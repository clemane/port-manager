# Port Manager — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Tauri 2 desktop app on Linux to manage system ports and Kubernetes port-forwards with a multi-theme Vue 3 UIKit.

**Architecture:** Rust backend handles K8s operations (kube-rs), system port scanning (/proc/net/tcp), SQLite persistence (sqlx), and kubeconfig encryption (libsecret). Vue 3 frontend communicates via Tauri IPC (`invoke`). UIKit uses CSS custom properties for multi-theme support.

**Tech Stack:** Tauri 2, Rust, kube-rs, sqlx (SQLite), Vue 3, TypeScript, Vite

**Prerequisites:**
- Rust 1.93+ (installed)
- Node 25+ (installed)
- webkit2gtk-4.1 (installed)
- libsecret (installed)
- `sudo pacman -S libappindicator-gtk3` (missing)
- `cargo install tauri-cli --version "^2"` (missing)

---

## Phase 1: Project Scaffolding

### Task 1: Initialize Tauri 2 + Vue 3 project

**Files:**
- Create: Full project scaffold via `create-tauri-app`

**Step 1: Install Tauri CLI**

Run: `cargo install tauri-cli --version "^2"`

**Step 2: Scaffold the project**

Run (from `/home/clement/Documents/GlobalTi/git/port-manager`):
```bash
npm create tauri-app@latest . -- --template vue-ts --manager npm
```

If the CLI asks questions:
- Package name: `port-manager`
- Identifier: `com.portmanager.app`
- Frontend: Vue + TypeScript (Vite)

**Step 3: Verify it builds**

Run:
```bash
npm install
cargo tauri dev
```
Expected: A Tauri window opens with the default Vue template.

**Step 4: Commit**

```bash
git add -A
git commit -m "chore: scaffold Tauri 2 + Vue 3 + TypeScript project"
```

---

### Task 2: Configure Rust dependencies

**Files:**
- Modify: `src-tauri/Cargo.toml`

**Step 1: Add Rust dependencies**

Add to `[dependencies]` in `src-tauri/Cargo.toml`:
```toml
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
kube = { version = "0.98", features = ["client", "runtime", "derive"] }
k8s-openapi = { version = "0.24", features = ["latest"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
tokio = { version = "1", features = ["full"] }
anyhow = "1"
keyring = { version = "3", features = ["linux-native"] }
```

**Step 2: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: Compiles with no errors.

**Step 3: Commit**

```bash
git add src-tauri/Cargo.toml
git commit -m "chore: add Rust dependencies (sqlx, kube-rs, keyring)"
```

---

### Task 3: Configure Vue frontend dependencies

**Files:**
- Modify: `package.json`

**Step 1: Install frontend deps**

Run:
```bash
npm install vue-router@4 pinia @vueuse/core
npm install -D sass-embedded
```

**Step 2: Set up project structure**

Create the following directory structure:
```
src/
  assets/
    tokens/
      colors.css
      semantic.css
      themes/
        dark.css
        light.css
        cyberpunk.css
  components/
    ui/          # UIKit components (Pm*)
    layout/      # PmSidebar, PmStatusBar
  views/
    DashboardView.vue
    K8sBrowserView.vue
    ForwardsView.vue
    SettingsView.vue
  composables/   # Vue composables (useTheme, usePorts, etc.)
  stores/        # Pinia stores
  types/         # TypeScript interfaces
  router/
    index.ts
```

Run:
```bash
mkdir -p src/{assets/tokens/themes,components/{ui,layout},views,composables,stores,types,router}
```

**Step 3: Commit**

```bash
git add -A
git commit -m "chore: add Vue dependencies and project structure"
```

---

## Phase 2: UIKit & Theming

### Task 4: Create CSS token system

**Files:**
- Create: `src/assets/tokens/colors.css`
- Create: `src/assets/tokens/semantic.css`
- Create: `src/assets/tokens/themes/dark.css`
- Create: `src/assets/tokens/themes/light.css`
- Create: `src/assets/tokens/themes/cyberpunk.css`

**Step 1: Create primitive color tokens**

