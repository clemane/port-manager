<script setup lang="ts">
import PmStatusDot from '@/components/ui/PmStatusDot.vue'

defineProps<{
  forwardCount: number
  listeningPorts: number
  tunnelCount: number
}>()
</script>

<template>
  <div class="status-bar">
    <div class="status-bar__left">
      <router-link to="/forwards" class="status-item status-item--link">
        <PmStatusDot :status="forwardCount > 0 ? 'running' : 'idle'" :size="6" />
        <span>{{ forwardCount }} forward{{ forwardCount !== 1 ? 's' : '' }}</span>
      </router-link>
      <router-link to="/" class="status-item status-item--link">
        <span>{{ listeningPorts }} ports</span>
      </router-link>
      <router-link v-if="tunnelCount > 0" to="/ngrok" class="status-item status-item--link">
        <PmStatusDot status="running" :size="6" />
        <span>{{ tunnelCount }} tunnel{{ tunnelCount !== 1 ? 's' : '' }}</span>
      </router-link>
    </div>
    <div class="status-bar__right">
      <span class="status-item">
        <PmStatusDot status="running" :size="5" />
        <span>Connected</span>
      </span>
      <span class="status-item">Port Manager v0.1.0</span>
    </div>
  </div>
</template>

<style scoped>
.status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  background: var(--pm-surface);
  border-top: 1px solid var(--pm-border);
  font-size: 11px;
  font-family: var(--pm-font-body);
  color: var(--pm-text-muted);
  height: 32px;
  min-height: 32px;
  flex-shrink: 0;
}
.status-bar__left,
.status-bar__right {
  display: flex;
  align-items: center;
  gap: 16px;
}
.status-item {
  display: flex;
  align-items: center;
  gap: 6px;
}
.status-item--link {
  text-decoration: none;
  color: var(--pm-text-muted);
  transition: color 0.15s;
  cursor: pointer;
}
.status-item--link:hover {
  color: var(--pm-text-secondary);
}
</style>
