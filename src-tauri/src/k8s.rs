use std::collections::HashMap;

use k8s_openapi::api::core::v1::{Namespace, Pod, Secret, Service};
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use kube::{
    api::ListParams,
    config::{KubeConfigOptions, Kubeconfig},
    Api, Client, Config,
};
use regex::Regex;
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

// ---------------------------------------------------------------------------
// Database credential detection
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Clone)]
pub struct DetectedCredentials {
    pub source: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub ssl_mode: Option<String>,
    pub confidence: f32,
}

/// Well-known environment / secret key names grouped by database field.
const PASSWORD_KEYS: &[&str] = &[
    "POSTGRES_PASSWORD",
    "PGPASSWORD",
    "DB_PASSWORD",
    "DATABASE_PASSWORD",
    "password",
];
const USER_KEYS: &[&str] = &[
    "POSTGRES_USER",
    "PGUSER",
    "DB_USER",
    "DATABASE_USER",
    "username",
];
const DATABASE_KEYS: &[&str] = &[
    "POSTGRES_DB",
    "PGDATABASE",
    "DB_NAME",
    "DATABASE_NAME",
    "database",
];
const HOST_KEYS: &[&str] = &["DB_HOST", "DATABASE_HOST", "PGHOST", "host"];
const PORT_KEYS: &[&str] = &["DB_PORT", "DATABASE_PORT", "PGPORT", "port"];
const URL_KEYS: &[&str] = &["DATABASE_URL", "POSTGRES_URL", "PG_URL", "DB_URL"];

/// Intermediate mutable builder used while scanning keys.
#[derive(Default, Clone)]
struct CredBuilder {
    host: Option<String>,
    port: Option<u16>,
    database: Option<String>,
    username: Option<String>,
    password: Option<String>,
    ssl_mode: Option<String>,
}

impl CredBuilder {
    /// Try to match a key/value pair against all known DB key names.
    fn ingest(&mut self, key: &str, value: &str) {
        if PASSWORD_KEYS.contains(&key) {
            self.password = Some(value.to_string());
        } else if USER_KEYS.contains(&key) {
            self.username = Some(value.to_string());
        } else if DATABASE_KEYS.contains(&key) {
            self.database = Some(value.to_string());
        } else if HOST_KEYS.contains(&key) {
            self.host = Some(value.to_string());
        } else if PORT_KEYS.contains(&key) {
            if let Ok(p) = value.parse::<u16>() {
                self.port = Some(p);
            }
        }
    }

    /// Try to parse a DATABASE_URL style connection string and fill fields.
    fn ingest_url(&mut self, url: &str) {
        let re = Regex::new(r"postgres(?:ql)?://([^:]+):([^@]+)@([^:/]+):(\d+)/([^\s?]+)")
            .expect("invalid regex");
        if let Some(caps) = re.captures(url) {
            self.username = Some(caps[1].to_string());
            self.password = Some(caps[2].to_string());
            self.host = Some(caps[3].to_string());
            if let Ok(p) = caps[4].parse::<u16>() {
                self.port = Some(p);
            }
            self.database = Some(caps[5].to_string());
        }
    }

    /// Returns true when at least one DB-related field was detected.
    fn has_any(&self) -> bool {
        self.host.is_some()
            || self.port.is_some()
            || self.database.is_some()
            || self.username.is_some()
            || self.password.is_some()
    }

    /// Compute a confidence score: +0.25 per detected field, capped at 1.0.
    fn confidence(&self) -> f32 {
        let mut n: u32 = 0;
        if self.host.is_some() {
            n += 1;
        }
        if self.port.is_some() {
            n += 1;
        }
        if self.database.is_some() {
            n += 1;
        }
        if self.username.is_some() {
            n += 1;
        }
        if self.password.is_some() {
            n += 1;
        }
        (n as f32 * 0.25).min(1.0)
    }

    fn build(self, source: String) -> DetectedCredentials {
        let confidence = self.confidence();
        DetectedCredentials {
            source,
            host: self.host,
            port: self.port,
            database: self.database,
            username: self.username,
            password: self.password,
            ssl_mode: self.ssl_mode,
            confidence,
        }
    }
}

