<script setup lang="ts">
import { useToast } from '@/composables/useToast'

const { toasts } = useToast()

function iconForType(type: string) {
  switch (type) {
    case 'success': return 'M5 12l3 3 7-7'
    case 'error': return 'M6 6l12 12M18 6l-12 12'
    case 'warning': return 'M12 9v4m0 3h.01M12 3l9.5 16.5H2.5L12 3z'
    default: return 'M12 8v4m0 4h.01'
  }
}

function durationForType(type: string) {
  return type === 'error' ? 5 : 3
}
</script>

<template>
  <div class="toast-container">
    <TransitionGroup name="toast">
      <div
        v-for="toast in toasts"
        :key="toast.id"
        class="toast-item"
        :class="`toast-item--${toast.type}`"
      >
        <svg class="toast-item__icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path :d="iconForType(toast.type)" />
        </svg>
        <span class="toast-item__message">{{ toast.message }}</span>
        <div
          class="toast-item__progress"
          :style="{ animationDuration: `${durationForType(toast.type)}s` }"
        />
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.toast-container {
  position: fixed;
  bottom: 16px;
  right: 16px;
  z-index: 2000;
  display: flex;
  flex-direction: column-reverse;
  gap: 8px;
}
.toast-item {
  position: relative;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px 14px;
  border-radius: var(--pm-radius);
  font-size: 13px;
  font-family: var(--pm-font-body);
  box-shadow: var(--pm-shadow);
  min-width: 240px;
  max-width: 400px;
  overflow: hidden;
}
.toast-item__icon {
  width: 18px;
  height: 18px;
  flex-shrink: 0;
}
.toast-item__message {
  flex: 1;
}
.toast-item__progress {
  position: absolute;
  bottom: 0;
  left: 0;
  height: 3px;
  background: rgba(255, 255, 255, 0.3);
  animation: pm-toast-countdown linear forwards;
}
.toast-item--success { background: var(--pm-success); color: var(--white); }
.toast-item--error { background: var(--pm-danger); color: var(--white); }
.toast-item--info { background: var(--pm-info); color: var(--white); }
.toast-item--warning { background: var(--pm-warning); color: var(--black); }

.toast-enter-active { transition: all 0.3s ease; }
.toast-leave-active { transition: all 0.3s ease; }
.toast-enter-from { opacity: 0; transform: translateY(16px); }
.toast-leave-to { opacity: 0; transform: translateX(30px); }
</style>
