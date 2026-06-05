mod commands;
mod detect;
mod document;
mod encoding;
mod error;
mod file_io;

use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            document: std::sync::Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            commands::open_file,
            commands::decode_current_as,
            commands::save_current,
            commands::save_as,
            commands::get_supported_encodings,
        ])
        .run(tauri::generate_context!())
        .expect("启动 DecodeNote 失败");
}
