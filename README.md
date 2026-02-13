# Port Manager

Desktop application for managing Kubernetes port-forwards and ngrok tunnels. Built with **Tauri 2**, **Vue 3**, and **Rust**.

## Features

### Dashboard
- Real-time system port scanner (LISTEN, ESTABLISHED, TIME_WAIT)
- Search and filter by process, port, or state

### Kubernetes Port-Forwards
- Import and manage multiple kubeconfigs (encrypted storage via OS keyring)
- Browse namespaces, services, and pods
- Launch `kubectl port-forward` from the UI with auto-assigned or custom local ports
- Kill / restart forwards, save favorites for one-click launch
- Favorites grouped by namespace or custom labels

### Ngrok Tunnels
- Store reserved ngrok domains
- Sync domains automatically from your ngrok account (API key)
- Launch `ngrok http --domain=xxx.ngrok-free.app <port>` from the UI
- Detect already-running tunnels on the machine (via local API `127.0.0.1:4040`)
- Kill / restart tunnels, copy public URL to clipboard
- Tunnel URL auto-resolved from ngrok local API

### Settings
- Theme switcher (Dark, Light, Cyberpunk, Matrix)
- Kubeconfig import (file path or paste YAML)
- Port range configuration for auto-assigned ports
- Ngrok authtoken (for launching tunnels)
- Ngrok API key (for syncing reserved domains)

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop framework | [Tauri 2](https://tauri.app/) |
| Frontend | Vue 3 + TypeScript + Vite |
| Backend | Rust |
| Database | SQLite (via sqlx) |
| K8s client | kube-rs + k8s-openapi |
| HTTP tunnels | ngrok CLI |
| Secret storage | OS keyring (keyring crate) |

## Prerequisites

- [Rust](https://rustup.rs/) (1.81+)
- [Node.js](https://nodejs.org/) (18+)
- [Tauri CLI](https://tauri.app/start/): `cargo install tauri-cli`
- `kubectl` (for port-forward features)
- `ngrok` (for tunnel features): https://ngrok.com/download

## Getting Started

```bash
# Install frontend dependencies
npm install

# Run in development mode (hot-reload)
cargo tauri dev

# Build release
cargo tauri build --bundles deb
```

## Project Structure

```
port-manager/
├── src/                          # Vue 3 frontend
│   ├── components/
│   │   ├── layout/               # AppLayout, PmSidebar, PmStatusBar
│   │   └── ui/                   # Reusable components (PmButton, PmTable, PmSelect, ...)
│   ├── composables/              # Vue composables (useForwards, useNgrok, usePorts, ...)
│   ├── router/                   # Vue Router
│   ├── views/                    # Page views
│   │   ├── DashboardView.vue     # System port scanner
│   │   ├── K8sBrowserView.vue    # K8s namespace/service/pod browser
│   │   ├── ForwardsView.vue      # Port-forward management
│   │   ├── NgrokView.vue         # Ngrok tunnel management
│   │   └── SettingsView.vue      # Settings & configuration
│   └── assets/                   # Styles & themes
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── lib.rs                # Tauri app setup, command registration
│   │   ├── db.rs                 # SQLite init & migrations
│   │   ├── forward.rs            # kubectl port-forward subprocess management
│   │   ├── ngrok.rs              # ngrok tunnel management & cloud API sync
│   │   ├── k8s.rs                # Kubernetes API client
│   │   ├── kubeconfig.rs         # Kubeconfig import/export (encrypted)
│   │   ├── favorites.rs          # Favorite port-forward configs
│   │   ├── settings.rs           # Key-value settings store
│   │   └── ports.rs              # System port scanner
│   └── migrations/
│       ├── 001_init.sql          # Core tables
│       └── 002_ngrok.sql         # Ngrok tables
└── package.json
```

## Ngrok Setup

1. Open **Settings** in the app
2. Paste your **ngrok authtoken** (from https://dashboard.ngrok.com/get-started/your-authtoken) and click Save
3. Paste your **ngrok API key** (from https://dashboard.ngrok.com/api-keys) and click Save
4. Go to the **Ngrok** page
5. Click **"Sync from ngrok"** to import your reserved domains
6. Select a domain, enter a local port, and click **Launch**

To detect tunnels already running on the machine, click **"Detect running"**.

## License

MIT
