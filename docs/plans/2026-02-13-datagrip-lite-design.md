# DataGrip Lite — DB Manager UI/UX Redesign

> PostgreSQL database manager embedded in Port Manager, transformed from basic query tool to full IDE-style database client.

## Context

The existing DB manager has a solid backend (19 Rust commands, connection pooling, inline editing) but the frontend is a basic textarea + table layout. This redesign transforms it into a "DataGrip Lite" — a professional database IDE with the Mission Control dark cockpit aesthetic.

## Scope

- Full layout refactoring with multi-tab workspace
- CodeMirror 6 with SQL syntax highlighting and schema-aware autocomplete
- Enriched schema browser with Views, Indexes, Functions, context menus
- EXPLAIN ANALYZE visual tree renderer
- CSV/JSON export via Tauri file dialog
- Schema management (CREATE TABLE, ADD COLUMN, DROP, RENAME)
- Dedicated DB semantic tokens for all 4 themes

---

## Section 1: Layout & Navigation

```
+-----------------------------------------------------------+
| [Connection v] [Connect] [+ New]          [DB Status]     |  40px
+----------+------------------------------------------------+
|          | [Tab 1] [Tab 2] [+]                       [x]  |  32px
| SCHEMAS  +------------------------------------------------+
|          |                                                 |
| > public |  CodeMirror 6 SQL Editor                       |  resizable
|   users  |  (syntax highlight, autocomplete)               |
|   orders |                                                 |
|   items  +--[Run] [Explain] [Export v] [Format]-----------|  36px
|          +------------------------------------------------+
| > auth   |  Results Grid                                  |  resizable
|          |  id | name | email | ...                        |
| INDEXES  |  1  | Alice| a@... |                            |
| VIEWS    |                                                 |
|          |  1-100 of 1,234          < 1/13 >               |
+----------+------------------------------------------------+
| . Connected  mydb@127.0.0.1:5433    12ms last query       |  28px
+-----------------------------------------------------------+
```

- **Tab bar**: each tab = independent workspace (SQL + result)
- **Schema browser**: collapsible sections (Tables, Views, Indexes, Functions) with context menus
- **Toolbar**: between editor and results (Run, Explain, Export, Format + stats)
- **Status footer**: dedicated DB status bar (connection, latency, last query)
- **Split panes**: horizontal (schema 220px | editor+results flex), vertical (editor 40% | results 60%), all resizable with localStorage persistence

## Section 2: CodeMirror 6 SQL Editor

**Package**: `codemirror` + `@codemirror/lang-sql` + `@codemirror/autocomplete`

**Features**:
- SQL syntax highlighting using custom theme mapped to CSS vars
- Schema-aware autocomplete (tables, columns, schemas from browser cache)
- Bracket matching, multi-cursors (Alt+Click)
- Selection execution: Ctrl+Enter runs selection or full content
- SQL formatting via `sql-formatter` library
- Line numbers gutter
- Custom Mission Control theme adapting to all 4 app themes
- Per-tab editor state (content, cursor, undo history)

**Keyboard shortcuts**:
| Shortcut | Action |
|----------|--------|
| Ctrl+Enter | Execute (selection or all) |
| Ctrl+Shift+E | EXPLAIN ANALYZE |
| Ctrl+S | Save query |
| Ctrl+L | Format SQL |
| Ctrl+Space | Force autocomplete |
| Ctrl+T | New tab |
| Ctrl+W | Close current tab |

## Section 3: Multi-tab Queries

**Data model per tab**:
```typescript
interface QueryTab {
  id: string
  label: string          // "Query 1" or saved query name
  sql: string
  cursorPos: number
  result: PgQueryResult | null
  error: string | null
  loading: boolean
}
```

**Behavior**:
- New tab: `+` button or Ctrl+T, creates blank "Query N"
- Close: `x` on tab or Ctrl+W, no unsaved warning
- Rename: double-click tab title -> inline input
- From schema browser: double-click table -> new tab with SELECT * pre-executed
- From saved queries: click -> open in new tab with query label
- Persistence: module-level singleton state, survives navigation
- Limit: max 10 tabs

