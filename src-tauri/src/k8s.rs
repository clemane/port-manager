use k8s_openapi::api::core::v1::{Namespace, Pod, Service};
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use kube::{
    api::ListParams,
    config::{KubeConfigOptions, Kubeconfig},
    Api, Client, Config,
};
use serde::Serialize;
use tauri::State;

use crate::kubeconfig::get_kubeconfig_content;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct K8sService {
    pub name: String,
    pub namespace: String,
    pub ports: Vec<ServicePort>,
}

#[derive(Debug, Serialize)]
pub struct ServicePort {
    pub name: Option<String>,
    pub port: i32,
    pub target_port: Option<String>,
    pub protocol: String,
}

#[derive(Debug, Serialize)]
pub struct K8sPod {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub ports: Vec<PodPort>,
}

#[derive(Debug, Serialize)]
pub struct PodPort {
    pub name: Option<String>,
    pub container_port: i32,
    pub protocol: String,
}

/// Format an IntOrString value as a plain string for serialization.
fn format_int_or_string(ios: &IntOrString) -> String {
    match ios {
        IntOrString::Int(i) => i.to_string(),
        IntOrString::String(s) => s.clone(),
    }
}

async fn build_client(
    kubeconfig_id: &str,
    pool: &sqlx::sqlite::SqlitePool,
) -> Result<Client, String> {
    let content = get_kubeconfig_content(kubeconfig_id, pool).await?;
    let kubeconfig: Kubeconfig = serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse kubeconfig: {}", e))?;
    let config = Config::from_custom_kubeconfig(kubeconfig, &KubeConfigOptions::default())
        .await
        .map_err(|e| format!("Failed to build K8s config: {}", e))?;
    Client::try_from(config).map_err(|e| format!("Failed to create K8s client: {}", e))
}

#[tauri::command]
pub async fn list_namespaces(
    kubeconfig_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let client = build_client(&kubeconfig_id, &state.db).await?;
    let namespaces: Api<Namespace> = Api::all(client);
    let list = namespaces
        .list(&ListParams::default())
        .await
        .map_err(|e| e.to_string())?;

    Ok(list
        .items
        .into_iter()
        .filter_map(|ns| ns.metadata.name)
        .collect())
}

#[tauri::command]
pub async fn list_services(
    kubeconfig_id: String,
    namespace: String,
    state: State<'_, AppState>,
) -> Result<Vec<K8sService>, String> {
    let client = build_client(&kubeconfig_id, &state.db).await?;
    let services: Api<Service> = Api::namespaced(client, &namespace);
    let list = services
        .list(&ListParams::default())
        .await
        .map_err(|e| e.to_string())?;

    Ok(list
        .items
        .into_iter()
        .map(|svc| {
            let name = svc.metadata.name.unwrap_or_default();
            let ns = svc.metadata.namespace.unwrap_or_default();
            let ports = svc
                .spec
                .as_ref()
                .and_then(|s| s.ports.as_ref())
                .map(|ports| {
                    ports
                        .iter()
                        .map(|p| ServicePort {
                            name: p.name.clone(),
                            port: p.port,
                            target_port: p.target_port.as_ref().map(format_int_or_string),
                            protocol: p.protocol.clone().unwrap_or_else(|| "TCP".to_string()),
                        })
                        .collect()
                })
                .unwrap_or_default();
            K8sService {
                name,
                namespace: ns,
                ports,
            }
        })
        .collect())
}

#[tauri::command]
pub async fn list_pods(
    kubeconfig_id: String,
    namespace: String,
    state: State<'_, AppState>,
) -> Result<Vec<K8sPod>, String> {
    let client = build_client(&kubeconfig_id, &state.db).await?;
    let pods: Api<Pod> = Api::namespaced(client, &namespace);
    let list = pods
        .list(&ListParams::default())
        .await
        .map_err(|e| e.to_string())?;

    Ok(list
        .items
        .into_iter()
        .map(|pod| {
            let name = pod.metadata.name.unwrap_or_default();
            let ns = pod.metadata.namespace.unwrap_or_default();
            let status = pod
                .status
                .as_ref()
                .and_then(|s| s.phase.clone())
                .unwrap_or_else(|| "Unknown".to_string());
            let ports = pod
                .spec
                .as_ref()
                .map(|spec| {
                    spec.containers
                        .iter()
                        .flat_map(|c| {
                            c.ports
                                .as_ref()
                                .map(|ports| {
                                    ports
                                        .iter()
                                        .map(|p| PodPort {
                                            name: p.name.clone(),
                                            container_port: p.container_port,
                                            protocol: p
                                                .protocol
                                                .clone()
                                                .unwrap_or_else(|| "TCP".to_string()),
                                        })
                                        .collect::<Vec<_>>()
                                })
                                .unwrap_or_default()
                        })
                        .collect()
                })
                .unwrap_or_default();
            K8sPod {
                name,
                namespace: ns,
                status,
                ports,
            }
        })
        .collect())
}
