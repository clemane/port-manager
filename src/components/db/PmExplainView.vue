<script setup lang="ts">
import { computed } from 'vue'
import type { ExplainNode } from '@/types/pgmanager'
import PmExplainNode from './PmExplainNode.vue'

const props = defineProps<{
  plan: unknown
}>()

function parseExplainJson(raw: unknown): ExplainNode | null {
  try {
    // EXPLAIN (FORMAT JSON) returns an array with one element containing { "Plan": {...} }
    const arr = Array.isArray(raw) ? raw : JSON.parse(String(raw))
    const root = arr?.[0]?.Plan ?? arr?.[0]?.['Plan'] ?? arr?.[0]
    if (!root) return null

    let nodeId = 0
    const totalTime = root['Actual Total Time'] ?? root['Total Cost'] ?? 1

    function walk(plan: Record<string, unknown>): ExplainNode {
      const actualTime = plan['Actual Total Time'] as number | null ?? null
      const children = ((plan['Plans'] as Record<string, unknown>[]) ?? []).map(walk)

      return {
        id: String(nodeId++),
        type: (plan['Node Type'] as string) ?? 'Unknown',
        relation: (plan['Relation Name'] as string) ?? undefined,
        alias: (plan['Alias'] as string) ?? undefined,
        startupCost: (plan['Startup Cost'] as number) ?? 0,
        totalCost: (plan['Total Cost'] as number) ?? 0,
        planRows: (plan['Plan Rows'] as number) ?? 0,
        actualTime,
        actualRows: (plan['Actual Rows'] as number) ?? null,
        loops: (plan['Actual Loops'] as number) ?? 1,
        filter: (plan['Filter'] as string) ?? undefined,
        indexName: (plan['Index Name'] as string) ?? undefined,
        sharedHitBlocks: (plan['Shared Hit Blocks'] as number) ?? undefined,
        sharedReadBlocks: (plan['Shared Read Blocks'] as number) ?? undefined,
        children,
        percentOfTotal: totalTime > 0 && actualTime !== null
          ? (actualTime / totalTime) * 100
          : 0,
      }
    }

    return walk(root)
  } catch {
    return null
  }
}

const rootNode = computed(() => parseExplainJson(props.plan))
</script>

<template>
  <div class="pm-explain-view">
    <div v-if="!rootNode" class="explain-empty">
      Unable to parse EXPLAIN output
    </div>
    <PmExplainNode v-else :node="rootNode" :depth="0" />
  </div>
</template>

<style scoped>
.pm-explain-view {
  padding: 12px;
  overflow: auto;
  height: 100%;
}

.explain-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--pm-text-muted);
  font-size: 13px;
}
</style>
