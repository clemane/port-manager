<script setup lang="ts">
import { useToast } from '@/composables/useToast'

const { toasts } = useToast()
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
        {{ toast.message }}
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.toast-container {
  position: fixed;
  top: 16px;
  right: 16px;
  z-index: 2000;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.toast-item {
  padding: 10px 16px;
  border-radius: var(--pm-radius);
  font-size: 13px;
  box-shadow: var(--pm-shadow);
  min-width: 200px;
  max-width: 400px;
}
.toast-item--success { background: var(--pm-success); color: var(--white); }
.toast-item--error { background: var(--pm-danger); color: var(--white); }
.toast-item--info { background: var(--pm-info); color: var(--white); }
.toast-item--warning { background: var(--pm-warning); color: var(--black); }

.toast-enter-active,
.toast-leave-active { transition: all 0.3s ease; }
.toast-enter-from { opacity: 0; transform: translateX(30px); }
.toast-leave-to { opacity: 0; transform: translateX(30px); }
</style>
