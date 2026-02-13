<script setup lang="ts">
withDefaults(defineProps<{
  variant?: 'table' | 'card' | 'text'
  lines?: number
}>(), {
  variant: 'table',
  lines: 5,
})

const tableWidths = ['70%', '40%', '90%', '55%', '80%', '65%', '45%', '85%']
</script>

<template>
  <div class="pm-skeleton" :class="`pm-skeleton--${variant}`">
    <template v-if="variant === 'table'">
      <div v-for="i in lines" :key="i" class="pm-skeleton__row">
        <div
          v-for="j in 4"
          :key="j"
          class="pm-skeleton__bar"
          :style="{ width: tableWidths[(i * 4 + j) % tableWidths.length] }"
        />
      </div>
    </template>
    <template v-else-if="variant === 'card'">
      <div class="pm-skeleton__bar pm-skeleton__bar--title" />
      <div class="pm-skeleton__bar pm-skeleton__bar--subtitle" />
      <div class="pm-skeleton__bar pm-skeleton__bar--body" />
    </template>
    <template v-else>
      <div
        v-for="i in lines"
        :key="i"
        class="pm-skeleton__bar"
        :style="{ width: tableWidths[(i - 1) % tableWidths.length] }"
      />
    </template>
  </div>
</template>

<style scoped>
.pm-skeleton__row {
  display: flex;
  gap: 12px;
  padding: 12px;
  border-bottom: 1px solid var(--pm-border-subtle);
}

.pm-skeleton__bar {
  height: 12px;
  border-radius: 6px;
  background: linear-gradient(90deg, var(--pm-surface-hover) 25%, var(--pm-surface-elevated) 50%, var(--pm-surface-hover) 75%);
  background-size: 200% 100%;
  animation: pm-shimmer 1.5s ease-in-out infinite;
  flex-shrink: 0;
}

.pm-skeleton__row .pm-skeleton__bar {
  flex: 1;
}

.pm-skeleton--card {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 16px;
}

.pm-skeleton__bar--title { width: 60%; height: 16px; }
.pm-skeleton__bar--subtitle { width: 40%; height: 12px; }
.pm-skeleton__bar--body { width: 85%; height: 12px; }

.pm-skeleton--text {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
</style>
