//! Window policy stays separate from native tray/instance plumbing.
#![forbid(unsafe_code)]

pub fn start_hidden(preference: bool, backend_ready: bool, tray_ready: bool, reopen: bool) -> bool {
    preference && backend_ready && tray_ready && !reopen
}

pub fn reveal_second_launch(args: &[String]) -> bool {
    !args.iter().any(|arg| arg == "--autostart")
}

// AppIndicator creation can succeed without a visible tray host. No observed
// host means no hiding, including on desktops with only a legacy XEmbed tray.
#[cfg(target_os = "linux")]
pub fn tray_host_ready() -> Result<(), String> {
    let observed = (|| -> zbus::Result<bool> {
        let connection = zbus::blocking::connection::Builder::session()?
            .method_timeout(std::time::Duration::from_secs(2))
            .build()?;
        let proxy: zbus::blocking::Proxy<'_> = zbus::blocking::proxy::Builder::new(&connection)
            .destination("org.kde.StatusNotifierWatcher")?
            .path("/StatusNotifierWatcher")?
            .interface("org.kde.StatusNotifierWatcher")?
            .cache_properties(zbus::proxy::CacheProperties::No)
            .build()?;
        proxy.get_property("IsStatusNotifierHostRegistered")
    })();
    match observed {
        Ok(true) => Ok(()),
        Ok(false) => Err("No desktop tray host is registered".into()),
        Err(e) => Err(format!("Cannot confirm a desktop tray host: {e}")),
    }
}

#[cfg(any(target_os = "linux", test))]
#[derive(Default)]
pub struct TrayRecovery {
    previous: Option<bool>,
}

#[cfg(any(target_os = "linux", test))]
impl TrayRecovery {
    /// Reveal on initial failure or loss, once per transition. Recovery never hides.
    pub fn observe(&mut self, ready: bool) -> bool {
        let reveal = !ready && self.previous != Some(false);
        self.previous = Some(ready);
        reveal
    }
}

#[cfg(any(target_os = "linux", test))]
pub struct TrayMonitor {
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    join: Option<std::thread::JoinHandle<()>>,
}

#[cfg(any(target_os = "linux", test))]
impl TrayMonitor {
    pub fn start(
        interval: std::time::Duration,
        mut probe: impl FnMut() -> Result<(), String> + Send + 'static,
        mut publish: impl FnMut(Result<(), String>) + Send + 'static,
    ) -> std::io::Result<Self> {
        use std::sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        };
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = stop.clone();
        let join = std::thread::Builder::new()
            .name("veek-tray".into())
            .spawn(move || {
                while !stopping.load(Ordering::Acquire) {
                    let result = probe();
                    if stopping.load(Ordering::Acquire) {
                        break;
                    }
                    publish(result);
                    // Drop unparks the worker, so shutdown never waits for this interval.
                    std::thread::park_timeout(interval);
                }
            })?;
        Ok(Self {
            stop,
            join: Some(join),
        })
    }
}

#[cfg(any(target_os = "linux", test))]
impl Drop for TrayMonitor {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Release);
        if let Some(join) = self.join.take() {
            join.thread().unpark();
            let _ = join.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hiding_requires_a_working_recovery_route() {
        for preference in [false, true] {
            for backend in [false, true] {
                for tray in [false, true] {
                    for reopen in [false, true] {
                        assert_eq!(
                            start_hidden(preference, backend, tray, reopen),
                            preference && backend && tray && !reopen
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn manual_launch_reopens_but_duplicate_login_launch_stays_quiet() {
        assert!(reveal_second_launch(&["veekpanel".into()]));
        assert!(!reveal_second_launch(&[
            "veekpanel".into(),
            "--autostart".into()
        ]));
    }

    #[test]
    fn tray_loss_reveals_once_and_recovery_does_not_hide() {
        let mut recovery = TrayRecovery::default();
        assert!(recovery.observe(false));
        assert!(!recovery.observe(false));
        assert!(!recovery.observe(true));
        assert!(recovery.observe(false));
        assert!(!recovery.observe(false));
        assert!(!recovery.observe(true));
        let mut healthy = TrayRecovery::default();
        assert!(!healthy.observe(true));
        assert!(!healthy.observe(true));
        assert!(healthy.observe(false));
    }

    #[test]
    fn monitor_shutdown_wakes_idle_worker_and_releases_owned_probe() {
        use std::sync::mpsc;
        use std::time::{Duration, Instant};
        struct OwnedProbe(mpsc::Sender<()>);
        impl Drop for OwnedProbe {
            fn drop(&mut self) {
                let _ = self.0.send(());
            }
        }
        let (released, release) = mpsc::channel();
        let owner = OwnedProbe(released);
        let (published, received) = mpsc::channel();
        let monitor = TrayMonitor::start(
            Duration::from_secs(60),
            move || {
                let _keep_alive = &owner;
                Err("no host".into())
            },
            move |value| {
                let _ = published.send(value);
            },
        )
        .unwrap();
        assert!(received
            .recv_timeout(Duration::from_secs(2))
            .unwrap()
            .is_err());
        let started = Instant::now();
        drop(monitor);
        assert!(started.elapsed() < Duration::from_secs(2));
        release.recv_timeout(Duration::from_secs(1)).unwrap();
        assert!(received.recv().is_err()); // No surviving publisher/thread.
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[ignore = "requires dbus-run-session and VEEK_PRIVATE_DBUS_TEST=1"]
    fn private_bus_tray_host_loss_and_recovery() {
        use std::sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        };
        assert_eq!(
            std::env::var("VEEK_PRIVATE_DBUS_TEST").as_deref(),
            Ok("1"),
            "Run only on the disposable CI/test session bus"
        );
        struct Watcher(Arc<AtomicBool>);
        #[zbus::interface(name = "org.kde.StatusNotifierWatcher")]
        impl Watcher {
            #[zbus(property)]
            fn is_status_notifier_host_registered(&self) -> bool {
                self.0.load(Ordering::SeqCst)
            }
        }
        let registered = Arc::new(AtomicBool::new(false));
        let serve = || {
            zbus::blocking::connection::Builder::session()
                .unwrap()
                .name("org.kde.StatusNotifierWatcher")
                .unwrap()
                .serve_at("/StatusNotifierWatcher", Watcher(registered.clone()))
                .unwrap()
                .build()
                .unwrap()
        };
        let server = serve();
        assert!(tray_host_ready().is_err());
        registered.store(true, Ordering::SeqCst);
        assert!(tray_host_ready().is_ok());
        registered.store(false, Ordering::SeqCst);
        assert!(tray_host_ready().is_err()); // Must not reuse cached true.
        server.close().unwrap();
        assert!(tray_host_ready().is_err());
        let _replacement = serve();
        registered.store(true, Ordering::SeqCst);
        assert!(tray_host_ready().is_ok());
    }
}
