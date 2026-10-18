//! App-level commands: start with Windows (autostart) and a real Quit.
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

/// True when desk-monitor is registered to start with Windows (HKCU Run key).
#[tauri::command]
pub async fn autostart_get(app: AppHandle) -> Result<bool, String> {
    app.autolaunch()
        .is_enabled()
        .map_err(|e| format!("can't read autostart: {e}"))
}

/// Turn start-with-Windows on or off. Only ever called from the UI toggle.
#[tauri::command]
pub async fn autostart_set(app: AppHandle, enabled: bool) -> Result<(), String> {
    let m = app.autolaunch();
    let r = if enabled { m.enable() } else { m.disable() };
    r.map_err(|e| {
        format!(
            "can't {} autostart: {e}",
            if enabled { "enable" } else { "disable" }
        )
    })
}

/// End the whole process (the window close button only hides to the tray).
#[tauri::command]
pub async fn app_quit(app: AppHandle) -> Result<(), String> {
    quit(&app);
    Ok(())
}

/// Shared by `app_quit` and the tray "Quit" item. Children we track are stopped in the
/// `RunEvent::Exit` handler in lib.rs; mstsc and the .ps1 windows are detached and stay.
pub fn quit(app: &AppHandle) {
    app.exit(0);
}
