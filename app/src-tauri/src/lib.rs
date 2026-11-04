mod client_runtime;
mod commands;
mod config;
mod host_runtime;
mod network;

use tauri::Manager;

use crate::client_runtime::ClientRuntime;
use crate::config::ConfigStore;
use crate::host_runtime::HostRuntime;

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
            let store = ConfigStore::new(&dir);
            let saved = store.load().ok().flatten();
            let runtime = HostRuntime::new(dir.join("host"));
            app.manage(store);
            app.manage(runtime);
            app.manage(ClientRuntime::new());

            // Pick up where we left off: host starts serving, client
            // reconnects to its host.
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                handle.state::<HostRuntime>().apply(saved.as_ref()).await;
                handle.state::<ClientRuntime>().apply(saved.as_ref()).await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::load_config,
            commands::save_config,
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
        ])
        .run(tauri::generate_context!())
        .expect("couldn't start Larder");
}
