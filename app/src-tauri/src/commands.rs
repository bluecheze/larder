use std::collections::BTreeMap;
use std::path::PathBuf;

use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

use crate::client_runtime::{ClientRuntime, ClientStatus};
use crate::config::{self, AppConfig, ConfigStore, Defaults};
use crate::host_runtime::{DeviceView, HostRuntime, HostStatus};
use crate::pin::{pin, Pinned};
use crate::sync_runtime::{Activity, ShelfSync, SyncRuntime};

type Result<T> = std::result::Result<T, String>;

fn text(err: impl std::fmt::Display) -> String {
    err.to_string()
}

fn home(app: &AppHandle) -> Result<PathBuf> {
    app.path().home_dir().map_err(text)
}

fn stored(store: &ConfigStore) -> Result<AppConfig> {
    store.load().map_err(text)?.ok_or_else(|| "Larder isn't set up yet".to_string())
}

#[tauri::command]
pub fn load_config(store: State<'_, ConfigStore>) -> Result<Option<AppConfig>> {
    Ok(store.load().map_err(text)?.map(|c| c.for_ui()))
}

#[tauri::command]
pub async fn save_config(store: State<'_, ConfigStore>, host: State<'_, HostRuntime>, config: AppConfig) -> Result<()> {
    store.save(&config).map_err(text)?;
    host.apply(Some(&config)).await;
    Ok(())
}

#[tauri::command]
pub async fn save_settings(
    app: AppHandle,
    store: State<'_, ConfigStore>,
    host: State<'_, HostRuntime>,
    sync: State<'_, SyncRuntime>,
    edit: AppConfig,
) -> Result<AppConfig> {
    let before = stored(&store)?;
    let after = before.with_edit(edit);
    store.save(&after).map_err(text)?;
    // Restarting the server drops connections for a second, only do it when
    // something it cares about actually changed.
    if before.host != after.host || before.device_name != after.device_name {
        host.apply(Some(&after)).await;
    }
    sync.apply(Some(&after), &home(&app)?).await;
    Ok(after.for_ui())
}

#[tauri::command]
pub async fn join_host(
    app: AppHandle,
    store: State<'_, ConfigStore>,
    host: State<'_, HostRuntime>,
    client: State<'_, ClientRuntime>,
    sync: State<'_, SyncRuntime>,
    invite: String,
    device_name: String,
) -> Result<AppConfig> {
    let config = ClientRuntime::join(&invite, &device_name).await.map_err(text)?;
    store.save(&config).map_err(text)?;
    host.stop().await;
    client.apply(Some(&config)).await;
    sync.apply(Some(&config), &home(&app)?).await;
    Ok(config.for_ui())
}

#[tauri::command]
pub async fn reset_config(
    store: State<'_, ConfigStore>,
    host: State<'_, HostRuntime>,
    client: State<'_, ClientRuntime>,
    sync: State<'_, SyncRuntime>,
) -> Result<()> {
    host.stop().await;
    client.apply(None).await;
    sync.apply(None, std::path::Path::new("")).await;
    store.reset().map_err(text)
}

#[tauri::command]
pub fn defaults(app: AppHandle) -> Result<Defaults> {
    Ok(config::defaults(&home(&app)?))
}

#[tauri::command]
pub async fn host_status(host: State<'_, HostRuntime>) -> Result<HostStatus> {
    Ok(host.status().await)
}

#[tauri::command]
pub async fn create_invite(host: State<'_, HostRuntime>, address: String) -> Result<String> {
    host.create_invite(&address).await.map_err(text)
}

#[tauri::command]
pub async fn list_devices(host: State<'_, HostRuntime>) -> Result<Vec<DeviceView>> {
    Ok(host.devices().await)
}

#[tauri::command]
pub async fn revoke_device(host: State<'_, HostRuntime>, id: String) -> Result<bool> {
    host.revoke(&id).await.map_err(text)
}

#[tauri::command]
pub async fn host_shelves(host: State<'_, HostRuntime>) -> Result<Vec<String>> {
    host.shelves().await.map_err(text)
}

#[tauri::command]
pub async fn create_shelf(host: State<'_, HostRuntime>, name: String) -> Result<()> {
    host.create_shelf(&name).await.map_err(text)
}

#[tauri::command]
pub async fn client_status(client: State<'_, ClientRuntime>) -> Result<ClientStatus> {
    Ok(client.status().await)
}

#[tauri::command]
pub async fn set_synced(
    app: AppHandle,
    store: State<'_, ConfigStore>,
    sync: State<'_, SyncRuntime>,
    shelf: String,
    on: bool,
) -> Result<AppConfig> {
    let mut config = stored(&store)?;
    let client = config.client.as_mut().ok_or("this computer isn't a client")?;
    client.synced.retain(|s| s != &shelf);
    if on {
        client.synced.push(shelf);
        client.synced.sort();
    }
    store.save(&config).map_err(text)?;
    sync.apply(Some(&config), &home(&app)?).await;
    Ok(config.for_ui())
}

#[tauri::command]
pub async fn sync_now(sync: State<'_, SyncRuntime>, shelf: String) -> Result<()> {
    sync.sync_now(&shelf).await;
    Ok(())
}

#[tauri::command]
pub fn sync_status(sync: State<'_, SyncRuntime>) -> BTreeMap<String, ShelfSync> {
    sync.status()
}

#[tauri::command]
pub fn sync_activity(sync: State<'_, SyncRuntime>) -> Vec<Activity> {
    sync.activity()
}

#[tauri::command]
pub fn larder_folder(app: AppHandle, store: State<'_, ConfigStore>) -> Result<Option<String>> {
    let config = stored(&store)?;
    Ok(config.larder_folder(&home(&app)?).map(|p| p.to_string_lossy().into_owned()))
}

/// Opens the Larder folder, or one shelf inside it, in Finder or Explorer.
#[tauri::command]
pub fn open_folder(app: AppHandle, store: State<'_, ConfigStore>, shelf: Option<String>) -> Result<()> {
    let config = stored(&store)?;
    let mut folder = config.larder_folder(&home(&app)?).ok_or("this computer isn't a client")?;
    if let Some(shelf) = shelf {
        folder = folder.join(shelf);
    }
    std::fs::create_dir_all(&folder).map_err(text)?;
    app.opener().open_path(folder.to_string_lossy(), None::<&str>).map_err(text)
}

#[tauri::command]
pub fn pin_folder(app: AppHandle, store: State<'_, ConfigStore>) -> Result<Pinned> {
    let config = stored(&store)?;
    let folder = config.larder_folder(&home(&app)?).ok_or("this computer isn't a client")?;
    std::fs::create_dir_all(&folder).map_err(text)?;
    pin(&folder)
}
