<div align="center">

# Port Manager

**A desktop command center for Kubernetes port-forwards, ngrok tunnels, and database connections.**

Built with [Tauri 2](https://tauri.app/) + [Vue 3](https://vuejs.org/) + [Rust](https://www.rust-lang.org/)

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2.x-FFC131?logo=tauri&logoColor=white)](https://tauri.app/)
[![Vue](https://img.shields.io/badge/Vue-3.x-4FC08D?logo=vue.js&logoColor=white)](https://vuejs.org/)
[![Rust](https://img.shields.io/badge/Rust-1.81+-DEA584?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.x-3178C6?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)

</div>

---

## Overview

Port Manager is a lightweight desktop application for developers and DevOps engineers who juggle multiple Kubernetes clusters, tunnels, and database connections daily. Instead of managing scattered terminal tabs, everything lives in one clean interface.

- Scan system ports in real time
- Launch and manage `kubectl port-forward` sessions
- Spin up ngrok tunnels with your reserved domains
- Connect to PostgreSQL databases and run queries with a built-in SQL editor
- Switch between 4 themes: Dark, Light, Cyberpunk, Matrix

## Features

### Dashboard — System Port Scanner
- Real-time view of all active ports (LISTEN, ESTABLISHED, TIME_WAIT)
- Filter by process name, port number, or connection state
- Quick overview of what's running on your machine

### Kubernetes Port-Forwards
- Import and manage multiple kubeconfigs with encrypted storage via OS keyring
- Browse namespaces, services, and pods from a tree view
- Launch `kubectl port-forward` directly from the UI with auto-assigned or custom local ports
- Save favorites with custom labels and group them by namespace or category
- One-click launch, kill, or restart any forward

### Ngrok Tunnels
- Manage reserved ngrok domains
- Sync domains from your ngrok account via API
- Launch tunnels with a single click
- Detect already-running tunnels on the machine (via `127.0.0.1:4040`)
- Copy public URLs to clipboard instantly

### Database Manager (DataGrip Lite)
- Connect to PostgreSQL databases (standalone or through active port-forwards)
- SQL editor powered by CodeMirror 6 with syntax highlighting and autocomplete
- Schema browser: tables, views, functions, and DDL inspection
- Query history and saved queries
- EXPLAIN query plan visualization
- Create tables, add columns, and manage schema from the UI

### Settings
- Theme switcher (Dark, Light, Cyberpunk, Matrix)
- Kubeconfig import (file picker or paste YAML)
- Port range configuration for auto-assigned local ports
- Ngrok authtoken and API key management

## Tech Stack

| Layer | Technology |
|---|---|
| Desktop framework | [Tauri 2](https://tauri.app/) |
| Frontend | Vue 3 + TypeScript + Vite |
| Backend | Rust |
| Local database | SQLite (sqlx) |
| K8s client | kube-rs + k8s-openapi |
| HTTP tunnels | ngrok CLI |
| SQL editor | CodeMirror 6 |
| PostgreSQL driver | tokio-postgres + deadpool |
| Secret storage | OS keyring |

## Prerequisites

- [Rust](https://rustup.rs/) 1.81+
- [Node.js](https://nodejs.org/) 18+
- [Tauri CLI](https://tauri.app/start/) — `cargo install tauri-cli`
- `kubectl` — required for port-forward features
- `ngrok` — required for tunnel features ([download](https://ngrok.com/download))

## Getting Started

```bash
# Clone the repository
git clone https://github.com/clemane/port-manager.git
cd port-manager

# Install frontend dependencies
npm install

# Run in development mode (hot-reload)
cargo tauri dev

# Build for production (Linux)
cargo tauri build --bundles deb
```

## Project Structure

```
port-manager/
├── src/                            # Vue 3 frontend
│   ├── components/
│   │   ├── db/                     # Database manager components
│   │   ├── layout/                 # AppLayout, Sidebar, StatusBar
│   │   └── ui/                     # Reusable UI kit (Button, Table, Modal, ...)
│   ├── composables/                # Vue composables (useForwards, useNgrok, usePgManager, ...)
│   ├── router/                     # Vue Router configuration
│   ├── types/                      # TypeScript type definitions
│   ├── views/                      # Page views
│   │   ├── DashboardView.vue       # System port scanner
│   │   ├── ForwardsView.vue        # K8s port-forward management
│   │   ├── K8sBrowserView.vue      # K8s resource browser
│   │   ├── NgrokView.vue           # Ngrok tunnel management
│   │   ├── DatabaseView.vue        # SQL editor & schema browser
│   │   └── SettingsView.vue        # App configuration
│   └── assets/                     # CSS design tokens & themes
├── src-tauri/                      # Rust backend
│   ├── src/
│   │   ├── lib.rs                  # Tauri app setup & command registration
│   │   ├── db.rs                   # SQLite initialization & migrations
│   │   ├── crypto.rs               # Encryption utilities
│   │   ├── forward.rs              # kubectl port-forward process management
│   │   ├── ngrok.rs                # Ngrok tunnel management & API sync
│   │   ├── k8s.rs                  # Kubernetes API client
│   │   ├── kubeconfig.rs           # Kubeconfig import/export (encrypted)
│   │   ├── pgmanager.rs            # PostgreSQL connection & query execution
│   │   ├── favorites.rs            # Favorite configurations
│   │   ├── settings.rs             # Key-value settings store
│   │   └── ports.rs                # System port scanner
│   └── migrations/                 # SQLite schema migrations
└── docs/plans/                     # Design & implementation documents
```

## Ngrok Setup

1. Open **Settings** in the app
2. Paste your **ngrok authtoken** ([get it here](https://dashboard.ngrok.com/get-started/your-authtoken)) and save
3. Paste your **ngrok API key** ([get it here](https://dashboard.ngrok.com/api-keys)) and save
4. Go to the **Ngrok** page
5. Click **Sync from ngrok** to import your reserved domains
6. Select a domain, enter a local port, and click **Launch**

> To detect tunnels already running on the machine, click **Detect running**.

## Security

- Kubeconfigs are encrypted at rest using the OS keyring
- Database credentials are stored encrypted in the local SQLite database
- No data is sent to external servers — everything runs locally
- ngrok API keys are stored in the local settings database, never exposed in config files

## License

[MIT](LICENSE)
