mod commands;
mod config;

use tauri::Manager;

use crate::config::ConfigStore;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // LARDER_CONFIG_DIR lets you run a host and a client on the same
            // machine while testing. Without it both would share one config.
            let dir = match std::env::var_os("LARDER_CONFIG_DIR") {
                Some(dir) => dir.into(),
                None => app.path().app_config_dir()?,
            };
            app.manage(ConfigStore::new(&dir));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::load_config,
            commands::save_config,
            commands::reset_config,
            commands::defaults,
        ])
        .run(tauri::generate_context!())
        .expect("couldn't start Larder");
}
