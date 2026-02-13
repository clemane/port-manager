<script setup lang="ts">
defineProps<{
  variant?: 'primary' | 'ghost' | 'danger' | 'icon'
  size?: 'sm' | 'md' | 'lg'
  disabled?: boolean
  loading?: boolean
}>()
</script>

<template>
  <button
    class="pm-btn"
    :class="[`pm-btn--${variant ?? 'primary'}`, `pm-btn--${size ?? 'md'}`]"
    :disabled="disabled || loading"
  >
    <span v-if="loading" class="pm-btn__spinner" />
    <slot />
  </button>
</template>

<style scoped>
.pm-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  border: 1px solid transparent;
  border-radius: var(--pm-radius-sm);
  font-weight: 500;
  cursor: pointer;
  font-family: var(--pm-font-body);
  transition: all 0.15s ease;
}
.pm-btn:focus-visible {
  outline: none;
  box-shadow: 0 0 0 2px var(--pm-accent-glow);
}
.pm-btn--sm { padding: 4px 10px; font-size: 12px; }
.pm-btn--md { padding: 6px 14px; font-size: 13px; }
.pm-btn--lg { padding: 8px 18px; font-size: 14px; }

.pm-btn--primary {
  background: linear-gradient(180deg, color-mix(in srgb, var(--pm-accent) 100%, white 8%) 0%, var(--pm-accent) 100%);
  color: var(--pm-accent-text);
  box-shadow: 0 1px 3px var(--pm-accent-glow);
}
.pm-btn--primary:hover:not(:disabled) { background: var(--pm-accent-hover); }

.pm-btn--ghost {
  background: transparent;
  color: var(--pm-text-secondary);
  transform: scale(0.97);
}
.pm-btn--ghost:hover:not(:disabled) {
  background: var(--pm-surface-hover);
  color: var(--pm-text-primary);
  transform: scale(1);
}

.pm-btn--danger {
  background: var(--pm-danger);
  color: var(--white);
}
.pm-btn--danger:hover:not(:disabled) {
  background: color-mix(in srgb, var(--pm-danger) 90%, black);
}

.pm-btn--icon {
  background: transparent;
  color: var(--pm-text-secondary);
  padding: 6px;
}
.pm-btn--icon:hover:not(:disabled) {
  background: var(--pm-surface-hover);
  color: var(--pm-text-primary);
}

.pm-btn:disabled { opacity: 0.5; cursor: not-allowed; }

.pm-btn__spinner {
  width: 14px;
  height: 14px;
  border: 2px solid transparent;
  border-top-color: currentColor;
  border-radius: 50%;
  animation: pm-spin 0.6s linear infinite;
}
</style>
