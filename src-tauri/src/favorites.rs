use serde::{Deserialize, Serialize};
use tauri::State;
use crate::AppState;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Favorite {
    pub id: String,
    pub kubeconfig_id: String,
    pub namespace: String,
    pub resource_type: String,
    pub resource_name: String,
    pub remote_port: i64,
    pub local_port: Option<i64>,
    pub label: String,
    pub group_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SaveFavoriteRequest {
    pub kubeconfig_id: String,
    pub namespace: String,
    pub resource_type: String,
    pub resource_name: String,
    pub remote_port: i64,
    pub local_port: Option<i64>,
    pub label: String,
    pub group_name: Option<String>,
}

#[tauri::command]
pub async fn save_favorite(
    req: SaveFavoriteRequest,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO favorites (id, kubeconfig_id, namespace, resource_type, resource_name, remote_port, local_port, label, group_name) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.kubeconfig_id)
    .bind(&req.namespace)
    .bind(&req.resource_type)
    .bind(&req.resource_name)
    .bind(req.remote_port)
    .bind(req.local_port)
    .bind(&req.label)
    .bind(&req.group_name)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(id)
}

#[tauri::command]
pub async fn list_favorites(
    state: State<'_, AppState>,
) -> Result<Vec<Favorite>, String> {
    sqlx::query_as::<_, Favorite>(
        "SELECT id, kubeconfig_id, namespace, resource_type, resource_name, remote_port, local_port, label, group_name FROM favorites ORDER BY group_name, label"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_favorite(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    sqlx::query("DELETE FROM favorites WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