File `src/assets/tokens/colors.css`:
```css
:root {
  /* Gray scale */
  --gray-50: #f9fafb;
  --gray-100: #f3f4f6;
  --gray-200: #e5e7eb;
  --gray-300: #d1d5db;
  --gray-400: #9ca3af;
  --gray-500: #6b7280;
  --gray-600: #4b5563;
  --gray-700: #374151;
  --gray-800: #1f2937;
  --gray-900: #111827;
  --gray-950: #030712;

  /* Blue */
  --blue-400: #60a5fa;
  --blue-500: #3b82f6;
  --blue-600: #2563eb;

  /* Green */
  --green-400: #4ade80;
  --green-500: #22c55e;

  /* Red */
  --red-400: #f87171;
  --red-500: #ef4444;

  /* Yellow */
  --yellow-400: #facc15;
  --yellow-500: #eab308;

  /* Cyan */
  --cyan-400: #22d3ee;
  --cyan-500: #06b6d4;

  /* Magenta */
  --magenta-400: #e879f9;
  --magenta-500: #d946ef;

  /* Base */
  --white: #ffffff;
  --black: #000000;
}
```

**Step 2: Create theme files**

File `src/assets/tokens/themes/dark.css`:
```css
[data-theme="dark"] {
  --pm-bg: var(--gray-950);
  --pm-surface: var(--gray-900);
  --pm-surface-hover: var(--gray-800);
  --pm-surface-active: var(--gray-700);
  --pm-border: var(--gray-700);
  --pm-border-subtle: var(--gray-800);

  --pm-text-primary: var(--gray-50);
  --pm-text-secondary: var(--gray-400);
  --pm-text-muted: var(--gray-500);

  --pm-accent: var(--blue-500);
  --pm-accent-hover: var(--blue-400);
  --pm-accent-text: var(--white);

  --pm-success: var(--green-500);
  --pm-danger: var(--red-500);
  --pm-warning: var(--yellow-500);
  --pm-info: var(--cyan-500);

  --pm-sidebar-bg: var(--gray-900);
  --pm-sidebar-active: var(--gray-800);
  --pm-sidebar-text: var(--gray-400);
  --pm-sidebar-text-active: var(--white);

  --pm-badge-running-bg: rgba(34, 197, 94, 0.15);
  --pm-badge-running-text: var(--green-400);
  --pm-badge-error-bg: rgba(239, 68, 68, 0.15);
  --pm-badge-error-text: var(--red-400);
  --pm-badge-stopped-bg: rgba(107, 114, 128, 0.15);
  --pm-badge-stopped-text: var(--gray-400);

  --pm-shadow: 0 1px 3px rgba(0, 0, 0, 0.4);
  --pm-radius: 8px;
  --pm-radius-sm: 4px;
  --pm-radius-lg: 12px;
}
```

File `src/assets/tokens/themes/light.css`:
```css
[data-theme="light"] {
  --pm-bg: var(--gray-50);
  --pm-surface: var(--white);
  --pm-surface-hover: var(--gray-100);
  --pm-surface-active: var(--gray-200);
  --pm-border: var(--gray-200);
  --pm-border-subtle: var(--gray-100);

  --pm-text-primary: var(--gray-900);
  --pm-text-secondary: var(--gray-600);
  --pm-text-muted: var(--gray-400);

  --pm-accent: var(--blue-600);
  --pm-accent-hover: var(--blue-500);
  --pm-accent-text: var(--white);

  --pm-success: var(--green-500);
  --pm-danger: var(--red-500);
  --pm-warning: var(--yellow-500);
  --pm-info: var(--cyan-500);

  --pm-sidebar-bg: var(--white);
  --pm-sidebar-active: var(--gray-100);
  --pm-sidebar-text: var(--gray-500);
  --pm-sidebar-text-active: var(--gray-900);

  --pm-badge-running-bg: rgba(34, 197, 94, 0.1);
  --pm-badge-running-text: var(--green-500);
  --pm-badge-error-bg: rgba(239, 68, 68, 0.1);
  --pm-badge-error-text: var(--red-500);
  --pm-badge-stopped-bg: rgba(107, 114, 128, 0.1);
  --pm-badge-stopped-text: var(--gray-500);

  --pm-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  --pm-radius: 8px;
  --pm-radius-sm: 4px;
  --pm-radius-lg: 12px;
}
```

