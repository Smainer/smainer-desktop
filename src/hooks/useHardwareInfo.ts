import { useQuery } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'

export interface GpuInfo {
  name: string
  vendor: string
  memory: number // Memory in MB
  compute_capability: string
  driver_version: string
  is_supported: boolean
}

export interface HardwareInfo {
  cpu_name: string
  cpu_cores: number
  total_ram: number // RAM in bytes
  available_ram: number // Available RAM in bytes
  gpus: GpuInfo[]
  os: string
  os_version: string
}

export interface SystemRequirements {
  meets_requirements: boolean
  ram_ok: boolean
  gpu_ok: boolean
  cpu_ok: boolean
  warnings: string[]
  errors: string[]
  system_info: HardwareInfo
}

export function useHardwareInfo() {
  return useQuery({
    queryKey: ['hardwareInfo'],
    queryFn: () => invoke<HardwareInfo>('get_system_info'),
    staleTime: 30000, // Hardware info is relatively static
    retry: 2,
  })
}

export function useGpuDetection() {
  return useQuery({
    queryKey: ['gpus'],
    queryFn: () => invoke<GpuInfo[]>('detect_gpus'),
    staleTime: 30000,
    retry: 2,
  })
}

export function useSystemRequirements() {
  return useQuery({
    queryKey: ['systemRequirements'],
    queryFn: () => invoke<SystemRequirements>('check_requirements'),
    staleTime: 30000,
    retry: 2,
  })
}