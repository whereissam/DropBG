mod commands;
mod imaging;
mod inference;
mod model;

use inference::session::SessionState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(SessionState::new())
        .invoke_handler(tauri::generate_handler![
            commands::check_model_ready,
            commands::get_model_info,
            commands::open_path_in_finder,
            commands::get_output_dir,
            commands::set_output_dir,
            commands::set_model_dir,
            commands::delete_model,
            commands::download_model,
            commands::remove_background,
            commands::save_image,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
