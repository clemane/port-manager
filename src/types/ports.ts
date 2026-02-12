export interface SystemPort {
  protocol: string
  local_port: number
  remote_port: number
  state: string
  pid: number | null
  process_name: string | null
}
