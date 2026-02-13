# DataGrip Lite Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task.

**Goal:** Transform the PostgreSQL database manager from a basic query tool into a full IDE-style database client with CodeMirror 6, multi-tab workspaces, EXPLAIN visual, export, and schema management.

**Architecture:** Frontend-heavy refactoring (new DB components in `src/components/db/`), CodeMirror 6 as the SQL editor, enriched schema browser with context menus, multi-tab state in the singleton composable. Backend additions for views/functions listing, DDL commands, and export. Tauri dialog plugin for native file picker.

**Tech Stack:** codemirror 6, @codemirror/lang-sql, @codemirror/autocomplete, sql-formatter, @tauri-apps/plugin-dialog, tauri-plugin-dialog (Rust)

---

## Task 1: Install frontend dependencies

**Files:**
- Modify: `package.json`

**Step 1: Install CodeMirror 6 + sql-formatter + Tauri dialog plugin**

```bash
npm install codemirror @codemirror/view @codemirror/state @codemirror/lang-sql @codemirror/autocomplete @codemirror/commands @codemirror/language @codemirror/search sql-formatter @tauri-apps/plugin-dialog
```

**Step 2: Verify build still works**

```bash
npx vite build
```
Expected: build succeeds

**Step 3: Commit**

```bash
git add package.json package-lock.json
git commit -m "chore: add codemirror 6, sql-formatter, tauri dialog deps"
```

---

## Task 2: Add Rust tauri-plugin-dialog dependency

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Add dependency to Cargo.toml**

Add under `[dependencies]`:
```toml
tauri-plugin-dialog = "2"
```

**Step 2: Register plugin in lib.rs**

In the `run()` function, add the dialog plugin before `.invoke_handler`:
```rust
.plugin(tauri_plugin_dialog::init())
```

**Step 3: Verify Rust compilation**

```bash
cargo build
```

**Step 4: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/lib.rs
git commit -m "chore: add tauri-plugin-dialog for native file picker"
```

---

## Task 3: Add DB semantic tokens to all 4 themes

**Files:**
- Modify: `src/assets/tokens/themes/dark.css`
- Modify: `src/assets/tokens/themes/light.css`
- Modify: `src/assets/tokens/themes/cyberpunk.css`
- Modify: `src/assets/tokens/themes/matrix.css`

**Step 1: Add tokens to each theme**

In each theme file, add these CSS variables inside the existing `:root[data-theme="<name>"]` block:

**dark.css:**
```css
  --pm-db-keyword: #58a6ff;
  --pm-db-string: #7ee787;
  --pm-db-number: #d29922;
  --pm-db-comment: #6e7681;
  --pm-db-table: #79c0ff;
  --pm-db-column: #c9d1d9;
  --pm-db-function: #d2a8ff;
```

**light.css:**
```css
  --pm-db-keyword: #0550ae;
  --pm-db-string: #116329;
  --pm-db-number: #953800;
  --pm-db-comment: #6e7781;
  --pm-db-table: #0969da;
  --pm-db-column: #24292f;
  --pm-db-function: #8250df;
```

**cyberpunk.css:**
```css
  --pm-db-keyword: #00e5ff;
  --pm-db-string: #76ff03;
  --pm-db-number: #ffea00;
  --pm-db-comment: #616161;
  --pm-db-table: #18ffff;
  --pm-db-column: #e0e0e0;
  --pm-db-function: #ea80fc;
```

**matrix.css:**
```css
  --pm-db-keyword: #00ff41;
  --pm-db-string: #7dff7d;
  --pm-db-number: #b5ff6a;
  --pm-db-comment: #2d5a2d;
  --pm-db-table: #39ff14;
  --pm-db-column: #a0ffa0;
  --pm-db-function: #00e676;
```

**Step 2: Verify build**

```bash
npx vite build
```

**Step 3: Commit**

```bash
git add src/assets/tokens/themes/
git commit -m "style: add DB semantic color tokens to all 4 themes"
```

---

## Task 4: Add new TypeScript types

**Files:**
- Modify: `src/types/pgmanager.ts`

**Step 1: Add QueryTab, ExplainNode, view/function types**

Append to `src/types/pgmanager.ts`:

```typescript
export interface QueryTab {
  id: string
  label: string
  sql: string
  cursorPos: number
  result: PgQueryResult | null
  error: string | null
  loading: boolean
}

