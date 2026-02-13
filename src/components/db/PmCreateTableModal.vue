<script setup lang="ts">
import { ref, computed } from 'vue'
import PmModal from '../ui/PmModal.vue'
import PmButton from '../ui/PmButton.vue'
import PmInput from '../ui/PmInput.vue'
import PmSelect from '../ui/PmSelect.vue'

interface ColumnDef {
  name: string
  data_type: string
  is_primary_key: boolean
  is_nullable: boolean
  default_value: string
}

const props = defineProps<{
  visible: boolean
  schema: string
}>()

const emit = defineEmits<{
  close: []
  create: [schema: string, tableName: string, columns: { name: string; data_type: string; is_primary_key: boolean; is_nullable: boolean; default_value: string | null }[]]
}>()

const tableName = ref('')
const activePreviewTab = ref<'form' | 'sql'>('form')

function emptyColumn(): ColumnDef {
  return { name: '', data_type: 'text', is_primary_key: false, is_nullable: true, default_value: '' }
}

const columns = ref<ColumnDef[]>([emptyColumn()])

const columnTypes = [
  'serial', 'bigserial', 'integer', 'bigint', 'text', 'varchar',
  'boolean', 'timestamp', 'timestamptz', 'date', 'uuid', 'jsonb',
  'numeric', 'float8', 'bytea',
]

const columnTypeOptions = columnTypes.map(t => ({ value: t, label: t }))

function addColumn() {
  columns.value.push(emptyColumn())
}

function removeColumn(index: number) {
  if (columns.value.length > 1) {
    columns.value.splice(index, 1)
  }
}

const generatedSql = computed(() => {
  if (!tableName.value) return ''
  const cols = columns.value
    .filter(c => c.name)
    .map(c => {
      let def = `"${c.name}" ${c.data_type}`
      if (c.is_primary_key) def += ' PRIMARY KEY'
      if (!c.is_nullable && !c.is_primary_key) def += ' NOT NULL'
      if (c.default_value) def += ` DEFAULT ${c.default_value}`
      return def
    })
  return `CREATE TABLE "${props.schema}"."${tableName.value}" (\n  ${cols.join(',\n  ')}\n);`
})

const canCreate = computed(() => {
  return tableName.value.trim() !== '' && columns.value.some(c => c.name.trim() !== '')
})

function handleCreate() {
  const validColumns = columns.value
    .filter(c => c.name.trim() !== '')
    .map(c => ({
      name: c.name,
      data_type: c.data_type,
      is_primary_key: c.is_primary_key,
      is_nullable: c.is_nullable,
      default_value: c.default_value || null,
    }))
  emit('create', props.schema, tableName.value, validColumns)
}

function handleClose() {
  tableName.value = ''
  activePreviewTab.value = 'form'
  columns.value = [emptyColumn()]
  emit('close')
}
</script>

