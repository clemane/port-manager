<script setup lang="ts">
defineProps<{
  open: boolean
  title?: string
}>()

defineEmits<{
  close: []
}>()
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="pm-modal-overlay" @click.self="$emit('close')">
      <div class="pm-modal">
        <div class="pm-modal__header">
          <h3 class="pm-modal__title">{{ title }}</h3>
          <button class="pm-modal__close" @click="$emit('close')">&times;</button>
        </div>
        <div class="pm-modal__body">
          <slot />
        </div>
        <div class="pm-modal__footer">
          <slot name="footer" />
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.pm-modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}
.pm-modal {
  background: var(--pm-surface);
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-lg);
  box-shadow: var(--pm-shadow);
  min-width: 400px;
  max-width: 90vw;
  max-height: 80vh;
  overflow: auto;
}
.pm-modal__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--pm-border-subtle);
}
.pm-modal__title {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--pm-text-primary);
}
.pm-modal__close {
  background: none;
  border: none;
  color: var(--pm-text-muted);
  font-size: 20px;
  cursor: pointer;
  padding: 0 4px;
  transition: color 0.15s;
}
.pm-modal__close:hover { color: var(--pm-text-primary); }
.pm-modal__body { padding: 20px; }
.pm-modal__footer {
  padding: 12px 20px;
  border-top: 1px solid var(--pm-border-subtle);
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
