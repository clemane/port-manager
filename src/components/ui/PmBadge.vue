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
  gap: 6px;
  padding: 2px 8px;
  border-radius: 9999px;
  font-size: 11px;
  font-weight: 500;
  font-family: var(--pm-font-body);
}
.pm-badge::before {
  content: '';
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}
.pm-badge--running {
  background: var(--pm-badge-running-bg);
  color: var(--pm-badge-running-text);
  border: 1px solid color-mix(in srgb, var(--pm-badge-running-text) 20%, transparent);
}
.pm-badge--running::before { background: var(--pm-badge-running-text); }
.pm-badge--error {
  background: var(--pm-badge-error-bg);
  color: var(--pm-badge-error-text);
  border: 1px solid color-mix(in srgb, var(--pm-badge-error-text) 20%, transparent);
}
.pm-badge--error::before { background: var(--pm-badge-error-text); }
.pm-badge--stopped {
  background: var(--pm-badge-stopped-bg);
  color: var(--pm-badge-stopped-text);
  border: 1px solid color-mix(in srgb, var(--pm-badge-stopped-text) 20%, transparent);
}
.pm-badge--stopped::before { background: var(--pm-badge-stopped-text); }
.pm-badge--info {
  background: var(--pm-surface-hover);
  color: var(--pm-info);
  border: 1px solid color-mix(in srgb, var(--pm-info) 20%, transparent);
}
.pm-badge--info::before { background: var(--pm-info); }
</style>
