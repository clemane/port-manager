<script setup lang="ts">
import PmSidebar from './PmSidebar.vue'

defineProps<{
  currentTheme: string
}>()

defineEmits<{
  themeChange: [theme: string]
}>()
</script>

<template>
  <div class="app-layout">
    <PmSidebar
      :current-theme="currentTheme"
      @theme-change="$emit('themeChange', $event)"
    />
    <main class="app-layout__main">
      <slot />
    </main>
  </div>
</template>

<style scoped>
.app-layout {
  display: flex;
  height: 100vh;
  background: var(--pm-bg);
  color: var(--pm-text-primary);
}
.app-layout__main {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
}
</style>
