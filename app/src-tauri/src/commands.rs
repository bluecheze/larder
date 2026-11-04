use tauri::{AppHandle, Manager, State};

use crate::config::{self, AppConfig, ConfigStore, Defaults};
use crate::host_runtime::{DeviceView, HostRuntime, HostStatus};

fn text(err: impl std::fmt::Display) -> String {
    err.to_string()
}

#[tauri::command]
pub fn load_config(store: State<'_, ConfigStore>) -> Result<Option<AppConfig>, String> {
    store.load().map_err(text)
}

#[tauri::command]
pub async fn save_config(
    store: State<'_, ConfigStore>,
    host: State<'_, HostRuntime>,
    config: AppConfig,
) -> Result<(), String> {
    store.save(&config).map_err(text)?;
    host.apply(Some(&config)).await;
    Ok(())
}

#[tauri::command]
pub async fn reset_config(store: State<'_, ConfigStore>, host: State<'_, HostRuntime>) -> Result<(), String> {
    host.stop().await;
    store.reset().map_err(text)
}

#[tauri::command]
pub fn defaults(app: AppHandle) -> Result<Defaults, String> {
    let home = app.path().home_dir().map_err(text)?;
    Ok(config::defaults(&home))
}

#[tauri::command]
pub async fn host_status(host: State<'_, HostRuntime>) -> Result<HostStatus, String> {
    Ok(host.status().await)
}

#[tauri::command]
pub async fn create_invite(host: State<'_, HostRuntime>, address: String) -> Result<String, String> {
    host.create_invite(&address).await.map_err(text)
}

#[tauri::command]
pub async fn list_devices(host: State<'_, HostRuntime>) -> Result<Vec<DeviceView>, String> {
    Ok(host.devices().await)
}

#[tauri::command]
pub async fn revoke_device(host: State<'_, HostRuntime>, id: String) -> Result<bool, String> {
    host.revoke(&id).await.map_err(text)
}
