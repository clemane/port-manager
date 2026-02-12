<script setup lang="ts">
defineProps<{
  current: string
  themes: { value: string; label: string }[]
}>()

defineEmits<{
  change: [theme: string]
}>()
</script>

<template>
  <div class="pm-theme-switcher">
    <button
      v-for="theme in themes"
      :key="theme.value"
      class="pm-theme-switcher__btn"
      :class="{ 'pm-theme-switcher__btn--active': theme.value === current }"
      @click="$emit('change', theme.value)"
    >
      {{ theme.label }}
    </button>
  </div>
</template>

<style scoped>
.pm-theme-switcher {
  display: flex;
  gap: 4px;
  background: var(--pm-surface);
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-sm);
  padding: 2px;
}
.pm-theme-switcher__btn {
  background: none;
  border: none;
  color: var(--pm-text-muted);
  padding: 4px 8px;
  font-size: 11px;
  border-radius: var(--pm-radius-sm);
  cursor: pointer;
  font-family: inherit;
  transition: background 0.15s, color 0.15s;
}
.pm-theme-switcher__btn:hover { color: var(--pm-text-primary); }
.pm-theme-switcher__btn--active {
  background: var(--pm-accent);
  color: var(--pm-accent-text);
}
</style>
