# Port Manager — Design Document

## Objectif

Application desktop Linux pour gérer tous les ports de la machine et les port-forwards Kubernetes depuis une interface graphique moderne et multi-thème.

---

## Stack technique

| Couche | Techno |
|--------|--------|
| Desktop runtime | Tauri 2 |
| Backend | Rust (kube-rs, sqlx) |
| Frontend | Vue 3 + TypeScript + Vite |
| Base de données | SQLite embarqué |
| Client K8s | kube-rs (natif Rust) |
| Scan ports système | /proc/net/tcp (Linux natif) |

---

## Architecture

```
┌─────────────────────────────────────┐
│           Tauri (Rust)              │
│                                     │
│  ┌───────────┐  ┌────────────────┐  │
│  │ K8s Client│  │ System Ports   │  │
│  │ (kube-rs) │  │ (/proc/net/tcp)│  │
│  └───────────┘  └────────────────┘  │
│  ┌───────────┐  ┌────────────────┐  │
│  │ Port-Fwd  │  │ SQLite Store   │  │
│  │ Manager   │  │ (sqlx)         │  │
│  └───────────┘  └────────────────┘  │
├─────────────────────────────────────┤
│        Vue 3 + TypeScript           │
│  ┌────────────────────────────────┐ │
│  │  UIKit (multi-theme)           │ │
│  │  ├── Dashboard (tous les ports)│ │
│  │  ├── K8s Browser (namespaces…) │ │
│  │  ├── Port-Forwards actifs      │ │
│  │  └── Settings (kubeconfigs)    │ │
│  └────────────────────────────────┘ │
└─────────────────────────────────────┘
```

Communication frontend → backend via `tauri::invoke` (IPC natif Tauri).

---

## Écrans

### 1. Dashboard — Vue globale des ports

- Tableau de tous les ports utilisés sur la machine (PID, process name, port, protocole, état)
- Filtres : par port, par process, par état (LISTEN, ESTABLISHED...)
- Les port-forwards K8s sont tagués visuellement pour les distinguer
- Indicateur de conflits (port utilisé par 2 sources)

### 2. K8s Browser — Navigation dans les clusters

- Sélecteur de cluster/kubeconfig en haut
- Arborescence : Namespace → Services / Pods
- Pour chaque service : liste des ports exposés
- Bouton "Forward" direct avec mini formulaire (port local souhaité, auto-detect si dispo)

### 3. Port-Forwards — Gestion des forwards actifs

- Liste des port-forwards en cours avec statut (actif, erreur, stoppé)
- Actions par ligne : Kill, Restart, Copier l'URL locale
- Section "Favoris" en haut pour relancer des forwards sauvegardés en un clic
- Groupement possible par cluster/namespace

### 4. Settings

- Import/suppression de kubeconfigs
- Choix du thème
- Port range préféré (ex: 3000-4000) pour l'auto-assign

---

## Modèle de données (SQLite)

### kubeconfigs
| Colonne | Type | Description |
|---------|------|-------------|
| id | TEXT PK | UUID |
| name | TEXT | Nom affiché ("prod-cluster") |
| content | BLOB | Kubeconfig chiffré |
| created_at | DATETIME | Date d'import |
| last_used | DATETIME | Dernier usage |

### favorites
| Colonne | Type | Description |
|---------|------|-------------|
| id | TEXT PK | UUID |
| kubeconfig_id | TEXT FK | Référence kubeconfig |
| namespace | TEXT | Namespace K8s |
| resource_type | TEXT | "service" ou "pod" |
| resource_name | TEXT | Nom de la ressource |
| remote_port | INTEGER | Port distant |
| local_port | INTEGER | Port local préféré (nullable = auto) |
| label | TEXT | Nom custom ("API Staging") |
| group_name | TEXT | Groupement dans l'UI |

### active_forwards
| Colonne | Type | Description |
|---------|------|-------------|
| id | TEXT PK | UUID |
| favorite_id | TEXT FK | Nullable, si lancé depuis un favori |
| kubeconfig_id | TEXT FK | Référence kubeconfig |
| namespace | TEXT | Namespace K8s |
| resource_type | TEXT | "service" ou "pod" |
| resource_name | TEXT | Nom de la ressource |
| remote_port | INTEGER | Port distant |
| local_port | INTEGER | Port local |
| pid | INTEGER | PID du process |
| status | TEXT | "running", "error", "stopped" |
| started_at | DATETIME | Heure de lancement |
| error_msg | TEXT | Nullable, message d'erreur |

