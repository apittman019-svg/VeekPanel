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
        zbus::blocking::Proxy::new(
            &connection,
            "org.kde.StatusNotifierWatcher",
            "/StatusNotifierWatcher",
            "org.kde.StatusNotifierWatcher",
        )?
        .get_property("IsStatusNotifierHostRegistered")
    })();
    match observed {
        Ok(true) => Ok(()),
        Ok(false) => Err("No desktop tray host is registered".into()),
        Err(e) => Err(format!("Cannot confirm a desktop tray host: {e}")),
    }
}

#[cfg(not(target_os = "linux"))]
pub fn tray_host_ready() -> Result<(), String> {
    Ok(()) // Native tray creation is the Windows recovery-route check.
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
}