File `src/assets/tokens/themes/cyberpunk.css`:
```css
[data-theme="cyberpunk"] {
  --pm-bg: var(--black);
  --pm-surface: #0a0a1a;
  --pm-surface-hover: #12122a;
  --pm-surface-active: #1a1a3a;
  --pm-border: #1e1e3e;
  --pm-border-subtle: #14142a;

  --pm-text-primary: var(--cyan-400);
  --pm-text-secondary: #8888cc;
  --pm-text-muted: #555577;

  --pm-accent: var(--magenta-500);
  --pm-accent-hover: var(--magenta-400);
  --pm-accent-text: var(--black);

  --pm-success: var(--cyan-400);
  --pm-danger: var(--magenta-500);
  --pm-warning: var(--yellow-400);
  --pm-info: var(--cyan-500);

  --pm-sidebar-bg: #050510;
  --pm-sidebar-active: #12122a;
  --pm-sidebar-text: #555577;
  --pm-sidebar-text-active: var(--cyan-400);

  --pm-badge-running-bg: rgba(34, 211, 238, 0.1);
  --pm-badge-running-text: var(--cyan-400);
  --pm-badge-error-bg: rgba(217, 70, 239, 0.1);
  --pm-badge-error-text: var(--magenta-400);
  --pm-badge-stopped-bg: rgba(85, 85, 119, 0.1);
  --pm-badge-stopped-text: #555577;

  --pm-shadow: 0 0 15px rgba(6, 182, 212, 0.1);
  --pm-radius: 4px;
  --pm-radius-sm: 2px;
  --pm-radius-lg: 8px;

  /* Cyberpunk-specific glow */
  --pm-glow-cyan: 0 0 10px rgba(34, 211, 238, 0.3);
  --pm-glow-magenta: 0 0 10px rgba(217, 70, 239, 0.3);
}
```

File `src/assets/tokens/semantic.css`:
```css
@import './colors.css';
@import './themes/dark.css';
@import './themes/light.css';
@import './themes/cyberpunk.css';

/* Transitions for theme switching */
*,
*::before,
*::after {
  transition: background-color 0.2s ease, color 0.2s ease, border-color 0.2s ease, box-shadow 0.2s ease;
}
```

**Step 3: Commit**

```bash
git add src/assets/tokens/
git commit -m "feat: add CSS token system with dark, light, and cyberpunk themes"
```

---

### Task 5: Build core UIKit components

**Files:**
- Create: `src/components/ui/PmButton.vue`
- Create: `src/components/ui/PmInput.vue`
- Create: `src/components/ui/PmBadge.vue`
- Create: `src/components/ui/PmModal.vue`
- Create: `src/components/ui/PmToast.vue`
- Create: `src/components/ui/PmTable.vue`
- Create: `src/components/ui/PmSelect.vue`
- Create: `src/components/ui/PmTreeView.vue`
- Create: `src/components/ui/PmThemeSwitcher.vue`
- Create: `src/components/ui/index.ts` (barrel export)

Each component uses only `--pm-*` CSS variables. Full implementation for each component:

**Step 1: Create PmButton**

File `src/components/ui/PmButton.vue`:
```vue
<script setup lang="ts">
defineProps<{
  variant?: 'primary' | 'ghost' | 'danger' | 'icon'
  size?: 'sm' | 'md' | 'lg'
  disabled?: boolean
  loading?: boolean
}>()
</script>

<template>
  <button
    class="pm-btn"
    :class="[`pm-btn--${variant ?? 'primary'}`, `pm-btn--${size ?? 'md'}`]"
    :disabled="disabled || loading"
  >
    <span v-if="loading" class="pm-btn__spinner" />
    <slot />
  </button>
</template>

<style scoped>
.pm-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  border: 1px solid transparent;
  border-radius: var(--pm-radius-sm);
  font-weight: 500;
  cursor: pointer;
  font-family: inherit;
}
.pm-btn--sm { padding: 4px 10px; font-size: 12px; }
.pm-btn--md { padding: 6px 14px; font-size: 13px; }
.pm-btn--lg { padding: 8px 18px; font-size: 14px; }

.pm-btn--primary {
  background: var(--pm-accent);
  color: var(--pm-accent-text);
}
.pm-btn--primary:hover:not(:disabled) { background: var(--pm-accent-hover); }

.pm-btn--ghost {
  background: transparent;
  color: var(--pm-text-secondary);
}
.pm-btn--ghost:hover:not(:disabled) {
  background: var(--pm-surface-hover);
  color: var(--pm-text-primary);
}

.pm-btn--danger {
  background: var(--pm-danger);
  color: var(--white);
}
.pm-btn--danger:hover:not(:disabled) { opacity: 0.9; }

.pm-btn--icon {
  background: transparent;
  color: var(--pm-text-secondary);
  padding: 6px;
  border-radius: var(--pm-radius-sm);
}
.pm-btn--icon:hover:not(:disabled) {
  background: var(--pm-surface-hover);
  color: var(--pm-text-primary);
}

.pm-btn:disabled { opacity: 0.5; cursor: not-allowed; }

.pm-btn__spinner {
  width: 14px;
  height: 14px;
  border: 2px solid transparent;
  border-top-color: currentColor;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }
</style>
```