export interface PgViewInfo {
  schema_name: string
  view_name: string
  definition: string | null
}

export interface PgFunctionInfo {
  schema_name: string
  function_name: string
  result_type: string
  argument_types: string
  function_type: string
}

export interface ExplainNode {
  id: string
  type: string
  relation?: string
  alias?: string
  startupCost: number
  totalCost: number
  planRows: number
  actualTime: number | null
  actualRows: number | null
  loops: number
  filter?: string
  indexName?: string
  sharedHitBlocks?: number
  sharedReadBlocks?: number
  children: ExplainNode[]
  percentOfTotal: number
}

export type ContextMenuAction =
  | { type: 'select-star'; schema: string; table: string }
  | { type: 'select-count'; schema: string; table: string }
  | { type: 'drop-table'; schema: string; table: string }
  | { type: 'drop-view'; schema: string; view: string }
  | { type: 'drop-index'; schema: string; index: string }
  | { type: 'drop-function'; schema: string; func: string }
  | { type: 'export-csv'; schema: string; table: string }
  | { type: 'export-json'; schema: string; table: string }
  | { type: 'create-table'; schema: string }
  | { type: 'add-column'; schema: string; table: string }
  | { type: 'rename-table'; schema: string; table: string }
  | { type: 'refresh' }
```

**Step 2: Verify TypeScript**

```bash
npx vue-tsc --noEmit
```

**Step 3: Commit**

```bash
git add src/types/pgmanager.ts
git commit -m "feat: add QueryTab, ExplainNode, view/function types"
```

---

## Task 5: Backend — add views, functions, DDL, export commands

**Files:**
- Modify: `src-tauri/src/pgmanager.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Add new models in pgmanager.rs**

Add after existing models:
```rust
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PgViewInfo {
    pub schema_name: String,
    pub view_name: String,
    pub definition: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PgFunctionInfo {
    pub schema_name: String,
    pub function_name: String,
    pub result_type: String,
    pub argument_types: String,
    pub function_type: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateColumnDef {
    pub name: String,
    pub data_type: String,
    pub is_primary_key: bool,
    pub is_nullable: bool,
    pub default_value: Option<String>,
}
```

**Step 2: Add list_views command**

```rust
#[tauri::command]
pub async fn pg_list_views(id: String, schema: String, state: State<'_, AppState>) -> Result<Vec<PgViewInfo>, String> {
    let pools = state.pg_pools.lock().await;
    let pool = pools.get(&id).ok_or("Not connected")?;
    let client = pool.get().await.map_err(|e| e.to_string())?;
    let rows = client.query(
        "SELECT schemaname AS schema_name, viewname AS view_name, definition FROM pg_views WHERE schemaname = $1 ORDER BY viewname",
        &[&schema],
    ).await.map_err(|e| e.to_string())?;
    Ok(rows.iter().map(|r| PgViewInfo {
        schema_name: r.get("schema_name"),
        view_name: r.get("view_name"),
        definition: r.get("definition"),
    }).collect())
}
```

**Step 3: Add list_functions command**

```rust
#[tauri::command]
pub async fn pg_list_functions(id: String, schema: String, state: State<'_, AppState>) -> Result<Vec<PgFunctionInfo>, String> {
    let pools = state.pg_pools.lock().await;
    let pool = pools.get(&id).ok_or("Not connected")?;
    let client = pool.get().await.map_err(|e| e.to_string())?;
    let rows = client.query(
        "SELECT n.nspname AS schema_name, p.proname AS function_name, \
         pg_catalog.pg_get_function_result(p.oid) AS result_type, \
         pg_catalog.pg_get_function_arguments(p.oid) AS argument_types, \
         CASE p.prokind WHEN 'f' THEN 'function' WHEN 'p' THEN 'procedure' WHEN 'a' THEN 'aggregate' WHEN 'w' THEN 'window' END AS function_type \
         FROM pg_catalog.pg_proc p \
         JOIN pg_catalog.pg_namespace n ON n.oid = p.pronamespace \
         WHERE n.nspname = $1 AND p.prokind IN ('f', 'p') \
         ORDER BY p.proname",
        &[&schema],
    ).await.map_err(|e| e.to_string())?;
    Ok(rows.iter().map(|r| PgFunctionInfo {
        schema_name: r.get("schema_name"),
        function_name: r.get("function_name"),
        result_type: r.get("result_type"),
        argument_types: r.get("argument_types"),
        function_type: r.get("function_type"),
    }).collect())
}
```

