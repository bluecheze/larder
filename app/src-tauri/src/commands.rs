use tauri::{AppHandle, Manager, State};

use crate::client_runtime::{ClientRuntime, ClientStatus};
use crate::config::{self, AppConfig, ConfigStore, Defaults};
use crate::host_runtime::{DeviceView, HostRuntime, HostStatus};

fn text(err: impl std::fmt::Display) -> String {
    err.to_string()
}

#[tauri::command]
pub fn load_config(store: State<'_, ConfigStore>) -> Result<Option<AppConfig>, String> {
    Ok(store.load().map_err(text)?.map(|c| c.for_ui()))
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
pub async fn join_host(
    store: State<'_, ConfigStore>,
    host: State<'_, HostRuntime>,
    client: State<'_, ClientRuntime>,
    invite: String,
    device_name: String,
) -> Result<AppConfig, String> {
    let config = ClientRuntime::join(&invite, &device_name).await.map_err(text)?;
    store.save(&config).map_err(text)?;
    host.stop().await;
    client.apply(Some(&config)).await;
    Ok(config.for_ui())
}

#[tauri::command]
pub async fn reset_config(
    store: State<'_, ConfigStore>,
    host: State<'_, HostRuntime>,
    client: State<'_, ClientRuntime>,
) -> Result<(), String> {
    host.stop().await;
    client.apply(None).await;
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

#[tauri::command]
pub async fn host_shelves(host: State<'_, HostRuntime>) -> Result<Vec<String>, String> {
    host.shelves().await.map_err(text)
}

#[tauri::command]
pub async fn create_shelf(host: State<'_, HostRuntime>, name: String) -> Result<(), String> {
    host.create_shelf(&name).await.map_err(text)
}

#[tauri::command]
pub async fn client_status(client: State<'_, ClientRuntime>) -> Result<ClientStatus, String> {
    Ok(client.status().await)
}
