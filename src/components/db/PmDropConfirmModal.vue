<script setup lang="ts">
import { ref, computed } from 'vue'
import PmModal from '../ui/PmModal.vue'
import PmButton from '../ui/PmButton.vue'
import PmInput from '../ui/PmInput.vue'

const props = defineProps<{
  visible: boolean
  objectType: string
  schema: string
  name: string
  rowCount?: number
}>()

const emit = defineEmits<{
  close: []
  confirm: []
}>()

const confirmText = ref('')

const isConfirmed = computed(() => confirmText.value === props.name)

const dropSql = computed(() => `DROP ${props.objectType} "${props.schema}"."${props.name}"`)

function handleConfirm() {
  if (isConfirmed.value) {
    emit('confirm')
  }
}

function handleClose() {
  confirmText.value = ''
  emit('close')
}
</script>

<template>
  <PmModal
    :open="visible"
    :title="`Drop ${objectType.toLowerCase()}`"
    @close="handleClose"
  >
    <div class="drop-confirm">
      <div class="drop-sql">
        <pre class="drop-sql__code">{{ dropSql }}</pre>
      </div>

      <div v-if="objectType === 'TABLE' && rowCount != null" class="warning">
        <svg class="warning__icon" viewBox="0 0 16 16" fill="currentColor" width="16" height="16">
          <path d="M8 1l7 14H1L8 1zm0 3.5v5m0 2v1" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linecap="round" />
        </svg>
        <span>This table contains <strong>{{ rowCount.toLocaleString() }}</strong> rows. This action cannot be undone.</span>
      </div>
      <div v-else class="warning">
        <svg class="warning__icon" viewBox="0 0 16 16" fill="currentColor" width="16" height="16">
          <path d="M8 1l7 14H1L8 1zm0 3.5v5m0 2v1" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linecap="round" />
        </svg>
        <span>This action cannot be undone.</span>
      </div>

      <div class="confirm-field">
        <label class="confirm-field__label">
          Type <strong>{{ name }}</strong> to confirm:
        </label>
        <PmInput
          v-model="confirmText"
          :placeholder="name"
        />
      </div>
    </div>

    <template #footer>
      <PmButton variant="ghost" @click="handleClose">Cancel</PmButton>
      <PmButton variant="danger" :disabled="!isConfirmed" @click="handleConfirm">
        Drop {{ objectType.toLowerCase() }}
      </PmButton>
    </template>
  </PmModal>
</template>

<style scoped>
.drop-confirm {
  min-width: 380px;
}

.drop-sql {
  margin-bottom: 16px;
}

.drop-sql__code {
  background: var(--pm-surface-elevated);
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-sm);
  padding: 10px 14px;
  font-family: var(--pm-font-mono);
  font-size: 12px;
  color: var(--pm-danger);
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
}

.warning {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 10px 12px;
  background: rgba(248, 81, 73, 0.08);
  border: 1px solid rgba(248, 81, 73, 0.2);
  border-radius: var(--pm-radius-sm);
  margin-bottom: 16px;
  font-size: 12px;
  color: var(--pm-text-secondary);
  line-height: 1.5;
}

.warning__icon {
  flex-shrink: 0;
  color: var(--pm-danger);
  margin-top: 1px;
}

.warning strong {
  color: var(--pm-text-primary);
}

.confirm-field {
  margin-bottom: 4px;
}

.confirm-field__label {
  display: block;
  font-size: 12px;
  color: var(--pm-text-secondary);
  margin-bottom: 6px;
}

.confirm-field__label strong {
  color: var(--pm-text-primary);
  font-family: var(--pm-font-mono);
  font-size: 12px;
}
</style>