**Step 4: Add DDL commands (create table, add column, drop object, rename)**

```rust
#[tauri::command]
pub async fn pg_create_table(
    id: String, schema: String, table_name: String,
    columns: Vec<CreateColumnDef>, state: State<'_, AppState>,
) -> Result<(), String> {
    let pools = state.pg_pools.lock().await;
    let pool = pools.get(&id).ok_or("Not connected")?;
    let client = pool.get().await.map_err(|e| e.to_string())?;

    let col_defs: Vec<String> = columns.iter().map(|c| {
        let mut def = format!("\"{}\" {}", c.name, c.data_type);
        if c.is_primary_key { def.push_str(" PRIMARY KEY"); }
        if !c.is_nullable && !c.is_primary_key { def.push_str(" NOT NULL"); }
        if let Some(ref d) = c.default_value { def.push_str(&format!(" DEFAULT {}", d)); }
        def
    }).collect();

    let sql = format!("CREATE TABLE \"{}\".\"{}\" ({})", schema, table_name, col_defs.join(", "));
    client.execute(&sql, &[]).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn pg_add_column(
    id: String, schema: String, table: String,
    name: String, data_type: String, is_nullable: bool, default_value: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pools = state.pg_pools.lock().await;
    let pool = pools.get(&id).ok_or("Not connected")?;
    let client = pool.get().await.map_err(|e| e.to_string())?;

    let mut sql = format!("ALTER TABLE \"{}\".\"{}\" ADD COLUMN \"{}\" {}", schema, table, name, data_type);
    if !is_nullable { sql.push_str(" NOT NULL"); }
    if let Some(ref d) = default_value { sql.push_str(&format!(" DEFAULT {}", d)); }

    client.execute(&sql, &[]).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn pg_drop_object(
    id: String, object_type: String, schema: String, name: String, cascade: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pools = state.pg_pools.lock().await;
    let pool = pools.get(&id).ok_or("Not connected")?;
    let client = pool.get().await.map_err(|e| e.to_string())?;

    let obj_type = match object_type.as_str() {
        "table" => "TABLE",
        "view" => "VIEW",
        "index" => "INDEX",
        "function" => "FUNCTION",
        _ => return Err(format!("Unknown object type: {}", object_type)),
    };

    let cascade_str = if cascade { " CASCADE" } else { "" };
    let sql = format!("DROP {} \"{}\".\"{}\"{}",  obj_type, schema, name, cascade_str);
    client.execute(&sql, &[]).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn pg_rename_table(
    id: String, schema: String, old_name: String, new_name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pools = state.pg_pools.lock().await;
    let pool = pools.get(&id).ok_or("Not connected")?;
    let client = pool.get().await.map_err(|e| e.to_string())?;

    let sql = format!("ALTER TABLE \"{}\".\"{}\" RENAME TO \"{}\"", schema, old_name, new_name);
    client.execute(&sql, &[]).await.map_err(|e| e.to_string())?;
    Ok(())
}
```

**Step 5: Add export commands**

