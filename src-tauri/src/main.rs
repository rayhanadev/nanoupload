// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use arboard::{Clipboard, ImageData};
use reqwest::{multipart, Client};
use serde_json::json;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Mutex;
use tauri::api::notification::Notification;
use tauri::async_runtime::{self};
use tauri::{command, State};
use tauri::{
    ActivationPolicy, AppHandle, CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem,
};
use tauri::{GlobalShortcutManager, WindowEvent};
use url::Url;

struct AppState {
    upload_endpoint: Mutex<String>,
}

#[command]
fn get_upload_endpoint(state: State<'_, AppState>) -> String {
    state.upload_endpoint.lock().unwrap().clone()
}

#[command]
fn set_upload_endpoint(state: State<'_, AppState>, endpoint: String) {
    *state.upload_endpoint.lock().unwrap() = endpoint;
}

const API_ENDPOINT: &str = "https://nano.rayhanadev.com";

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
        .invoke_handler(tauri::generate_handler![
            get_upload_endpoint,
            set_upload_endpoint
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(ActivationPolicy::Accessory);

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
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "about" => {
                    // TODO: open browser to website
                }
                "check_for_updates" => {
                    // TODO: check for updates
                }
                "settings" => {
                    tauri::WindowBuilder::new(
                        app,
                        "settings",
                        tauri::WindowUrl::App("index.html".into()),
                    )
                    .build()
                    .unwrap();
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
    let client = Client::new();

    let clipboard_image_result = clipboard.get_image();

    if clipboard_image_result.is_ok() {
        let clipboard_image = clipboard_image_result.unwrap();

        let image_bytes = match image_to_png_bytes(&clipboard_image) {
            Ok(bytes) => bytes,
            Err(e) => {
                println!("Failed to convert image to PNG bytes: {:?}", e);
                return;
            }
        };

        let img_type = infer::get(&image_bytes);
        let ext = match img_type {
            Some(img_type) => ".".to_owned() + img_type.extension(),
            None => {
                println!("Failed to get image type");
                return;
            }
        };

        let form = multipart::Form::new()
            .text("type", "i")
            .text("ext", ext)
            .part(
                "file",
                multipart::Part::bytes(image_bytes).file_name("image"),
            );

        let response = client
            .post(API_ENDPOINT.to_owned() + "/upload")
            .multipart(form)
            .send()
            .await;

        match response {
            Ok(resp) => {
                let response_text = resp.text().await.unwrap();
                let json_response: serde_json::Value =
                    serde_json::from_str(&response_text).unwrap();
                let url = API_ENDPOINT.to_owned() + json_response["url"].as_str().unwrap();
                show_notification(
                    app,
                    "NanoUpload",
                    "Image uploaded successfully, the link has been copied to your clipboard.",
                );
                clipboard.set_text(url).unwrap();
            }
            Err(e) => println!("Failed to upload clipboard content: {:?}", e),
        }
    } else {
        let clipboard_text_result = clipboard.get_text();
        if clipboard_text_result.is_ok() {
            let clipboard_text = clipboard_text_result.unwrap();

            if is_url(&clipboard_text) {
                let payload = json!({
                    "payload": clipboard_text,
                    "type": "l",
                });

                let response = client
                    .post(API_ENDPOINT.to_owned() + "/create")
                    .body(payload.to_string())
                    .send()
                    .await;

                match response {
                    Ok(resp) => {
                        let response_text = resp.text().await.unwrap();
                        let json_response: serde_json::Value =
                            serde_json::from_str(&response_text).unwrap();
                        let url = API_ENDPOINT.to_owned() + json_response["url"].as_str().unwrap();
                        show_notification(app, "NanoUpload", "Shortlink created successfully, the link has been copied to your clipboard.");
                        clipboard.set_text(url).unwrap();
                    }
                    Err(e) => println!("Failed to upload clipboard content: {:?}", e),
                }
            } else if is_file_path(&clipboard_text) {
                if let Ok((file_bytes, ext)) = read_file(&clipboard_text) {
                    let form = multipart::Form::new()
                        .text("type", "f")
                        .text("ext", ext)
                        .part("file", multipart::Part::bytes(file_bytes).file_name("file"));

                    let response = client
                        .post(API_ENDPOINT.to_owned() + "/upload")
                        .multipart(form)
                        .send()
                        .await;

                    match response {
                        Ok(resp) => {
                            let response_text = resp.text().await.unwrap();
                            let json_response: serde_json::Value =
                                serde_json::from_str(&response_text).unwrap();
                            let url =
                                API_ENDPOINT.to_owned() + json_response["url"].as_str().unwrap();
                            show_notification(app, "NanoUpload", "File uploaded successfully, the link has been copied to your clipboard.");
                            clipboard.set_text(url).unwrap();
                        }
                        Err(e) => println!("Failed to upload file: {:?}", e),
                    }
                }
            } else if clipboard_text.len() > 0 {
                let payload = json!({
                    "payload": clipboard_text,
                    "type": "t",
                });

                let response = client
                    .post(API_ENDPOINT.to_owned() + "/create")
                    .body(payload.to_string())
                    .send()
                    .await;

                match response {
                    Ok(resp) => {
                        let response_text = resp.text().await.unwrap();
                        let json_response: serde_json::Value =
                            serde_json::from_str(&response_text).unwrap();
                        let url = API_ENDPOINT.to_owned() + json_response["url"].as_str().unwrap();
                        show_notification(app, "NanoUpload", "Text uploaded successfully, the link has been copied to your clipboard.");
                        clipboard.set_text(url).unwrap();
                    }
                    Err(e) => println!("Failed to upload clipboard content: {:?}", e),
                }
            } else {
                println!("clipboard is empty")
            }

            println!("clipboard text: {:?}", clipboard_text);
        }
    }
}

fn read_file(file_path: &str) -> Result<(Vec<u8>, String), std::io::Error> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let ext = Path::new(file_path)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("txt")
        .to_string();
    Ok((buffer, ext))
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

fn image_to_png_bytes(image: &ImageData) -> Result<Vec<u8>, png::EncodingError> {
    let width = image.width as u32;
    let height = image.height as u32;
    let buffer = &image.bytes;

    let mut cursor = std::io::Cursor::new(Vec::new());
    {
        let mut encoder = png::Encoder::new(&mut cursor, width, height);
        encoder.set_color(png::ColorType::Rgba);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&buffer)?;
    }

    Ok(cursor.into_inner())
}
