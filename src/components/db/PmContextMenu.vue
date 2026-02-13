<script setup lang="ts">
import { ref, watch, onBeforeUnmount, nextTick } from 'vue'

interface ContextMenuItem {
  label: string
  icon?: string
  danger?: boolean
  separator?: boolean
}

const props = defineProps<{
  items: ContextMenuItem[]
  x: number
  y: number
  visible: boolean
}>()

const emit = defineEmits<{
  select: [index: number]
  close: []
}>()

const menuRef = ref<HTMLDivElement>()

function handleClickOutside(e: MouseEvent) {
  if (menuRef.value && !menuRef.value.contains(e.target as Node)) {
    emit('close')
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    emit('close')
  }
}

watch(() => props.visible, (visible) => {
  if (visible) {
    nextTick(() => {
      document.addEventListener('mousedown', handleClickOutside)
      document.addEventListener('keydown', handleKeydown)
    })
  } else {
    document.removeEventListener('mousedown', handleClickOutside)
    document.removeEventListener('keydown', handleKeydown)
  }
})

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', handleClickOutside)
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <Teleport to="body">
    <div
      v-if="visible"
      ref="menuRef"
      class="pm-context-menu"
      :style="{ left: x + 'px', top: y + 'px' }"
    >
      <template v-for="(item, i) in items" :key="i">
        <div v-if="item.separator" class="menu-separator" />
        <button
          v-else
          class="menu-item"
          :class="{ 'menu-item--danger': item.danger }"
          @click="emit('select', i); emit('close')"
        >
          {{ item.label }}
        </button>
      </template>
    </div>
  </Teleport>
</template>

<style scoped>
.pm-context-menu {
  position: fixed;
  z-index: 1000;
  min-width: 160px;
  background: var(--pm-surface-elevated);
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-sm);
  box-shadow: var(--pm-shadow);
  padding: 4px 0;
  animation: pm-fade-in 0.1s ease;
}

@keyframes pm-fade-in {
  from { opacity: 0; transform: scale(0.95); }
  to { opacity: 1; transform: scale(1); }
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 6px 12px;
  background: none;
  border: none;
  color: var(--pm-text-primary);
  font-size: 12px;
  font-family: var(--pm-font-body);
  cursor: pointer;
  text-align: left;
  transition: background 0.1s;
}
.menu-item:hover {
  background: var(--pm-surface-hover);
}
.menu-item--danger {
  color: var(--pm-danger);
}
.menu-item--danger:hover {
  background: rgba(248, 81, 73, 0.1);
}

.menu-separator {
  height: 1px;
  background: var(--pm-border);
  margin: 4px 0;
}
</style>