Build all other components following the same pattern. Each one:
- Accepts props via `defineProps`
- Uses only `--pm-*` variables
- Scoped styles
- Minimal, focused API

**Step 2: Create remaining components**

Create all remaining components: PmInput, PmBadge, PmModal, PmToast, PmTable, PmSelect, PmTreeView, PmThemeSwitcher. Each one follows the same pattern.

**Step 3: Create barrel export**

File `src/components/ui/index.ts`:
```ts
export { default as PmButton } from './PmButton.vue'
export { default as PmInput } from './PmInput.vue'
export { default as PmBadge } from './PmBadge.vue'
export { default as PmModal } from './PmModal.vue'
export { default as PmToast } from './PmToast.vue'
export { default as PmTable } from './PmTable.vue'
export { default as PmSelect } from './PmSelect.vue'
export { default as PmTreeView } from './PmTreeView.vue'
export { default as PmThemeSwitcher } from './PmThemeSwitcher.vue'
```

**Step 4: Commit**

```bash
git add src/components/ui/
git commit -m "feat: add UIKit components (PmButton, PmTable, PmBadge, etc.)"
```

---

### Task 6: Set up layout shell and routing

**Files:**
- Create: `src/components/layout/PmSidebar.vue`
- Create: `src/components/layout/PmStatusBar.vue`
- Create: `src/components/layout/AppLayout.vue`
- Create: `src/router/index.ts`
- Create: `src/views/DashboardView.vue` (placeholder)
- Create: `src/views/K8sBrowserView.vue` (placeholder)
- Create: `src/views/ForwardsView.vue` (placeholder)
- Create: `src/views/SettingsView.vue` (placeholder)
- Modify: `src/main.ts`
- Modify: `src/App.vue`

**Step 1: Create router**

File `src/router/index.ts`:
```ts
import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', name: 'dashboard', component: () => import('../views/DashboardView.vue') },
    { path: '/k8s', name: 'k8s-browser', component: () => import('../views/K8sBrowserView.vue') },
    { path: '/forwards', name: 'forwards', component: () => import('../views/ForwardsView.vue') },
    { path: '/settings', name: 'settings', component: () => import('../views/SettingsView.vue') },
  ],
})

export default router
```

**Step 2: Create sidebar**

File `src/components/layout/PmSidebar.vue`:
```vue
<script setup lang="ts">
import { useRoute } from 'vue-router'
const route = useRoute()

const navItems = [
  { path: '/', label: 'Dashboard', icon: 'monitor' },
  { path: '/k8s', label: 'Kubernetes', icon: 'ship' },
  { path: '/forwards', label: 'Forwards', icon: 'arrow-right-left' },
  { path: '/settings', label: 'Settings', icon: 'settings' },
]
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar__logo">PM</div>
    <nav class="sidebar__nav">
      <router-link
        v-for="item in navItems"
        :key="item.path"
        :to="item.path"
        class="sidebar__item"
        :class="{ 'sidebar__item--active': route.path === item.path }"
      >
        <span class="sidebar__label">{{ item.label }}</span>
      </router-link>
    </nav>
    <div class="sidebar__footer">
      <slot name="theme-switcher" />
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 200px;
  height: 100vh;
  background: var(--pm-sidebar-bg);
  border-right: 1px solid var(--pm-border);
  display: flex;
  flex-direction: column;
  padding: 12px 0;
}
.sidebar__logo {
  font-size: 18px;
  font-weight: 700;
  color: var(--pm-accent);
  padding: 8px 16px 20px;
}
.sidebar__nav { flex: 1; display: flex; flex-direction: column; gap: 2px; padding: 0 8px; }
.sidebar__item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  border-radius: var(--pm-radius-sm);
  color: var(--pm-sidebar-text);
  text-decoration: none;
  font-size: 13px;
}
.sidebar__item:hover { background: var(--pm-sidebar-active); }
.sidebar__item--active {
  background: var(--pm-sidebar-active);
  color: var(--pm-sidebar-text-active);
  font-weight: 500;
}
.sidebar__footer { padding: 8px 16px; }
</style>
```

**Step 3: Create AppLayout, placeholder views, wire up App.vue and main.ts**

