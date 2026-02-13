<script setup lang="ts">
withDefaults(defineProps<{
  modelValue?: string
  placeholder?: string
}>(), {
  modelValue: '',
  placeholder: 'Enter SQL query...',
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'execute': []
}>()

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter' && (event.ctrlKey || event.metaKey)) {
    event.preventDefault()
    emit('execute')
    return
  }

  if (event.key === 'Tab') {
    event.preventDefault()
    const textarea = event.target as HTMLTextAreaElement
    const start = textarea.selectionStart
    const end = textarea.selectionEnd
    const value = textarea.value
    const newValue = value.substring(0, start) + '  ' + value.substring(end)
    emit('update:modelValue', newValue)
    // Restore cursor position after the inserted spaces
    requestAnimationFrame(() => {
      textarea.selectionStart = start + 2
      textarea.selectionEnd = start + 2
    })
  }
}
</script>

<template>
  <textarea
    class="pm-code-editor"
    :value="modelValue"
    :placeholder="placeholder"
    spellcheck="false"
    autocomplete="off"
    autocorrect="off"
    autocapitalize="off"
    @input="emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
    @keydown="onKeydown"
  />
</template>

<style scoped>
.pm-code-editor {
  font-family: var(--pm-font-mono);
  background: var(--pm-surface-elevated);
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius);
  padding: 12px;
  min-height: 120px;
  color: var(--pm-text-primary);
  font-size: 13px;
  resize: vertical;
  width: 100%;
  line-height: 1.5;
  outline: none;
  transition: border-color 0.15s, box-shadow 0.15s;
  box-sizing: border-box;
  tab-size: 2;
}
.pm-code-editor:focus {
  border-color: var(--pm-accent);
  box-shadow: 0 0 0 3px var(--pm-accent-glow);
}
.pm-code-editor::placeholder {
  color: var(--pm-text-muted);
}
</style>
