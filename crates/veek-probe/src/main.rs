mod capture;
mod mini;

use clap::{Parser, Subcommand, ValueEnum};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader, Read, Write},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use veek_hardware::{
    protocol::{parse_hid, OriginalDecoder, SerialMessage},
    transport::{self, Session},
    Model,
};

#[derive(Parser)]
#[command(
    version,
    about = "PCPanel hardware probe. No audio control. Original/Maple support is experimental."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Enumerate recognized HID devices and serial candidates without opening ports.
    List,
    /// Guided Original test: compare ports, select explicitly, collect 60 seconds of raw evidence.
    TestOriginal,
    /// Mini-only HID test: auto-detect, guide all four knobs/buttons, save bounded evidence.
    TestMini {
        #[arg(long, default_value_t = 90, value_parser = clap::value_parser!(u64).range(1..=300))]
        duration: u64,
        #[arg(long, default_value_t = 30, value_parser = clap::value_parser!(u64).range(1..=120))]
        wait: u64,
        /// New capture directory; parent must exist. Default: captures/mini-unique-id.
        #[arg(long)]
        output: Option<PathBuf>,
        /// Only needed if more than one matching Mini HID interface is present.
        #[arg(long)]
        hid_path: Option<String>,
        /// Diagnostic comparison without the documented HID state request.
        #[arg(long)]
        no_init: bool,
    },
    /// Save raw Original serial bytes for investigation; never validates hardware identity.
    Capture {
        /// Explicit port only. No discovery probes, initialization writes or automatic reconnect.
        #[arg(long)]
        serial: String,
        /// New directory beneath an existing parent; existing evidence is never overwritten.
        #[arg(long)]
        output: PathBuf,
        #[arg(long, default_value_t = 30, value_parser = clap::value_parser!(u64).range(1..=300))]
        duration: u64,
        /// Raw byte cap, excluding the bounded hex/metadata representation.
        #[arg(long, default_value_t = 1_048_576, value_parser = clap::value_parser!(u32).range(1..=16_777_216))]
        max_bytes: u32,
    },
    /// Read real controls; automatically discover/reconnect known HID devices.
    Watch {
        /// Explicit Original/Maple port (COM3 or /dev/serial/by-id/...). No generic serial probing.
        #[arg(long)]
        serial: Option<String>,
        /// Print raw reports/chunks in hex as well as decoded events.
        #[arg(long)]
        raw: bool,
        /// Skip HID initialization write (Original serial never receives writes).
        #[arg(long)]
        no_init: bool,
        /// Stop after this many seconds (otherwise Ctrl+C).
        #[arg(long, value_parser = clap::value_parser!(u64).range(1..=86400))]
        duration: Option<u64>,
    },
    /// Decode a fixture/capture OFFLINE; this never opens hardware.
    Replay {
        #[arg(long, value_enum)]
        model: ReplayModel,
        /// Original: raw serial bytes. HID: one hex report per line, # comments allowed.
        file: PathBuf,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum ReplayModel {
    Original,
    Rgb,
    Mini,
    Pro,
}
impl From<ReplayModel> for Model {
    fn from(value: ReplayModel) -> Self {
        match value {
            ReplayModel::Original => Self::Original,
            ReplayModel::Rgb => Self::Rgb,
            ReplayModel::Mini => Self::Mini,
            ReplayModel::Pro => Self::Pro,
        }
    }
}

fn main() {
    if let Err(error) = run() {
        // A closed output pipe is an ordinary diagnostic CLI shutdown.
        if error
            .downcast_ref::<io::Error>()
            .is_some_and(|e| e.kind() == io::ErrorKind::BrokenPipe)
        {
            return;
        }
        eprintln!("ERROR: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    match Cli::parse().command {
        Command::List => list(),
        Command::TestOriginal => test_original(),
        Command::TestMini {
            duration,
            wait,
            output,
            hid_path,
            no_init,
        } => mini::run(duration, wait, output, hid_path, !no_init),
        Command::Capture {
            serial,
            output,
            duration,
            max_bytes,
        } => capture_serial(serial, output, duration, max_bytes as usize),
        Command::Watch {
            serial,
            raw,
            no_init,
            duration,
        } => watch(serial, raw, !no_init, duration),
        Command::Replay { model, file } => replay(model.into(), file),
    }
}

fn prompt(message: &str) -> Result<String, Box<dyn Error>> {
    print!("{message}");
    io::stdout().flush()?;
    let mut answer = String::new();
    let n = io::stdin().lock().take(1025).read_line(&mut answer)?;
    if n == 0 {
        return Err("Input closed; no device was selected".into());
    }
    if n > 1024 {
        return Err("Input exceeds 1024 bytes".into());
    }
    Ok(answer.trim().to_owned())
}

fn test_original() -> Result<(), Box<dyn Error>> {
    println!("Original PCPanel evidence collection — hardware behavior is UNVERIFIED.\nClose other PCPanel software. Run as your ordinary user. No audio changes or protocol writes.");
    prompt("Unplug the PCPanel, then press Enter to list the baseline ports: ")?;
    list()?;
    prompt("Connect the PCPanel, wait for the OS to recognize it, then press Enter: ")?;
    list()?;
    let port =
        prompt("Enter its newly appeared serial port exactly (for example COM3); empty cancels: ")?;
    if port.is_empty() {
        return Err("Canceled; no device was opened".into());
    }
    println!("The chosen port is not verified as a PCPanel. Opening it may reset some boards.\nDuring the 60-second capture: slowly turn each of the four knobs fully both ways,\nthen press and release each knob separately. Note which physical knob you used.\nRecord any mismatch in RESULTS.txt afterward.");
    if !prompt("Press Enter to start, or type anything to cancel: ")?.is_empty() {
        return Err("Canceled; no device was opened".into());
    }
    let parent = PathBuf::from("captures");
    std::fs::create_dir_all(&parent)?;
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let output = parent.join(format!("original-{stamp}-{}", std::process::id()));
    println!("Evidence directory: {:?}", output);
    capture_serial(port, output.clone(), 60, 1_048_576)?;
    std::fs::write(
        output.join("RESULTS.txt"),
        include_str!("../../../tests/manual/RESULTS.txt"),
    )?;
    println!("Captured raw transport data. The following replay is OFFLINE and cannot confirm physical correctness:");
    if let Err(error) = replay(Model::Original, output.join("serial.bin")) {
        println!("Replay reported: {error}. Raw evidence is preserved; partial lines or a different stock protocol need investigation.");
    }
    println!("Fill in RESULTS.txt and review the folder before sharing. Hardware validation remains pending human review.");
    Ok(())
}

fn capture_serial(
    port: String,
    output: PathBuf,
    duration: u64,
    max_bytes: usize,
) -> Result<(), Box<dyn Error>> {
    let running = Arc::new(AtomicBool::new(true));
    let signal = running.clone();
    ctrlc::set_handler(move || signal.store(false, Ordering::Relaxed))?;
    // Create evidence first so a failed device open is recorded, too. No HID devices are opened.
    let mut capture = capture::Capture::create(&output, max_bytes)?;
    let start = Instant::now();
    let descriptor = transport::DeviceDescriptor::selected_original(port);
    let mut device = match transport::open(&descriptor) {
        Ok(device) => device,
        Err(error) => {
            capture.finish("open_error", start.elapsed())?;
            return Err(error.into());
        }
    };
    let mut out = io::stdout().lock();
    let mut reason = "duration";
    let result = (|| -> Result<(), Box<dyn Error>> {
        writeln!(out, "CAPTURE_OPENED_UNVERIFIED: serial at 9600 8N1; saving received bytes, not certifying PCPanel identity.\nNo protocol writes. Opening serial may reset some boards. Ctrl+C stops.")?;
        out.flush()?;
        let mut bytes = [0; 256];
        while running.load(Ordering::Relaxed) && start.elapsed() < Duration::from_secs(duration) {
            let n = match device.read(&mut bytes) {
                Ok(n) if n <= bytes.len() => n,
                Ok(n) => {
                    reason = "read_error";
                    return Err(transport::HardwareError::InvalidRead(n).into());
                }
                Err(error) => {
                    reason = "read_error";
                    return Err(error.into());
                }
            };
            if !capture.record(&bytes[..n], start.elapsed())? {
                reason = "byte_limit";
                break;
            }
        }
        if !running.load(Ordering::Relaxed) {
            reason = "interrupted";
        }
        Ok(())
    })();
    if result.is_err() && reason != "read_error" {
        reason = "output_error";
    }
    capture.finish(reason, start.elapsed())?;
    result?;
    writeln!(
        out,
        "Capture stopped ({reason}). Hardware validation remains unverified. Evidence: {:?}",
        output
    )?;
    Ok(())
}

fn list() -> Result<(), Box<dyn Error>> {
    let mut out = io::stdout().lock();
    let mut api = hidapi::HidApi::new()?;
    let devices = transport::discover_hid(&mut api)?;
    writeln!(out, "Recognized PCPanel HID interfaces: {}", devices.len())?;
    for d in devices {
        writeln!(out, "PCPanel {} {:04x}:{:04x} path={:?} product={:?} serial={:?} bcdDevice={:?} interface={:?} usage={:?}:{:?}",
            d.model, d.vid.unwrap_or(0), d.pid.unwrap_or(0), d.key(), d.product, d.serial, d.release_number, d.interface, d.usage_page, d.usage)?;
    }
    writeln!(out, "Serial candidates (NOT identified as PCPanel):")?;
    for port in transport::serial_candidates()? {
        writeln!(out, "  {:?}: {:?}", port.port_name, port.port_type)?;
    }
    writeln!(out, "Original/Maple has no verified unique USB ID. Select its port with watch --serial PORT.\nClose other PCPanel software before watch. See docs/LINUX_SETUP.md or docs/WINDOWS_SETUP.md.")?;
    Ok(())
}

struct Worker {
    stop: Arc<AtomicBool>,
    handle: thread::JoinHandle<()>,
}

fn watch(
    serial: Option<String>,
    raw: bool,
    initialize: bool,
    duration: Option<u64>,
) -> Result<(), Box<dyn Error>> {
    let running = Arc::new(AtomicBool::new(true));
    let signal = running.clone();
    ctrlc::set_handler(move || signal.store(false, Ordering::Relaxed))?;
    let mut api = hidapi::HidApi::new()?;
    let (tx, rx) = mpsc::sync_channel::<String>(256);
    let mut workers: HashMap<String, Worker> = HashMap::new();
    let start = Instant::now();
    let mut next_scan = start;
    let mut out = io::stdout().lock();
    writeln!(
        out,
        "LIVE hardware mode. Waiting for PCPanel. Ctrl+C stops. No audio is changed."
    )?;
    if serial.is_some() {
        writeln!(out, "Original/Maple: experimental serial grammar; stock firmware not yet verified. Opening a serial port may reset its board.")?;
    }
    // Keep cleanup outside the fallible loop, including output and enumeration errors.
    let result = (|| -> Result<(), Box<dyn Error>> {
        while running.load(Ordering::Relaxed)
            && duration.is_none_or(|s| start.elapsed() < Duration::from_secs(s))
        {
            if Instant::now() >= next_scan {
                // 3s discovery/retry fallback, independent of blocking input-reader threads.
                next_scan = Instant::now() + Duration::from_secs(3);
                let mut scan_ok = true;
                let mut devices = match transport::discover_hid(&mut api) {
                    Ok(devices) => devices,
                    Err(error) => {
                        scan_ok = false;
                        writeln!(out, "DISCOVERY_ERROR: {error}; retrying in 3 seconds")?;
                        // Do not tear down healthy handles on a transient enumeration failure.
                        Vec::new()
                    }
                };
                if let Some(port) = &serial {
                    devices.push(transport::DeviceDescriptor::selected_original(port.clone()));
                }
                if scan_ok {
                    let present: HashSet<_> = devices.iter().map(|d| d.key()).collect();
                    for (key, worker) in &workers {
                        if !present.contains(key) && !worker.stop.swap(true, Ordering::Relaxed) {
                            writeln!(out, "DISCONNECTED [{key:?}]: interface removed")?;
                        }
                    }
                }
                // Read errors detect unplug. A finished worker is joined before reopening its path.
                let finished: Vec<_> = workers
                    .iter()
                    .filter(|(_, w)| w.handle.is_finished())
                    .map(|(key, _)| key.clone())
                    .collect();
                for key in finished {
                    if let Some(worker) = workers.remove(&key) {
                        if worker.handle.join().is_err() {
                            writeln!(out, "WORKER_ERROR [{key:?}]: reader panicked")?;
                        }
                    }
                }
                let mut seen = HashSet::new();
                for d in devices {
                    let key = d.key();
                    if !seen.insert(key.clone()) || workers.contains_key(&key) {
                        continue;
                    }
                    let stop = Arc::new(AtomicBool::new(false));
                    let worker_stop = stop.clone();
                    let sender = tx.clone();
                    let handle = thread::Builder::new().name("pcpanel-reader".into()).spawn(move || {
                        let send = |message: String| sender.send(format!("[{:?}] {message}", d.key())).is_ok();
                        let result = (|| -> Result<(), transport::HardwareError> {
                            let mut session = Session::new(d.model, transport::open(&d)?, initialize)?;
                            let state = if d.model == Model::Original { "OPENED_UNVERIFIED" } else { "CONNECTED" };
                            if !send(format!("{state}: PCPanel {}", d.model)) { return Ok(()); }
                            let mut got_control = false;
                            let mut got_heartbeat = false;
                            let mut identified = false;
                            while !worker_stop.load(Ordering::Relaxed) {
                                let batch = session.read()?;
                                if raw && !batch.raw.is_empty() && !send(format!("RAW {:02x?}", batch.raw)) { break; }
                                got_heartbeat |= batch.heartbeats > 0;
                                for event in batch.events {
                                    let text = match event {
                                        Ok(event) => { got_control = true; event.display(d.model) },
                                        Err(error) => format!("PARSE_ERROR: {error}"),
                                    };
                                    if !send(text) { return Ok(()); }
                                }
                                if d.model == Model::Original && got_control && got_heartbeat && !identified {
                                    identified = true;
                                    if !send("PROTOCOL_MATCH: Original/Maple-compatible traffic (identity still unverified)".into()) { break; }
                                }
                            }
                            Ok(())
                        })();
                        if let Err(error) = result { send(format!("DISCONNECTED_OR_OPEN_FAILED: {error}; retrying")); }
                    })?;
                    workers.insert(key, Worker { stop, handle });
                }
            }
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(message) => {
                    writeln!(out, "{message}")?;
                    out.flush()?;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        Ok(())
    })();
    for worker in workers.values() {
        worker.stop.store(true, Ordering::Relaxed);
    }
    drop(rx); // Unblock senders before joining, even when stdout failed.
    for (_, worker) in workers {
        let _ = worker.handle.join();
    }
    result?;
    writeln!(
        out,
        "Stopped. Hardware validation requires observed physical control events."
    )?;
    Ok(())
}

fn replay(model: Model, file: PathBuf) -> Result<(), Box<dyn Error>> {
    let mut out = io::stdout().lock();
    writeln!(out, "OFFLINE REPLAY — no hardware is opened or validated")?;
    let input = File::open(file)?;
    if model == Model::Original {
        let mut decoder = OriginalDecoder::default();
        let mut reader = BufReader::new(input);
        let mut buffer = [0; 256];
        loop {
            let n = reader.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            for message in decoder.feed(&buffer[..n]) {
                match message? {
                    SerialMessage::Control(event) => writeln!(out, "{}", event.display(model))?,
                    SerialMessage::Heartbeat => writeln!(out, "HEARTBEAT")?,
                }
            }
        }
        decoder.finish()?;
    } else {
        let mut reader = BufReader::new(input);
        loop {
            // Bounded line reads even for malicious fixture files.
            let mut line = String::new();
            let n = reader.by_ref().take(1025).read_line(&mut line)?;
            if n == 0 {
                break;
            }
            if n > 1024 {
                return Err("hex report line exceeds 1024 bytes".into());
            }
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let bytes = line
                .split_whitespace()
                .map(|b| u8::from_str_radix(b, 16))
                .collect::<Result<Vec<_>, _>>()?;
            writeln!(out, "{}", parse_hid(model, &bytes)?.display(model))?;
        }
    }
    Ok(())
}
