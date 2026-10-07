#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod lifecycle;
mod startup;

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
struct Desktop {
    tray_ready: AtomicBool,
    reopen_requested: AtomicBool,
    tray_error: Mutex<Option<String>>,
    // Serialize registration reads/writes; all OS I/O runs off the UI thread.
    startup: Mutex<Result<startup::Registration, String>>,
}

fn reveal(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command]
async fn snapshot(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let handle = app
        .try_state::<veek_runtime::Handle>()
        .ok_or_else(|| app.state::<StartupFailure>().0.clone())?;
    let state = handle.state();
    let desktop = app.state::<Arc<Desktop>>().inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mappings = state
            .audio
            .as_ref()
            .map(|audio| veek_core::statuses(&state.config, audio))
            .unwrap_or_default();
        let registered = desktop
            .startup
            .lock()
            .map_err(|e| e.to_string())?
            .as_ref()
            .map_err(Clone::clone)
            .and_then(|r| r.registered());
        let startup_error = registered.as_ref().err().cloned();
        let startup_registered = registered.ok();
        let tray_error = desktop
            .tray_error
            .lock()
            .map_err(|e| e.to_string())?
            .clone();
        Ok(
            serde_json::json!({"state":state,"mappings":mappings,"desktop":{
                "tray_ready":desktop.tray_ready.load(Ordering::SeqCst),
                "tray_error":tray_error,"startup_supported":cfg!(any(windows, target_os="linux")),
                "startup_registered":startup_registered,"startup_error":startup_error
            }}),
        )
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn set_login_startup(app: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let desktop = app.state::<Arc<Desktop>>().inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        desktop
            .startup
            .lock()
            .map_err(|e| e.to_string())?
            .as_ref()
            .map_err(Clone::clone)?
            .set_registered(enabled)
    })
    .await
    .map_err(|e| e.to_string())?
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

fn create_tray(app: &tauri::App) -> tauri::Result<()> {
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
    TrayIconBuilder::with_id("veekpanel")
        .icon(tauri::image::Image::new_owned(pixels, 32, 32))
        .tooltip("VeekPanel")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => reveal(app),
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
        .build(app)?;
    Ok(())
}

fn main() {
    let desktop = Arc::new(Desktop {
        tray_ready: AtomicBool::new(false),
        reopen_requested: AtomicBool::new(false),
        tray_error: Mutex::new(None),
        startup: Mutex::new(Err("Login startup has not initialized".into())),
    });
    let instance_state = desktop.clone();
    let setup_state = desktop.clone();
    let close_state = desktop.clone();
    tauri::Builder::default()
        // First plugin: duplicate processes exit before opening hardware/config.
        .plugin(tauri_plugin_single_instance::init(move |app, args, _cwd| {
            if lifecycle::reveal_second_launch(&args) {
                instance_state.reopen_requested.store(true, Ordering::SeqCst);
                reveal(app);
            }
        }))
        .manage(desktop)
        .setup(move |app| {
            let executable = std::env::current_exe().ok();
            #[cfg(target_os = "linux")]
            let executable = app.env().appimage.map(std::path::PathBuf::from).or(executable);
            *setup_state.startup.lock().expect("startup mutex") = executable
                .ok_or_else(|| "Cannot locate this executable for login startup".to_string())
                .and_then(|exe| app.path().config_dir().map_err(|e| e.to_string())
                    .and_then(|dir| startup::Registration::new(&exe, &dir)));
            let path = app.path().app_config_dir()?.join("config.json");
            let runtime = match veek_runtime::start(path) {
                Ok(r) => r,
                Err(e) => {
                    app.manage(StartupFailure(e));
                    reveal(app.handle()); // Bad config/lock must never strand a hidden app.
                    return Ok(());
                }
            };
            let minimized = runtime.handle.state().config.settings.start_minimized;
            app.manage(StartupFailure("Backend unavailable".into()));
            app.manage(runtime.handle.clone());
            app.manage(Mutex::new(Some(runtime)));
            // Normal launches appear immediately; tray-host I/O stays off-thread.
            if !minimized { reveal(app.handle()); }
            match create_tray(app) {
                Ok(()) => {
                    let state = setup_state.clone();
                    let handle = app.handle().clone();
                    tauri::async_runtime::spawn_blocking(move || {
                        match lifecycle::tray_host_ready() {
                            Ok(()) => state.tray_ready.store(true, Ordering::SeqCst),
                            Err(e) => {
                                *state.tray_error.lock().expect("tray mutex") = Some(format!(
                                    "{e}. Start in tray and close-to-tray are disabled for this session."
                                ));
                            }
                        }
                        if !lifecycle::start_hidden(minimized, true,
                            state.tray_ready.load(Ordering::SeqCst),
                            state.reopen_requested.load(Ordering::SeqCst)) {
                            reveal(&handle);
                        }
                    });
                }
                Err(e) => {
                    *setup_state.tray_error.lock().expect("tray mutex") = Some(format!(
                        "Tray unavailable: {e}. Start in tray and close-to-tray are disabled for this session."
                    ));
                    reveal(app.handle());
                }
            }
            Ok(())
        })
        .on_window_event(move |window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let h = window.try_state::<veek_runtime::Handle>();
                if close_state.tray_ready.load(Ordering::SeqCst)
                    && h.is_some_and(|h| h.state().config.settings.close_to_tray)
                    && window.hide().is_ok()
                { api.prevent_close(); }
            }
        })
        .invoke_handler(tauri::generate_handler![snapshot, action, set_login_startup])
        .build(tauri::generate_context!())
        .expect("VeekPanel startup failed; configuration has not been reset")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                // Join owner thread before process exit: release config/hardware/audio.
                if let Some(runtime) = app.try_state::<Mutex<Option<veek_runtime::Runtime>>>() {
                    if let Ok(mut owner) = runtime.lock() { drop(owner.take()); }
                }
            }
        });
}
