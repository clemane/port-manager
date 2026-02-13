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
  <input
    class="pm-input"
    :class="{ 'pm-input--search': type === 'search' }"
    :type="type ?? 'text'"
    :value="modelValue"
    :placeholder="placeholder"
    :disabled="disabled"
    @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
  />
</template>

<style scoped>
.pm-input {
  background: var(--pm-surface);
  color: var(--pm-text-primary);
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-sm);
  padding: 6px 12px;
  font-size: 13px;
  font-family: inherit;
  outline: none;
  width: 100%;
  transition: border-color 0.15s;
}
.pm-input:focus { border-color: var(--pm-accent); }
.pm-input::placeholder { color: var(--pm-text-muted); }
.pm-input:disabled { opacity: 0.5; cursor: not-allowed; }
.pm-input--search { padding-left: 32px; }
</style>
