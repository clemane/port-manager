<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'

const props = defineProps<{
  active: boolean
}>()

const canvasRef = ref<HTMLCanvasElement>()
let animId = 0
let columns: number[] = []
let activeCtx: CanvasRenderingContext2D | null = null
let activeCvs: HTMLCanvasElement | null = null
let resizeHandler: (() => void) | null = null

const CHARS = 'アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワヲン0123456789ABCDEF'
const FONT_SIZE = 14
const FADE = 'rgba(0, 0, 0, 0.05)'
const COLOR_HEAD = '#ffffff'
const COLOR_TAIL = '#00ff41'

function stop() {
  cancelAnimationFrame(animId)
  if (resizeHandler) {
    window.removeEventListener('resize', resizeHandler)
    resizeHandler = null
  }
  activeCtx = null
  activeCvs = null
}

function resize() {
  if (!activeCvs) return
  activeCvs.width = activeCvs.offsetWidth
  activeCvs.height = activeCvs.offsetHeight
  const colCount = Math.floor(activeCvs.width / FONT_SIZE)
  columns = Array.from({ length: colCount }, () =>
    Math.floor(Math.random() * -40)
  )
}

function draw() {
  if (!activeCtx || !activeCvs) return
  activeCtx.fillStyle = FADE
  activeCtx.fillRect(0, 0, activeCvs.width, activeCvs.height)

  for (let i = 0; i < columns.length; i++) {
    const col = columns[i] ?? 0
    const char = CHARS.charAt(Math.floor(Math.random() * CHARS.length))
    const x = i * FONT_SIZE
    const y = col * FONT_SIZE

    activeCtx.font = `${FONT_SIZE}px monospace`
    activeCtx.fillStyle = COLOR_HEAD
    activeCtx.fillText(char, x, y)

    if (col > 0) {
      const prevChar = CHARS.charAt(Math.floor(Math.random() * CHARS.length))
      activeCtx.fillStyle = COLOR_TAIL
      activeCtx.fillText(prevChar, x, y - FONT_SIZE)
    }

    if (y > activeCvs.height && Math.random() > 0.975) {
      columns[i] = Math.floor(Math.random() * -20)
    }
    columns[i] = col + 1
  }

  animId = requestAnimationFrame(draw)
}

function start() {
  const cvs = canvasRef.value
  if (!cvs) return
  const ctx = cvs.getContext('2d')
  if (!ctx) return

  activeCvs = cvs
  activeCtx = ctx

  resize()
  resizeHandler = resize
  window.addEventListener('resize', resize)
  draw()
}

watch(
  () => props.active,
  (active) => {
    if (active) {
      requestAnimationFrame(() => start())
    } else {
      stop()
      const cvs = canvasRef.value
      if (cvs) {
        const ctx = cvs.getContext('2d')
        if (ctx) ctx.clearRect(0, 0, cvs.width, cvs.height)
      }
    }
  },
  { immediate: true }
)

onMounted(() => {
  if (props.active) start()
})

onUnmounted(() => {
  stop()
})
</script>

<template>
  <canvas
    v-show="active"
    ref="canvasRef"
    class="matrix-rain"
  />
</template>

<style scoped>
.matrix-rain {
  position: fixed;
  inset: 0;
  z-index: 0;
  pointer-events: none;
  opacity: 0.12;
}
</style>
