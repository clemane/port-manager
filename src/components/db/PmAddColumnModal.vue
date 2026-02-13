<script setup lang="ts">
import { ref } from 'vue'
import PmModal from '../ui/PmModal.vue'
import PmButton from '../ui/PmButton.vue'
import PmInput from '../ui/PmInput.vue'
import PmSelect from '../ui/PmSelect.vue'

const props = defineProps<{
  visible: boolean
  schema: string
  table: string
}>()

const emit = defineEmits<{
  close: []
  add: [name: string, dataType: string, isNullable: boolean, defaultValue: string | null]
}>()

const name = ref('')
const dataType = ref('text')
const isNullable = ref(true)
const defaultValue = ref('')

const columnTypes = [
  'serial', 'bigserial', 'integer', 'bigint', 'text', 'varchar',
  'boolean', 'timestamp', 'timestamptz', 'date', 'uuid', 'jsonb',
  'numeric', 'float8', 'bytea',
]

const columnTypeOptions = columnTypes.map(t => ({ value: t, label: t }))

function handleAdd() {
  emit('add', name.value, dataType.value, isNullable.value, defaultValue.value || null)
}

function handleClose() {
  name.value = ''
  dataType.value = 'text'
  isNullable.value = true
  defaultValue.value = ''
  emit('close')
}
</script>

<template>
  <PmModal
    :open="visible"
    :title="`Add Column to ${schema}.${table}`"
    @close="handleClose"
  >
    <div class="add-column-form">
      <div class="field">
        <label class="field__label">Column Name</label>
        <PmInput v-model="name" placeholder="column_name" />
      </div>

      <div class="field">
        <label class="field__label">Data Type</label>
        <PmSelect
          v-model="dataType"
          :options="columnTypeOptions"
          :searchable="true"
        />
      </div>

      <div class="field field--inline">
        <input
          v-model="isNullable"
          type="checkbox"
          class="checkbox"
          id="add-col-nullable"
        />
        <label for="add-col-nullable" class="field__label field__label--inline">Allow NULL</label>
      </div>

      <div class="field">
        <label class="field__label">Default Value</label>
        <PmInput v-model="defaultValue" placeholder="e.g. now(), 0, 'value'" />
      </div>
    </div>

    <template #footer>
      <PmButton variant="ghost" @click="handleClose">Cancel</PmButton>
      <PmButton :disabled="!name.trim()" @click="handleAdd">Add Column</PmButton>
    </template>
  </PmModal>
</template>

<style scoped>
.add-column-form {
  min-width: 360px;
}

.field {
  margin-bottom: 14px;
}

.field--inline {
  display: flex;
  align-items: center;
  gap: 8px;
}

.field__label {
  display: block;
  font-size: 12px;
  font-weight: 500;
  color: var(--pm-text-secondary);
  margin-bottom: 6px;
}

.field__label--inline {
  margin-bottom: 0;
  cursor: pointer;
}

.checkbox {
  width: 14px;
  height: 14px;
  accent-color: var(--pm-accent);
  cursor: pointer;
}
</style>
