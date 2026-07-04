use serde::{Deserialize, Serialize};
use std::{fs, sync::{atomic::{AtomicBool, Ordering}, Mutex}};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder, WindowEvent,
};

#[derive(Clone, Serialize, Deserialize)]
struct Settings {
    minimize_to_tray: bool,
}

impl Default for Settings {
    fn default() -> Self { Self { minimize_to_tray: true } }
}

struct AppState {
    settings: Mutex<Settings>,
    quitting: AtomicBool,
}

fn settings_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path().app_config_dir().map(|p| p.join("settings.json")).map_err(|e| e.to_string())
}

fn load_settings(app: &AppHandle) -> Settings {
    settings_path(app).ok()
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_settings(app: &AppHandle, value: &Settings) -> Result<(), String> {
    let path = settings_path(app)?;
    if let Some(parent) = path.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
    fs::write(path, serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_settings(state: State<'_, AppState>) -> Settings {
    state.settings.lock().expect("settings lock poisoned").clone()
}

#[tauri::command]
fn set_minimize_to_tray(app: AppHandle, state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    settings.minimize_to_tray = enabled;
    save_settings(&app, &settings)
}

fn clear_webview(app: &AppHandle) -> Result<(), String> {
    let window = app.get_webview_window("main").ok_or("main window not found")?;
    window.clear_all_browsing_data().map_err(|e| e.to_string())?;
    window.eval("location.reload()").map_err(|e| e.to_string())
}

#[tauri::command]
fn clear_session(app: AppHandle) -> Result<(), String> { clear_webview(&app) }

#[tauri::command]
fn shortcut_action(app: AppHandle, state: State<'_, AppState>, action: &str) -> Result<(), String> {
    match action {
        "reload" => app.get_webview_window("main").ok_or("main window not found")?.eval("location.reload()").map_err(|e| e.to_string()),
        "logout" => clear_webview(&app),
        "quit" => { state.quitting.store(true, Ordering::SeqCst); app.exit(0); Ok(()) },
        _ => Err("unknown action".into()),
    }
}

fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn tray_image() -> Image<'static> {
    let mut rgba = vec![0u8; 32 * 32 * 4];
    for y in 0..32 { for x in 0..32 {
        let i = (y * 32 + x) * 4;
        let inside = (x as i32 - 16).pow(2) + (y as i32 - 15).pow(2) < 13_i32.pow(2);
        if inside { rgba[i..i + 4].copy_from_slice(&[37, 211, 102, 255]); }
    }}
    Image::new_owned(rgba, 32, 32)
}

const INIT_SCRIPT: &str = r#"
(() => {
  document.addEventListener('keydown', (event) => {
    if (!event.ctrlKey) return;
    let action = null;
    if (event.key.toLowerCase() === 'r') action = 'reload';
    if (event.key.toLowerCase() === 'q') action = 'quit';
    if (event.shiftKey && event.key.toLowerCase() === 'l') action = 'logout';
    if (action) {
      event.preventDefault();
      window.__TAURI_INTERNALS__?.invoke('shortcut_action', { action });
    }
  }, true);
})();
"#;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .invoke_handler(tauri::generate_handler![get_settings, set_minimize_to_tray, clear_session, shortcut_action])
        .setup(|app| {
            let initial = load_settings(app.handle());
            app.manage(AppState { settings: Mutex::new(initial), quitting: AtomicBool::new(false) });

            WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::External("https://web.whatsapp.com/".parse().expect("valid WhatsApp URL")),
            )
            .title("WhatsUp Lite")
            .inner_size(1200.0, 800.0)
            .min_inner_size(720.0, 520.0)
            .initialization_script(INIT_SCRIPT)
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            .build()?;

            let open = MenuItem::with_id(app, "open", "Open WhatsApp", true, None::<&str>)?;
            let reload = MenuItem::with_id(app, "reload", "Reload", true, None::<&str>)?;
            let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let logout = MenuItem::with_id(app, "logout", "Logout / Clear Session", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &reload, &settings, &logout, &quit])?;
            TrayIconBuilder::new()
                .icon(tray_image())
                .menu(&menu)
                .tooltip("WhatsUp Lite")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => show_main(app),
                    "reload" => { let _ = app.get_webview_window("main").map(|w| w.eval("location.reload()")); },
                    "settings" => { if let Some(w) = app.get_webview_window("settings") { let _ = w.show(); let _ = w.set_focus(); } },
                    "logout" => { let _ = clear_webview(app); },
                    "quit" => { app.state::<AppState>().quitting.store(true, Ordering::SeqCst); app.exit(0); },
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                        show_main(tray.app_handle());
                    }
                })
                .build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<AppState>();
                if !state.quitting.load(Ordering::SeqCst) {
                    if window.label() == "settings" || state.settings.lock().map(|s| s.minimize_to_tray).unwrap_or(true) {
                        api.prevent_close();
                        let _ = window.hide();
                    } else if window.label() == "main" {
                        state.quitting.store(true, Ordering::SeqCst);
                        window.app_handle().exit(0);
                    }
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("failed to build WhatsUp Lite");

    app.run(|_, _| {});
}
