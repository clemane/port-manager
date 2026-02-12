CREATE TABLE IF NOT EXISTS kubeconfigs (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    content BLOB NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_used TEXT
);

CREATE TABLE IF NOT EXISTS favorites (
    id TEXT PRIMARY KEY NOT NULL,
    kubeconfig_id TEXT NOT NULL,
    namespace TEXT NOT NULL,
    resource_type TEXT NOT NULL CHECK (resource_type IN ('service', 'pod')),
    resource_name TEXT NOT NULL,
    remote_port INTEGER NOT NULL,
    local_port INTEGER,
    label TEXT NOT NULL,
    group_name TEXT,
    FOREIGN KEY (kubeconfig_id) REFERENCES kubeconfigs(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS active_forwards (
    id TEXT PRIMARY KEY NOT NULL,
    favorite_id TEXT,
    kubeconfig_id TEXT NOT NULL,
    namespace TEXT NOT NULL,
    resource_type TEXT NOT NULL CHECK (resource_type IN ('service', 'pod')),
    resource_name TEXT NOT NULL,
    remote_port INTEGER NOT NULL,
    local_port INTEGER NOT NULL,
    pid INTEGER,
    status TEXT NOT NULL DEFAULT 'stopped' CHECK (status IN ('running', 'error', 'stopped')),
    started_at TEXT,
    error_msg TEXT,
    FOREIGN KEY (favorite_id) REFERENCES favorites(id) ON DELETE SET NULL,
    FOREIGN KEY (kubeconfig_id) REFERENCES kubeconfigs(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

INSERT OR IGNORE INTO settings (key, value) VALUES ('theme', '"dark"');
INSERT OR IGNORE INTO settings (key, value) VALUES ('port_range_start', '3000');
INSERT OR IGNORE INTO settings (key, value) VALUES ('port_range_end', '4000');
