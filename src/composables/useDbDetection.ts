import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { DetectedCredentials } from '@/types/k8s'

export function useDbDetection() {
  const detecting = ref(false)
  const detectedCreds = ref<DetectedCredentials[]>([])
  const detectError = ref<string | null>(null)

  async function detectCredentials(kubeconfigId: string, namespace: string) {
    detecting.value = true
    detectError.value = null
    try {
      detectedCreds.value = await invoke<DetectedCredentials[]>('detect_db_credentials', {
        kubeconfigId,
        namespace,
      })
    } catch (e) {
      detectError.value = String(e)
      detectedCreds.value = []
    } finally {
      detecting.value = false
    }
  }

  function reset() {
    detecting.value = false
    detectedCreds.value = []
    detectError.value = null
  }

  return { detecting, detectedCreds, detectError, detectCredentials, reset }
}
