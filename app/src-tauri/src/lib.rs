mod client_runtime;
mod commands;
mod config;
mod host_runtime;
mod network;
mod pin;
mod sync_runtime;

use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;

use crate::client_runtime::ClientRuntime;
use crate::config::ConfigStore;
use crate::host_runtime::HostRuntime;
use crate::sync_runtime::SyncRuntime;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .setup(|app| {
            // LARDER_CONFIG_DIR lets you run a host and a client on the same
            // machine while testing. Without it both would share one config.
            let dir = match std::env::var_os("LARDER_CONFIG_DIR") {
                Some(dir) => dir.into(),
                None => app.path().app_config_dir()?,
            };
            let home = app.path().home_dir()?;
            let store = ConfigStore::new(&dir);
            let saved = store.load().ok().flatten();
            app.manage(store);
            app.manage(HostRuntime::new(dir.join("host")));
            app.manage(ClientRuntime::new());
            app.manage(SyncRuntime::new(dir.join("sync")));

            // Pick up where we left off: the host starts serving, a client
            // reconnects and its synced shelves start syncing again.
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                handle.state::<HostRuntime>().apply(saved.as_ref()).await;
                handle.state::<ClientRuntime>().apply(saved.as_ref()).await;
                handle.state::<SyncRuntime>().apply(saved.as_ref(), &home).await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::load_config,
            commands::save_config,
            commands::save_settings,
            commands::join_host,
            commands::reset_config,
            commands::defaults,
            commands::host_status,
            commands::create_invite,
            commands::list_devices,
            commands::revoke_device,
            commands::host_shelves,
            commands::create_shelf,
            commands::client_status,
            commands::set_synced,
            commands::sync_now,
            commands::sync_status,
            commands::sync_activity,
            commands::larder_folder,
            commands::open_folder,
            commands::pin_folder,
        ])
        .run(tauri::generate_context!())
        .expect("couldn't start Larder");
}
