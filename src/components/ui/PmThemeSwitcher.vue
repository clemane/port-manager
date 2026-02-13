<script setup lang="ts">
const props = defineProps<{
  current: string
  themes: { value: string; label: string }[]
  collapsed?: boolean
}>()

defineEmits<{
  change: [theme: string]
}>()

const themeColors: Record<string, string> = {
  dark: '#58a6ff',
  light: '#0969da',
  cyberpunk: '#d946ef',
  matrix: '#00ff41',
}

function nextTheme() {
  const idx = props.themes.findIndex(t => t.value === props.current)
  const next = props.themes[(idx + 1) % props.themes.length]
  return next.value
}
</script>

<template>
  <div v-if="!collapsed" class="pm-theme-switcher">
    <button
      v-for="theme in themes"
      :key="theme.value"
      class="pm-theme-switcher__btn"
      :class="{ 'pm-theme-switcher__btn--active': theme.value === current }"
      @click="$emit('change', theme.value)"
    >
      <span
        class="pm-theme-switcher__dot"
        :style="{ background: themeColors[theme.value] || 'var(--pm-accent)' }"
      />
      {{ theme.label }}
    </button>
  </div>
  <button
    v-else
    class="pm-theme-switcher__cycle"
    title="Switch theme"
    @click="$emit('change', nextTheme())"
  >
    <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" width="18" height="18">
      <circle cx="8" cy="8" r="3" />
      <path d="M8 1v2m0 10v2m-7-7h2m10 0h2m-2.5-5.5L11 4.5m-6 7l-1.5 1.5m9-1.5L11 11.5m-6-7L3.5 3" stroke-linecap="round" />
    </svg>
  </button>
</template>

<style scoped>
.pm-theme-switcher {
  display: flex;
  gap: 2px;
  background: var(--pm-surface);
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-sm);
  padding: 2px;
}
.pm-theme-switcher__btn {
  display: flex;
  align-items: center;
  gap: 6px;
  background: none;
  border: none;
  color: var(--pm-text-muted);
  padding: 4px 8px;
  font-size: 11px;
  border-radius: var(--pm-radius-sm);
  cursor: pointer;
  font-family: var(--pm-font-body);
  transition: background 0.15s, color 0.15s;
}
.pm-theme-switcher__btn:hover { color: var(--pm-text-primary); }
.pm-theme-switcher__btn--active {
  background: var(--pm-accent);
  color: var(--pm-accent-text);
}
.pm-theme-switcher__dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}
.pm-theme-switcher__btn--active .pm-theme-switcher__dot {
  box-shadow: 0 0 4px currentColor;
}
.pm-theme-switcher__cycle {
  background: transparent;
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-sm);
  color: var(--pm-text-muted);
  padding: 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: color 0.15s, border-color 0.15s;
}
.pm-theme-switcher__cycle:hover {
  color: var(--pm-accent);
  border-color: var(--pm-accent);
}
</style>
