import { ref, watchEffect } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export type Theme = 'dark' | 'light' | 'cyberpunk'

const currentTheme = ref<Theme>('dark')

export function useTheme() {
  async function loadTheme() {
    try {
      const saved = await invoke<string | null>('get_setting', { key: 'theme' })
      if (saved) {
        const parsed = JSON.parse(saved) as Theme
        currentTheme.value = parsed
      }
    } catch {
      // Use default dark theme
    }
  }

  async function setTheme(theme: Theme) {
    currentTheme.value = theme
    document.documentElement.setAttribute('data-theme', theme)
    try {
      await invoke('set_setting', { key: 'theme', value: JSON.stringify(theme) })
    } catch {
      // Silently fail on save
    }
  }

  watchEffect(() => {
    document.documentElement.setAttribute('data-theme', currentTheme.value)
  })

  return { currentTheme, loadTheme, setTheme }
}
