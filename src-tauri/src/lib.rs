mod commands;
mod imaging;
mod inference;
mod model;

use inference::face_detect::FaceDetectState;
use inference::refine::RefineState;
use inference::session::SessionState;
use inference::upscale::UpscaleSessionState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(SessionState::new())
        .manage(UpscaleSessionState::new())
        .manage(FaceDetectState::new())
        .manage(RefineState::new())
        .invoke_handler(tauri::generate_handler![
            commands::apple_vision_available,
            commands::remove_background_apple_vision,
            commands::check_model_ready,
            commands::is_onboarding_done,
            commands::complete_onboarding,
            commands::get_model_info,
            commands::open_path_in_finder,
            commands::open_url_in_browser,
            commands::get_output_dir,
            commands::set_output_dir,
            commands::set_model_dir,
            commands::set_model_variant,
            commands::delete_model,
            commands::download_model,
            commands::remove_background,
            commands::remove_background_batch,
            commands::replace_background_color,
            commands::replace_background_gradient,
            commands::replace_background_image,
            commands::auto_crop,
            commands::get_upscale_model_info,
            commands::download_upscale_model,
            commands::upscale_image,
            commands::save_image,
            commands::get_auto_routing,
            commands::set_auto_routing,
            commands::get_refine_model_info,
            commands::download_refine_model,
            commands::refine_result,
            commands::get_cloud_config,
            commands::set_cloud_enabled,
            commands::set_cloud_provider,
            commands::set_cloud_api_key,
            commands::remove_background_cloud,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
