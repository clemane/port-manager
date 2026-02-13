<script setup lang="ts">
withDefaults(defineProps<{
  status: 'running' | 'error' | 'stopped' | 'idle'
  size?: number
}>(), {
  size: 8,
})
</script>

<template>
  <span
    class="pm-status-dot"
    :class="`pm-status-dot--${status}`"
    :style="{ width: `${size}px`, height: `${size}px` }"
  />
</template>

<style scoped>
.pm-status-dot {
  display: inline-block;
  border-radius: 50%;
  flex-shrink: 0;
}

.pm-status-dot--running {
  background: var(--pm-success);
  animation: pm-pulse 2s ease-in-out infinite;
  will-change: transform, opacity;
}

.pm-status-dot--error {
  background: var(--pm-danger);
  animation: pm-blink 1s ease-in-out infinite;
  will-change: opacity;
}

.pm-status-dot--stopped {
  background: var(--pm-text-muted);
  opacity: 0.5;
}

.pm-status-dot--idle {
  background: var(--pm-text-muted);
  opacity: 0.3;
}
</style>