```rust
#[tauri::command]
pub async fn pg_export_csv(
    id: String, sql: String, file_path: String,
    state: State<'_, AppState>,
) -> Result<i64, String> {
    let pools = state.pg_pools.lock().await;
    let pool = pools.get(&id).ok_or("Not connected")?;
    let client = pool.get().await.map_err(|e| e.to_string())?;

    let rows = client.query(&sql, &[]).await.map_err(|e| e.to_string())?;
    let mut wtr = csv::Writer::from_path(&file_path).map_err(|e| e.to_string())?;

    // Header
    if let Some(first) = rows.first() {
        let headers: Vec<&str> = first.columns().iter().map(|c| c.name()).collect();
        wtr.write_record(&headers).map_err(|e| e.to_string())?;
    }

    // Data
    for row in &rows {
        let json_vals = row_to_json(row);
        let strs: Vec<String> = json_vals.iter().map(|v| match v {
            serde_json::Value::Null => String::new(),
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        }).collect();
        wtr.write_record(&strs).map_err(|e| e.to_string())?;
    }

    wtr.flush().map_err(|e| e.to_string())?;
    Ok(rows.len() as i64)
}

#[tauri::command]
pub async fn pg_export_json(
    id: String, sql: String, file_path: String,
    state: State<'_, AppState>,
) -> Result<i64, String> {
    let pools = state.pg_pools.lock().await;
    let pool = pools.get(&id).ok_or("Not connected")?;
    let client = pool.get().await.map_err(|e| e.to_string())?;

    let rows = client.query(&sql, &[]).await.map_err(|e| e.to_string())?;
    let mut result = Vec::new();

    for row in &rows {
        let mut obj = serde_json::Map::new();
        let json_vals = row_to_json(row);
        for (i, col) in row.columns().iter().enumerate() {
            obj.insert(col.name().to_string(), json_vals[i].clone());
        }
        result.push(serde_json::Value::Object(obj));
    }

    let json_str = serde_json::to_string_pretty(&result).map_err(|e| e.to_string())?;
    std::fs::write(&file_path, json_str).map_err(|e| e.to_string())?;
    Ok(result.len() as i64)
}
```

**Step 6: Register all new commands in lib.rs**

Add to invoke_handler:
```rust
pgmanager::pg_list_views,
pgmanager::pg_list_functions,
pgmanager::pg_create_table,
pgmanager::pg_add_column,
pgmanager::pg_drop_object,
pgmanager::pg_rename_table,
pgmanager::pg_export_csv,
pgmanager::pg_export_json,
```

**Step 7: Verify Rust compilation**

```bash
cargo build
```

**Step 8: Commit**

```bash
git add src-tauri/src/pgmanager.rs src-tauri/src/lib.rs
git commit -m "feat: add views, functions, DDL, export backend commands"
```

---

## Task 6: Update usePgManager composable — tabs, views, functions, DDL, export

**Files:**
- Modify: `src/composables/usePgManager.ts`

**Step 1: Add tab state and new methods**

Add module-level refs after existing ones:
```typescript
const tabs = ref<QueryTab[]>([{ id: crypto.randomUUID(), label: 'Query 1', sql: '', cursorPos: 0, result: null, error: null, loading: false }])
const activeTabId = ref<string>(tabs.value[0].id)
const views = ref<PgViewInfo[]>([])
const functions = ref<PgFunctionInfo[]>([])
```

Add new methods inside `usePgManager()`:
```typescript
// Tab management
function createTab(label?: string, sql?: string) { ... }
function closeTab(tabId: string) { ... }
function setActiveTab(tabId: string) { ... }
const activeTab = computed(() => tabs.value.find(t => t.id === activeTabId.value) ?? null)

// Schema: views, functions
async function loadViews(schema: string) { ... }
async function loadFunctions(schema: string) { ... }

// DDL
async function createTable(schema: string, tableName: string, columns: CreateColumnDef[]) { ... }
async function addColumn(schema: string, table: string, name: string, dataType: string, isNullable: boolean, defaultValue?: string) { ... }
async function dropObject(objectType: string, schema: string, name: string, cascade: boolean) { ... }
async function renameTable(schema: string, oldName: string, newName: string) { ... }

// Export
async function exportCsv(sql: string, filePath: string) { ... }
async function exportJson(sql: string, filePath: string) { ... }
```

Each method follows the same pattern as existing ones: check `activeConnectionId`, `invoke()`, handle errors.

Update the return object to include all new state and methods.

**Step 2: Verify TypeScript**

```bash
npx vue-tsc --noEmit
```

**Step 3: Commit**

```bash
git add src/composables/usePgManager.ts
git commit -m "feat: add tabs, views, functions, DDL, export to usePgManager"
```

---

## Task 7: PmSqlEditor — CodeMirror 6 wrapper component

**Files:**
- Create: `src/components/db/PmSqlEditor.vue`

