import React, { useState } from 'react'
import { useRegisterNode } from '../../hooks/useProviderCommands.ts'
import { HardwareInfo } from '../../hooks/useHardwareInfo.ts'
import { Button } from '../ui/button.tsx'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../ui/card.tsx'
import { toast } from 'sonner'

interface NodeRegistrationProps {
  walletAddress: string
  hardwareInfo?: HardwareInfo
  onComplete: (walletAddress: string, nodeId: string) => void
  onBack: () => void
}

export default function NodeRegistration({ 
  walletAddress, 
  hardwareInfo, 
  onComplete, 
  onBack 
}: NodeRegistrationProps) {
  const [relayerUrl, setRelayerUrl] = useState((import.meta as any).env?.VITE_RELAYER_URL || 'https://api.smainer.io')
  const [nodeName, setNodeName] = useState('')
  const [autoStart, setAutoStart] = useState(true)
  const [contactInfo, setContactInfo] = useState('')

  const registerNode = useRegisterNode()

  const handleRegister = () => {
    if (!hardwareInfo) {
      toast.error('Hardware detection incomplete. Please restart the application.')
      return
    }

    if (!relayerUrl.trim()) {
      toast.error('Relayer endpoint required. Please enter a valid URL.')
      return
    }

    const registration = {
      wallet_address: walletAddress,
      hardware_capabilities: hardwareInfo,
      relayer_endpoint: relayerUrl,
      node_name: nodeName || undefined,
      contact_info: contactInfo || undefined,
    }

    toast.info('Starting provider daemon and registering with network...')

    registerNode.mutate(registration, {
      onSuccess: (nodeId) => {
        toast.success('Node registered successfully! Provider daemon started.')
        onComplete(walletAddress, nodeId)
      },
      onError: (error: any) => {
        const errorMsg = error.toString()
        if (errorMsg.includes('provider daemon')) {
          toast.error('Provider daemon not found. Download the installer version or set SMAINER_PROVIDER_CMD environment variable.')
        } else if (errorMsg.includes('wallet')) {
          toast.error('Wallet access error. Check your wallet configuration and try again.')
        } else if (errorMsg.includes('network') || errorMsg.includes('timeout')) {
          toast.error('Network connection failed. Check your internet connection and relayer endpoint.')
        } else {
          toast.error(`Registration failed: ${errorMsg}`)
        }
      }
    })
  }

  const supportedGpus = hardwareInfo?.gpus.filter(gpu => gpu.is_supported) || []
  const totalVram = supportedGpus.reduce((sum, gpu) => sum + gpu.memory, 0)
  const ramGB = hardwareInfo ? Math.round(hardwareInfo.total_ram / (1024 * 1024 * 1024)) : 0

  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h2 className="text-2xl font-bold mb-4">Register Provider Node</h2>
        <p className="text-muted-foreground">
          Connect your node to the Smainer network. This starts the provider daemon and registers your hardware capabilities for task assignment.
        </p>
      </div>

      <Card className="bg-card border-border">
        <CardHeader>
          <CardTitle className="text-primary">Your Node Overview</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-4">
            {/* Wallet Address - always full width to prevent overlap */}
            <div>
              <span className="text-muted-foreground text-sm">Wallet Address:</span>
              <p className="font-mono text-xs mt-1 bg-muted p-2 rounded border border-border break-all word-break-all">
                {walletAddress}
              </p>
            </div>
            
            {/* System Specs - separate row */}
            <div>
              <span className="text-muted-foreground text-sm">System Specifications:</span>
              <ul className="mt-1 space-y-1 text-xs bg-muted p-2 rounded border border-border">
                <li><strong>RAM:</strong> {ramGB}GB total</li>
                <li><strong>CPU:</strong> {hardwareInfo?.cpu_cores} cores</li>
                <li><strong>GPU:</strong> {supportedGpus.length} device(s), {Math.round(totalVram / 1024)}GB VRAM total</li>
                <li><strong>OS:</strong> {hardwareInfo?.os} {hardwareInfo?.os_version}</li>
              </ul>
            </div>
          </div>
          
          {supportedGpus.length > 0 && (
            <div>
              <span className="text-muted-foreground block mb-2">Available GPUs:</span>
              <div className="space-y-1">
                {supportedGpus.map((gpu, idx) => (
                  <div key={idx} className="bg-muted p-2 rounded border border-border text-xs">
                    <span className="font-medium">{gpu.name}</span>
                    <span className="text-muted-foreground ml-2">
                      {Math.round(gpu.memory / 1024)}GB VRAM
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Registration Settings</CardTitle>
          <CardDescription>
            Configure how your node connects to the Smainer network.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <label className="block text-sm font-medium mb-2">
              Relayer Endpoint
            </label>
            <input
              type="text"
              value={relayerUrl}
              onChange={(e) => setRelayerUrl(e.target.value)}
              placeholder="https://api.smainer.io"
              className="w-full px-3 py-2 border border-border rounded-lg focus:ring-2 focus:ring-ring focus:border-ring"
            />
            <p className="text-xs text-muted-foreground mt-1">
              URL of the Smainer relayer service. Use http://localhost:8000 for local development.
            </p>
          </div>

          <div>
            <label className="block text-sm font-medium mb-2">
              Node Name (Optional)
            </label>
            <input
              type="text"
              value={nodeName}
              onChange={(e) => setNodeName(e.target.value)}
              placeholder="My Smainer Node"
              className="w-full px-3 py-2 border border-border rounded-lg focus:ring-2 focus:ring-ring focus:border-ring"
            />
            <p className="text-xs text-muted-foreground mt-1">
              A friendly name for your node to help identify it.
            </p>
          </div>

          <div>
            <label className="block text-sm font-medium mb-2">
              Contact Information (Optional)
            </label>
            <input
              type="email"
              value={contactInfo}
              onChange={(e) => setContactInfo(e.target.value)}
              placeholder="your.email@example.com"
              className="w-full px-3 py-2 border border-border rounded-lg focus:ring-2 focus:ring-ring focus:border-ring"
            />
            <p className="text-xs text-muted-foreground mt-1">
              Optional email for node operator communications.
            </p>
          </div>

          <div className="flex items-center space-x-2">
            <input
              type="checkbox"
              id="autoStart"
              checked={autoStart}
              onChange={(e) => setAutoStart(e.target.checked)}
              className="rounded"
            />
            <label htmlFor="autoStart" className="text-sm text-muted-foreground">
              Start provider automatically after registration
            </label>
          </div>
        </CardContent>
      </Card>

      <Card className="bg-card border-border">
        <CardHeader>
          <CardTitle className="text-primary">Earning Potential</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="text-sm text-muted-foreground">
            <p className="mb-2">
              Based on your hardware configuration, you can expect:
            </p>
            <ul className="space-y-1 list-disc list-inside">
              <li>~${(totalVram * 0.001).toFixed(2)}-${(totalVram * 0.003).toFixed(2)} per hour (GPU tasks)</li>
              <li>~$0.10-$0.50 per hour (CPU/memory tasks)</li>
              <li>Higher earnings during peak demand periods</li>
            </ul>
            <p className="text-xs mt-3 text-muted-foreground">
              * Estimates based on current network activity. Actual earnings may vary.
            </p>
          </div>
        </CardContent>
      </Card>

      <div className="flex justify-between">
        <Button variant="outline" onClick={onBack} disabled={registerNode.isPending}>
          Back
        </Button>
        <Button
          onClick={handleRegister}
          disabled={registerNode.isPending || !relayerUrl.trim()}
          className="px-8"
        >
          {registerNode.isPending ? (
            <div className="flex items-center space-x-2">
              <div className="w-4 h-4 border-2 border-primary border-t-transparent rounded-full animate-spin" />
              <span>Registering Node...</span>
            </div>
          ) : (
            'Register & Complete Setup'
          )}
        </Button>
      </div>
    </div>
  )
}