<template>
  <PmModal :open="visible" :title="`Create Table in ${schema}`" @close="handleClose">
    <div class="create-table">
      <!-- Tab switcher -->
      <div class="tab-bar">
        <button
          class="tab-btn"
          :class="{ 'tab-btn--active': activePreviewTab === 'form' }"
          @click="activePreviewTab = 'form'"
        >
          Form
        </button>
        <button
          class="tab-btn"
          :class="{ 'tab-btn--active': activePreviewTab === 'sql' }"
          @click="activePreviewTab = 'sql'"
        >
          SQL Preview
        </button>
      </div>

      <!-- Form tab -->
      <div v-if="activePreviewTab === 'form'" class="form-tab">
        <div class="field">
          <label class="field__label">Table Name</label>
          <PmInput v-model="tableName" placeholder="my_table" />
        </div>

        <div class="columns-section">
          <label class="field__label">Columns</label>
          <div class="columns-grid">
            <div class="columns-header">
              <span class="col-name">Name</span>
              <span class="col-type">Type</span>
              <span class="col-pk">PK</span>
              <span class="col-nullable">Nullable</span>
              <span class="col-default">Default</span>
              <span class="col-actions" />
            </div>
            <div v-for="(col, i) in columns" :key="i" class="column-row">
              <div class="col-name">
                <PmInput v-model="col.name" placeholder="column_name" />
              </div>
              <div class="col-type">
                <PmSelect
                  v-model="col.data_type"
                  :options="columnTypeOptions"
                  :searchable="true"
                />
              </div>
              <div class="col-pk">
                <input
                  v-model="col.is_primary_key"
                  type="checkbox"
                  class="checkbox"
                />
              </div>
              <div class="col-nullable">
                <input
                  v-model="col.is_nullable"
                  type="checkbox"
                  class="checkbox"
                />
              </div>
              <div class="col-default">
                <PmInput v-model="col.default_value" placeholder="DEFAULT" />
              </div>
              <div class="col-actions">
                <button
                  class="remove-btn"
                  :disabled="columns.length <= 1"
                  @click="removeColumn(i)"
                  title="Remove column"
                >
                  &times;
                </button>
              </div>
            </div>
          </div>
          <PmButton variant="ghost" size="sm" @click="addColumn">+ Add Column</PmButton>
        </div>
      </div>

      <!-- SQL Preview tab -->
      <div v-if="activePreviewTab === 'sql'" class="sql-tab">
        <pre v-if="generatedSql" class="sql-preview">{{ generatedSql }}</pre>
        <div v-else class="sql-empty">Enter a table name and at least one column to see the SQL preview.</div>
      </div>
    </div>

    <template #footer>
      <PmButton variant="ghost" @click="handleClose">Cancel</PmButton>
      <PmButton :disabled="!canCreate" @click="handleCreate">Create Table</PmButton>
    </template>
  </PmModal>
</template>

<style scoped>
.create-table {
  min-width: 560px;
}

.tab-bar {
  display: flex;
  gap: 0;
  border-bottom: 1px solid var(--pm-border-subtle);
  margin-bottom: 16px;
}

.tab-btn {
  padding: 6px 16px;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--pm-text-secondary);
  font-size: 12px;
  font-family: var(--pm-font-body);
  cursor: pointer;
  transition: color 0.15s, border-color 0.15s;
}
.tab-btn:hover {
  color: var(--pm-text-primary);
}
.tab-btn--active {
  color: var(--pm-accent);
  border-bottom-color: var(--pm-accent);
}

.field {
  margin-bottom: 16px;
}
.field__label {
  display: block;
  font-size: 12px;
  font-weight: 500;
  color: var(--pm-text-secondary);
  margin-bottom: 6px;
}

.columns-section {
  margin-top: 4px;
}

.columns-grid {
  margin-bottom: 8px;
}

.columns-header {
  display: grid;
  grid-template-columns: 1.5fr 1fr 36px 56px 1fr 32px;
  gap: 6px;
  padding: 0 0 6px 0;
  font-size: 11px;
  font-weight: 500;
  color: var(--pm-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.column-row {
  display: grid;
  grid-template-columns: 1.5fr 1fr 36px 56px 1fr 32px;
  gap: 6px;
  align-items: center;
  margin-bottom: 6px;
}

.col-pk,
.col-nullable {
  display: flex;
  align-items: center;
  justify-content: center;
}

.checkbox {
  width: 14px;
  height: 14px;
  accent-color: var(--pm-accent);
  cursor: pointer;
}

.remove-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: none;
  border: none;
  border-radius: var(--pm-radius-sm);
  color: var(--pm-text-muted);
  font-size: 16px;
  cursor: pointer;
  transition: background 0.15s, color 0.15s;
}
.remove-btn:hover:not(:disabled) {
  background: rgba(248, 81, 73, 0.1);
  color: var(--pm-danger);
}
.remove-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.sql-tab {
  min-height: 120px;
}

.sql-preview {
  background: var(--pm-surface-elevated);
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-sm);
  padding: 12px 16px;
  font-family: var(--pm-font-mono);
  font-size: 12px;
  color: var(--pm-text-primary);
  white-space: pre-wrap;
  word-break: break-word;
  margin: 0;
  line-height: 1.6;
}

.sql-empty {
  color: var(--pm-text-muted);
  font-size: 13px;
  text-align: center;
  padding: 32px 0;
}
</style>