This replaces PmCodeEditor (the textarea) with a full CodeMirror 6 editor.

**Props:**
- `modelValue: string` — SQL content (v-model)
- `tables: string[]` — table names for autocomplete
- `columns: Map<string, string[]>` — table→columns map for autocomplete
- `schemas: string[]` — schema names for autocomplete
- `readOnly: boolean` — for SQL preview in modals

**Emits:**
- `update:modelValue` — content change
- `execute` — Ctrl+Enter
- `explain` — Ctrl+Shift+E
- `save` — Ctrl+S
- `format` — Ctrl+L

**Implementation highlights:**
- Use `@codemirror/lang-sql` with `PostgreSQL` dialect
- Custom theme using `EditorView.theme()` mapped to CSS vars via `getComputedStyle()`
- Autocomplete via `schemaCompletionSource` from `@codemirror/lang-sql` fed with schema/table/column data
- `sql-formatter` for Format action
- `onMounted` creates the EditorView, `onBeforeUnmount` destroys it
- `watch(modelValue)` syncs external changes to editor state

**Step 1: Create the component**

Full CodeMirror 6 setup with:
```typescript
import { EditorView, keymap, lineNumbers, highlightActiveLine } from '@codemirror/view'
import { EditorState } from '@codemirror/state'
import { sql, PostgreSQL } from '@codemirror/lang-sql'
import { autocompletion } from '@codemirror/autocomplete'
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands'
import { syntaxHighlighting, defaultHighlightStyle, bracketMatching } from '@codemirror/language'
import { searchKeymap, highlightSelectionMatches } from '@codemirror/search'
import { format } from 'sql-formatter'
```

Build the editor theme using CSS custom properties read from document for each `--pm-db-*` token. Setup keybindings for Ctrl+Enter (execute), Ctrl+Shift+E (explain), Ctrl+S (save), Ctrl+L (format). Autocomplete fed from props.

**Step 2: Verify build**

```bash
npx vue-tsc --noEmit && npx vite build
```

**Step 3: Commit**

```bash
git add src/components/db/PmSqlEditor.vue
git commit -m "feat: add PmSqlEditor with CodeMirror 6, autocomplete, theme"
```

---

## Task 8: PmQueryTabs — tab bar component

**Files:**
- Create: `src/components/db/PmQueryTabs.vue`

**Props:**
- `tabs: QueryTab[]`
- `activeTabId: string`

**Emits:**
- `select(tabId: string)` — switch tab
- `close(tabId: string)` — close tab
- `create()` — new tab
- `rename(tabId: string, label: string)` — rename tab

**Template:** Horizontal scrollable row of tab buttons. Active tab has 2px bottom accent border. Each tab shows: dot if modified (sql changed since last execute), spinner if loading, label text, close X on hover. `+` button at the end.

Double-click label → inline input for rename.

**Step 1: Create the component** with full template + scoped CSS.

**Step 2: Verify build**

**Step 3: Commit**

```bash
git add src/components/db/PmQueryTabs.vue
git commit -m "feat: add PmQueryTabs component"
```

---

## Task 9: PmContextMenu — right-click menu component

**Files:**
- Create: `src/components/db/PmContextMenu.vue`

**Props:**
- `items: { label: string; icon?: string; danger?: boolean; separator?: boolean }[]`
- `x: number` — position X
- `y: number` — position Y
- `visible: boolean`

**Emits:**
- `select(index: number)`
- `close()`

**Behavior:** Positioned at (x,y) with `position: fixed`. Click-outside closes. Escape closes. Items with `danger: true` are styled in `--pm-danger`. Separators render a 1px border line.

**Step 1: Create the component**

**Step 2: Verify build**

**Step 3: Commit**

```bash
git add src/components/db/PmContextMenu.vue
git commit -m "feat: add PmContextMenu for schema browser"
```

---

## Task 10: PmExplainView + PmExplainNode — EXPLAIN visual

**Files:**
- Create: `src/components/db/PmExplainView.vue`
- Create: `src/components/db/PmExplainNode.vue`

**PmExplainView props:**
- `plan: object` — raw JSON from EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON)

