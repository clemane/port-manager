CREATE TABLE IF NOT EXISTS pg_connections (
    id TEXT PRIMARY KEY NOT NULL,
    label TEXT,
    forward_id TEXT,
    favorite_id TEXT,
    host TEXT NOT NULL DEFAULT '127.0.0.1',
    port INTEGER NOT NULL DEFAULT 5432,
    database_name TEXT NOT NULL DEFAULT 'postgres',
    username TEXT NOT NULL DEFAULT 'postgres',
    password BLOB,
    ssl_mode TEXT NOT NULL DEFAULT 'prefer',
    color TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_used TEXT,
    FOREIGN KEY (forward_id) REFERENCES active_forwards(id) ON DELETE SET NULL,
    FOREIGN KEY (favorite_id) REFERENCES favorites(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS pg_query_history (
    id TEXT PRIMARY KEY NOT NULL,
    connection_id TEXT NOT NULL,
    sql_text TEXT NOT NULL,
    executed_at TEXT NOT NULL DEFAULT (datetime('now')),
    duration_ms INTEGER,
    row_count INTEGER,
    error TEXT,
    FOREIGN KEY (connection_id) REFERENCES pg_connections(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS pg_saved_queries (
    id TEXT PRIMARY KEY NOT NULL,
    connection_id TEXT,
    label TEXT NOT NULL,
    sql_text TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (connection_id) REFERENCES pg_connections(id) ON DELETE SET NULL
)