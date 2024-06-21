// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use arboard::Clipboard;
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
                // attempt to get an image from the clipboard and if that is not possible attempt to get text
                let mut clipboard = Clipboard::new().unwrap();

                let clipboard_image_result = clipboard.get_image();

                if clipboard_image_result.is_ok() {
                    let _clipboard_image = clipboard_image_result.unwrap();
                    println!("clipboard content is an image");
                } else {
                    let clipboard_text_result = clipboard.get_text();
                    if clipboard_text_result.is_ok() {
                        let clipboard_text = clipboard_text_result.unwrap();

                        if is_url(&clipboard_text) {
                            println!("clipboard content is a url")
                        }
                        
                        else if is_file_path(&clipboard_text) {
                            println!("clipboard content is a file path")
                        }

                        else if clipboard_text.len() > 0 {
                            println!("clipboard content is text")
                        }

                        else {
                            println!("clipboard is empty")
                        }

                        println!("clipboard text: {:?}", clipboard_text);
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

fn is_url(content: &str) -> bool {
    Url::parse(content).is_ok()
}

fn is_file_path(content: &str) -> bool {
    Path::new(content).exists()
}
