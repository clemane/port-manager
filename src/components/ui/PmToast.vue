<script setup lang="ts">
defineProps<{
  message: string
  type?: 'success' | 'error' | 'info' | 'warning'
  visible: boolean
}>()
</script>

<template>
  <Transition name="toast">
    <div v-if="visible" class="pm-toast" :class="`pm-toast--${type ?? 'info'}`">
      {{ message }}
    </div>
  </Transition>
</template>

<style scoped>
.pm-toast {
  position: fixed;
  top: 16px;
  right: 16px;
  padding: 10px 16px;
  border-radius: var(--pm-radius);
  font-size: 13px;
  z-index: 2000;
  box-shadow: var(--pm-shadow);
}
.pm-toast--success { background: var(--pm-success); color: var(--white); }
.pm-toast--error { background: var(--pm-danger); color: var(--white); }
.pm-toast--info { background: var(--pm-info); color: var(--white); }
.pm-toast--warning { background: var(--pm-warning); color: var(--black); }

.toast-enter-active { transition: all 0.3s ease; }
.toast-leave-active { transition: all 0.3s ease; }
.toast-enter-from { opacity: 0; transform: translateX(30px); }
.toast-leave-to { opacity: 0; transform: translateX(30px); }
</style>
