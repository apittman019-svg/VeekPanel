//! M2 diagnostics only. Persistent mappings/profiles and desktop UI belong to later milestones.
#![forbid(unsafe_code)]
use clap::{Args, Parser, Subcommand};
use std::{
    error::Error,
    io::{self, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};
use veek_audio::{resolve, Change, Controller, Pickup, PressEdge};
#[cfg(target_os = "linux")]
use veek_audio_linux::PipeWire as Native;
#[cfg(windows)]
use veek_audio_windows::CoreAudio as Native;
use veek_hardware::{
    transport::{self, DeviceDescriptor, Session},
    ControlEvent, Model,
};
type AppResult<T> = Result<T, Box<dyn Error + Send + Sync>>;
#[derive(Parser)]
#[command(
    version,
    about = "Experimental M2 native audio diagnostic. JSON output; no saved mappings or GUI."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    /// Read-only snapshot of outputs, inputs and live application streams/sessions.
    List,
    /// Observe native changes and reconnect after audio service failure. No writes.
    Watch {
        #[arg(long,default_value_t=30,value_parser=clap::value_parser!(u64).range(1..=86400))]
        duration: u64,
    },
    /// Set a selected live target's volume (0..100) OR mute, then report OS readback.
    Set {
        #[arg(long)]
        target: String,
        #[command(flatten)]
        change: ChangeArgs,
    },
    /// Temporary explicit hardware binding. Initial positions/held presses do not act.
    Bind {
        #[arg(
            long,
            required_unless_present = "hid_path",
            conflicts_with = "hid_path"
        )]
        serial: Option<String>,
        /// Exact allowlisted HID path printed by veek-probe list.
        #[arg(long, required_unless_present = "serial", conflicts_with = "serial")]
        hid_path: Option<String>,
        #[arg(long)]
        target: String,
        /// One-based wire knob index; physical ordering still requires hardware validation.
        #[arg(long,default_value_t=1,value_parser=clap::value_parser!(u8).range(1..=5))]
        knob: u8,
        /// Optional separate mute target for this knob's push button; disabled unless provided.
        #[arg(long)]
        button_target: Option<String>,
        #[arg(long,default_value_t=60,value_parser=clap::value_parser!(u64).range(1..=86400))]
        duration: u64,
    },
}
#[derive(Args)]
#[group(required = true, multiple = false)]
struct ChangeArgs {
    #[arg(long,value_parser=parse_percent)]
    volume: Option<f32>,
    #[arg(long,value_parser=["on","off"])]
    mute: Option<String>,
}
fn parse_percent(value: &str) -> Result<f32, String> {
    let value = value
        .parse::<f32>()
        .map_err(|_| "expected a number".to_string())?;
    if !value.is_finite() || !(0.0..=100.0).contains(&value) {
        return Err("volume must be finite and between 0 and 100".into());
    }
    Ok(value / 100.0)
}
fn main() {
    let cli = Cli::parse();
    let stop = Arc::new(AtomicBool::new(false));
    let cancelled = stop.clone();
    if let Err(e) = ctrlc::set_handler(move || cancelled.store(true, Ordering::Relaxed)) {
        eprintln!("ERROR: {e}");
        std::process::exit(1);
    }
    // Native objects are created, called and dropped exclusively on this worker.
    let result = thread::Builder::new()
        .name("veek-audio".into())
        .spawn(move || run(cli.command, stop))
        .and_then(|worker| {
            worker
                .join()
                .map_err(|_| io::Error::other("audio worker panicked"))
        });
    match result {
        Ok(Ok(())) => (),
        Ok(Err(e)) => {
            eprintln!("ERROR: {e}");
            std::process::exit(1)
        }
        Err(e) => {
            eprintln!("ERROR: {e}");
            std::process::exit(1)
        }
    }
}
fn output(value: serde_json::Value) -> AppResult<()> {
    let mut stdout = io::stdout().lock();
    serde_json::to_writer(&mut stdout, &value)?;
    writeln!(stdout)?;
    stdout.flush()?;
    Ok(())
}
fn run(command: Command, stop: Arc<AtomicBool>) -> AppResult<()> {
    if let Command::Watch { duration } = command {
        return watch(duration, &stop);
    }
    let mut controller = Controller::new(Native::connect()?);
    match command {
        Command::List => output(serde_json::to_value(controller.snapshot()?)?),
        Command::Set { target, change } => {
            let selection = controller.select(&target)?;
            let change = if let Some(volume) = change.volume {
                Change::Volume(volume)
            } else {
                Change::Mute(change.mute.as_deref() == Some("on"))
            };
            let receipt = controller.apply(&selection, change)?;
            let confirmed = receipt.confirmed;
            output(serde_json::to_value(receipt)?)?;
            if !confirmed {
                return Err(
                    "native write returned, but the requested state was not confirmed by readback"
                        .into(),
                );
            }
            Ok(())
        }
        Command::Bind {
            serial,
            hid_path,
            target,
            knob,
            button_target,
            duration,
        } => {
            let descriptor = if let Some(port) = serial {
                DeviceDescriptor::selected_original(port)
            } else {
                let path = hid_path.expect("clap requires exactly one hardware address");
                let mut api = hidapi::HidApi::new()?;
                transport::discover_hid(&mut api)?.into_iter().find(|d|matches!(&d.address,transport::Address::Hid(p) if p.to_string_lossy()==path)).ok_or("selected HID path is not a recognized PCPanel")?
            };
            if knob > if descriptor.model == Model::Pro { 5 } else { 4 } {
                return Err("selected model does not have that knob".into());
            }
            let selected = controller.select(&target)?;
            let button = button_target
                .as_deref()
                .map(|id| controller.select(id))
                .transpose()?;
            let initial = controller.snapshot()?;
            if resolve(&initial, &selected.id)?.volume.is_none() {
                return Err("selected target has no volume control".into());
            }
            if let Some(b) = &button {
                if resolve(&initial, &b.id)?.muted.is_none() {
                    return Err("button target has no mute control".into());
                }
            }
            let mut session = Session::new(descriptor.model, transport::open(&descriptor)?, true)?;
            let mut pickup = Pickup::default();
            let mut edge = PressEdge::default();
            output(
                serde_json::json!({"status":"binding","hardware_validation":"unverified","target":selected,"button_target":button,"knob":knob,"pickup":"move through the current OS volume; release before pressing"}),
            )?;
            let deadline = Instant::now() + Duration::from_secs(duration);
            let mut reconcile = Instant::now();
            while Instant::now() < deadline && !stop.load(Ordering::Relaxed) {
                controller.wait(Duration::from_millis(1))?;
                if Instant::now() >= reconcile {
                    let snapshot = controller.snapshot()?;
                    resolve(&snapshot, &selected.id)?;
                    if let Some(button) = &button {
                        resolve(&snapshot, &button.id)?;
                    }
                    reconcile = Instant::now() + Duration::from_secs(1);
                }
                let batch = session.read()?;
                for event in batch.events {
                    let event = match event {
                        Ok(e) => e,
                        Err(e) => {
                            eprintln!("Ignored unrecognized hardware input: {e}");
                            continue;
                        }
                    };
                    let action = match event {
                        ControlEvent::Analog {
                            index,
                            raw,
                            maximum,
                        } if index == knob - 1 => {
                            let snapshot = controller.snapshot()?;
                            let observed = resolve(&snapshot, &selected.id)?
                                .volume
                                .ok_or("volume capability disappeared")?;
                            pickup
                                .update(raw as f32 / maximum as f32, observed)
                                .map(|value| (&selected, Change::Volume(value)))
                        }
                        ControlEvent::Button { index, pressed }
                            if index == knob - 1 && edge.update(pressed) =>
                        {
                            if let Some(b) = &button {
                                let snapshot = controller.snapshot()?;
                                let muted = resolve(&snapshot, &b.id)?
                                    .muted
                                    .ok_or("mute capability disappeared")?;
                                Some((b, Change::Mute(!muted)))
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };
                    if let Some((selection, change)) = action {
                        let receipt = controller.apply(selection, change)?;
                        let confirmed = receipt.confirmed;
                        output(serde_json::to_value(receipt)?)?;
                        if !confirmed {
                            return Err(
                                "binding stopped: OS did not confirm the requested audio state"
                                    .into(),
                            );
                        }
                    }
                }
            }
            output(serde_json::json!({"status":"stopped","reason":"duration or cancellation"}))
        }
        Command::Watch { .. } => unreachable!(),
    }
}
fn watch(duration: u64, stop: &AtomicBool) -> AppResult<()> {
    let deadline = Instant::now() + Duration::from_secs(duration);
    let mut controller = None;
    let mut previous = None;
    let mut retry = Instant::now();
    while Instant::now() < deadline && !stop.load(Ordering::Relaxed) {
        if controller.is_none() && Instant::now() >= retry {
            match Native::connect() {
                Ok(backend) => {
                    controller = Some(Controller::new(backend));
                    previous = None;
                }
                Err(e) => {
                    output(serde_json::json!({"status":"unavailable","error":e.to_string()}))?;
                    retry = Instant::now() + Duration::from_secs(2);
                }
            }
        }
        if let Some(c) = controller.as_mut() {
            match c.snapshot().and_then(|snapshot| {
                c.wait(Duration::from_millis(200))?;
                Ok(snapshot)
            }) {
                Ok(snapshot) => {
                    if previous.as_ref() != Some(&snapshot) {
                        output(serde_json::to_value(&snapshot)?)?;
                        previous = Some(snapshot);
                    }
                }
                Err(e) => {
                    output(serde_json::json!({"status":"unavailable","error":e.to_string()}))?;
                    controller = None;
                    previous = None;
                    retry = Instant::now() + Duration::from_secs(2);
                }
            }
        } else {
            thread::sleep(Duration::from_millis(100));
        }
    }
    Ok(())
}
