mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::advance_combat_turn,
            commands::create_homebrew_monster,
            commands::load_basic_map,
            commands::load_bestiary,
            commands::load_character_sheet,
            commands::move_map_token,
            commands::roll_formula,
            commands::start_basic_combat
        ])
        .run(tauri::generate_context!())
        .expect("error while running RPG Engine desktop application");
}
