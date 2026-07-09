mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::load_character_sheet])
        .run(tauri::generate_context!())
        .expect("error while running RPG Engine desktop application");
}