**Behavior:** Parses the JSON into an `ExplainNode[]` tree. Computes `percentOfTotal` for each node. Renders tree with `PmExplainNode` components connected by CSS lines.

**PmExplainNode props:**
- `node: ExplainNode`
- `depth: number`

**Template per node:** Card with:
- Operation type label (bold)
- Relation name if present
- Cost bar (4px height, gradient green→yellow→red based on percentOfTotal)
- Actual time and rows
- Filter/index info
- Hover tooltip with full details

Bottleneck nodes (>50% of total time): 3px left border `--pm-danger`, background tinted 5% red.

Staggered entrance animation: each node delays by `depth * 50ms`.

**Step 1: Create both components**

**Step 2: Verify build**

**Step 3: Commit**

```bash
git add src/components/db/PmExplainView.vue src/components/db/PmExplainNode.vue
git commit -m "feat: add EXPLAIN ANALYZE visual tree renderer"
```

---

## Task 11: PmSchemaTree — enriched schema browser

**Files:**
- Create: `src/components/db/PmSchemaTree.vue`

Replaces the inline schema tree in DatabaseView. Extracts it into a dedicated component with:

**Props:**
- `schemas: string[]`
- `connectionId: string | null`
- `isConnected: boolean`

**Emits:**
- `selectTable(schema: string, table: string)`
- `doubleClickTable(schema: string, table: string)` — opens new tab
- `contextMenu(action: ContextMenuAction)`
- `refresh()`

**Features:**
- Search bar at top (filters all nodes)
- Sections per schema: Tables (count), Views (count), Indexes (count), Functions (count)
- Each section lazy-loads on expand (calls usePgManager methods)
- SVG icons per node type: grid (table), eye (view), bolt (index), f (function)
- Table items show estimated rows + size on the right
- Right-click triggers PmContextMenu with appropriate actions per node type
- Selected item: 2px left border accent + surface-hover bg

**Step 1: Create the component** (reads from usePgManager, emits actions to parent)

**Step 2: Verify build**

**Step 3: Commit**

```bash
git add src/components/db/PmSchemaTree.vue
git commit -m "feat: add PmSchemaTree with sections, search, context menu"
```

---

## Task 12: PmCreateTableModal + PmAddColumnModal + PmDropConfirmModal

**Files:**
- Create: `src/components/db/PmCreateTableModal.vue`
- Create: `src/components/db/PmAddColumnModal.vue`
- Create: `src/components/db/PmDropConfirmModal.vue`

**PmCreateTableModal:**
- Two-tab modal: Form | SQL Preview
- Form: table name input, schema select, columns grid (add/remove rows)
- Column row: name input, type select (serial, bigserial, integer, bigint, text, varchar, boolean, timestamp, timestamptz, date, uuid, jsonb, numeric, float8, bytea), PK checkbox, nullable checkbox, default input
- SQL Preview: PmSqlEditor in readOnly mode showing generated CREATE TABLE
- Emits: `create(schema, tableName, columns)`

**PmAddColumnModal:**
- Simple modal: name, type select, nullable checkbox, default input
- Emits: `add(name, dataType, isNullable, defaultValue)`

**PmDropConfirmModal:**
- Shows the DROP SQL that will be executed
- Row count warning for tables
- Type-to-confirm input (must match object name)
- Danger-styled Drop button, disabled until confirmed
- Emits: `confirm()`, `cancel()`

**Step 1: Create all 3 modal components**

**Step 2: Verify build**

**Step 3: Commit**

```bash
git add src/components/db/PmCreateTableModal.vue src/components/db/PmAddColumnModal.vue src/components/db/PmDropConfirmModal.vue
git commit -m "feat: add schema management modals (create table, add column, drop)"
```

---

## Task 13: Rewrite DatabaseView — full DataGrip Lite layout

**Files:**
- Modify: `src/views/DatabaseView.vue`

This is the main integration task. Complete rewrite of the template and script.

