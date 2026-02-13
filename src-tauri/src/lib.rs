mod crypto;
mod db;
mod favorites;
mod forward;
mod k8s;
mod kubeconfig;
mod ngrok;
mod pgmanager;
mod ports;
mod settings;

use std::collections::HashMap;
use std::sync::Arc;
use tauri::Manager;

pub struct AppState {
    pub db: sqlx::sqlite::SqlitePool,
    pub pg_pools: Arc<tokio::sync::Mutex<HashMap<String, deadpool_postgres::Pool>>>,
}

#[tauri::command]
fn get_system_ports() -> Vec<ports::SystemPort> {
    ports::scan_ports()
}

#[tauri::command]
fn read_file_content(path: String) -> Result<String, String> {
    // Expand ~ to home directory
    let expanded = if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            format!("{}/{}", home, &path[2..])
        } else {
            path.clone()
        }
    } else {
        path.clone()
    };
    std::fs::read_to_string(&expanded)
        .map_err(|e| format!("Cannot read {}: {}", expanded, e))
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

            // Clean up forwards that were "running" in a previous session.
            // After a restart, those kubectl processes are dead so we mark them stopped.
            let stale_count = tauri::async_runtime::block_on(forward::cleanup_stale_forwards(&pool))
                .map(|stale| stale.len())
                .unwrap_or(0);

            if stale_count > 0 {
                log::info!("{} stale forwards cleaned up on startup", stale_count);
            }

            let ngrok_stale_count =
                tauri::async_runtime::block_on(ngrok::cleanup_stale_tunnels(&pool))
                    .map(|stale| stale.len())
                    .unwrap_or(0);

            if ngrok_stale_count > 0 {
                log::info!("{} stale ngrok tunnels cleaned up on startup", ngrok_stale_count);
            }

            app.manage(AppState {
                db: pool,
                pg_pools: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            });

            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_system_ports,
            read_file_content,
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
            k8s::detect_db_credentials,
            forward::create_forward,
            forward::kill_forward,
            forward::restart_forward,
            forward::list_forwards,
            ngrok::add_ngrok_domain,
            ngrok::list_ngrok_domains,
            ngrok::delete_ngrok_domain,
            ngrok::create_tunnel,
            ngrok::kill_tunnel,
            ngrok::restart_tunnel,
            ngrok::list_tunnels,
            ngrok::sync_ngrok_domains,
            ngrok::detect_running_tunnels,
            pgmanager::pg_save_connection,
            pgmanager::pg_list_connections,
            pgmanager::pg_delete_connection,
            pgmanager::pg_test_connection,
            pgmanager::pg_connect,
            pgmanager::pg_disconnect,
            pgmanager::pg_list_schemas,
            pgmanager::pg_list_tables,
            pgmanager::pg_list_columns,
            pgmanager::pg_list_indexes,
            pgmanager::pg_table_row_count,
            pgmanager::pg_execute_query,
            pgmanager::pg_list_query_history,
            pgmanager::pg_save_query,
            pgmanager::pg_list_saved_queries,
            pgmanager::pg_delete_saved_query,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
