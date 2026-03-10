import { useQuery } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'

export interface NodeStatus {
  is_online: boolean
  node_id: string
  uptime: number
  last_heartbeat: string
  tasks_active: number
  tasks_completed_today: number
  earnings_today: number
  cpu_usage: number
  memory_usage: number
  gpu_usage?: number
  network_status: string
  relayer_connected: boolean
  provider_version: string
  node_tier: string
}

export function useNodeStatus() {
  return useQuery({
    queryKey: ['nodeStatus'],
    queryFn: () => invoke<NodeStatus>('get_node_status'),
    refetchInterval: 5000, // Refresh every 5 seconds
    retry: 1,
    staleTime: 1000,
  })
}

export interface ProviderStatus {
  is_running: boolean
  uptime: number
  tasks_completed: number
  tasks_active: number
  last_heartbeat: string
  earnings_today: number
  cpu_usage: number
  memory_usage: number
  gpu_usage?: number
  network_status: string
  relayer_connected: boolean
  last_task_time?: string
  error_message?: string
}

export function useProviderStatus() {
  return useQuery({
    queryKey: ['providerStatus'],
    queryFn: () => invoke<ProviderStatus>('get_provider_status'),
    refetchInterval: 3000, // Refresh every 3 seconds
    retry: 1,
  })
}