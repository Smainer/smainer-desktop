import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../ui/card'
import { Button } from '../ui/button'
import { Bug } from 'lucide-react'
import { useStartProvider, useStopProvider } from '../../hooks/useProviderCommands'
import type { NodeStatus as NodeStatusType } from '../../hooks/useNodeStatus'

interface NodeStatusProps {
  status?: NodeStatusType
}

interface DiagnosticsBundle {
  bundle_path: string
  created_at: string
  items_collected: string[]
  summary?: {
    provider_running: boolean
    relayer_health_ok: boolean
    ollama_api_ok: boolean
    ai_enabled: boolean
    node_id: string
    relayer_url: string
  }
}

export default function NodeStatus({ status }: NodeStatusProps) {
  const startProvider = useStartProvider()
  const stopProvider = useStopProvider()

  const [isExportingDiagnostics, setIsExportingDiagnostics] = useState(false)
  const [lastBundle, setLastBundle] = useState<DiagnosticsBundle | null>(null)
  const handleToggleProvider = async () => {
    if (status?.is_online) {
      stopProvider.mutate()
    } else {
      // Get actual wallet address before starting
      try {
        const walletAddress = await invoke('get_wallet_address')
        startProvider.mutate({
          wallet_address: walletAddress as string,
          relayer_url: 'https://api.smainer.io',
          port: 8080,
          max_tasks: 10,
          gpu_enabled: true,
          auto_start: false,
        })
      } catch (error) {
        console.error('Failed to get wallet address:', error)
        const { toast } = await import('sonner')
        toast.error('No wallet found. Please complete onboarding first.')
      }
    }
  }

  const handleExportDiagnostics = async () => {
    if (isExportingDiagnostics) return
    
    setIsExportingDiagnostics(true)
    try {
      const { toast } = await import('sonner')
      const bundle = await invoke<DiagnosticsBundle>('export_diagnostics_bundle')
      setLastBundle(bundle)
      
      if (bundle && typeof bundle === 'object' && 'bundle_path' in bundle) {
        toast.success('Debug bundle created!', {
          description: `Bundle saved. Click Copy to use path.`,
          action: {
            label: 'Copy Path',
            onClick: () => {
              navigator.clipboard.writeText(bundle.bundle_path)
              toast.success('Path copied to clipboard')
            },
          },
          duration: 10000,
        })
      } else {
        throw new Error('Invalid bundle response')
      }
    } catch (error) {
      const { toast } = await import('sonner')
      console.error('Failed to export diagnostics:', error)
      toast.error('Debug bundle failed', {
        description: String(error),
      })
    } finally {
      setIsExportingDiagnostics(false)
    }
  }
  const uptime = status?.uptime || 0
  const uptimeHours = Math.floor(uptime / 3600)
  const uptimeMinutes = Math.floor((uptime % 3600) / 60)
  const providerFailed = status?.network_status === 'provider_failed'
  const statusDescription = startProvider.isPending
    ? 'Connecting to network...'
    : stopProvider.isPending
      ? 'Shutting down...'
      : status?.is_online
        ? 'Your node is running and accepting tasks'
        : providerFailed
          ? 'Provider failed to start - click Debug for logs'
          : status?.network_status === 'connecting'
            ? 'Starting up - connecting to relayer...'
            : status?.relayer_connected === false && status?.network_status === 'disconnected'
              ? 'Node offline - provider is not running'
              : 'Node is offline'
  const relayerLabel = status?.relayer_connected ? 'Online' : providerFailed ? 'Provider failed' : status?.network_status === 'connecting' ? 'Starting' : 'Offline'

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center space-x-2">
                <span>Node Status</span>
                <div className={`w-3 h-3 rounded-full ${
                  status?.is_online ? 'bg-primary' : status?.network_status === 'connecting' ? 'bg-yellow-500' : 'bg-destructive'
                }`} />
              </CardTitle>
              <CardDescription>
                {statusDescription}
              </CardDescription>
            </div>
                       <div className="flex gap-2">
                         <Button
                           onClick={handleExportDiagnostics}
                           disabled={isExportingDiagnostics}
                           variant="outline"
                           size="sm"
                           className="text-xs"
                         >
                           {isExportingDiagnostics ? (
                             <>
                               <span className="inline-block animate-spin mr-1">o</span>
                               Debug...
                             </>
                           ) : (
                             <>
                               <Bug className="w-3 h-3 mr-1" />
                               Debug
                             </>
                           )}
                         </Button>
            <Button
              onClick={handleToggleProvider}
              variant={status?.is_online ? 'destructive' : 'default'}
              disabled={startProvider.isPending || stopProvider.isPending}
            >
              {startProvider.isPending ? 'Starting...' : stopProvider.isPending ? 'Stopping...' : status?.is_online ? 'Stop Node' : 'Start Node'}
            </Button>
                     </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="text-center p-3 bg-card rounded-lg">
              <div className="text-2xl font-bold text-primary">{uptimeHours}h {uptimeMinutes}m</div>
              <div className="text-xs text-muted-foreground">Uptime</div>
            </div>
            <div className="text-center p-3 bg-card rounded-lg">
              <div className="text-2xl font-bold text-primary">{status?.tasks_completed_today || 0}</div>
              <div className="text-xs text-muted-foreground">Tasks Today</div>
            </div>
            <div className="text-center p-3 bg-card rounded-lg">
              <div className="text-2xl font-bold text-primary">{status?.tasks_active || 0}</div>
              <div className="text-xs text-muted-foreground">Active Tasks</div>
            </div>
            <div className="text-center p-3 bg-card rounded-lg">
              <div className="text-2xl font-bold text-primary">
                {relayerLabel}
              </div>
              <div className="text-xs text-muted-foreground">Relayer</div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* System Resources */}
      <Card>
        <CardHeader>
          <CardTitle>System Resources</CardTitle>
          <CardDescription>Real-time resource usage</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div>
              <div className="flex justify-between text-sm mb-1">
                <span>CPU Usage</span>
                <span>{(status?.cpu_usage || 0).toFixed(1)}%</span>
              </div>
              <div className="w-full bg-secondary rounded-full h-2">
                <div 
                  className="bg-primary h-2 rounded-full transition-all duration-300"
                  style={{ width: `${status?.cpu_usage || 0}%` }}
                />
              </div>
            </div>
            
            <div>
              <div className="flex justify-between text-sm mb-1">
                <span>Memory Usage</span>
                <span>{(status?.memory_usage || 0).toFixed(1)}%</span>
              </div>
              <div className="w-full bg-secondary rounded-full h-2">
                <div 
                  className="bg-primary h-2 rounded-full transition-all duration-300"
                  style={{ width: `${status?.memory_usage || 0}%` }}
                />
              </div>
            </div>
            
            {status?.gpu_usage && (
              <div>
                <div className="flex justify-between text-sm mb-1">
                  <span>GPU Usage</span>
                  <span>{status.gpu_usage.toFixed(1)}%</span>
                </div>
                <div className="w-full bg-secondary rounded-full h-2">
                  <div 
                    className="bg-primary h-2 rounded-full transition-all duration-300"
                    style={{ width: `${status.gpu_usage}%` }}
                  />
                </div>
              </div>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Node Info */}
      {status && (
        <Card>
          <CardHeader>
            <CardTitle>Node Information</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-6 text-sm">
              <div className="space-y-2">
                <div className="font-medium text-muted-foreground">Node ID:</div>
                <div className="font-mono text-xs break-all bg-muted p-3 rounded-lg border">
                  {status.node_id}
                </div>
              </div>
              <div className="grid grid-cols-1 gap-4">
                <div className="space-y-1">
                  <div className="font-medium text-muted-foreground">Version:</div>
                  <div>{status.provider_version}</div>
                </div>
                <div className="space-y-1">
                  <div className="font-medium text-muted-foreground">Tier:</div>
                  <div className="capitalize">{status.node_tier}</div>
                </div>
                <div className="space-y-1">
                  <div className="font-medium text-muted-foreground">Network Status:</div>
                  <div className="capitalize">{status.network_status}</div>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {lastBundle && (
        <Card>
          <CardHeader>
            <CardTitle>Latest Debug Bundle</CardTitle>
            <CardDescription>
              Redacted diagnostics snapshot for relayer and provider troubleshooting
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3 text-sm">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
              <div>Provider: {lastBundle.summary?.provider_running ? 'running' : 'stopped'}</div>
              <div>Relayer health: {lastBundle.summary?.relayer_health_ok ? 'ok' : 'failed'}</div>
              <div>Ollama API: {lastBundle.summary?.ollama_api_ok ? 'ok' : 'failed'}</div>
              <div>AI enabled: {lastBundle.summary?.ai_enabled ? 'yes' : 'no'}</div>
            </div>
            <div className="text-xs text-muted-foreground break-all">
              {lastBundle.bundle_path}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  )
}