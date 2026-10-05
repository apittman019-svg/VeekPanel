#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};
struct StartupFailure(String);
#[tauri::command]
fn snapshot(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let handle = app
        .try_state::<veek_runtime::Handle>()
        .ok_or_else(|| app.state::<StartupFailure>().0.clone())?;
    let state = handle.state();
    let mappings = state
        .audio
        .as_ref()
        .map(|audio| veek_core::statuses(&state.config, audio))
        .unwrap_or_default();
    Ok(serde_json::json!({"state":state,"mappings":mappings}))
}
#[tauri::command]
async fn action(app: tauri::AppHandle, request: veek_runtime::Command) -> Result<(), String> {
    let handle = app
        .try_state::<veek_runtime::Handle>()
        .ok_or_else(|| app.state::<StartupFailure>().0.clone())?
        .inner()
        .clone();
    tauri::async_runtime::spawn_blocking(move || handle.request(request))
        .await
        .map_err(|e| e.to_string())?
}
fn main() {
    let tray_ready = Arc::new(AtomicBool::new(false));
    let ready = tray_ready.clone();
    tauri::Builder::default()
        .setup(move |app| {
            let path = app.path().app_config_dir()?.join("config.json");
            let runtime = match veek_runtime::start(path) {
                Ok(r) => r,
                Err(e) => {
                    app.manage(StartupFailure(e));
                    return Ok(());
                }
            };
            app.manage(StartupFailure("Backend unavailable".into()));
            app.manage(runtime.handle.clone());
            app.manage(Mutex::new(runtime));
            let open = MenuItem::with_id(app, "open", "Open VeekPanel", true, None::<&str>)?;
            let next = MenuItem::with_id(app, "next", "Next profile", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &next, &quit])?;
            let mut pixels = vec![0u8; 32 * 32 * 4];
            for y in 0..32 {
                for x in 0..32 {
                    let d = (x as i32 - 16).pow(2) + (y as i32 - 16).pow(2);
                    let i = (y * 32 + x) * 4;
                    if d < 190 {
                        pixels[i..i + 4].copy_from_slice(&[124, 108, 244, 255]);
                    }
                    if d < 55 {
                        pixels[i..i + 4].copy_from_slice(&[239, 238, 255, 255]);
                    }
                }
            }
            let tray = TrayIconBuilder::with_id("veekpanel")
                .icon(tauri::image::Image::new_owned(pixels, 32, 32))
                .tooltip("VeekPanel")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "next" => {
                        let h = app.state::<veek_runtime::Handle>().inner().clone();
                        tauri::async_runtime::spawn_blocking(move || {
                            let s = h.state();
                            if let Some(i) = s
                                .config
                                .profiles
                                .iter()
                                .position(|p| p.id == s.config.active_profile)
                            {
                                let id = s.config.profiles[(i + 1) % s.config.profiles.len()]
                                    .id
                                    .clone();
                                let _ = h.request(veek_runtime::Command::Activate { id });
                            }
                        });
                    }
                    "quit" => app.exit(0),
                    _ => (),
                })
                .build(app);
            if tray.is_ok() {
                ready.store(true, Ordering::Relaxed);
            } else {
                eprintln!("Tray unavailable: closing the window will exit");
            }
            Ok(())
        })
        .on_window_event(move |window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let h = window.try_state::<veek_runtime::Handle>();
                if tray_ready.load(Ordering::Relaxed)
                    && h.is_some_and(|h| h.state().config.settings.close_to_tray)
                    && window.hide().is_ok()
                {
                    api.prevent_close();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![snapshot, action])
        .run(tauri::generate_context!())
        .expect("VeekPanel startup failed; configuration has not been reset");
}
