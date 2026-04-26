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
  const [relayerUrl, setRelayerUrl] = useState(import.meta.env.VITE_RELAYER_URL || 'https://api.smainer.io')
  const [nodeName, setNodeName] = useState('')
  const [autoStart, setAutoStart] = useState(true)
  const [contactInfo, setContactInfo] = useState('')

  const registerNode = useRegisterNode()

  const handleRegister = () => {
    if (!hardwareInfo) {
      toast.error('Hardware information not available')
      return
    }

    const registration = {
      wallet_address: walletAddress,
      hardware_capabilities: hardwareInfo,
      relayer_endpoint: relayerUrl,
      node_name: nodeName || undefined,
      contact_info: contactInfo || undefined,
    }

    registerNode.mutate(registration, {
      onSuccess: (nodeId) => {
        onComplete(walletAddress, nodeId)
      }
    })
  }

  const supportedGpus = hardwareInfo?.gpus.filter(gpu => gpu.is_supported) || []
  const totalVram = supportedGpus.reduce((sum, gpu) => sum + gpu.memory, 0)
  const ramGB = hardwareInfo ? Math.round(hardwareInfo.total_ram / (1024 * 1024 * 1024)) : 0

  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h2 className="text-2xl font-bold mb-4">Register Your Node</h2>
        <p className="text-muted-foreground">
          Register your provider node with the Smainer network to start earning.
        </p>
      </div>

      <Card className="bg-card border-border">
        <CardHeader>
          <CardTitle className="text-primary">Your Node Overview</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div>
              <span className="text-muted-foreground">Wallet Address:</span>
              <p className="font-mono text-xs mt-1 bg-white p-2 rounded border">
                {walletAddress}
              </p>
            </div>
            <div>
              <span className="text-muted-foreground">System Specs:</span>
              <ul className="mt-1 space-y-1 text-xs">
                <li>{ramGB}GB RAM, {hardwareInfo?.cpu_cores} CPU cores</li>
                <li>{supportedGpus.length} GPU(s), {Math.round(totalVram / 1024)}GB VRAM</li>
                <li>{hardwareInfo?.os} {hardwareInfo?.os_version}</li>
              </ul>
            </div>
          </div>
          
          {supportedGpus.length > 0 && (
            <div>
              <span className="text-muted-foreground block mb-2">Available GPUs:</span>
              <div className="space-y-1">
                {supportedGpus.map((gpu, idx) => (
                  <div key={idx} className="bg-white p-2 rounded border text-xs">
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
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
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
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
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
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
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