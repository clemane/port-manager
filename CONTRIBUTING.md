# Contributing to Port Manager

Thanks for your interest in contributing! Port Manager is a desktop app built with **Tauri 2 + Vue 3 + Rust**, and contributions of all kinds are welcome — bug reports, feature requests, docs improvements, and code.

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) 1.81+
- [Node.js](https://nodejs.org/) 18+
- [Tauri CLI](https://tauri.app/start/) — `cargo install tauri-cli`
- `kubectl` (for port-forward features)
- `ngrok` (for tunnel features)

### Setup

```bash
git clone https://github.com/clemane/port-manager.git
cd port-manager
npm install
cargo tauri dev
```

## How to Contribute

### Reporting Bugs

Open an [issue](https://github.com/clemane/port-manager/issues/new?template=bug_report.md) with:
- Steps to reproduce
- Expected vs actual behavior
- Your OS and app version

### Suggesting Features

Open an [issue](https://github.com/clemane/port-manager/issues/new?template=feature_request.md) describing the feature and why it would be useful.

### Submitting Code

1. **Fork** the repository
2. **Create a branch** from `main`: `git checkout -b feat/my-feature`
3. **Make your changes**
4. **Test locally** — run `cargo tauri dev` and verify everything works
5. **Commit** with a clear message (see conventions below)
6. **Push** to your fork and open a **Pull Request**

## Branch Naming

| Type | Pattern | Example |
|------|---------|---------|
| Feature | `feat/short-description` | `feat/docker-support` |
| Bug fix | `fix/short-description` | `fix/tunnel-crash` |
| Docs | `docs/short-description` | `docs/setup-guide` |
| Refactor | `refactor/short-description` | `refactor/k8s-client` |

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add Docker container management
fix: resolve port conflict on tunnel launch
docs: update ngrok setup instructions
refactor: simplify kubeconfig encryption flow
```

## Project Structure

- **`src/`** — Vue 3 frontend (TypeScript)
- **`src-tauri/`** — Rust backend (Tauri commands, SQLite, K8s, PostgreSQL)
- **`src/components/ui/`** — Reusable UI components
- **`src/composables/`** — Vue composables
- **`src/views/`** — Page views

## Code Style

### Frontend (TypeScript / Vue)
- Vue 3 Composition API with `<script setup lang="ts">`
- TypeScript strict mode
- Use existing UI components from `src/components/ui/`

### Backend (Rust)
- Follow standard Rust conventions (`cargo fmt`, `cargo clippy`)
- Tauri commands go in their respective module files
- Use `sqlx` for database operations

## Review Process

1. A maintainer will review your PR
2. Changes may be requested — this is normal and collaborative
3. Once approved, your PR will be merged into `main`

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