/// Decode the data map of a Kubernetes Secret into plain UTF-8 strings keyed
/// by the original key name.
fn decode_secret_data(secret: &Secret) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Some(data) = &secret.data {
        for (key, byte_string) in data {
            if let Ok(value) = String::from_utf8(byte_string.0.clone()) {
                map.insert(key.clone(), value);
            }
        }
    }
    map
}

#[tauri::command]
pub async fn detect_db_credentials(
    kubeconfig_id: String,
    namespace: String,
    state: State<'_, AppState>,
) -> Result<Vec<DetectedCredentials>, String> {
    let client = build_client(&kubeconfig_id, &state.db).await?;

    // ------------------------------------------------------------------
    // 1. Fetch all secrets in the namespace and build a lookup cache.
    // ------------------------------------------------------------------
    let secrets_api: Api<Secret> = Api::namespaced(client.clone(), &namespace);
    let secret_list = secrets_api
        .list(&ListParams::default())
        .await
        .map_err(|e| format!("Failed to list secrets: {}", e))?;

    // Cache: secret_name -> decoded key/value pairs
    let mut secret_cache: HashMap<String, HashMap<String, String>> = HashMap::new();
    for secret in &secret_list.items {
        let name = secret.metadata.name.clone().unwrap_or_default();
        let decoded = decode_secret_data(secret);
        secret_cache.insert(name, decoded);
    }

    let mut results: Vec<DetectedCredentials> = Vec::new();

    // ------------------------------------------------------------------
    // 2. Scan secrets for DB credentials.
    // ------------------------------------------------------------------
    for secret in &secret_list.items {
        let secret_name = secret.metadata.name.clone().unwrap_or_default();
        let decoded = secret_cache.get(&secret_name).cloned().unwrap_or_default();

        let mut builder = CredBuilder::default();

        for (key, value) in &decoded {
            if URL_KEYS.contains(&key.as_str()) {
                builder.ingest_url(value);
            }
            builder.ingest(key.as_str(), value);
        }

        if builder.has_any() {
            results.push(builder.build(format!("secret/{}", secret_name)));
        }
    }

    // ------------------------------------------------------------------
    // 3. Scan pods for DB credentials via env vars.
    // ------------------------------------------------------------------
    let pods_api: Api<Pod> = Api::namespaced(client.clone(), &namespace);
    let pod_list = pods_api
        .list(&ListParams::default())
        .await
        .map_err(|e| format!("Failed to list pods: {}", e))?;

    for pod in &pod_list.items {
        let pod_name = pod.metadata.name.clone().unwrap_or_default();
        let spec = match &pod.spec {
            Some(s) => s,
            None => continue,
        };

        for container in &spec.containers {
            let env_vars = match &container.env {
                Some(envs) => envs,
                None => continue,
            };

            let mut builder = CredBuilder::default();

            for env_var in env_vars {
                let key = &env_var.name;

                // Direct value
                if let Some(value) = &env_var.value {
                    if URL_KEYS.contains(&key.as_str()) {
                        builder.ingest_url(value);
                    }
                    builder.ingest(key.as_str(), value);
                    continue;
                }

                // Resolve secretKeyRef
                if let Some(value_from) = &env_var.value_from {
                    if let Some(secret_key_ref) = &value_from.secret_key_ref {
                        let secret_name = &secret_key_ref.name;
                        if secret_name.is_empty() {
                            continue;
                        }
                        let secret_key = &secret_key_ref.key;

                        if let Some(secret_data) = secret_cache.get(secret_name.as_str()) {
                            if let Some(resolved_value) = secret_data.get(secret_key) {
                                if URL_KEYS.contains(&key.as_str()) {
                                    builder.ingest_url(resolved_value);
                                }
                                builder.ingest(key.as_str(), resolved_value);
                            }
                        }
                    }
                }
            }

            if builder.has_any() {
                results.push(
                    builder.build(format!("pod/{}/env/{}", pod_name, container.name)),
                );
            }
        }
    }

    // ------------------------------------------------------------------
    // 4. Sort by confidence descending.
    // ------------------------------------------------------------------
    results.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(results)
}
