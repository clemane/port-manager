export interface KubeconfigInfo {
  id: string
  name: string
  created_at: string
  last_used: string | null
}

export interface K8sService {
  name: string
  namespace: string
  ports: ServicePort[]
}

export interface ServicePort {
  name: string | null
  port: number
  target_port: string | null
  protocol: string
}

export interface K8sPod {
  name: string
  namespace: string
  status: string
  ports: PodPort[]
}

export interface PodPort {
  name: string | null
  container_port: number
  protocol: string
}
