import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { KubeconfigInfo, K8sService, K8sPod } from '@/types/k8s'

export function useK8s() {
  const kubeconfigs = ref<KubeconfigInfo[]>([])
  const namespaces = ref<string[]>([])
  const services = ref<K8sService[]>([])
  const pods = ref<K8sPod[]>([])
  const loading = ref(false)

  async function loadKubeconfigs() {
    kubeconfigs.value = await invoke<KubeconfigInfo[]>('list_kubeconfigs')
  }

  async function loadNamespaces(kubeconfigId: string) {
    loading.value = true
    try {
      namespaces.value = await invoke<string[]>('list_namespaces', { kubeconfigId })
    } finally {
      loading.value = false
    }
  }

  async function loadServices(kubeconfigId: string, namespace: string) {
    services.value = await invoke<K8sService[]>('list_services', { kubeconfigId, namespace })
  }

  async function loadPods(kubeconfigId: string, namespace: string) {
    pods.value = await invoke<K8sPod[]>('list_pods', { kubeconfigId, namespace })
  }

  return { kubeconfigs, namespaces, services, pods, loading, loadKubeconfigs, loadNamespaces, loadServices, loadPods }
}
