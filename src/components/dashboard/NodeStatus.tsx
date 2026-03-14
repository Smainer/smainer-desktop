import React from 'react'
import { useQuery } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../ui/card'
import { Button } from '../ui/button'
import { useStartProvider, useStopProvider } from '../../hooks/useProviderCommands'
import type { NodeStatus as NodeStatusType } from '../../hooks/useNodeStatus'

interface NodeStatusProps {
  status?: NodeStatusType
}

export default function NodeStatus({ status }: NodeStatusProps) {
  const startProvider = useStartProvider()
  const stopProvider = useStopProvider()

  const { data: metrics } = useQuery({
    queryKey: ['nodeMetrics'],
    queryFn: () => invoke('get_node_metrics'),
    refetchInterval: 5000,
  })

  const handleToggleProvider = () => {
    if (status?.is_online) {
      stopProvider.mutate()
    } else {
      // Start with basic config
      startProvider.mutate({
        wallet_address: 'mock_address',
        relayer_url: 'http://localhost:8000',
        port: 8080,
        max_tasks: 10,
        gpu_enabled: true,
        auto_start: false,
      })
    }
  }

  const uptime = status?.uptime || 0
  const uptimeHours = Math.floor(uptime / 3600)
  const uptimeMinutes = Math.floor((uptime % 3600) / 60)

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center space-x-2">
                <span>Node Status</span>
                <div className={`w-3 h-3 rounded-full ${
                  status?.is_online ? 'bg-green-500' : 'bg-red-500'
                }`} />
              </CardTitle>
              <CardDescription>
                {status?.is_online ? 'Your node is running and accepting tasks' : 'Node is offline'}
              </CardDescription>
            </div>
            <Button
              onClick={handleToggleProvider}
              variant={status?.is_online ? 'destructive' : 'default'}
              disabled={startProvider.isPending || stopProvider.isPending}
            >
              {status?.is_online ? 'Stop Node' : 'Start Node'}
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="text-center p-3 bg-gray-50 rounded-lg">
              <div className="text-2xl font-bold text-blue-600">{uptimeHours}h {uptimeMinutes}m</div>
              <div className="text-xs text-gray-600">Uptime</div>
            </div>
            <div className="text-center p-3 bg-gray-50 rounded-lg">
              <div className="text-2xl font-bold text-green-600">{status?.tasks_completed_today || 0}</div>
              <div className="text-xs text-gray-600">Tasks Today</div>
            </div>
            <div className="text-center p-3 bg-gray-50 rounded-lg">
              <div className="text-2xl font-bold text-purple-600">{status?.tasks_active || 0}</div>
              <div className="text-xs text-gray-600">Active Tasks</div>
            </div>
            <div className="text-center p-3 bg-gray-50 rounded-lg">
              <div className="text-2xl font-bold text-orange-600">
                {status?.relayer_connected ? '🟢' : '🔴'}
              </div>
              <div className="text-xs text-gray-600">Relayer</div>
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
              <div className="w-full bg-gray-200 rounded-full h-2">
                <div 
                  className="bg-blue-500 h-2 rounded-full transition-all duration-300"
                  style={{ width: `${status?.cpu_usage || 0}%` }}
                />
              </div>
            </div>
            
            <div>
              <div className="flex justify-between text-sm mb-1">
                <span>Memory Usage</span>
                <span>{(status?.memory_usage || 0).toFixed(1)}%</span>
              </div>
              <div className="w-full bg-gray-200 rounded-full h-2">
                <div 
                  className="bg-green-500 h-2 rounded-full transition-all duration-300"
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
                <div className="w-full bg-gray-200 rounded-full h-2">
                  <div 
                    className="bg-purple-500 h-2 rounded-full transition-all duration-300"
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
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
              <div>
                <span className="font-medium text-gray-600">Node ID:</span>
                <p className="font-mono mt-1">{status.node_id}</p>
              </div>
              <div>
                <span className="font-medium text-gray-600">Version:</span>
                <p className="mt-1">{status.provider_version}</p>
              </div>
              <div>
                <span className="font-medium text-gray-600">Tier:</span>
                <p className="mt-1 capitalize">{status.node_tier}</p>
              </div>
              <div>
                <span className="font-medium text-gray-600">Network Status:</span>
                <p className="mt-1 capitalize">{status.network_status}</p>
              </div>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  )
}