//! Machine-local, per-user registration. Never called by config load/import.
//! No shell execution, elevation, HKLM writes or implicit startup registration.
#![forbid(unsafe_code)]

use std::path::Path;

#[cfg(target_os = "linux")]
const HEADER: &str = "[Desktop Entry]\nX-VeekPanel-Managed=true\n";

pub struct Registration {
    #[cfg(target_os = "linux")]
    path: std::path::PathBuf,
    #[cfg(target_os = "linux")]
    entry: String,
    #[cfg(windows)]
    command: String,
}

impl Registration {
    pub fn new(executable: &Path, config_dir: &Path) -> Result<Self, String> {
        #[cfg(target_os = "linux")]
        {
            Ok(Self {
                path: config_dir.join("autostart/org.veekpanel.desktop.desktop"),
                entry: desktop_entry(executable)?,
            })
        }
        #[cfg(windows)]
        {
            let _ = config_dir;
            Ok(Self {
                command: windows_command(executable)?,
            })
        }
        #[cfg(not(any(windows, target_os = "linux")))]
        {
            let _ = (executable, config_dir);
            Err("Login startup is supported on Windows and Linux only".into())
        }
    }

    pub fn registered(&self) -> Result<bool, String> {
        #[cfg(target_os = "linux")]
        {
            match std::fs::read_to_string(&self.path) {
                Ok(entry) if entry == self.entry => Ok(true),
                Ok(_) => Err("Login entry was changed or points to another install. Enable to repair VeekPanel's entry, or disable to remove it.".into()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
                Err(e) => Err(e.to_string()),
            }
        }
        #[cfg(windows)]
        {
            match registry_value() {
                Ok(Some(command)) if command == self.command => Ok(true),
                Ok(Some(_)) => Err("Login entry points to another install. Enable to update VeekPanel's entry, or disable to remove it.".into()),
                Ok(None) => Ok(false),
                Err(e) => Err(e.to_string()),
            }
        }
        #[cfg(not(any(windows, target_os = "linux")))]
        Err("Login startup is unsupported on this platform".into())
    }

    pub fn set_registered(&self, enabled: bool) -> Result<(), String> {
        self.write(enabled)?;
        verify_registration(enabled, self.registered())
    }

    fn write(&self, enabled: bool) -> Result<(), String> {
        #[cfg(target_os = "linux")]
        {
            use std::io::Write;
            // Refuse to overwrite/remove a foreign file at our reserved name.
            match std::fs::read_to_string(&self.path) {
                Ok(entry) if !entry.starts_with(HEADER) => {
                    return Err("Unrecognized login entry; review it in your desktop's startup settings before changing it".into());
                }
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                    return Err(e.to_string());
                }
                _ => (),
            }
            if enabled {
                let parent = self.path.parent().expect("autostart path has a parent");
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                let mut temp =
                    tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
                temp.write_all(self.entry.as_bytes())
                    .map_err(|e| e.to_string())?;
                temp.as_file().sync_all().map_err(|e| e.to_string())?;
                temp.persist(&self.path).map_err(|e| e.to_string())?;
            } else {
                match std::fs::remove_file(&self.path) {
                    Ok(()) => (),
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
                    Err(e) => return Err(e.to_string()),
                }
            }
            Ok(())
        }
        #[cfg(windows)]
        {
            use windows_registry::CURRENT_USER;
            if enabled {
                CURRENT_USER
                    .create(RUN_KEY)
                    .and_then(|key| key.set_string(VALUE_NAME, &self.command))
                    .map_err(|e| e.to_string())
            } else {
                // Opening a missing key for disable must not create it.
                match CURRENT_USER.options().write().open(RUN_KEY) {
                    Ok(key) => match key.remove_value(VALUE_NAME) {
                        Ok(()) => Ok(()),
                        Err(e) if missing_registry_error(e.code().0) => Ok(()),
                        Err(e) => Err(e.to_string()),
                    },
                    Err(e) if missing_registry_error(e.code().0) => Ok(()),
                    Err(e) => Err(e.to_string()),
                }
            }
        }
        #[cfg(not(any(windows, target_os = "linux")))]
        {
            let _ = enabled;
            Err("Login startup is unsupported on this platform".into())
        }
    }
}

fn verify_registration(expected: bool, observed: Result<bool, String>) -> Result<(), String> {
    match observed {
        Ok(value) if value == expected => Ok(()),
        Ok(_) => Err("Login startup did not match the requested state after writing; inspect your OS startup settings".into()),
        Err(e) => Err(format!("Login startup may have changed, but readback failed: {e}")),
    }
}

#[cfg(any(windows, test))]
fn windows_command(path: &Path) -> Result<String, String> {
    let path = path
        .to_str()
        .ok_or("Executable path is not valid Unicode")?;
    if path.is_empty() || path.contains(['"', '\r', '\n', '\0']) {
        return Err("Executable path cannot be represented safely for login startup".into());
    }
    let command = format!("\"{path}\" --autostart");
    if command.encode_utf16().count() > 260 {
        return Err("Executable path exceeds the Windows Run command limit".into());
    }
    Ok(command)
}

