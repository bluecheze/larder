use tauri::{AppHandle, Manager, State};

use crate::config::{self, AppConfig, ConfigStore, Defaults};

fn text(err: impl std::fmt::Display) -> String {
    err.to_string()
}

#[tauri::command]
pub fn load_config(store: State<ConfigStore>) -> Result<Option<AppConfig>, String> {
    store.load().map_err(text)
}

#[tauri::command]
pub fn save_config(store: State<ConfigStore>, config: AppConfig) -> Result<(), String> {
    store.save(&config).map_err(text)
}

#[tauri::command]
pub fn reset_config(store: State<ConfigStore>) -> Result<(), String> {
    store.reset().map_err(text)
}

#[tauri::command]
pub fn defaults(app: AppHandle) -> Result<Defaults, String> {
    let home = app.path().home_dir().map_err(text)?;
    Ok(config::defaults(&home))
}
