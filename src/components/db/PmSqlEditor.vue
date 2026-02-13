<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch, toRefs } from 'vue'
import { EditorView, keymap, lineNumbers, highlightActiveLine, drawSelection } from '@codemirror/view'
import { EditorState, Compartment } from '@codemirror/state'
import { sql, PostgreSQL } from '@codemirror/lang-sql'
import { autocompletion } from '@codemirror/autocomplete'
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands'
import { syntaxHighlighting, HighlightStyle, bracketMatching } from '@codemirror/language'
import { searchKeymap, highlightSelectionMatches } from '@codemirror/search'
import { tags } from '@lezer/highlight'
import { format as formatSQL } from 'sql-formatter'

const props = withDefaults(defineProps<{
  modelValue?: string
  tables?: string[]
  columns?: Record<string, string[]>
  schemas?: string[]
  readOnly?: boolean
}>(), {
  modelValue: '',
  tables: () => [],
  columns: () => ({}),
  schemas: () => [],
  readOnly: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  execute: []
  explain: []
  save: []
  format: []
}>()

const { tables, columns, schemas } = toRefs(props)

const editorRef = ref<HTMLDivElement>()
let view: EditorView | null = null
const sqlCompartment = new Compartment()

// ---------------------------------------------------------------------------
// Theme (reads CSS custom properties at mount time)
// ---------------------------------------------------------------------------
function buildTheme() {
  return EditorView.theme({
    '&': {
      backgroundColor: 'var(--pm-surface-elevated)',
      color: 'var(--pm-text-primary)',
      height: '100%',
    },
    '.cm-content': {
      fontFamily: 'var(--pm-font-mono)',
      fontSize: '13px',
      caretColor: 'var(--pm-accent)',
      lineHeight: '1.5',
    },
    '.cm-cursor': {
      borderLeftColor: 'var(--pm-accent)',
      borderLeftWidth: '2px',
    },
    '.cm-selectionBackground': {
      backgroundColor: 'var(--pm-accent-glow) !important',
    },
    '&.cm-focused .cm-selectionBackground': {
      backgroundColor: 'var(--pm-accent-glow) !important',
    },
    '.cm-gutters': {
      backgroundColor: 'var(--pm-surface)',
      color: 'var(--pm-text-muted)',
      border: 'none',
      borderRight: '1px solid var(--pm-border-subtle)',
    },
    '.cm-activeLineGutter': {
      backgroundColor: 'var(--pm-surface-hover)',
    },
    '.cm-activeLine': {
      backgroundColor: 'var(--pm-surface-hover)',
    },
    '.cm-tooltip.cm-tooltip-autocomplete': {
      backgroundColor: 'var(--pm-surface-elevated)',
      border: '1px solid var(--pm-border)',
      borderRadius: 'var(--pm-radius-sm)',
    },
    '.cm-tooltip.cm-tooltip-autocomplete > ul > li': {
      color: 'var(--pm-text-primary)',
    },
    '.cm-tooltip.cm-tooltip-autocomplete > ul > li[aria-selected]': {
      backgroundColor: 'var(--pm-accent-glow)',
      color: 'var(--pm-text-primary)',
    },
  })
}

// ---------------------------------------------------------------------------
// Syntax highlighting (uses DB semantic tokens)
// ---------------------------------------------------------------------------
function buildHighlightStyle() {
  return HighlightStyle.define([
    { tag: tags.keyword, color: 'var(--pm-db-keyword)', fontWeight: '600' },
    { tag: tags.string, color: 'var(--pm-db-string)' },
    { tag: tags.number, color: 'var(--pm-db-number)' },
    { tag: tags.comment, color: 'var(--pm-db-comment)', fontStyle: 'italic' },
    { tag: tags.typeName, color: 'var(--pm-db-table)' },
    { tag: tags.propertyName, color: 'var(--pm-db-column)' },
    { tag: tags.function(tags.variableName), color: 'var(--pm-db-function)' },
    { tag: tags.operator, color: 'var(--pm-db-keyword)' },
    { tag: tags.punctuation, color: 'var(--pm-text-secondary)' },
  ])
}

// ---------------------------------------------------------------------------
// SQL dialect with schema-aware autocomplete
// ---------------------------------------------------------------------------
function buildSqlConfig() {
  const schema: Record<string, string[]> = {}
  for (const table of props.tables) {
    schema[table] = props.columns[table] ?? []
  }
  return sql({
    dialect: PostgreSQL,
    upperCaseKeywords: true,
    schema,
  })
}

// ---------------------------------------------------------------------------
// Custom keybindings
// ---------------------------------------------------------------------------
const customKeymap = keymap.of([
  { key: 'Ctrl-Enter', run: () => { emit('execute'); return true } },
  { key: 'Ctrl-Shift-e', run: () => { emit('explain'); return true } },
  { key: 'Ctrl-s', run: () => { emit('save'); return true } },
  { key: 'Ctrl-l', run: () => { emit('format'); return true } },
])

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------
onMounted(() => {
  if (!editorRef.value) return

  const state = EditorState.create({
    doc: props.modelValue,
    extensions: [
      customKeymap,
      lineNumbers(),
      highlightActiveLine(),
      drawSelection(),
      bracketMatching(),
      history(),
      keymap.of([...defaultKeymap, ...historyKeymap, ...searchKeymap]),
      highlightSelectionMatches(),
      sqlCompartment.of(buildSqlConfig()),
      autocompletion(),
      syntaxHighlighting(buildHighlightStyle()),
      buildTheme(),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          emit('update:modelValue', update.state.doc.toString())
        }
      }),
      ...(props.readOnly ? [EditorState.readOnly.of(true)] : []),
    ],
  })

  view = new EditorView({ state, parent: editorRef.value })
})

onBeforeUnmount(() => {
  view?.destroy()
  view = null
})

// ---------------------------------------------------------------------------
// Watchers
// ---------------------------------------------------------------------------

// Sync external v-model changes into the editor
watch(() => props.modelValue, (newVal) => {
  if (!view) return
  const current = view.state.doc.toString()
  if (newVal !== current) {
    view.dispatch({
      changes: { from: 0, to: current.length, insert: newVal },
    })
  }
})

// Update SQL autocomplete schema when table/column/schema props change
watch([() => props.tables, () => props.columns, () => props.schemas], () => {
  if (!view) return
  view.dispatch({
    effects: sqlCompartment.reconfigure(buildSqlConfig()),
  })
})

// ---------------------------------------------------------------------------
// Exposed API
// ---------------------------------------------------------------------------
function formatSql() {
  if (!view) return
  const formatted = formatSQL(view.state.doc.toString(), { language: 'postgresql' })
  view.dispatch({
    changes: { from: 0, to: view.state.doc.length, insert: formatted },
  })
}

defineExpose({ formatSql })
</script>

<template>
  <div ref="editorRef" class="pm-sql-editor" />
</template>

<style scoped>
.pm-sql-editor {
  height: 100%;
  overflow: auto;
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius);
}
.pm-sql-editor :deep(.cm-editor) {
  height: 100%;
}
.pm-sql-editor :deep(.cm-scroller) {
  overflow: auto;
}
</style>