Integrate router + pinia into `main.ts`, replace `App.vue` with layout shell containing sidebar + `<router-view>`.

**Step 4: Verify app renders**

Run: `cargo tauri dev`
Expected: App shows sidebar with 4 nav items, clicking navigates between placeholder views.

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: add app layout with sidebar, router, and placeholder views"
```

---

## Phase 3: Rust Backend — Database & System Ports

### Task 7: Set up SQLite database with migrations

**Files:**
- Create: `src-tauri/src/db.rs`
- Create: `src-tauri/migrations/001_init.sql`
- Modify: `src-tauri/src/main.rs` (or `lib.rs`)

**Step 1: Create migration file**

File `src-tauri/migrations/001_init.sql`:
```sql
CREATE TABLE IF NOT EXISTS kubeconfigs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    content BLOB NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_used TEXT
);

CREATE TABLE IF NOT EXISTS favorites (
    id TEXT PRIMARY KEY,
    kubeconfig_id TEXT NOT NULL REFERENCES kubeconfigs(id) ON DELETE CASCADE,
    namespace TEXT NOT NULL,
    resource_type TEXT NOT NULL CHECK (resource_type IN ('service', 'pod')),
    resource_name TEXT NOT NULL,
    remote_port INTEGER NOT NULL,
    local_port INTEGER,
    label TEXT NOT NULL,
    group_name TEXT
);

