<script setup lang="ts">
import { ref } from 'vue'
import { useRoute } from 'vue-router'
import { PmThemeSwitcher } from '@/components/ui'

const route = useRoute()

defineProps<{
  currentTheme: string
}>()

defineEmits<{
  themeChange: [theme: string]
}>()

const collapsed = ref(true)
let hoverTimeout: ReturnType<typeof setTimeout> | null = null

function onMouseEnter() {
  hoverTimeout = setTimeout(() => { collapsed.value = false }, 300)
}

function onMouseLeave() {
  if (hoverTimeout) clearTimeout(hoverTimeout)
  collapsed.value = true
}

function toggleCollapse() {
  if (hoverTimeout) clearTimeout(hoverTimeout)
  collapsed.value = !collapsed.value
}

const navItems = [
  { path: '/', label: 'Dashboard', icon: 'dashboard' },
  { path: '/k8s', label: 'Kubernetes', icon: 'k8s' },
  { path: '/forwards', label: 'Forwards', icon: 'forwards' },
  { path: '/ngrok', label: 'Ngrok', icon: 'ngrok' },
  { path: '/database', label: 'Database', icon: 'database' },
  { path: '/settings', label: 'Settings', icon: 'settings' },
]

const themes = [
  { value: 'dark', label: 'Dark' },
  { value: 'light', label: 'Light' },
  { value: 'cyberpunk', label: 'Cyber' },
  { value: 'matrix', label: 'Matrix' },
]
</script>

<template>
  <aside
    class="sidebar"
    :class="{ 'sidebar--collapsed': collapsed }"
    @mouseenter="onMouseEnter"
    @mouseleave="onMouseLeave"
  >
    <div class="sidebar__header">
      <button class="sidebar__toggle" @click="toggleCollapse">
        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
          <path d="M2 4h12M2 8h12M2 12h12" />
        </svg>
      </button>
      <span class="sidebar__logo" :class="{ 'sidebar__logo--hidden': collapsed }">PM</span>
    </div>

    <nav class="sidebar__nav">
      <router-link
        v-for="item in navItems"
        :key="item.path"
        :to="item.path"
        class="sidebar__item"
        :class="{ 'sidebar__item--active': route.path === item.path }"
      >
        <span class="sidebar__icon">
          <!-- Dashboard -->
          <svg v-if="item.icon === 'dashboard'" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="2" y="2" width="7" height="7" rx="1" />
            <rect x="11" y="2" width="7" height="4" rx="1" />
            <rect x="2" y="11" width="7" height="4" rx="1" />
            <rect x="11" y="8" width="7" height="7" rx="1" />
          </svg>
          <!-- Kubernetes -->
          <svg v-else-if="item.icon === 'k8s'" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5">
            <circle cx="10" cy="10" r="7.5" />
            <path d="M10 5v10M5.5 7.5l9 5M5.5 12.5l9-5" />
          </svg>
          <!-- Forwards -->
          <svg v-else-if="item.icon === 'forwards'" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
            <path d="M3 7h14m-4-4l4 4-4 4" />
            <path d="M17 13H3m4 4l-4-4 4-4" />
          </svg>
          <!-- Ngrok -->
          <svg v-else-if="item.icon === 'ngrok'" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
            <path d="M4 16l6-12 6 12" />
            <path d="M6.5 11h7" />
          </svg>
          <!-- Database -->
          <svg v-else-if="item.icon === 'database'" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5">
            <ellipse cx="10" cy="5" rx="7" ry="3" />
            <path d="M3 5v10c0 1.66 3.13 3 7 3s7-1.34 7-3V5" />
            <path d="M3 10c0 1.66 3.13 3 7 3s7-1.34 7-3" />
          </svg>
          <!-- Settings -->
          <svg v-else-if="item.icon === 'settings'" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5">
            <circle cx="10" cy="10" r="3" />
            <path d="M10 2v3m0 10v3m-8-8h3m10 0h3M4.2 4.2l2.1 2.1m7.4 7.4l2.1 2.1M15.8 4.2l-2.1 2.1M6.3 13.7l-2.1 2.1" stroke-linecap="round" />
          </svg>
        </span>
        <span class="sidebar__label">{{ item.label }}</span>
      </router-link>
    </nav>

    <div class="sidebar__footer">
      <PmThemeSwitcher
        :current="currentTheme"
        :themes="themes"
        :collapsed="collapsed"
        @change="$emit('themeChange', $event)"
      />
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 200px;
  min-width: 200px;
  height: 100vh;
  background: var(--pm-sidebar-bg);
  border-right: 1px solid var(--pm-border);
  display: flex;
  flex-direction: column;
  padding: 8px 0;
  transition: width 200ms ease-out, min-width 200ms ease-out;
  overflow: hidden;
  z-index: 10;
}

.sidebar--collapsed {
  width: 56px;
  min-width: 56px;
}

.sidebar__header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px 16px;
}

.sidebar__toggle {
  background: none;
  border: none;
  color: var(--pm-text-muted);
  cursor: pointer;
  padding: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--pm-radius-sm);
  transition: color 0.15s, background 0.15s;
  flex-shrink: 0;
}
.sidebar__toggle svg {
  width: 18px;
  height: 18px;
}
.sidebar__toggle:hover {
  color: var(--pm-text-primary);
  background: var(--pm-sidebar-active);
}

.sidebar__logo {
  font-family: var(--pm-font-display);
  font-size: 18px;
  font-weight: 700;
  color: var(--pm-accent);
  letter-spacing: 2px;
  white-space: nowrap;
  overflow: hidden;
  transition: opacity 150ms ease, width 200ms ease;
}
.sidebar__logo--hidden {
  opacity: 0;
  width: 0;
}

.sidebar__nav {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 0 8px;
}

.sidebar__item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border-radius: var(--pm-radius-sm);
  color: var(--pm-sidebar-text);
  text-decoration: none;
  font-size: 13px;
  font-family: var(--pm-font-body);
  transition: background 0.15s, color 0.15s;
  position: relative;
  white-space: nowrap;
  overflow: hidden;
}

.sidebar__item:hover {
  background: var(--pm-sidebar-active);
  color: var(--pm-sidebar-text-active);
}

.sidebar__item--active {
  color: var(--pm-sidebar-text-active);
  font-weight: 500;
}

.sidebar__item--active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 6px;
  bottom: 6px;
  width: 3px;
  background: var(--pm-accent);
  border-radius: 0 2px 2px 0;
}

.sidebar__icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  flex-shrink: 0;
}

.sidebar__icon svg {
  width: 20px;
  height: 20px;
}

.sidebar__label {
  overflow: hidden;
  transition: opacity 150ms ease;
}

.sidebar--collapsed .sidebar__label {
  opacity: 0;
  width: 0;
}

.sidebar__footer {
  padding: 12px 8px;
}
</style>
