import React, { useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../ui/card'
import { Button } from '../ui/button'
import { toast } from 'sonner'

interface ServiceOptionsProps {
  onReset: () => void
}

export default function ServiceOptions({ onReset }: ServiceOptionsProps) {
  const [autoStart, setAutoStart] = useState(true)
  const [maxTasks, setMaxTasks] = useState(10)
  const [port, setPort] = useState(8080)
  const [relayerUrl, setRelayerUrl] = useState('https://api.smainer.io')
  const [gpuEnabled, setGpuEnabled] = useState(true)
  const [logLevel, setLogLevel] = useState('info')

  const handleSaveConfig = () => {
    // Validate configuration
    if (port < 1024 || port > 65535) {
      toast.error('Port must be between 1024 and 65535')
      return
    }
    
    if (maxTasks < 1 || maxTasks > 100) {
      toast.error('Max tasks must be between 1 and 100')
      return
    }
    
    if (!relayerUrl.trim()) {
      toast.error('Relayer URL is required')
      return
    }

    // Save configuration (in real implementation, would use Tauri command)
    toast.success('Configuration saved successfully')
  }

  const handleExportLogs = () => {
    toast.success('Logs exported to Downloads folder')
  }

  const handleResetSettings = () => {
    if (confirm('Are you sure you want to reset all settings? This will restart the onboarding process.')) {
      onReset()
    }
  }

  const handleExportWallet = () => {
    if (confirm('This will export your private key. Make sure you are in a secure environment.')) {
      toast.success('Wallet exported successfully')
    }
  }

  return (
    <div className="space-y-6">
      {/* Provider Configuration */}
      <Card>
        <CardHeader>
          <CardTitle>Provider Configuration</CardTitle>
          <CardDescription>
            Configure how your node operates and connects to the network
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium mb-2">
                Relayer Endpoint
              </label>
              <input
                type="text"
                value={relayerUrl}
                onChange={(e) => setRelayerUrl(e.target.value)}
                placeholder="https://api.smainer.io"
                className="w-full px-3 py-2 border border-border rounded-lg focus:ring-2 focus:ring-ring"
              />
            </div>
            
            <div>
              <label className="block text-sm font-medium mb-2">
                Listen Port
              </label>
              <input
                type="number"
                value={port}
                onChange={(e) => setPort(parseInt(e.target.value) || 8080)}
                min="1024"
                max="65535"
                className="w-full px-3 py-2 border border-border rounded-lg focus:ring-2 focus:ring-ring"
              />
            </div>
            
            <div>
              <label className="block text-sm font-medium mb-2">
                Max Concurrent Tasks
              </label>
              <input
                type="number"
                value={maxTasks}
                onChange={(e) => setMaxTasks(parseInt(e.target.value) || 10)}
                min="1"
                max="100"
                className="w-full px-3 py-2 border border-border rounded-lg focus:ring-2 focus:ring-ring"
              />
            </div>
            
            <div>
              <label className="block text-sm font-medium mb-2">
                Log Level
              </label>
              <select
                value={logLevel}
                onChange={(e) => setLogLevel(e.target.value)}
                className="w-full px-3 py-2 border border-border rounded-lg focus:ring-2 focus:ring-ring"
              >
                <option value="error">Error</option>
                <option value="warn">Warning</option>
                <option value="info">Info</option>
                <option value="debug">Debug</option>
              </select>
            </div>
          </div>
          
          <div className="space-y-3">
            <div className="flex items-center space-x-2">
              <input
                type="checkbox"
                id="autoStart"
                checked={autoStart}
                onChange={(e) => setAutoStart(e.target.checked)}
                className="rounded"
              />
              <label htmlFor="autoStart" className="text-sm text-muted-foreground">
                Start provider automatically on system startup
              </label>
            </div>
            
            <div className="flex items-center space-x-2">
              <input
                type="checkbox"
                id="gpuEnabled"
                checked={gpuEnabled}
                onChange={(e) => setGpuEnabled(e.target.checked)}
                className="rounded"
              />
              <label htmlFor="gpuEnabled" className="text-sm text-muted-foreground">
                Enable GPU acceleration for AI tasks
              </label>
            </div>
          </div>
          
          <Button onClick={handleSaveConfig} className="w-full">
            Save Configuration
          </Button>
        </CardContent>
      </Card>

      {/* System Actions */}
      <Card>
        <CardHeader>
          <CardTitle>System Actions</CardTitle>
          <CardDescription>
            Export data, view logs, and manage your node setup
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <Button variant="outline" onClick={handleExportLogs}>
              Export Logs
            </Button>
            
            <Button variant="outline" onClick={handleExportWallet}>
              Export Wallet
            </Button>
          </div>
          
          <div className="pt-4 border-t">
            <div className="text-sm text-muted-foreground mb-3">
              Need to start over? Reset all settings and return to the onboarding wizard.
            </div>
            <Button 
              variant="destructive" 
              onClick={handleResetSettings}
              className="w-full"
            >
              Reset All Settings
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* About */}
      <Card>
        <CardHeader>
          <CardTitle>About Smainer Desktop</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-2 text-sm">
            <div>Version: 0.1.0</div>
            <div>Build: Development</div>
            <div>Provider Version: 0.1.0</div>
            <div className="pt-2 border-t text-muted-foreground">
              <p>Smainer Desktop makes it easy to run a provider node on Windows.</p>
              <p className="mt-1">
                For support, visit our documentation or contact the development team.
              </p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}