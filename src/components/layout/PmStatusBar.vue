<script setup lang="ts">
defineProps<{
  forwardCount: number
  listeningPorts: number
}>()
</script>

<template>
  <div class="status-bar">
    <div class="status-bar__left">
      <span class="status-item">
        <span class="status-dot status-dot--active" />
        {{ forwardCount }} forward{{ forwardCount !== 1 ? 's' : '' }} active
      </span>
      <span class="status-item">
        {{ listeningPorts }} ports listening
      </span>
    </div>
    <div class="status-bar__right">
      <span class="status-item">Port Manager v0.1.0</span>
    </div>
  </div>
</template>

<style scoped>
.status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 16px;
  background: var(--pm-surface);
  border-top: 1px solid var(--pm-border);
  font-size: 11px;
  color: var(--pm-text-muted);
  height: 28px;
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
  gap: 4px;
}
.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
}
.status-dot--active { background: var(--pm-success); }
</style>
