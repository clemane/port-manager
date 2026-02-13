<script setup lang="ts">
defineProps<{
  modelValue?: string | number
  type?: 'text' | 'search' | 'number' | 'password'
  placeholder?: string
  disabled?: boolean
}>()

defineEmits<{
  'update:modelValue': [value: string | number]
}>()
</script>

<template>
  <div class="pm-input-wrapper" :class="{ 'pm-input-wrapper--search': type === 'search' }">
    <svg v-if="type === 'search'" class="pm-input-wrapper__icon" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
      <circle cx="7" cy="7" r="4.5" />
      <path d="M10.5 10.5L14 14" stroke-linecap="round" />
    </svg>
    <input
      class="pm-input"
      :type="type === 'search' ? 'text' : (type ?? 'text')"
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
    />
  </div>
</template>

<style scoped>
.pm-input-wrapper {
  position: relative;
  width: 100%;
}
.pm-input-wrapper--search .pm-input {
  padding-left: 36px;
}
.pm-input-wrapper__icon {
  position: absolute;
  left: 10px;
  top: 50%;
  transform: translateY(-50%);
  width: 16px;
  height: 16px;
  color: var(--pm-text-muted);
  pointer-events: none;
}
.pm-input {
  background: var(--pm-surface-elevated);
  color: var(--pm-text-primary);
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-sm);
  padding: 7px 12px;
  height: 36px;
  font-size: 13px;
  font-family: var(--pm-font-body);
  outline: none;
  width: 100%;
  transition: border-color 0.15s, box-shadow 0.15s;
}
.pm-input:focus {
  border-color: var(--pm-accent);
  box-shadow: 0 0 0 3px var(--pm-accent-glow);
}
.pm-input::placeholder { color: var(--pm-text-muted); }
.pm-input:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