#[cfg(target_os = "linux")]
fn desktop_entry(path: &Path) -> Result<String, String> {
    let path = path.to_str().ok_or("Executable path is not valid UTF-8")?;
    // Percent in a quoted Exec argument is undefined by the desktop-entry spec;
    // reject it rather than inventing a field-code expansion. No shell involved.
    if path.is_empty() || path.contains(['=', '%', '\r', '\n', '\t', '\0']) {
        return Err("Executable path cannot be represented safely in an XDG login entry".into());
    }
    let mut quoted = String::new();
    for ch in path.chars() {
        match ch {
            '\\' => quoted.push_str("\\\\\\\\"),
            '"' | '$' | '`' => {
                quoted.push_str("\\\\");
                quoted.push(ch);
            }
            _ => quoted.push(ch),
        }
    }
    Ok(format!(
        "{HEADER}Type=Application\nName=VeekPanel\nExec=\"{quoted}\" --autostart\nTerminal=false\n"
    ))
}

#[cfg(windows)]
const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
#[cfg(windows)]
const VALUE_NAME: &str = "org.veekpanel.desktop";

#[cfg(windows)]
fn missing_registry_error(code: i32) -> bool {
    // HRESULT_FROM_WIN32(ERROR_FILE_NOT_FOUND / ERROR_PATH_NOT_FOUND).
    matches!(code as u32, 0x80070002 | 0x80070003)
}

#[cfg(windows)]
fn registry_value() -> windows_registry::Result<Option<String>> {
    match windows_registry::CURRENT_USER.open(RUN_KEY) {
        Ok(key) => match key.get_string(VALUE_NAME) {
            Ok(value) => Ok(Some(value)),
            Err(e) if missing_registry_error(e.code().0) => Ok(None),
            Err(e) => Err(e),
        },
        Err(e) if missing_registry_error(e.code().0) => Ok(None),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_command_quotes_paths_and_rejects_unsafe_or_long_values() {
        assert_eq!(
            windows_command(Path::new(r"C:\Program Files\VeekPanel\veekpanel.exe")).unwrap(),
            r#""C:\Program Files\VeekPanel\veekpanel.exe" --autostart"#
        );
        for path in ["", "bad\npath", "bad\"path", &"x".repeat(260)] {
            assert!(windows_command(Path::new(path)).is_err());
        }
    }

    #[test]
    fn readback_never_reports_unconfirmed_success() {
        for enabled in [true, false] {
            assert!(verify_registration(enabled, Ok(enabled)).is_ok());
            assert!(verify_registration(enabled, Ok(!enabled)).is_err());
            assert!(
                verify_registration(enabled, Err("permission denied".into()))
                    .unwrap_err()
                    .contains("may have changed")
            );
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn xdg_registration_is_opt_in_idempotent_and_scoped_to_config_dir() {
        let folder = tempfile::tempdir().unwrap();
        let registration =
            Registration::new(Path::new("/opt/Veek Panel/app"), folder.path()).unwrap();
        assert!(!registration.registered().unwrap());
        assert!(!registration.path.exists()); // Construct/read never writes.
        registration.set_registered(false).unwrap();
        assert!(!registration.path.exists());
        for _ in 0..2 {
            registration.set_registered(true).unwrap();
            assert!(registration.registered().unwrap());
            assert!(std::fs::read_to_string(&registration.path)
                .unwrap()
                .contains("Exec=\"/opt/Veek Panel/app\" --autostart"));
        }
        std::fs::write(&registration.path, format!("{HEADER}Hidden=true\n")).unwrap();
        assert!(registration.registered().is_err());
        registration.set_registered(true).unwrap(); // Explicit repair of our own entry.
        registration.set_registered(false).unwrap();
        registration.set_registered(false).unwrap();
        assert!(!registration.registered().unwrap());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn foreign_entries_and_io_failures_are_not_overwritten() {
        let folder = tempfile::tempdir().unwrap();
        let registration = Registration::new(Path::new("/opt/veekpanel"), folder.path()).unwrap();
        std::fs::create_dir_all(registration.path.parent().unwrap()).unwrap();
        std::fs::write(&registration.path, "foreign desktop entry").unwrap();
        for enabled in [true, false] {
            assert!(registration.set_registered(enabled).is_err());
        }
        assert_eq!(
            std::fs::read_to_string(&registration.path).unwrap(),
            "foreign desktop entry"
        );
        let blocked = folder.path().join("file-not-directory");
        std::fs::write(&blocked, "blocked").unwrap();
        let registration = Registration::new(Path::new("/opt/veekpanel"), &blocked).unwrap();
        assert!(registration.registered().is_err());
        assert!(registration.set_registered(true).is_err());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn desktop_exec_uses_both_escape_layers_and_rejects_field_codes() {
        let entry = desktop_entry(Path::new("/opt/a b/$`\"\\app")).unwrap();
        assert!(entry.contains(r#"Exec="/opt/a b/\\$\\`\\"\\\\app" --autostart"#));
        for path in ["/opt/a%f", "/opt/a=b", "/opt/a\nName=bad"] {
            assert!(desktop_entry(Path::new(path)).is_err());
        }
    }
}