CREATE TABLE IF NOT EXISTS active_forwards (
    id TEXT PRIMARY KEY,
    favorite_id TEXT REFERENCES favorites(id) ON DELETE SET NULL,
    kubeconfig_id TEXT NOT NULL REFERENCES kubeconfigs(id) ON DELETE CASCADE,
    namespace TEXT NOT NULL,
    resource_type TEXT NOT NULL CHECK (resource_type IN ('service', 'pod')),
    resource_name TEXT NOT NULL,
    remote_port INTEGER NOT NULL,
    local_port INTEGER NOT NULL,
    pid INTEGER,
    status TEXT NOT NULL DEFAULT 'stopped' CHECK (status IN ('running', 'error', 'stopped')),
    started_at TEXT,
    error_msg TEXT
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Default settings
INSERT OR IGNORE INTO settings (key, value) VALUES ('theme', '"dark"');
INSERT OR IGNORE INTO settings (key, value) VALUES ('port_range_start', '3000');
INSERT OR IGNORE INTO settings (key, value) VALUES ('port_range_end', '4000');
```

**Step 2: Create db module**

File `src-tauri/src/db.rs`:
```rust
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::PathBuf;

pub async fn init_db(app_dir: PathBuf) -> Result<SqlitePool, sqlx::Error> {
    std::fs::create_dir_all(&app_dir).ok();
    let db_path = app_dir.join("data.db");
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    sqlx::query(include_str!("../migrations/001_init.sql"))
        .execute(&pool)
        .await?;

    Ok(pool)
}
```

Note: `include_str!` with multiple statements needs splitting. Use `sqlx::raw_sql` or run each statement separately. Adjust in implementation.

**Step 3: Wire into Tauri app state**

Add `SqlitePool` as managed Tauri state in `main.rs`/`lib.rs`.

**Step 4: Verify with `cargo check`**

Run: `cd src-tauri && cargo check`
Expected: Compiles.

**Step 5: Commit**

```bash
git add src-tauri/
git commit -m "feat: add SQLite database setup with migrations"
```

---

### Task 8: System port scanner

**Files:**
- Create: `src-tauri/src/ports.rs`
- Modify: `src-tauri/src/main.rs` (register commands)

**Step 1: Implement /proc/net/tcp parser**

File `src-tauri/src/ports.rs`:
```rust
use serde::Serialize;
use std::fs;

#[derive(Debug, Serialize, Clone)]
pub struct SystemPort {
    pub protocol: String,       // "tcp" or "tcp6"
    pub local_port: u16,
    pub remote_port: u16,
    pub state: String,          // "LISTEN", "ESTABLISHED", etc.
    pub pid: Option<u32>,
    pub process_name: Option<String>,
}

fn parse_hex_port(hex: &str) -> u16 {
    u16::from_str_radix(hex, 16).unwrap_or(0)
}

fn tcp_state(state: &str) -> &str {
    match state {
        "0A" => "LISTEN",
        "01" => "ESTABLISHED",
        "02" => "SYN_SENT",
        "03" => "SYN_RECV",
        "04" => "FIN_WAIT1",
        "05" => "FIN_WAIT2",
        "06" => "TIME_WAIT",
        "07" => "CLOSE",
        "08" => "CLOSE_WAIT",
        "09" => "LAST_ACK",
        _ => "UNKNOWN",
    }
}

fn get_process_for_inode(inode: &str) -> Option<(u32, String)> {
    // Read /proc/*/fd/* to find which PID owns the socket inode
    let proc_dir = fs::read_dir("/proc").ok()?;
    for entry in proc_dir.flatten() {
        let pid_str = entry.file_name().to_string_lossy().to_string();
        let pid: u32 = pid_str.parse().ok()?;
        let fd_dir = format!("/proc/{}/fd", pid);
        if let Ok(fds) = fs::read_dir(&fd_dir) {
            for fd in fds.flatten() {
                if let Ok(link) = fs::read_link(fd.path()) {
                    let link_str = link.to_string_lossy();
                    if link_str.contains(&format!("socket:[{}]", inode)) {
                        let comm = fs::read_to_string(format!("/proc/{}/comm", pid))
                            .unwrap_or_default()
                            .trim()
                            .to_string();
                        return Some((pid, comm));
                    }
                }
            }
        }
    }
    None
}

pub fn scan_ports() -> Vec<SystemPort> {
    let mut ports = Vec::new();
    for (file, proto) in [("/proc/net/tcp", "tcp"), ("/proc/net/tcp6", "tcp6")] {
        if let Ok(content) = fs::read_to_string(file) {
            for line in content.lines().skip(1) {
                let fields: Vec<&str> = line.split_whitespace().collect();
                if fields.len() < 10 { continue; }

                let local_parts: Vec<&str> = fields[1].split(':').collect();
                let remote_parts: Vec<&str> = fields[2].split(':').collect();
                if local_parts.len() < 2 || remote_parts.len() < 2 { continue; }

                let local_port = parse_hex_port(local_parts.last().unwrap_or(&"0"));
                let remote_port = parse_hex_port(remote_parts.last().unwrap_or(&"0"));
                let state = tcp_state(fields[3]).to_string();
                let inode = fields[9];

                let (pid, process_name) = get_process_for_inode(inode)
                    .map(|(p, n)| (Some(p), Some(n)))
                    .unwrap_or((None, None));

                ports.push(SystemPort {
                    protocol: proto.to_string(),
                    local_port,
                    remote_port,
                    state,
                    pid,
                    process_name,
                });
            }
        }
    }
    ports.sort_by_key(|p| p.local_port);
    ports.dedup_by_key(|p| (p.local_port, p.pid));
    ports
}
```

**Step 2: Register Tauri command**

```rust
#[tauri::command]
fn get_system_ports() -> Vec<SystemPort> {
    ports::scan_ports()
}
```

**Step 3: Test manually**

Run: `cargo tauri dev`, then in browser console:
```js
await window.__TAURI__.core.invoke('get_system_ports')
```
Expected: Returns array of port objects.

**Step 4: Commit**

```bash
git add src-tauri/src/ports.rs src-tauri/src/main.rs
git commit -m "feat: add system port scanner via /proc/net/tcp"
```

---

## Phase 4: Rust Backend — Kubeconfig & K8s

### Task 9: Kubeconfig management (import, list, delete with encryption)

**Files:**
- Create: `src-tauri/src/kubeconfig.rs`
- Modify: `src-tauri/src/main.rs` (register commands)

**Step 1: Implement kubeconfig CRUD with keyring encryption**

Use `keyring` crate for encryption key storage, encrypt kubeconfig content before writing to SQLite. Implement commands:
- `import_kubeconfig(name: String, content: String)` — encrypt content, insert into DB
- `list_kubeconfigs()` — return id + name + dates (NOT content)
- `delete_kubeconfig(id: String)` — delete from DB
- `get_kubeconfig_content(id: String)` — decrypt and return (internal use only)

**Step 2: Register Tauri commands**

**Step 3: Verify with `cargo check`**

**Step 4: Commit**

```bash
git add src-tauri/src/kubeconfig.rs src-tauri/src/main.rs
git commit -m "feat: add kubeconfig management with keyring encryption"
```

---

### Task 10: K8s browser commands (namespaces, services, pods)

**Files:**
- Create: `src-tauri/src/k8s.rs`
- Modify: `src-tauri/src/main.rs`

**Step 1: Implement K8s client builder**

Use `kube-rs` with `Config::from_custom_kubeconfig` to create a client from stored kubeconfig content.

**Step 2: Implement browse commands**

```rust
#[tauri::command]
async fn list_namespaces(kubeconfig_id: String, state: State<'_, AppState>) -> Result<Vec<String>, String>

#[tauri::command]
async fn list_services(kubeconfig_id: String, namespace: String, state: State<'_, AppState>) -> Result<Vec<K8sService>, String>

#[tauri::command]
async fn list_pods(kubeconfig_id: String, namespace: String, state: State<'_, AppState>) -> Result<Vec<K8sPod>, String>
```

Where `K8sService` and `K8sPod` are serializable structs with name + ports.

**Step 3: Verify with `cargo check`**

**Step 4: Commit**

```bash
git add src-tauri/src/k8s.rs src-tauri/src/main.rs
git commit -m "feat: add K8s browser commands (namespaces, services, pods)"
```

---

### Task 11: Port-forward management (create, kill, restart)

**Files:**
- Create: `src-tauri/src/forward.rs`
- Modify: `src-tauri/src/main.rs`

**Step 1: Implement port-forward lifecycle**

Use `kube-rs` port-forward API or spawn `kubectl port-forward` as child process. Track PIDs in `active_forwards` table.

Commands:
- `create_forward(...)` — Start a port-forward, insert into active_forwards, return the record
- `kill_forward(id)` — Kill process by PID, update status to "stopped"
- `restart_forward(id)` — Kill + re-create
- `list_forwards()` — Return all active_forwards with current status

**Step 2: Add health check loop**

Spawn a background task that periodically checks if port-forward processes are still alive and updates their status.

**Step 3: Commit**

```bash
git add src-tauri/src/forward.rs src-tauri/src/main.rs
git commit -m "feat: add port-forward management (create, kill, restart)"
```

---

### Task 12: Favorites and settings commands

**Files:**
- Create: `src-tauri/src/favorites.rs`
- Create: `src-tauri/src/settings.rs`
- Modify: `src-tauri/src/main.rs`

**Step 1: Implement favorites CRUD**

- `save_favorite(...)` — Insert into favorites table
- `list_favorites()` — Return all favorites
- `launch_favorite(id)` — Create forward from favorite template
- `delete_favorite(id)` — Delete

**Step 2: Implement settings CRUD**

- `get_setting(key)` — Return value
- `set_setting(key, value)` — Upsert

**Step 3: Commit**

```bash
git add src-tauri/src/favorites.rs src-tauri/src/settings.rs src-tauri/src/main.rs
git commit -m "feat: add favorites and settings management"
```

---

## Phase 5: Frontend — Views

### Task 13: Theme composable and integration

**Files:**
- Create: `src/composables/useTheme.ts`
- Create: `src/stores/settings.ts`
- Modify: `src/App.vue`

**Step 1: Create useTheme composable**

```ts
// src/composables/useTheme.ts
import { ref, watchEffect } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export type Theme = 'dark' | 'light' | 'cyberpunk'

const currentTheme = ref<Theme>('dark')

export function useTheme() {
  async function loadTheme() {
    const saved = await invoke<string>('get_setting', { key: 'theme' })
    if (saved) currentTheme.value = JSON.parse(saved) as Theme
  }

  async function setTheme(theme: Theme) {
    currentTheme.value = theme
    document.documentElement.setAttribute('data-theme', theme)
    await invoke('set_setting', { key: 'theme', value: JSON.stringify(theme) })
  }

  watchEffect(() => {
    document.documentElement.setAttribute('data-theme', currentTheme.value)
  })

  return { currentTheme, loadTheme, setTheme }
}
```

**Step 2: Apply theme on app mount in `App.vue`**

**Step 3: Commit**

```bash
git add src/composables/ src/stores/ src/App.vue
git commit -m "feat: add theme composable with persistence"
```

---

### Task 14: Dashboard view (system ports)

**Files:**
- Create: `src/composables/usePorts.ts`
- Modify: `src/views/DashboardView.vue`

**Step 1: Create usePorts composable**

Calls `get_system_ports` via invoke, stores result in reactive ref, provides polling (every 5s).

**Step 2: Build DashboardView**

- Use PmTable to display ports
- Add filter bar (PmInput for search, PmSelect for state filter)
- Tag K8s port-forwards with PmBadge
- Highlight conflicts (same port, different PIDs)

**Step 3: Verify UI renders**

Run: `cargo tauri dev`
Expected: Dashboard shows system ports in a styled table with filters.

**Step 4: Commit**

```bash
git add src/composables/usePorts.ts src/views/DashboardView.vue
git commit -m "feat: add Dashboard view with system port table"
```

---

### Task 15: K8s Browser view

**Files:**
- Create: `src/composables/useK8s.ts`
- Modify: `src/views/K8sBrowserView.vue`

**Step 1: Create useK8s composable**

Wraps all K8s invoke calls: `listNamespaces`, `listServices`, `listPods`.

**Step 2: Build K8sBrowserView**

- PmSelect for cluster/kubeconfig selection
- PmTreeView for Namespace → Service/Pod hierarchy
- Right panel with port details + "Forward" button
- Forward button opens PmModal with local port input (pre-filled with auto-detected free port)

**Step 3: Commit**

```bash
git add src/composables/useK8s.ts src/views/K8sBrowserView.vue
git commit -m "feat: add K8s Browser view with cluster navigation"
```

---

### Task 16: Port-Forwards view

**Files:**
- Create: `src/composables/useForwards.ts`
- Modify: `src/views/ForwardsView.vue`

**Step 1: Create useForwards composable**

Wraps forward invoke calls + favorites. Polls for status updates.

**Step 2: Build ForwardsView**

- Favorites section at top (cards, click to launch)
- Active forwards table with PmBadge for status
- Action buttons: Kill (PmButton danger), Restart (ghost), Copy URL (icon)
- Group by cluster/namespace toggle

**Step 3: Commit**

```bash
git add src/composables/useForwards.ts src/views/ForwardsView.vue
git commit -m "feat: add Port-Forwards view with favorites and actions"
```

---

### Task 17: Settings view

**Files:**
- Modify: `src/views/SettingsView.vue`

**Step 1: Build SettingsView**

- Kubeconfig management: list with delete button, import button (opens file picker via Tauri dialog API)
- Theme selector: PmThemeSwitcher with live preview
- Port range: two PmInput (number) for start/end

**Step 2: Commit**

```bash
git add src/views/SettingsView.vue
git commit -m "feat: add Settings view with kubeconfig import and theme selection"
```

---

## Phase 6: Polish & Integration

### Task 18: Status bar and global notifications

**Files:**
- Create: `src/components/layout/PmStatusBar.vue`
- Create: `src/composables/useToast.ts`
- Modify: `src/components/layout/AppLayout.vue`

**Step 1: Build status bar**

Shows: active forwards count, cluster connected, current port conflicts.

**Step 2: Build toast system**

Global toast notifications for: forward started/stopped, errors, conflicts.

**Step 3: Commit**

```bash
git add src/components/layout/PmStatusBar.vue src/composables/useToast.ts src/components/layout/AppLayout.vue
git commit -m "feat: add status bar and toast notification system"
```

---

### Task 19: Auto-restore forwards on app start

**Files:**
- Modify: `src-tauri/src/forward.rs`
- Modify: `src-tauri/src/main.rs` (setup hook)

**Step 1: On app startup, read active_forwards with status "running"**

Mark them as "stopped" (processes are dead after app restart), then optionally offer to re-launch them.

**Step 2: Emit event to frontend**

Use Tauri events to notify frontend of forwards that need re-launching.

**Step 3: Commit**

```bash
git add src-tauri/src/forward.rs src-tauri/src/main.rs
git commit -m "feat: add forward auto-restore on app startup"
```

---

### Task 20: Final integration test and build

**Step 1: Full manual test**

- Launch app, import a kubeconfig
- Browse namespaces/services
- Create a port-forward, verify it appears in Dashboard
- Kill and restart the forward
- Save as favorite, relaunch from favorites
- Switch themes
- Check port conflicts

**Step 2: Build release binary**

Run: `cargo tauri build`
Expected: `.deb` and `.AppImage` in `src-tauri/target/release/bundle/`

**Step 3: Commit**

```bash
git add -A
git commit -m "chore: final polish and integration"
```

---

## Summary

| Phase | Tasks | Description |
|-------|-------|-------------|
| 1 | 1-3 | Project scaffolding (Tauri + Vue + deps) |
| 2 | 4-6 | UIKit & Theming (tokens, components, layout) |
| 3 | 7-8 | Rust backend: DB + system ports |
| 4 | 9-12 | Rust backend: K8s, forwards, favorites, settings |
| 5 | 13-17 | Frontend views (dashboard, browser, forwards, settings) |
| 6 | 18-20 | Polish (status bar, toasts, auto-restore, build) |
