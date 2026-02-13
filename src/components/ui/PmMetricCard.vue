<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount } from 'vue'

const props = withDefaults(defineProps<{
  label: string
  value: number
  color?: 'accent' | 'success' | 'danger' | 'warning' | 'muted'
  animate?: boolean
}>(), {
  color: 'accent',
  animate: true,
})

const displayValue = ref(0)
let rafId = 0

function animateValue(from: number, to: number) {
  if (!props.animate) {
    displayValue.value = to
    return
  }
  cancelAnimationFrame(rafId)
  const duration = 400
  const start = performance.now()
  const startVal = from

  function tick(now: number) {
    const elapsed = now - start
    const progress = Math.min(elapsed / duration, 1)
    const eased = 1 - Math.pow(1 - progress, 3) // ease-out cubic
    displayValue.value = Math.round(startVal + (to - startVal) * eased)
    if (progress < 1) {
      rafId = requestAnimationFrame(tick)
    }
  }

  rafId = requestAnimationFrame(tick)
}

watch(() => props.value, (newVal, oldVal) => {
  animateValue(oldVal ?? 0, newVal)
})

onMounted(() => {
  animateValue(0, props.value)
})

onBeforeUnmount(() => {
  cancelAnimationFrame(rafId)
})
</script>

<template>
  <div class="pm-metric-card" :class="`pm-metric-card--${color}`">
    <div class="pm-metric-card__content">
      <span class="pm-metric-card__value">{{ displayValue }}</span>
      <span class="pm-metric-card__label">{{ label }}</span>
    </div>
    <div class="pm-metric-card__icon">
      <slot name="icon" />
    </div>
  </div>
</template>

<style scoped>
.pm-metric-card {
  background: var(--pm-surface);
  border: 1px solid var(--pm-border);
  border-left: 3px solid var(--pm-accent);
  border-radius: var(--pm-radius);
  padding: 16px 20px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  animation: pm-count-up 0.3s ease-out;
}

.pm-metric-card--accent { border-left-color: var(--pm-accent); }
.pm-metric-card--success { border-left-color: var(--pm-success); }
.pm-metric-card--danger { border-left-color: var(--pm-danger); }
.pm-metric-card--warning { border-left-color: var(--pm-warning); }
.pm-metric-card--muted { border-left-color: var(--pm-text-muted); }

.pm-metric-card__content {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.pm-metric-card__value {
  font-family: var(--pm-font-display);
  font-size: 28px;
  font-weight: 700;
  line-height: 1;
  color: var(--pm-text-primary);
}

.pm-metric-card__label {
  font-family: var(--pm-font-body);
  font-size: 12px;
  color: var(--pm-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.pm-metric-card__icon {
  color: var(--pm-text-muted);
  opacity: 0.5;
  font-size: 24px;
}
</style>