**New layout:**
```
┌─────────────────────────────────────────────────────────┐
│ [Connection ▼] [Connect] [+ New]          [DB Status]   │
├──────────┬──────────────────────────────────────────────┤
│          │ [Tab 1] [Tab 2] [+]                    [×]  │
│ PmSchema ├──────────────────────────────────────────────┤
│ Tree     │ PmSqlEditor (CodeMirror 6)                   │
│          ├──[Run][Explain][Export▼][Format]──stats──────│
│          ├──────────────────────────────────────────────┤
│          │ [Results] [Explain] (toggle)                  │
│          │ PmTable / PmExplainView                      │
│          │ + edit toolbar if pending edits               │
├──────────┴──────────────────────────────────────────────┤
│ ● Connected  mydb@127.0.0.1:5433    12ms last query    │
└─────────────────────────────────────────────────────────┘
```

**Key changes from current:**
- Replace PmCodeEditor with PmSqlEditor (CodeMirror 6)
- Add PmQueryTabs between connection bar and editor
- Replace inline schema tree with PmSchemaTree component
- Add export dropdown in toolbar (Export CSV / Export JSON / Export Table CSV / Export Table JSON)
- Add Format button (calls sql-formatter)
- Add Explain button (wraps query with EXPLAIN, shows PmExplainView)
- Add toggle between Results and Explain view below toolbar
- Add status footer bar at the bottom
- Move toolbar between editor and results (was above editor)
- Tab state drives which SQL/result is shown

**Script changes:**
- Use tabs from usePgManager
- Active tab's SQL synced with PmSqlEditor
- Active tab's result shown in results area
- Double-click table in PmSchemaTree → createTab with SELECT *
- Context menu actions → open appropriate modals
- Export uses `@tauri-apps/plugin-dialog` save dialog
- Explain parses JSON and toggles to PmExplainView
- Duration color: green <100ms, orange <1s, red >1s

**Step 1: Full rewrite of DatabaseView.vue**

Include all imports, script setup, template, and scoped CSS for the new layout. Preserve existing inline editing logic.

**Step 2: Verify TypeScript + build**

```bash
npx vue-tsc --noEmit && npx vite build
```

**Step 3: Commit**

```bash
git add src/views/DatabaseView.vue
git commit -m "feat: rewrite DatabaseView with DataGrip Lite layout"
```

---

## Task 14: Final integration — route, sidebar, component cleanup

**Files:**
- Modify: `src/components/ui/index.ts` (no changes needed if db/ components imported directly in DatabaseView)
- Verify: `src/router/index.ts` (route already exists)
- Verify: `src/components/layout/PmSidebar.vue` (database nav already exists)

**Step 1: Verify routing and navigation work**

```bash
npx vue-tsc --noEmit && npx vite build
```

**Step 2: Commit any remaining changes**

```bash
git add -A && git commit -m "feat: DataGrip Lite integration complete"
```

---

## Verification

After all tasks:

1. `cargo build` — Rust compilation with new commands
2. `npx vue-tsc --noEmit` — TypeScript check
3. `npx vite build` — Frontend build
4. Manual tests:
   - Connect to a PG database via port-forward
   - Browse schemas, tables, views, indexes, functions in the tree
   - Search in schema browser
   - Open multiple query tabs
   - Run SQL with CodeMirror autocomplete
   - Use EXPLAIN ANALYZE visual view
   - Export results as CSV and JSON
   - Create a table via modal
   - Add a column to existing table
   - Drop a table with confirmation
   - Inline edit cells and save
   - Switch between tabs preserving state
   - Navigate away and back — state persists

---

## File Inventory

**New files (10):**
- `src/components/db/PmSqlEditor.vue`
- `src/components/db/PmQueryTabs.vue`
- `src/components/db/PmContextMenu.vue`
- `src/components/db/PmExplainView.vue`
- `src/components/db/PmExplainNode.vue`
- `src/components/db/PmSchemaTree.vue`
- `src/components/db/PmCreateTableModal.vue`
- `src/components/db/PmAddColumnModal.vue`
- `src/components/db/PmDropConfirmModal.vue`

**Modified files (8):**
- `package.json` + `package-lock.json`
- `src-tauri/Cargo.toml`
- `src-tauri/src/lib.rs`
- `src-tauri/src/pgmanager.rs`
- `src/types/pgmanager.ts`
- `src/composables/usePgManager.ts`
- `src/views/DatabaseView.vue`
- `src/assets/tokens/themes/*.css` (4 files)
