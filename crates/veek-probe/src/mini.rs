//! Bounded Mini HID evidence collection. Counts are observations, never acceptance.
use std::{
    error::Error,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use veek_hardware::{
    protocol::ControlEvent,
    transport::{self, Address, DeviceDescriptor, ReadBatch, Session},
    Model,
};

const LIMIT: usize = 1_048_576;
#[derive(Default)]
struct Knob {
    min: Option<u8>,
    max: Option<u8>,
    presses: usize,
    releases: usize,
}
struct Evidence {
    raw: File,
    events: File,
    metadata: File,
    directory: PathBuf,
    bytes: usize,
    reports: usize,
    errors: usize,
    knobs: [Knob; 4],
}
impl Evidence {
    fn create(directory: &Path, initialize: bool) -> io::Result<Self> {
        fs::create_dir(directory)?;
        let mut raw = File::create_new(directory.join("reports.hex"))?;
        writeln!(
            raw,
            "# Physical Mini HID reads, not a hardware-validation verdict"
        )?;
        let mut events = File::create_new(directory.join("events.txt"))?;
        writeln!(events, "# elapsed_ms; decoded events / parser errors")?;
        let mut metadata = File::create_new(directory.join("metadata.txt"))?;
        writeln!(metadata, "format_version=1\nproducer=veek-probe {}\nos={}\nmodel_assumption=mini\nrevision=unknown_user_reports_Mini_1.0\ntransport=hid\nexpected_vid_pid=0483:a3c4\nhardware_validation=unverified\npaths_and_usb_serial=omitted\ninitialization_requested={initialize}\nmax_raw_bytes={LIMIT}\nstarted_unix_seconds={}\nstatus=in_progress", env!("CARGO_PKG_VERSION"), std::env::consts::OS, SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs())?;
        metadata.flush()?;
        fs::write(
            directory.join("RESULTS.txt"),
            include_str!("../../../tests/manual/MINI-RESULTS.txt"),
        )?;
        if let Ok(commit) = fs::read_to_string("BUILD_COMMIT.txt") {
            let commit = commit.trim();
            if commit.len() == 40 && commit.bytes().all(|b| b.is_ascii_hexdigit()) {
                fs::write(directory.join("BUILD_COMMIT.txt"), commit)?;
            }
        }
        Ok(Self {
            raw,
            events,
            metadata,
            directory: directory.to_owned(),
            bytes: 0,
            reports: 0,
            errors: 0,
            knobs: std::array::from_fn(|_| Knob::default()),
        })
    }
    fn descriptor(&mut self, device: &DeviceDescriptor) -> io::Result<()> {
        // Intentionally omit paths, serial values and arbitrary product strings.
        writeln!(self.metadata, "observed_vid={:04x}\nobserved_pid={:04x}\nbcdDevice={:?}\ninterface={:?}\nusage_page={:?}\nusage={:?}\nserial_present={}", device.vid.unwrap_or(0), device.pid.unwrap_or(0), device.release_number, device.interface, device.usage_page, device.usage, device.serial.is_some())?;
        self.metadata.flush()
    }
    fn record(&mut self, batch: ReadBatch, elapsed: Duration) -> io::Result<bool> {
        if batch.raw.is_empty() {
            return Ok(true);
        }
        if batch.raw.len() > LIMIT - self.bytes {
            return Ok(false);
        }
        writeln!(
            self.raw,
            "# elapsed_ms={} length={}",
            elapsed.as_millis(),
            batch.raw.len()
        )?;
        for b in &batch.raw {
            write!(self.raw, "{b:02x} ")?;
        }
        writeln!(self.raw)?;
        self.raw.flush()?;
        self.bytes += batch.raw.len();
        self.reports += 1;
        for event in batch.events {
            let text = match event {
                Ok(event) => {
                    match event {
                        ControlEvent::Analog { index, raw, .. } => {
                            if let Some(k) = self.knobs.get_mut(index as usize) {
                                k.min = Some(k.min.map_or(raw, |v| v.min(raw)));
                                k.max = Some(k.max.map_or(raw, |v| v.max(raw)));
                            }
                        }
                        ControlEvent::Button { index, pressed } => {
                            if let Some(k) = self.knobs.get_mut(index as usize) {
                                if pressed {
                                    k.presses += 1;
                                } else {
                                    k.releases += 1;
                                }
                            }
                        }
                    }
                    event.display(Model::Mini)
                }
                Err(error) => {
                    self.errors += 1;
                    format!("PARSE_ERROR: {error}")
                }
            };
            writeln!(self.events, "{} {text}", elapsed.as_millis())?;
            println!("{text}");
        }
        self.events.flush()?;
        Ok(true)
    }
    fn finish(&mut self, reason: &str, elapsed: Duration) -> io::Result<()> {
        let mut summary = format!(
            "Mini test stopped: {reason}\nReports: {}; raw bytes: {}; parser errors: {}\n\n",
            self.reports, self.bytes, self.errors
        );
        if self.bytes == 0 {
            summary.push_str("NO INPUT RECEIVED. This is not a successful control test.\n");
        }
        for (i, k) in self.knobs.iter().enumerate() {
            let varied = matches!((k.min,k.max),(Some(a),Some(b)) if a < b);
            summary.push_str(&format!("Wire knob {}: raw min={:?}, max={:?}, varied={varied}; press reports={}, release reports={}\n",i+1,k.min,k.max,k.presses,k.releases));
        }
        summary.push_str("\nThese are report counts, NOT verified physical actions. Startup reports may contribute.\nConfirm physical order, full travel and each press/release in RESULTS.txt.\nNo volume changes are expected: this test only reads hardware.\nReview and zip this whole capture folder for return. Nothing uploads automatically.\n");
        fs::write(self.directory.join("SUMMARY.txt"), &summary)?;
        writeln!(self.metadata, "bytes_saved={}\nreports_saved={}\nparser_errors={}\nelapsed_ms={}\nstop_reason={reason}\nstatus=finished",self.bytes,self.reports,self.errors,elapsed.as_millis())?;
        self.raw.sync_all()?;
        self.events.sync_all()?;
        self.metadata.sync_all()?;
        println!("\n{summary}\nSaved to: {}", self.directory.display());
        Ok(())
    }
}

fn choose(
    devices: Vec<DeviceDescriptor>,
    path: Option<&str>,
) -> Result<Option<DeviceDescriptor>, &'static str> {
    let mut matches: Vec<_> = devices
        .into_iter()
        .filter(|d| {
            d.model == Model::Mini
                && d.vid == Some(0x0483)
                && d.pid == Some(0xa3c4)
                && match &d.address {
                    Address::Hid(p) => path.is_none_or(|wanted| p.to_string_lossy() == wanted),
                    Address::Serial(_) => false,
                }
        })
        .collect();
    if matches.len() > 1 {
        return Err("More than one Mini HID interface found. Unplug other Minis and retry; if still ambiguous, use veek-probe list and select an exact --hid-path. No device was opened.");
    }
    Ok(matches.pop())
}

