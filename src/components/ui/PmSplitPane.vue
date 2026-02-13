<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'

const props = withDefaults(defineProps<{
  direction?: 'horizontal' | 'vertical'
  initialRatio?: number
  minSize?: number
  storageKey?: string
}>(), {
  direction: 'horizontal',
  initialRatio: 0.3,
  minSize: 150,
})

const containerRef = ref<HTMLElement | null>(null)
const ratio = ref(props.initialRatio)
const isDragging = ref(false)

onMounted(() => {
  if (props.storageKey) {
    const saved = localStorage.getItem(`pm-split-${props.storageKey}`)
    if (saved) {
      const parsed = parseFloat(saved)
      if (!Number.isNaN(parsed) && parsed > 0 && parsed < 1) {
        ratio.value = parsed
      }
    }
  }
})

function onMouseDown(e: MouseEvent) {
  e.preventDefault()
  isDragging.value = true
  document.addEventListener('mousemove', onMouseMove)
  document.addEventListener('mouseup', onMouseUp)
}

function onMouseMove(e: MouseEvent) {
  if (!isDragging.value || !containerRef.value) return
  const rect = containerRef.value.getBoundingClientRect()
  const isHorizontal = props.direction === 'horizontal'
  const total = isHorizontal ? rect.width : rect.height
  const pos = isHorizontal ? e.clientX - rect.left : e.clientY - rect.top
  const minRatio = props.minSize / total
  const maxRatio = 1 - minRatio
  ratio.value = Math.min(maxRatio, Math.max(minRatio, pos / total))
}

function onMouseUp() {
  isDragging.value = false
  document.removeEventListener('mousemove', onMouseMove)
  document.removeEventListener('mouseup', onMouseUp)
  if (props.storageKey) {
    localStorage.setItem(`pm-split-${props.storageKey}`, ratio.value.toString())
  }
}

onUnmounted(() => {
  document.removeEventListener('mousemove', onMouseMove)
  document.removeEventListener('mouseup', onMouseUp)
})

const firstStyle = computed(() => {
  const pct = `${ratio.value * 100}%`
  return props.direction === 'horizontal'
    ? { width: pct, minWidth: `${props.minSize}px` }
    : { height: pct, minHeight: `${props.minSize}px` }
})

const secondStyle = computed(() => {
  const pct = `${(1 - ratio.value) * 100}%`
  return props.direction === 'horizontal'
    ? { width: pct, minWidth: `${props.minSize}px` }
    : { height: pct, minHeight: `${props.minSize}px` }
})
</script>

<template>
  <div
    ref="containerRef"
    class="pm-split-pane"
    :class="[
      `pm-split-pane--${direction}`,
      { 'pm-split-pane--dragging': isDragging },
    ]"
  >
    <div class="pm-split-pane__first" :style="firstStyle">
      <slot name="first" />
    </div>
    <div
      class="pm-split-pane__divider"
      @mousedown="onMouseDown"
    />
    <div class="pm-split-pane__second" :style="secondStyle">
      <slot name="second" />
    </div>
  </div>
</template>

<style scoped>
.pm-split-pane {
  display: flex;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

.pm-split-pane--horizontal {
  flex-direction: row;
}

.pm-split-pane--vertical {
  flex-direction: column;
}

.pm-split-pane--dragging {
  user-select: none;
}

.pm-split-pane__first,
.pm-split-pane__second {
  overflow: auto;
}

.pm-split-pane__divider {
  flex-shrink: 0;
  background: var(--pm-border);
  transition: background-color 0.2s ease;
}

.pm-split-pane__divider:hover,
.pm-split-pane--dragging .pm-split-pane__divider {
  background: var(--pm-accent);
}

.pm-split-pane--horizontal > .pm-split-pane__divider {
  width: 4px;
  cursor: col-resize;
}

.pm-split-pane--vertical > .pm-split-pane__divider {
  height: 4px;
  cursor: row-resize;
}
</style>
