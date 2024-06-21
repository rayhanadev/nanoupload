// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use arboard::Clipboard;
use std::path::Path;
use tauri::api::notification::Notification;
use tauri::{AppHandle, CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};
use tauri::{GlobalShortcutManager, WindowEvent};
use tauri::async_runtime::{self};
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
            let (tx, mut rx) = async_runtime::channel::<()>(1);
            
            let handle = app.handle().clone();

            async_runtime::spawn(async move {
                while rx.recv().await.is_some() {
                    handle_clipboard(&handle).await;
                }
            });

            let mut global_shortcut_manager = app.global_shortcut_manager();
            global_shortcut_manager
                .register("Cmd+Shift+U", move || {
                    let _ = tx.try_send(());
                })
                .expect("Failed to register global shortcut");

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
        .on_window_event(|event| {
            if let WindowEvent::CloseRequested { api, .. } = event.event() {
                let window = event.window().clone();
                api.prevent_close();
                window.hide().unwrap();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn handle_clipboard(app: &AppHandle) {
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
            } else if is_file_path(&clipboard_text) {
                println!("clipboard content is a file path")
            } else if clipboard_text.len() > 0 {
                println!("clipboard content is text")
            } else {
                println!("clipboard is empty")
            }

            println!("clipboard text: {:?}", clipboard_text);
        }
    }

    show_notification(app, "Clipboard", "Clipboard content has been processed");
}

fn is_url(content: &str) -> bool {
    Url::parse(content).is_ok()
}

fn is_file_path(content: &str) -> bool {
    Path::new(content).exists()
}

fn show_notification(app: &AppHandle, title: &str, message: &str) {
    Notification::new(&app.config().tauri.bundle.identifier)
        .title(title)
        .body(message)
        .show()
        .unwrap();
}