pub fn run(
    duration: u64,
    wait: u64,
    output: Option<PathBuf>,
    path: Option<String>,
    initialize: bool,
) -> Result<(), Box<dyn Error>> {
    let output = match output {
        Some(p) => p,
        None => {
            fs::create_dir_all("captures")?;
            PathBuf::from("captures").join(format!(
                "mini-{}-{}",
                SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
                std::process::id()
            ))
        }
    };
    let mut evidence = Evidence::create(&output, initialize)?;
    let running = Arc::new(AtomicBool::new(true));
    let signal = running.clone();
    ctrlc::set_handler(move || signal.store(false, Ordering::Relaxed))?;
    let start = Instant::now();
    let mut reason = "setup_error";
    let result = (|| -> Result<(), Box<dyn Error>> {
        println!("MINI HID TEST - no audio changes, LEDs or firmware writes.\nClose all other PCPanel apps. Plug in one Mini now.\nLooking for USB 0483:a3c4 for up to {wait} seconds. No COM port or Device Manager needed.\nEvidence folder: {}", output.display());
        let mut api = hidapi::HidApi::new()?;
        let discovery = Instant::now();
        let device = loop {
            if !running.load(Ordering::Relaxed) {
                reason = "interrupted";
                return Ok(());
            }
            if let Some(d) = choose(transport::discover_hid(&mut api)?, path.as_deref())? {
                break d;
            }
            if discovery.elapsed() >= Duration::from_secs(wait) {
                reason = "not_found";
                return Err("No matching Mini HID found. Check cable/connection. If it still cannot be found, return this capture folder; do not select a COM port or install a guessed driver.".into());
            }
            thread::sleep(Duration::from_millis(100));
        };
        evidence.descriptor(&device)?;
        reason = "open_or_init_error";
        let mut session = Session::new(Model::Mini, transport::open(&device)?, initialize)?;
        println!("\nMini HID opened. Recording for {duration} seconds.\nWork from LEFT to RIGHT: turn each knob fully both ways, then press/release it twice.\nThe displayed wire number may differ from physical order: record what you observe.\nKeep it connected for this run. Run the test again after unplug/replug. Ctrl+C stops early.");
        let recording = Instant::now();
        let mut stage = None;
        reason = "duration";
        while recording.elapsed() < Duration::from_secs(duration) {
            if !running.load(Ordering::Relaxed) {
                reason = "interrupted";
                break;
            }
            let next = (recording.elapsed().as_secs() * 5 / duration).min(4);
            if stage != Some(next) {
                stage = Some(next);
                if next < 4 {
                    println!("\n>>> Physical knob {} from the left: full travel both ways; press and release twice.",next+1);
                } else {
                    println!("\n>>> Repeat any missed controls; try quick turns and separate/simultaneous button presses.");
                }
            }
            let batch = match session.read() {
                Ok(batch) => batch,
                Err(error) => {
                    reason = "read_error_or_disconnect";
                    return Err(error.into());
                }
            };
            match evidence.record(batch, recording.elapsed()) {
                Ok(true) => (),
                Ok(false) => {
                    reason = "byte_limit";
                    break;
                }
                Err(error) => {
                    reason = "output_error";
                    return Err(error.into());
                }
            }
        }
        Ok(())
    })();
    evidence.finish(reason, start.elapsed())?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Directory(PathBuf);
    impl Directory {
        fn new() -> Self {
            Self(std::env::temp_dir().join(format!(
                    "veek-mini-{}-{}",
                    std::process::id(),
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_nanos()
                )))
        }
    }
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn descriptor(model: Model, vid: u16, pid: u16) -> DeviceDescriptor {
        let mut d = DeviceDescriptor::selected_original("NOT_OPENED".into());
        d.model = model;
        d.vid = Some(vid);
        d.pid = Some(pid);
        d.address = Address::Hid(std::ffi::CString::new("synthetic").unwrap());
        d
    }
    #[test]
    fn selection_never_opens_other_models_unknown_ids_or_ambiguous_interfaces() {
        let mini = descriptor(Model::Mini, 0x0483, 0xa3c4);
        assert!(choose(
            vec![
                descriptor(Model::Pro, 0x0483, 0xa3c5),
                descriptor(Model::Mini, 1, 2)
            ],
            None
        )
        .unwrap()
        .is_none());
        assert!(choose(vec![mini.clone(), mini.clone()], None).is_err());
        assert!(choose(vec![mini.clone()], Some("wrong")).unwrap().is_none());
        assert!(choose(vec![mini], Some("synthetic")).unwrap().is_some());
    }
    #[test]
    fn synthetic_reports_preserve_malformed_data_and_distinguish_counts_from_acceptance() {
        let dir = Directory::new();
        let mut e = Evidence::create(&dir.0, true).unwrap();
        for raw in [
            vec![1, 0, 0],
            vec![1, 0, 255],
            vec![2, 0, 1],
            vec![2, 0, 0],
            vec![0xff],
        ] {
            let events = vec![veek_hardware::protocol::parse_hid(Model::Mini, &raw)];
            assert!(e
                .record(
                    ReadBatch {
                        raw,
                        events,
                        heartbeats: 0
                    },
                    Duration::ZERO
                )
                .unwrap());
        }
        e.finish("duration", Duration::from_secs(1)).unwrap();
        let raw = fs::read_to_string(dir.0.join("reports.hex")).unwrap();
        assert!(raw.contains("ff "));
        let summary = fs::read_to_string(dir.0.join("SUMMARY.txt")).unwrap();
        assert!(summary.contains("raw min=Some(0), max=Some(255), varied=true"));
        assert!(summary.contains("press reports=1, release reports=1"));
        assert!(summary.contains("parser errors: 1"));
        assert!(summary.contains("NOT verified physical actions"));
        assert!(fs::read_to_string(dir.0.join("metadata.txt"))
            .unwrap()
            .contains("hardware_validation=unverified"));
        assert!(Evidence::create(&dir.0, true).is_err());
    }
    #[test]
    fn empty_and_capped_capture_cannot_look_like_a_pass() {
        let dir = Directory::new();
        let mut e = Evidence::create(&dir.0, false).unwrap();
        e.bytes = LIMIT;
        assert!(!e
            .record(
                ReadBatch {
                    raw: vec![1, 0, 5],
                    events: vec![],
                    heartbeats: 0
                },
                Duration::ZERO
            )
            .unwrap());
        e.bytes = 0;
        e.finish("not_found", Duration::ZERO).unwrap();
        assert!(fs::read_to_string(dir.0.join("SUMMARY.txt"))
            .unwrap()
            .contains("NO INPUT RECEIVED"));
        assert!(!fs::read_to_string(dir.0.join("reports.hex"))
            .unwrap()
            .contains("01 00 05"));
    }
}
