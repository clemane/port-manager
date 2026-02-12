<script setup lang="ts">
import { useRoute } from 'vue-router'
import { PmThemeSwitcher } from '@/components/ui'

const route = useRoute()

defineProps<{
  currentTheme: string
}>()

defineEmits<{
  themeChange: [theme: string]
}>()

const navItems = [
  { path: '/', label: 'Dashboard', icon: '\u229E' },
  { path: '/k8s', label: 'Kubernetes', icon: '\u2638' },
  { path: '/forwards', label: 'Forwards', icon: '\u21C4' },
  { path: '/settings', label: 'Settings', icon: '\u2699' },
]

const themes = [
  { value: 'dark', label: 'Dark' },
  { value: 'light', label: 'Light' },
  { value: 'cyberpunk', label: 'Cyber' },
]
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar__logo">PM</div>
    <nav class="sidebar__nav">
      <router-link
        v-for="item in navItems"
        :key="item.path"
        :to="item.path"
        class="sidebar__item"
        :class="{ 'sidebar__item--active': route.path === item.path }"
      >
        <span class="sidebar__icon">{{ item.icon }}</span>
        <span class="sidebar__label">{{ item.label }}</span>
      </router-link>
    </nav>
    <div class="sidebar__footer">
      <PmThemeSwitcher
        :current="currentTheme"
        :themes="themes"
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
  padding: 12px 0;
}
.sidebar__logo {
  font-size: 20px;
  font-weight: 700;
  color: var(--pm-accent);
  padding: 8px 16px 20px;
  letter-spacing: 2px;
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
  padding: 8px 12px;
  border-radius: var(--pm-radius-sm);
  color: var(--pm-sidebar-text);
  text-decoration: none;
  font-size: 13px;
}
.sidebar__item:hover { background: var(--pm-sidebar-active); }
.sidebar__item--active {
  background: var(--pm-sidebar-active);
  color: var(--pm-sidebar-text-active);
  font-weight: 500;
}
.sidebar__icon { font-size: 16px; }
.sidebar__footer { padding: 12px 8px; }
</style>