### settings
| Colonne | Type | Description |
|---------|------|-------------|
| key | TEXT PK | Clé du setting |
| value | TEXT | Valeur JSON sérialisée |

Notes :
- Kubeconfigs chiffrés au repos via keyring système Linux
- active_forwards persiste l'état pour restauration après crash/restart
- Favoris = templates réutilisables, indépendants des forwards actifs

---

## UIKit & Theming

### Architecture des tokens CSS

```
tokens/
  ├── colors.css        # Palette brute (gray-100, blue-500...)
  ├── themes/
  │   ├── dark.css      # --pm-bg: var(--gray-900)
  │   ├── light.css     # --pm-bg: var(--gray-50)
  │   └── cyberpunk.css # --pm-bg: var(--black), accents néon
  └── semantic.css      # --pm-surface, --pm-danger, --pm-success
```

3 niveaux de tokens :
- **Primitifs** — Couleurs brutes (`--gray-900`, `--blue-400`)
- **Sémantiques** — Sens (`--pm-surface`, `--pm-text-primary`, `--pm-danger`)
- **Composants** — Spécifiques (`--pm-btn-bg`, `--pm-table-row-hover`)

Changement de thème = swap classe sur `<html>` → toutes les variables suivent.

### Composants

| Composant | Description |
|-----------|-------------|
| PmButton | Variants: primary, ghost, danger, icon |
| PmInput | Text, search, number |
| PmTable | Sortable, filterable, avec slots |
| PmBadge | Status: active, error, stopped, info |
| PmSidebar | Navigation latérale |
| PmModal | Confirmations, formulaires |
| PmToast | Notifications |
| PmSelect | Dropdown avec search |
| PmTreeView | Arborescence K8s (namespaces → services) |
| PmThemeSwitcher | Toggle de thème |

Chaque composant utilise uniquement des variables sémantiques `--pm-*`.

### Thèmes de base

- **Dark** — Fond sombre, texte clair, accents bleus (défaut)
- **Light** — Fond clair, sobre et pro
- **Cyberpunk** — Noir profond, accents néon cyan/magenta, glow effects

---

## Commandes Tauri (IPC)

### Ports système
- `get_system_ports()` → Liste de tous les ports ouverts sur la machine
- `get_port_details(port)` → Détail d'un port (PID, process, état)

### Kubeconfigs
- `import_kubeconfig(name, content)` → Importe et chiffre un kubeconfig
- `list_kubeconfigs()` → Liste les kubeconfigs stockés
- `delete_kubeconfig(id)` → Supprime un kubeconfig

### K8s Browser
- `list_namespaces(kubeconfig_id)` → Liste les namespaces
- `list_services(kubeconfig_id, namespace)` → Liste les services
- `list_pods(kubeconfig_id, namespace)` → Liste les pods
- `get_service_ports(kubeconfig_id, namespace, service)` → Ports d'un service

### Port-Forwards
- `create_forward(kubeconfig_id, namespace, resource_type, resource_name, remote_port, local_port?)` → Crée un port-forward
- `kill_forward(id)` → Kill un forward
- `restart_forward(id)` → Restart un forward
- `list_forwards()` → Liste les forwards actifs

### Favoris
- `save_favorite(forward_config, label, group?)` → Sauvegarde un favori
- `list_favorites()` → Liste les favoris
- `launch_favorite(id)` → Lance un forward depuis un favori
- `delete_favorite(id)` → Supprime un favori

### Settings
- `get_setting(key)` → Récupère un setting
- `set_setting(key, value)` → Met à jour un setting

---

## Sécurité

- Kubeconfigs chiffrés via le keyring système (Secret Service API / libsecret sur Linux)
- Pas de stockage de secrets en clair dans SQLite
- Validation des ports (range 1-65535, ports privilégiés < 1024 signalés)
- Sanitization des inputs utilisateur avant passage au client K8s
