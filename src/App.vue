<script setup lang="ts">
import { ref, onMounted } from 'vue'
import AppLayout from '@/components/layout/AppLayout.vue'

const currentTheme = ref('dark')

function setTheme(theme: string) {
  currentTheme.value = theme
  document.documentElement.setAttribute('data-theme', theme)
}

onMounted(() => {
  document.documentElement.setAttribute('data-theme', currentTheme.value)
})
</script>

<template>
  <AppLayout :current-theme="currentTheme" @theme-change="setTheme">
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
