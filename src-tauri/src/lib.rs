mod app;
mod config;
mod daemon;
mod deskflow;
mod rdp;
mod shortcuts;
mod util;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, RunEvent, WindowEvent};

fn show_main(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // Never enables itself: only `autostart_set` from the UI toggle changes it.
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .setup(|app| {
            shortcuts::init(app.handle());
            // One app: start the Deskflow server core through its service if it is not running.
            tauri::async_runtime::spawn(deskflow::ensure_server_running());
            let open = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &quit])?;
            let mut tray = TrayIconBuilder::new()
                .tooltip("Desk Monitor")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "open" => show_main(app),
                    "quit" => crate::app::quit(app),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main(tray.app_handle());
                    }
                });
            // the crisp 2-screen mark; the window icon's 256 px master blurs when shrunk to tray size
            tray = tray.icon(tauri::include_image!("icons/32x32.png"));
            tray.build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            config::get_config,
            config::save_config,
            deskflow::deskflow_status,
            deskflow::deskflow_action,
            deskflow::login_server,
            deskflow::layout_get,
            deskflow::layout_save,
            rdp::rdp_status,
            rdp::rdp_connect,
            rdp::rdp_sessions,
            rdp::rdp_logoff,
            rdp::rdp_fix_paste,
            deskflow::hotkeys_get,
            deskflow::hotkeys_save,
            deskflow::hotkeys_default,
            shortcuts::hotkeys_status,
            shortcuts::displays_list,
            app::autostart_get,
            app::autostart_set,
            app::app_quit,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, event| {
            if let RunEvent::Exit = event {
                // Stop ssh/qwinsta/etc. still running; detached mstsc and .ps1 windows are untouched.
                util::kill_tracked();
            }
        });
}
