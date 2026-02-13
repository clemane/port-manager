<script setup lang="ts">
import { ref, nextTick } from 'vue'
import type { QueryTab } from '@/types/pgmanager'

const props = defineProps<{
  tabs: QueryTab[]
  activeTabId: string
}>()

const emit = defineEmits<{
  select: [tabId: string]
  close: [tabId: string]
  create: []
  rename: [tabId: string, label: string]
}>()

const renamingTabId = ref<string | null>(null)
const renameValue = ref('')

function startRename(tabId: string, currentLabel: string) {
  renamingTabId.value = tabId
  renameValue.value = currentLabel
  nextTick(() => {
    const input = document.querySelector('.tab-rename-input') as HTMLInputElement
    input?.focus()
    input?.select()
  })
}

function commitRename(tabId: string) {
  if (renameValue.value.trim()) {
    emit('rename', tabId, renameValue.value.trim())
  }
  renamingTabId.value = null
}

function cancelRename() {
  renamingTabId.value = null
}
</script>

<template>
  <div class="pm-query-tabs">
    <div class="tabs-scroll">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        class="tab-item"
        :class="{
          'tab-item--active': tab.id === activeTabId,
          'tab-item--loading': tab.loading,
          'tab-item--error': tab.error,
        }"
        @click="emit('select', tab.id)"
        @dblclick.stop="startRename(tab.id, tab.label)"
      >
        <span v-if="tab.loading" class="tab-spinner" />
        <span v-else-if="tab.error" class="tab-error-dot" />

        <input
          v-if="renamingTabId === tab.id"
          v-model="renameValue"
          class="tab-rename-input"
          @blur="commitRename(tab.id)"
          @keydown.enter="commitRename(tab.id)"
          @keydown.escape="cancelRename"
          @click.stop
        />
        <span v-else class="tab-label">{{ tab.label }}</span>

        <button
          v-if="tabs.length > 1"
          class="tab-close"
          @click.stop="emit('close', tab.id)"
        >
          &times;
        </button>
      </button>
    </div>

    <button class="tab-add" @click="emit('create')" title="New query (Ctrl+T)">
      +
    </button>
  </div>
</template>

<style scoped>
.pm-query-tabs {
  display: flex;
  align-items: stretch;
  height: 32px;
  background: var(--pm-surface);
  border-bottom: 1px solid var(--pm-border);
  flex-shrink: 0;
  overflow: hidden;
}

.tabs-scroll {
  display: flex;
  overflow-x: auto;
  flex: 1;
  scrollbar-width: none;
}
.tabs-scroll::-webkit-scrollbar {
  display: none;
}

.tab-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 12px;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--pm-text-muted);
  font-size: 12px;
  font-family: var(--pm-font-body);
  cursor: pointer;
  white-space: nowrap;
  transition: color 0.15s, border-color 0.15s, background 0.15s;
  flex-shrink: 0;
}
.tab-item:hover {
  color: var(--pm-text-secondary);
  background: var(--pm-surface-hover);
}
.tab-item--active {
  color: var(--pm-text-primary);
  border-bottom-color: var(--pm-accent);
}
.tab-item--loading {
  opacity: 0.7;
}

.tab-spinner {
  width: 10px;
  height: 10px;
  border: 2px solid var(--pm-text-muted);
  border-top-color: var(--pm-accent);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}
@keyframes spin {
  to { transform: rotate(360deg); }
}

.tab-error-dot {
  width: 6px;
  height: 6px;
  background: var(--pm-danger);
  border-radius: 50%;
  flex-shrink: 0;
}

.tab-label {
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tab-rename-input {
  background: var(--pm-surface-elevated);
  border: 1px solid var(--pm-accent);
  border-radius: 2px;
  color: var(--pm-text-primary);
  font-size: 12px;
  font-family: var(--pm-font-body);
  padding: 1px 4px;
  width: 100px;
  outline: none;
}

.tab-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  background: none;
  border: none;
  color: var(--pm-text-muted);
  font-size: 14px;
  cursor: pointer;
  border-radius: 2px;
  opacity: 0;
  transition: opacity 0.15s, background 0.15s, color 0.15s;
}
.tab-item:hover .tab-close {
  opacity: 1;
}
.tab-item--active .tab-close {
  opacity: 1;
}
.tab-close:hover {
  background: var(--pm-surface-active);
  color: var(--pm-danger);
}

.tab-add {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  background: none;
  border: none;
  border-left: 1px dashed var(--pm-border);
  color: var(--pm-text-muted);
  font-size: 16px;
  cursor: pointer;
  transition: color 0.15s, background 0.15s;
  flex-shrink: 0;
}
.tab-add:hover {
  color: var(--pm-accent);
  background: var(--pm-surface-hover);
}
</style>
