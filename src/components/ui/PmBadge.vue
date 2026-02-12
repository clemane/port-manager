<script setup lang="ts">
defineProps<{
  variant?: 'running' | 'error' | 'stopped' | 'info'
}>()
</script>

<template>
  <span class="pm-badge" :class="`pm-badge--${variant ?? 'info'}`">
    <slot />
  </span>
</template>

<style scoped>
.pm-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: 9999px;
  font-size: 11px;
  font-weight: 500;
}
.pm-badge--running { background: var(--pm-badge-running-bg); color: var(--pm-badge-running-text); }
.pm-badge--error { background: var(--pm-badge-error-bg); color: var(--pm-badge-error-text); }
.pm-badge--stopped { background: var(--pm-badge-stopped-bg); color: var(--pm-badge-stopped-text); }
.pm-badge--info { background: var(--pm-surface-hover); color: var(--pm-info); }
</style>