**Visual**:
- Active tab: 2px bottom accent border, text-primary
- Inactive: text-muted, hover surface-hover
- Dot indicator for modified content, spinner for running query
- `x` appears on hover of inactive tabs

## Section 4: Schema Browser

**Tree structure**:
```
> public
  > Tables (12)
      users         ~1.2K  2.1MB
      orders        ~45K   12MB
  > Views (3)
  > Indexes (8)
  > Functions (5)
> auth
```

**Node types**:
- Tables: grid icon, row count + size
- Views: eye icon, same click behavior as tables
- Indexes: bolt icon, columns + unique badge
- Functions: f icon, signature (args + return)

**Context menu (right-click)**:
| Node | Actions |
|------|---------|
| Schema | Refresh, Create Table |
| Table | SELECT *, SELECT COUNT(*), DROP TABLE, Export CSV, Export JSON |
| View | SELECT *, DROP VIEW |
| Index | DROP INDEX |
| Function | CALL/SELECT, DROP FUNCTION |

**Interactions**:
- Single click table: show columns in inline panel (name, type, PK, nullable, default, FK)
- Double click table/view: new tab with SELECT * pre-executed
- Search bar at top: real-time filter across all nodes
- Refresh button in header
- Rename table: inline input on right-click > Rename

**New backend commands**: `pg_list_views`, `pg_list_functions`, `pg_drop_object`, `pg_create_table`, `pg_add_column`, `pg_rename_table`

## Section 5: EXPLAIN ANALYZE Visual

**Trigger**: Explain button or Ctrl+Shift+E. Wraps query with `EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON)`.

**Rendering**: Tree plan view replacing results grid (toggle between Results/Explain).

Each node displays:
- Operation type (Seq Scan, Index Scan, Hash Join, Nested Loop, Sort, Aggregate...)
- Cost bar: proportional to node time vs total. Green (<20%), yellow (20-50%), red (>50%)
- Metrics: actual time, rows, loops, buffers
- Filters and index used

**Implementation**:
- JSON parsing frontend-side
- CSS Grid/Flexbox layout, no graph library
- `PmExplainNode.vue` component per node
- CSS lines for connections between nodes
- Staggered entrance animation (50ms per node, left to right)
- Hover tooltip with full JSON details
- Bottleneck nodes: 3px left border danger + 5% red tint

**Backend**: reuses `pg_execute_query` with EXPLAIN-wrapped SQL, no new command.

## Section 6: Export CSV/JSON

**Dropdown menu on Export button**:
- Export results as CSV / JSON (re-runs query without LIMIT)
- Export table as CSV / JSON (visible when table selected)

**Flow**:
1. Click export option
2. Tauri native save dialog (`tauri-plugin-dialog`) with pre-filled filename
3. Backend streams data directly to file
4. Success/error toast

**Backend commands**: `pg_export_csv`, `pg_export_json` (both take connection_id, sql, file_path)

**Formats**:
- CSV: UTF-8 BOM for Excel, NULL -> empty, timestamps ISO 8601
- JSON: array of objects, one per row

**Dependency**: `tauri-plugin-dialog` for native file picker.

## Section 7: Schema Management

**Create Table** (modal with Form/SQL Preview tabs):
- Fields: table name, schema select, columns grid (name, type, PK, nullable, default)
- SQL Preview tab: live-generated CREATE TABLE in read-only CodeMirror
- Available types: serial, bigserial, integer, bigint, text, varchar, boolean, timestamp, timestamptz, date, uuid, jsonb, numeric, float8, bytea

**Add Column** (simple modal from right-click on table):
- Fields: name, type, nullable, default
- Generates ALTER TABLE ADD COLUMN

**Drop Table/View/Index/Function** (confirmation modal):
- Shows the DROP SQL that will be executed
- Row count warning for tables
- Type-to-confirm (table name) for safety
- Danger-styled button

