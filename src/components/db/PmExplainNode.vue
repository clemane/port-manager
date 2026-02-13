<script setup lang="ts">
import type { ExplainNode } from '@/types/pgmanager'

const props = defineProps<{
  node: ExplainNode
  depth: number
}>()

const isBottleneck = props.node.percentOfTotal > 50

function formatTime(ms: number | null): string {
  if (ms === null) return '—'
  if (ms < 1) return `${(ms * 1000).toFixed(0)}μs`
  if (ms < 1000) return `${ms.toFixed(1)}ms`
  return `${(ms / 1000).toFixed(2)}s`
}

function formatRows(n: number | null): string {
  if (n === null) return '—'
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`
  return String(n)
}

function costBarColor(pct: number): string {
  if (pct < 20) return 'var(--pm-success)'
  if (pct < 50) return 'var(--pm-warning)'
  return 'var(--pm-danger)'
}
</script>

<template>
  <div
    class="explain-node"
    :class="{ 'explain-node--bottleneck': isBottleneck }"
    :style="{ animationDelay: `${depth * 50}ms` }"
  >
    <div class="node-card">
      <div class="node-header">
        <span class="node-type">{{ node.type }}</span>
        <span v-if="node.relation" class="node-relation">on {{ node.relation }}</span>
      </div>

      <div class="node-cost-bar">
        <div
          class="node-cost-fill"
          :style="{
            width: `${Math.max(node.percentOfTotal, 2)}%`,
            backgroundColor: costBarColor(node.percentOfTotal),
          }"
        />
      </div>

      <div class="node-metrics">
        <span class="metric">
          <span class="metric-label">Time</span>
          <span class="metric-value">{{ formatTime(node.actualTime) }}</span>
        </span>
        <span class="metric">
          <span class="metric-label">Rows</span>
          <span class="metric-value">{{ formatRows(node.actualRows) }}</span>
        </span>
        <span class="metric">
          <span class="metric-label">Loops</span>
          <span class="metric-value">{{ node.loops }}</span>
        </span>
        <span class="metric">
          <span class="metric-label">Cost</span>
          <span class="metric-value">{{ node.percentOfTotal.toFixed(0) }}%</span>
        </span>
      </div>

      <div v-if="node.filter" class="node-filter">
        Filter: {{ node.filter }}
      </div>
      <div v-if="node.indexName" class="node-index">
        Index: {{ node.indexName }}
      </div>
    </div>

    <div v-if="node.children.length" class="node-children">
      <PmExplainNode
        v-for="child in node.children"
        :key="child.id"
        :node="child"
        :depth="depth + 1"
      />
    </div>
  </div>
</template>

<style scoped>
.explain-node {
  animation: pm-slide-up 0.3s ease both;
}

@keyframes pm-slide-up {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
}

.node-card {
  background: var(--pm-surface-elevated);
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-sm);
  padding: 8px 12px;
  margin-bottom: 4px;
}

.explain-node--bottleneck > .node-card {
  border-left: 3px solid var(--pm-danger);
  background: rgba(248, 81, 73, 0.05);
}

.node-header {
  display: flex;
  align-items: baseline;
  gap: 6px;
  margin-bottom: 6px;
}

.node-type {
  font-weight: 600;
  font-size: 12px;
  color: var(--pm-text-primary);
}

.node-relation {
  font-size: 11px;
  color: var(--pm-text-secondary);
  font-family: var(--pm-font-mono);
}

.node-cost-bar {
  height: 4px;
  background: var(--pm-surface);
  border-radius: 2px;
  margin-bottom: 6px;
  overflow: hidden;
}

.node-cost-fill {
  height: 100%;
  border-radius: 2px;
  transition: width 0.5s ease;
}

.node-metrics {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.metric {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.metric-label {
  font-size: 10px;
  color: var(--pm-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.metric-value {
  font-size: 12px;
  font-family: var(--pm-font-mono);
  color: var(--pm-text-primary);
}

.node-filter,
.node-index {
  font-size: 11px;
  color: var(--pm-text-secondary);
  font-family: var(--pm-font-mono);
  margin-top: 4px;
  padding-top: 4px;
  border-top: 1px solid var(--pm-border-subtle);
}

.node-children {
  padding-left: 24px;
  border-left: 1px solid var(--pm-border-subtle);
  margin-left: 12px;
  margin-top: 4px;
}
</style>
