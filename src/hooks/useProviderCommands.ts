import { useMutation, useQuery } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { toast } from 'sonner'

export interface ProviderConfig {
  wallet_address: string
  relayer_url: string
  port: number
  max_tasks: number
  gpu_enabled: boolean
  auto_start: boolean
}

export interface NodeRegistration {
  wallet_address: string
  hardware_capabilities: any
  stake_amount?: number
  relayer_endpoint?: string
  node_name?: string
  contact_info?: string
}

export function useStartProvider() {
  return useMutation({
    mutationFn: (config: ProviderConfig) => 
      invoke<boolean>('start_provider', { config }),
    onSuccess: () => {
      toast.success('Node started successfully')
    },
    onError: (error: any) => {
      const errorMsg = error.toString()
      if (errorMsg.includes('provider daemon')) {
        toast.error('Provider daemon not found. Please use the installer version or configure SMAINER_PROVIDER_CMD environment variable.')
      } else if (errorMsg.includes('wallet')) {
        toast.error('Wallet configuration error. Please check your wallet setup.')
      } else {
        toast.error(`Failed to start node: ${errorMsg}`)
      }
    },
  })
}

export function useStopProvider() {
  return useMutation({
    mutationFn: () => invoke<boolean>('stop_provider'),
    onSuccess: () => {
      toast.success('Provider stopped successfully')
    },
    onError: (error: any) => {
      toast.error(`Failed to stop provider: ${error}`)
    },
  })
}

export function useRegisterNode() {
  return useMutation({
    mutationFn: (registration: NodeRegistration) => 
      invoke<string>('register_node', { registration }),
    onSuccess: (nodeId) => {
      toast.success(`Node registered successfully! ID: ${nodeId}`)
    },
    onError: (error: any) => {
      toast.error(`Failed to register node: ${error}`)
    },
  })
}

export function useProviderLogs() {
  return useQuery({
    queryKey: ['providerLogs'],
    queryFn: () => invoke<string[]>('get_provider_logs'),
    refetchInterval: 5000,
    retry: 1,
  })
}