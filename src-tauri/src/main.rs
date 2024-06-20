// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use copypasta::{ClipboardContext, ClipboardProvider};
use std::path::Path;
use tauri::GlobalShortcutManager;
use tauri::{CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};
use url::Url;

fn main() {
    let about = CustomMenuItem::new("about".to_string(), "About");
    let check_for_updates =
        CustomMenuItem::new("check_for_updates".to_string(), "Check for Updates");
    let settings = CustomMenuItem::new("settings".to_string(), "Settings");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let tray_menu = SystemTrayMenu::new()
        .add_item(about)
        .add_item(check_for_updates)
        .add_item(settings)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    let system_tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .setup(|app| {
            if let Err(e) = app.global_shortcut_manager().register("Cmd+Shift+U", || {
                let mut ctx = ClipboardContext::new().unwrap();
                let clipboard_content = ctx.get_contents().unwrap_or_else(|_| String::new());

                if clipboard_content.is_empty() {
                    println!("Clipboard is empty.");
                } else {
                    println!("Clipboard content: {}", clipboard_content);

                    if is_image(&clipboard_content) {
                        println!("Clipboard contains an image.");
                    } else if is_url(&clipboard_content) {
                        println!("Clipboard contains a URL.");
                    } else if is_file_path(&clipboard_content) {
                        println!("Clipboard contains a file path.");
                    } else {
                        println!("Clipboard contains text.");
                    }
                }
            }) {
                println!("global shortcut register failed: {}", e);
            };

            Ok(())
        })
        .system_tray(system_tray)
        .on_system_tray_event(|_app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "about" => {
                    // TODO: open browser to website
                }
                "check_for_updates" => {
                    // TODO: check for updates
                }
                "settings" => {
                    // TODO: open settings
                }
                "quit" => {
                    std::process::exit(0);
                }
                _ => {}
            },
            _ => {}
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
}

fn is_image(content: &str) -> bool {
    // TODO: research this check
    false
}

fn is_url(content: &str) -> bool {
    Url::parse(content).is_ok()
}

fn is_file_path(content: &str) -> bool {
    Path::new(content).exists()
}
