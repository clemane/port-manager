<script setup lang="ts">
import PmSidebar from './PmSidebar.vue'
import PmStatusBar from './PmStatusBar.vue'
import PmToastContainer from './PmToastContainer.vue'

defineProps<{
  currentTheme: string
  forwardCount: number
  listeningPorts: number
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
    <div class="app-layout__content">
      <main class="app-layout__main">
        <slot />
      </main>
      <PmStatusBar
        :forward-count="forwardCount"
        :listening-ports="listeningPorts"
      />
    </div>
    <PmToastContainer />
  </div>
</template>

<style scoped>
.app-layout {
  display: flex;
  height: 100vh;
  background: var(--pm-bg);
  color: var(--pm-text-primary);
}
.app-layout__content {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}
.app-layout__main {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
}
</style>
