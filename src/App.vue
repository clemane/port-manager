<script setup lang="ts">
import { onMounted } from 'vue'
import AppLayout from '@/components/layout/AppLayout.vue'
import { useTheme, type Theme } from '@/composables/useTheme'

const { currentTheme, loadTheme, setTheme } = useTheme()

onMounted(async () => {
  await loadTheme()
})

function onThemeChange(theme: string) {
  setTheme(theme as Theme)
}
</script>

<template>
  <AppLayout :current-theme="currentTheme" @theme-change="onThemeChange">
    <router-view />
  </AppLayout>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  background: var(--pm-bg);
  color: var(--pm-text-primary);
  -webkit-font-smoothing: antialiased;
}

::-webkit-scrollbar { width: 6px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: var(--pm-border); border-radius: 3px; }
</style>
