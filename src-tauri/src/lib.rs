mod db;
mod favorites;
mod forward;
mod k8s;
mod kubeconfig;
mod ports;
mod settings;

use tauri::Manager;

pub struct AppState {
    pub db: sqlx::sqlite::SqlitePool,
}

#[tauri::command]
fn get_system_ports() -> Vec<ports::SystemPort> {
    ports::scan_ports()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let app_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");

            let pool = tauri::async_runtime::block_on(db::init_db(app_dir))
                .expect("failed to initialize database");

            app.manage(AppState { db: pool });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_system_ports,
            kubeconfig::import_kubeconfig,
            kubeconfig::list_kubeconfigs,
            kubeconfig::delete_kubeconfig,
            favorites::save_favorite,
            favorites::list_favorites,
            favorites::delete_favorite,
            settings::get_setting,
            settings::set_setting,
            k8s::list_namespaces,
            k8s::list_services,
            k8s::list_pods,
            forward::create_forward,
            forward::kill_forward,
            forward::restart_forward,
            forward::list_forwards,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
