<script setup lang="ts">
import { useRoute } from 'vue-router'
import PmSidebar from './PmSidebar.vue'
import PmStatusBar from './PmStatusBar.vue'
import PmToastContainer from './PmToastContainer.vue'

const route = useRoute()

defineProps<{
  currentTheme: string
  forwardCount: number
  listeningPorts: number
  tunnelCount?: number
  updateAvailable?: boolean
  newVersion?: string
}>()

defineEmits<{
  themeChange: [theme: string]
  showUpdate: []
}>()
</script>

<template>
  <div class="app-layout">
    <PmSidebar
      :current-theme="currentTheme"
      @theme-change="$emit('themeChange', $event)"
    />
    <div class="app-layout__content">
      <header class="app-layout__header">
        <h1 class="app-layout__title">{{ route.meta.title || 'Port Manager' }}</h1>
        <div class="app-layout__header-actions">
          <slot name="header-actions" />
        </div>
      </header>
      <main class="app-layout__main">
        <router-view v-slot="{ Component }">
          <Transition name="page" mode="out-in">
            <component :is="Component" />
          </Transition>
        </router-view>
      </main>
      <PmStatusBar
        :forward-count="forwardCount"
        :listening-ports="listeningPorts"
        :tunnel-count="tunnelCount ?? 0"
        :update-available="updateAvailable"
        :new-version="newVersion"
        @show-update="$emit('showUpdate')"
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
.app-layout__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 24px;
  height: 56px;
  min-height: 56px;
  border-bottom: 1px solid var(--pm-border);
  background: var(--pm-surface);
}
.app-layout__title {
  font-family: var(--pm-font-display);
  font-size: 18px;
  font-weight: 600;
  color: var(--pm-text-primary);
  margin: 0;
}
.app-layout__header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}
.app-layout__main {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
  background: var(--pm-gradient-ambiance);
}

/* Page transitions */
.page-enter-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}
.page-leave-active {
  transition: opacity 0.1s ease, transform 0.1s ease;
}
.page-enter-from {
  opacity: 0;
  transform: translateY(4px);
}
.page-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
