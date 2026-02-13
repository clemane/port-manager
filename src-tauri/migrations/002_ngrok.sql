CREATE TABLE IF NOT EXISTS ngrok_domains (
    id TEXT PRIMARY KEY NOT NULL,
    domain TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS ngrok_tunnels (
    id TEXT PRIMARY KEY NOT NULL,
    domain_id TEXT,
    domain TEXT NOT NULL,
    local_port INTEGER NOT NULL,
    pid INTEGER,
    status TEXT NOT NULL DEFAULT 'stopped' CHECK (status IN ('running', 'error', 'stopped')),
    tunnel_url TEXT,
    started_at TEXT,
    error_msg TEXT,
    FOREIGN KEY (domain_id) REFERENCES ngrok_domains(id) ON DELETE SET NULL
)
