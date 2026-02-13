<script setup lang="ts">
import { ref, computed, watch, onBeforeUnmount } from 'vue'

const props = defineProps<{
  modelValue?: string
  options: { value: string; label: string }[]
  placeholder?: string
  searchable?: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const isOpen = ref(false)
const search = ref('')

const filtered = computed(() => {
  if (!search.value) return props.options
  const q = search.value.toLowerCase()
  return props.options.filter(o => o.label.toLowerCase().includes(q))
})

const selectedLabel = computed(() => {
  const opt = props.options.find(o => o.value === props.modelValue)
  return opt?.label ?? props.placeholder ?? 'Select...'
})

function select(value: string) {
  emit('update:modelValue', value)
  isOpen.value = false
  search.value = ''
}

const selectRef = ref<HTMLElement | null>(null)

function onClickOutside(e: MouseEvent) {
  if (selectRef.value && !selectRef.value.contains(e.target as Node)) {
    isOpen.value = false
    search.value = ''
  }
}

watch(isOpen, (open) => {
  if (open) document.addEventListener('click', onClickOutside, true)
  else document.removeEventListener('click', onClickOutside, true)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', onClickOutside, true)
})
</script>

<template>
  <div ref="selectRef" class="pm-select" :class="{ 'pm-select--open': isOpen }">
    <button class="pm-select__trigger" @click="isOpen = !isOpen">
      <span :class="{ 'pm-select__placeholder': !modelValue }">{{ selectedLabel }}</span>
      <span class="pm-select__arrow">&#9662;</span>
    </button>
    <Transition name="dropdown">
    <div v-if="isOpen" class="pm-select__dropdown">
      <input
        v-if="searchable"
        v-model="search"
        class="pm-select__search"
        placeholder="Search..."
        @click.stop
      />
      <div class="pm-select__options">
        <button
          v-for="opt in filtered"
          :key="opt.value"
          class="pm-select__option"
          :class="{ 'pm-select__option--selected': opt.value === modelValue }"
          @click="select(opt.value)"
        >
          {{ opt.label }}
        </button>
        <div v-if="filtered.length === 0" class="pm-select__empty">No results</div>
      </div>
    </div>
    </Transition>
  </div>
</template>

<style scoped>
.pm-select { position: relative; }
.pm-select__trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  height: 36px;
  background: var(--pm-surface-elevated);
  color: var(--pm-text-primary);
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-sm);
  padding: 6px 12px;
  font-size: 13px;
  cursor: pointer;
  font-family: var(--pm-font-body);
  transition: border-color 0.15s, box-shadow 0.15s;
}
.pm-select--open .pm-select__trigger {
  border-color: var(--pm-accent);
  box-shadow: 0 0 0 3px var(--pm-accent-glow);
}
.pm-select__placeholder { color: var(--pm-text-muted); }
.pm-select__arrow { font-size: 10px; color: var(--pm-text-muted); }
.pm-select__dropdown {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  right: 0;
  background: var(--pm-surface-elevated);
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-sm);
  box-shadow: var(--pm-shadow);
  z-index: 100;
  max-height: 200px;
  overflow: auto;
}
.pm-select__search {
  width: 100%;
  padding: 6px 12px;
  border: none;
  border-bottom: 1px solid var(--pm-border-subtle);
  background: transparent;
  color: var(--pm-text-primary);
  font-size: 13px;
  outline: none;
  font-family: var(--pm-font-body);
}
.pm-select__search::placeholder { color: var(--pm-text-muted); }
.pm-select__option {
  display: block;
  width: 100%;
  text-align: left;
  padding: 6px 12px;
  background: none;
  border: none;
  color: var(--pm-text-primary);
  font-size: 13px;
  cursor: pointer;
  font-family: var(--pm-font-body);
  transition: background 0.15s;
}
.pm-select__option:hover { background: var(--pm-surface-hover); }
.pm-select__option--selected { color: var(--pm-accent); }
.pm-select__empty { padding: 8px 12px; color: var(--pm-text-muted); font-size: 13px; }

.pm-select__trigger:focus-visible {
  outline: none;
  border-color: var(--pm-accent);
  box-shadow: 0 0 0 2px var(--pm-accent-glow);
}

.dropdown-enter-active,
.dropdown-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
  transform-origin: top;
}
.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: scaleY(0.95);
}
</style>
