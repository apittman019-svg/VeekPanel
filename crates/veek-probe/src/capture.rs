//! Local raw serial evidence, never a declaration of hardware compatibility.
use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub struct Capture {
    raw: File,
    chunks: File,
    metadata: File,
    bytes: usize,
    limit: usize,
}

impl Capture {
    /// A new directory is mandatory: never overwrite or append to prior evidence.
    pub fn create(directory: &Path, limit: usize) -> io::Result<Self> {
        fs::create_dir(directory)?;
        let raw = File::create_new(directory.join("serial.bin"))?;
        let mut chunks = File::create_new(directory.join("chunks.tsv"))?;
        let mut metadata = File::create_new(directory.join("metadata.txt"))?;
        writeln!(chunks, "elapsed_ms\tlength\thex")?;
        writeln!(metadata, "format_version=1\nproducer=veek-probe {}\ntransport=serial\nmodel_assumption=original_maple\nhardware_validation=unverified\nsource_identity=unverified\nsettings=9600_8N1_no_flow_control\nprotocol_writes=none\nport_and_usb_serial=omitted\nmax_bytes={limit}\nstarted_unix_seconds={}\nstatus=in_progress",
            env!("CARGO_PKG_VERSION"), SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs())?;
        metadata.flush()?;
        Ok(Self {
            raw,
            chunks,
            metadata,
            bytes: 0,
            limit,
        })
    }

    /// Returns false before a whole chunk would exceed the limit. No partial chunk is saved.
    pub fn record(&mut self, bytes: &[u8], elapsed: Duration) -> io::Result<bool> {
        if bytes.len() > self.limit - self.bytes {
            return Ok(false);
        }
        if bytes.is_empty() {
            return Ok(true);
        }
        self.raw.write_all(bytes)?;
        self.raw.flush()?;
        self.bytes += bytes.len();
        write!(self.chunks, "{}\t{}\t", elapsed.as_millis(), bytes.len())?;
        for byte in bytes {
            write!(self.chunks, "{byte:02x}")?;
        }
        writeln!(self.chunks)?;
        self.chunks.flush()?;
        Ok(true)
    }

    pub fn finish(&mut self, reason: &str, elapsed: Duration) -> io::Result<()> {
        // No OS error text or device paths in metadata; those remain local stderr diagnostics.
        writeln!(
            self.metadata,
            "bytes_saved={}\nelapsed_ms={}\nstop_reason={reason}\nstatus=finished",
            self.bytes,
            elapsed.as_millis()
        )?;
        self.raw.sync_all()?;
        self.chunks.sync_all()?;
        self.metadata.sync_all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
    };
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Directory(PathBuf);
    impl Directory {
        fn new() -> Self {
            Self(std::env::temp_dir().join(format!(
                    "veek-capture-test-{}-{}-{}",
                    std::process::id(),
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_nanos(),
                    NEXT.fetch_add(1, Ordering::Relaxed)
                )))
        }
    }
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn preserves_invalid_bytes_and_chunk_boundaries_without_certifying_hardware() {
        let dir = Directory::new();
        let mut capture = Capture::create(&dir.0, 100).unwrap();
        assert!(capture.record(b"v0x", Duration::from_millis(12)).unwrap());
        assert!(capture
            .record(&[0xff, 0, b'\n'], Duration::from_millis(34))
            .unwrap());
        capture.finish("duration", Duration::from_secs(1)).unwrap();
        drop(capture);
        assert_eq!(fs::read(dir.0.join("serial.bin")).unwrap(), b"v0x\xff\0\n");
        assert_eq!(
            fs::read_to_string(dir.0.join("chunks.tsv")).unwrap(),
            "elapsed_ms\tlength\thex\n12\t3\t763078\n34\t3\tff000a\n"
        );
        let metadata = fs::read_to_string(dir.0.join("metadata.txt")).unwrap();
        assert!(metadata.contains("hardware_validation=unverified"));
        assert!(metadata.contains("bytes_saved=6"));
        assert!(metadata.contains("stop_reason=duration"));
    }

    #[test]
    fn cap_preserves_complete_chunks_and_existing_evidence_is_not_overwritten() {
        let dir = Directory::new();
        let mut capture = Capture::create(&dir.0, 4).unwrap();
        assert!(capture.record(b"abc", Duration::ZERO).unwrap());
        assert!(!capture.record(b"de", Duration::ZERO).unwrap());
        capture.finish("byte_limit", Duration::ZERO).unwrap();
        drop(capture);
        assert_eq!(
            Capture::create(&dir.0, 4).err().unwrap().kind(),
            io::ErrorKind::AlreadyExists
        );
        assert_eq!(fs::read(dir.0.join("serial.bin")).unwrap(), b"abc");
    }

    #[test]
    fn unfinished_capture_stays_in_progress() {
        let dir = Directory::new();
        drop(Capture::create(&dir.0, 4).unwrap());
        let metadata = fs::read_to_string(dir.0.join("metadata.txt")).unwrap();
        assert!(metadata.contains("status=in_progress"));
        assert!(!metadata.contains("status=finished"));
    }
}
