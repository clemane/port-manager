<script setup lang="ts">
import { computed } from 'vue'
import PmModal from './PmModal.vue'
import PmButton from './PmButton.vue'
import PmBadge from './PmBadge.vue'
import type { DetectedCredentials } from '@/types/k8s'

const props = defineProps<{
  open: boolean
  credentials: DetectedCredentials[]
  loading: boolean
  error: string | null
}>()

defineEmits<{
  close: []
  select: [cred: DetectedCredentials]
  manual: []
}>()

function formatCredential(cred: DetectedCredentials): string {
  const user = cred.username ?? '?'
  const host = cred.host ?? '?'
  const port = cred.port ?? '?'
  const db = cred.database ?? '?'
  return `${user}@${host}:${port}/${db}`
}

function confidenceVariant(confidence: number): 'running' | 'info' | 'error' {
  if (confidence >= 0.75) return 'running'
  if (confidence >= 0.5) return 'info'
  return 'error'
}

function confidenceLabel(confidence: number): string {
  return `${Math.round(confidence * 100)}%`
}

const showEmpty = computed(() => !props.loading && !props.error && props.credentials.length === 0)
</script>

<template>
  <PmModal :open="open" title="Detected DB Credentials" @close="$emit('close')">
    <div class="pm-cred-picker">
      <!-- Loading state -->
      <div v-if="loading" class="pm-cred-picker__loading">
        <span class="pm-cred-picker__spinner" />
        <span class="pm-cred-picker__loading-text">Scanning K8s secrets and pods...</span>
      </div>

      <!-- Error state -->
      <div v-else-if="error" class="pm-cred-picker__error">
        {{ error }}
      </div>

      <!-- Empty state -->
      <div v-else-if="showEmpty" class="pm-cred-picker__empty">
        No credentials found in this namespace
      </div>

      <!-- Credentials list -->
      <div v-else class="pm-cred-picker__list">
        <div
          v-for="(cred, index) in credentials"
          :key="index"
          class="pm-cred-picker__card"
        >
          <div class="pm-cred-picker__card-info">
            <div class="pm-cred-picker__card-header">
              <span class="pm-cred-picker__source">{{ cred.source }}</span>
              <PmBadge :variant="confidenceVariant(cred.confidence)">
                {{ confidenceLabel(cred.confidence) }}
              </PmBadge>
            </div>
            <span class="pm-cred-picker__details">{{ formatCredential(cred) }}</span>
          </div>
          <PmButton size="sm" @click="$emit('select', cred)">Use</PmButton>
        </div>
      </div>
    </div>

    <template #footer>
      <PmButton variant="ghost" @click="$emit('close')">Cancel</PmButton>
      <PmButton variant="ghost" @click="$emit('manual')">Manual</PmButton>
    </template>
  </PmModal>
</template>

<style scoped>
.pm-cred-picker {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-width: 420px;
}

.pm-cred-picker__loading {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 16px 0;
}

.pm-cred-picker__spinner {
  display: inline-block;
  width: 14px;
  height: 14px;
  border: 2px solid transparent;
  border-top-color: var(--pm-accent);
  border-radius: 50%;
  animation: pm-spin 0.6s linear infinite;
}

.pm-cred-picker__loading-text {
  font-family: var(--pm-font-body);
  font-size: 13px;
  color: var(--pm-text-secondary);
}

.pm-cred-picker__error {
  font-family: var(--pm-font-body);
  font-size: 13px;
  color: var(--pm-danger);
  padding: 12px;
  border-radius: var(--pm-radius-sm);
  background: color-mix(in srgb, var(--pm-danger) 10%, transparent);
  border: 1px solid color-mix(in srgb, var(--pm-danger) 25%, transparent);
}

.pm-cred-picker__empty {
  font-family: var(--pm-font-body);
  font-size: 13px;
  color: var(--pm-text-muted);
  text-align: center;
  padding: 24px 0;
}

.pm-cred-picker__list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.pm-cred-picker__card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: var(--pm-surface-elevated);
  border: 1px solid var(--pm-border-subtle);
  border-radius: var(--pm-radius-md);
  transition: border-color 0.15s ease;
}

.pm-cred-picker__card:hover {
  border-color: var(--pm-border);
}

.pm-cred-picker__card-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
  min-width: 0;
}

.pm-cred-picker__card-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.pm-cred-picker__source {
  font-family: var(--pm-font-mono);
  font-size: 13px;
  font-weight: 500;
  color: var(--pm-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.pm-cred-picker__details {
  font-family: var(--pm-font-mono);
  font-size: 12px;
  color: var(--pm-text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
