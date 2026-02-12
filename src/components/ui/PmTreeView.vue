<script setup lang="ts">
import { ref } from 'vue'
import type { TreeNode } from '@/types/tree'

defineProps<{
  nodes: TreeNode[]
  depth?: number
}>()

const emit = defineEmits<{
  select: [node: TreeNode]
}>()

const expanded = ref<Set<string>>(new Set())

function toggle(id: string) {
  if (expanded.value.has(id)) {
    expanded.value.delete(id)
  } else {
    expanded.value.add(id)
  }
}
</script>

<template>
  <div class="pm-tree">
    <div
      v-for="node in nodes"
      :key="node.id"
      class="pm-tree__node"
    >
      <div
        class="pm-tree__item"
        :style="{ paddingLeft: `${(depth ?? 0) * 16 + 8}px` }"
        @click="node.children ? toggle(node.id) : emit('select', node)"
      >
        <span v-if="node.children" class="pm-tree__toggle">
          {{ expanded.has(node.id) ? '\u25BE' : '\u25B8' }}
        </span>
        <span v-else class="pm-tree__spacer" />
        <span class="pm-tree__label">{{ node.label }}</span>
      </div>
      <PmTreeView
        v-if="node.children && expanded.has(node.id)"
        :nodes="node.children"
        :depth="(depth ?? 0) + 1"
        @select="emit('select', $event)"
      />
    </div>
  </div>
</template>

<style scoped>
.pm-tree__item {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  cursor: pointer;
  color: var(--pm-text-primary);
  font-size: 13px;
  border-radius: var(--pm-radius-sm);
  transition: background 0.1s;
}
.pm-tree__item:hover { background: var(--pm-surface-hover); }
.pm-tree__toggle {
  width: 16px;
  text-align: center;
  font-size: 10px;
  color: var(--pm-text-muted);
}
.pm-tree__spacer { width: 16px; }
.pm-tree__label { flex: 1; }
</style>