**Rename Table** (inline in schema browser):
- Click rename in context menu -> name becomes editable input
- Enter to confirm, Escape to cancel
- Generates ALTER TABLE RENAME TO

**Backend commands**: `pg_create_table`, `pg_add_column`, `pg_drop_object`, `pg_rename_table`

## Section 8: Design & Aesthetics

**DB-specific semantic tokens** (all 4 theme files):
```css
--pm-db-keyword    /* SQL keywords */
--pm-db-string     /* string literals */
--pm-db-number     /* numeric literals */
--pm-db-comment    /* comments */
--pm-db-table      /* table/view names */
--pm-db-column     /* column names */
--pm-db-function   /* SQL functions */
```

**Schema browser**: surface bg, SVG icons per type, monospace row counts, search bar, left-border 2px accent on selected item.

**Tab bar**: 32px height, active = 2px bottom accent border, inactive = text-muted, `+` with dashed border, horizontal scroll overflow.

**CodeMirror**: surface-elevated bg, accent cursor 2px, selection = accent-glow 15%, autocomplete popup with type icons (T/C/K/F).

**Toolbar**: 36px, surface bg, compact buttons, stats right-aligned with color-coded duration (green <100ms, orange <1s, red >1s).

**Results grid**: font-mono 12px, sticky header, alternating row backgrounds (surface/transparent), NULL in italic muted, JSON cells truncated with `{}` expand, edited cells accent-glow bg.

**EXPLAIN view**: node cards surface-elevated, 4px cost gradient bar (green to red), SVG path connections, bottleneck = 3px left danger border, staggered entrance.

**Context menu**: surface-elevated, shadow, 32px items with SVG icons, danger actions in red.

**Animations**: tab switch crossfade 100ms, tree expand/collapse 200ms, EXPLAIN nodes pm-slide-up staggered, context menu pm-fade-in 100ms, modals scale 0.95->1.

---

## Files Inventory

**New files**:
- `src/components/db/PmSqlEditor.vue` (CodeMirror 6 wrapper)
- `src/components/db/PmQueryTabs.vue` (tab bar)
- `src/components/db/PmSchemaTree.vue` (enriched schema browser)
- `src/components/db/PmContextMenu.vue` (right-click menu)
- `src/components/db/PmExplainView.vue` (EXPLAIN tree renderer)
- `src/components/db/PmExplainNode.vue` (single plan node)
- `src/components/db/PmCreateTableModal.vue` (create table form)
- `src/components/db/PmDropConfirmModal.vue` (drop confirmation)
- `src/components/db/PmAddColumnModal.vue` (add column form)

**Modified files**:
- `src/views/DatabaseView.vue` (full rewrite with new layout)
- `src/composables/usePgManager.ts` (add tabs, views, functions, schema mgmt methods)
- `src/types/pgmanager.ts` (add QueryTab, PgViewInfo, PgFunctionInfo, ExplainNode types)
- `src/assets/tokens/themes/dark.css` (add --pm-db-* tokens)
- `src/assets/tokens/themes/light.css` (add --pm-db-* tokens)
- `src/assets/tokens/themes/cyberpunk.css` (add --pm-db-* tokens)
- `src/assets/tokens/themes/matrix.css` (add --pm-db-* tokens)
- `src-tauri/src/pgmanager.rs` (add views, functions, DDL, export commands)
- `src-tauri/Cargo.toml` (add tauri-plugin-dialog)
- `src-tauri/src/lib.rs` (register new commands, add dialog plugin)
- `package.json` (add codemirror, sql-formatter, @tauri-apps/plugin-dialog)

## Verification

1. `cargo build` - Rust compilation with new commands
2. `npx vue-tsc --noEmit` - TypeScript check
3. `npx vite build` - Frontend build
4. Manual test: connect to PG via forward, browse schemas, run query with autocomplete, switch tabs, EXPLAIN a query, export CSV, create table, drop table
