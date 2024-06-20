// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::{ClipboardManager, GlobalShortcutManager, Manager};

#[tauri::command]
async fn upload_file(file_data: String, file_type: String) -> Result<(), String> {
    // Your logic to upload the file based on its type
    match file_type.as_str() {
        "image" => {
            // print the file data
            println!("{}", file_data);
            // Logic to upload image
        }
        "text" => {
            // print the file data
            println!("{}", file_data);
            // Logic to upload text
        }
        _ => {
            return Err("Unsupported file type".into());
        }
    }
    Ok(())
}

#[tauri::command]
fn set_shortcut(app_handle: tauri::AppHandle, new_shortcut: String) -> Result<(), String> {
    let app_handle_clone = app_handle.clone();
    let mut shortcut_manager = app_handle.global_shortcut_manager();

    // Unregister previous shortcuts
    shortcut_manager
        .unregister_all()
        .map_err(|e| e.to_string())?;

    // Register new shortcut
    shortcut_manager
        .register(&new_shortcut, move || {
            let clipboard_content = app_handle_clone
                .clipboard_manager()
                .read_text()
                .unwrap()
                .unwrap_or_default();
            app_handle_clone
                .emit_all("hotkey-pressed", clipboard_content)
                .unwrap();
        })
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle();
            // Register initial shortcut
            let initial_shortcut = "Ctrl+U";
            set_shortcut(handle.clone(), initial_shortcut.to_string()).unwrap();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![upload_file, set_shortcut])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
