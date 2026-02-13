<script setup lang="ts">
import { watch, onBeforeUnmount } from 'vue'

const props = defineProps<{
  open: boolean
  title?: string
}>()

const emit = defineEmits<{
  close: []
}>()

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') emit('close')
}

watch(() => props.open, (open) => {
  if (open) document.addEventListener('keydown', onKeydown)
  else document.removeEventListener('keydown', onKeydown)
})

onBeforeUnmount(() => {
  document.removeEventListener('keydown', onKeydown)
})
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
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
    </Transition>
  </Teleport>
</template>

<style scoped>
.pm-modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
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
  padding: 20px 24px 16px;
}
.pm-modal__title {
  margin: 0;
  font-family: var(--pm-font-display);
  font-size: 16px;
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
.pm-modal__body { padding: 0 24px 24px; }
.pm-modal__footer {
  padding: 16px 24px;
  border-top: 1px solid var(--pm-border-subtle);
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}
.modal-enter-active .pm-modal,
.modal-leave-active .pm-modal {
  transition: transform 0.2s ease;
}
.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
.modal-enter-from .pm-modal {
  transform: scale(0.95);
}
.modal-leave-to .pm-modal {
  transform: scale(0.95);
}
</style>
