import React, { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Button } from '../ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../ui/card'
import { RadioGroup, RadioGroupItem } from '../ui/radio-group'
import { Label } from '../ui/label'
import { Checkbox } from '../ui/checkbox'
import { Alert, AlertDescription } from '../ui/alert'
import { Badge } from '../ui/badge'
import { useHardwareInfo } from '../../hooks/useHardwareInfo'

interface AISetupProps {
  onNext: () => void
  onBack: () => void
}

interface ModelConfig {
  name: string
  enabled: boolean
  priority: number
  requirements: {
    min_vram_gb: number
    min_ram_gb: number
    min_disk_gb: number
    requires_gpu: boolean
    network_bandwidth_mbps?: number
  }
}

interface AICapabilityConfig {
  schema_version: string
  contract_version: string
  ai_serving_enabled: boolean
  ollama_config?: {
    install_requested: boolean
    installation_path?: string
    api_endpoint: string
    models_to_install: string[]
    auto_update: boolean
  }
  model_preferences: ModelConfig[]
  privacy_mode: 'Standard' | 'Enhanced' | 'Maximum'
  resources: {
    max_cpu_percent: number
    max_ram_gb: number
    max_vram_gb?: number
    max_disk_io_mbps?: number
    max_network_mbps?: number
  }
}

const AVAILABLE_MODELS = [
  {
    name: 'llama3.1:8b',
    displayName: 'Llama 3.1 8B',
    description: 'Balanced performance and resource usage',
    requirements: { min_vram_gb: 6, min_ram_gb: 8, min_disk_gb: 5, requires_gpu: true, network_bandwidth_mbps: 50 }
  },
  {
    name: 'mistral:7b',
    displayName: 'Mistral 7B',
    description: 'Fast inference, good for most tasks',
    requirements: { min_vram_gb: 4, min_ram_gb: 8, min_disk_gb: 4, requires_gpu: true, network_bandwidth_mbps: 25 }
  },
  {
    name: 'phi3:mini',
    displayName: 'Phi3 Mini',
    description: 'Lightweight, runs on CPU',
    requirements: { min_vram_gb: 2, min_ram_gb: 4, min_disk_gb: 2, requires_gpu: false, network_bandwidth_mbps: 10 }
  }
]

export default function AISetup({ onNext, onBack }: AISetupProps) {
  const [config, setConfig] = useState<AICapabilityConfig>({
    schema_version: '1.0.0',
    contract_version: '2024.1',
    ai_serving_enabled: false,
    ollama_config: undefined,
    model_preferences: [],
    privacy_mode: 'Standard',
    resources: {
      max_cpu_percent: 80,
      max_ram_gb: 8,
      max_vram_gb: undefined,
      max_disk_io_mbps: undefined,
      max_network_mbps: undefined
    }
  })
  
  const [ollamaAvailable, setOllamaAvailable] = useState<boolean | null>(null)
  const [validationResult, setValidationResult] = useState<any>(null)
  const [isValidating, setIsValidating] = useState(false)
  const [isSaving, setIsSaving] = useState(false)
  const [acknowledgedRisks, setAcknowledgedRisks] = useState(false)
  
  const { data: hardware } = useHardwareInfo()

  useEffect(() => {
    loadExistingConfig()
    checkOllamaAvailability()
  }, [])

  useEffect(() => {
    if (config.ai_serving_enabled) {
      validateConfiguration()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [config])

  const loadExistingConfig = async () => {
    try {
      const existingConfig = await invoke<AICapabilityConfig>('load_ai_config')
      setConfig(existingConfig)
    } catch (error) {
      console.error('Failed to load existing config:', error)
    }
  }

  const checkOllamaAvailability = async () => {
    try {
      const response = await fetch('http://localhost:11434/api/version')
      setOllamaAvailable(response.ok)
    } catch {
      setOllamaAvailable(false)
    }
  }

  const validateConfiguration = async () => {
    if (!config.ai_serving_enabled) return
    
    setIsValidating(true)
    try {
      const result = await invoke('validate_ai_capabilities', { config })
      setValidationResult(result)
    } catch (error) {
      console.error('Validation failed:', error)
    } finally {
      setIsValidating(false)
    }
  }

  const updateAIEnabled = (enabled: boolean) => {
    setConfig(prev => ({
      ...prev,
      ai_serving_enabled: enabled,
      ollama_config: enabled ? {
        install_requested: !ollamaAvailable,
        api_endpoint: 'http://localhost:11434',
        models_to_install: ['llama3.1:8b'],
        auto_update: false
      } : undefined,
      model_preferences: enabled ? [
        {
          name: 'llama3.1:8b',
          enabled: true,
          priority: 8,
          requirements: AVAILABLE_MODELS[0].requirements
        }
      ] : []
    }))
  }

  const updateModelSelection = (modelName: string, enabled: boolean) => {
    setConfig(prev => ({
      ...prev,
      model_preferences: prev.model_preferences.map(model =>
        model.name === modelName ? { ...model, enabled } : model
      )
    }))
  }

  const updatePrivacyMode = (mode: 'Standard' | 'Enhanced' | 'Maximum') => {
    setConfig(prev => ({ ...prev, privacy_mode: mode }))
  }

  const handleSaveAndContinue = async () => {
    setIsSaving(true)
    try {
      await invoke('save_ai_config', { config })
      onNext()
    } catch (error) {
      console.error('Failed to save AI config:', error)
    } finally {
      setIsSaving(false)
    }
  }

  const getSystemCompatibility = () => {
    if (!hardware) return null
    
    const ramGB = Math.round(hardware.total_ram / (1024 * 1024 * 1024))
    const bestGpu = hardware.gpus.find(gpu => gpu.is_supported)
    const vramGB = bestGpu ? Math.round(bestGpu.memory / 1024) : 0
    
    return { ramGB, vramGB, hasGpu: !!bestGpu }
  }

  const compatibility = getSystemCompatibility()

  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h2 className="text-2xl font-bold mb-4">AI Capability Setup</h2>
        <p className="text-muted-foreground">
          Configure how your node will serve AI inference tasks to earn additional rewards.
        </p>
      </div>

      {/* AI Serving Toggle */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center justify-between">
            Enable AI Serving
            <Badge variant={config.ai_serving_enabled ? "default" : "secondary"}>
              {config.ai_serving_enabled ? "Enabled" : "Disabled"}
            </Badge>
          </CardTitle>
          <CardDescription>
            AI serving allows your node to run language models and earn higher rewards. 
            This requires additional system resources and may affect other applications.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center space-x-2">
            <Checkbox
              id="ai-enabled"
              checked={config.ai_serving_enabled}
              onCheckedChange={updateAIEnabled}
            />
            <Label htmlFor="ai-enabled" className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
              Enable AI inference serving on this node
            </Label>
          </div>
          
          {config.ai_serving_enabled && (
            <Alert>
              <AlertDescription>
                <strong>Why we ask:</strong> AI serving enables higher-paying tasks but requires significant system resources. 
                We validate your hardware to ensure stable operation and prevent system overload.
              </AlertDescription>
            </Alert>
          )}
        </CardContent>
      </Card>

      {/* Ollama Configuration */}
      {config.ai_serving_enabled && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center justify-between">
              Ollama Runtime
              <Badge variant={ollamaAvailable ? "default" : "destructive"}>
                {ollamaAvailable ? "Available" : "Required"}
              </Badge>
            </CardTitle>
            <CardDescription>
              Ollama is required to run AI models. It provides the runtime environment for language model inference.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            {!ollamaAvailable && (
              <Alert>
                <AlertDescription>
                  <strong>Action Required:</strong> Ollama is not installed or not running. 
                  Please install Ollama from <a href="https://ollama.ai" target="_blank" rel="noopener noreferrer" className="text-blue-600 hover:underline">ollama.ai</a> 
                  and ensure it's running before continuing.
                  <br /><br />
                  <strong>Why we ask:</strong> Ollama manages model loading, memory allocation, and inference execution. 
                  Without it, your node cannot serve AI tasks, resulting in missed earning opportunities.
                </AlertDescription>
              </Alert>
            )}
            
            <div className="flex items-center space-x-2">
              <Checkbox
                id="ollama-install"
                checked={config.ollama_config?.install_requested || false}
                onCheckedChange={(checked: boolean) => {
                  setConfig(prev => ({
                    ...prev,
                    ollama_config: prev.ollama_config ? {
                      ...prev.ollama_config,
                      install_requested: checked
                    } : undefined
                  }))
                }}
                disabled={ollamaAvailable ?? false}
              />
              <Label htmlFor="ollama-install">
                {ollamaAvailable ? "Ollama detected and available" : "Install Ollama automatically (if possible)"}
              </Label>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Model Selection */}
      {config.ai_serving_enabled && (
        <Card>
          <CardHeader>
            <CardTitle>Model Selection</CardTitle>
            <CardDescription>
              Choose which AI models your node can serve. Each model has different requirements and earning potential.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <Alert>
              <AlertDescription>
                <strong>Why we ask:</strong> Different models have varying resource requirements. 
                We match models to your hardware capabilities to ensure optimal performance and prevent system instability.
              </AlertDescription>
            </Alert>

            <div className="space-y-3">
              {AVAILABLE_MODELS.map((model) => {
                const meetsRequirements = compatibility && 
                  compatibility.ramGB >= model.requirements.min_ram_gb &&
                  (!model.requirements.requires_gpu || 
                   (compatibility.hasGpu && compatibility.vramGB >= model.requirements.min_vram_gb))
                
                const currentModelConfig = config.model_preferences.find(m => m.name === model.name)
                
                return (
                  <Card key={model.name} className={`p-4 ${!meetsRequirements ? 'opacity-60' : ''}`}>
                    <div className="flex items-center justify-between">
                      <div className="space-y-1">
                        <div className="flex items-center space-x-2">
                          <Checkbox
                            id={`model-${model.name}`}
                            checked={currentModelConfig?.enabled || false}
                            onCheckedChange={(checked: boolean) => updateModelSelection(model.name, checked)}
                            disabled={!meetsRequirements}
                          />
                          <Label htmlFor={`model-${model.name}`} className="font-medium">
                            {model.displayName}
                          </Label>
                          {meetsRequirements ? (
                            <Badge variant="default">Compatible</Badge>
                          ) : (
                            <Badge variant="destructive">Insufficient Resources</Badge>
                          )}
                        </div>
                        <p className="text-sm text-muted-foreground">{model.description}</p>
                        <p className="text-xs text-muted-foreground">
                          Requires: {model.requirements.min_ram_gb}GB RAM
                          {model.requirements.requires_gpu && `, ${model.requirements.min_vram_gb}GB VRAM`}
                          , {model.requirements.min_disk_gb}GB disk
                        </p>
                      </div>
                    </div>
                  </Card>
                )
              })}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Privacy Mode */}
      {config.ai_serving_enabled && (
        <Card>
          <CardHeader>
            <CardTitle>Privacy Mode</CardTitle>
            <CardDescription>
              Choose how much information your node shares during AI task execution.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <Alert>
              <AlertDescription>
                <strong>Why we ask:</strong> Privacy preferences affect task eligibility and logging levels. 
                Higher privacy may limit some high-paying tasks but provides better data protection.
              </AlertDescription>
            </Alert>

            <RadioGroup
              value={config.privacy_mode}
              onValueChange={(value: string) => updatePrivacyMode(value as 'Standard' | 'Enhanced' | 'Maximum')}
            >
              <div className="space-y-3">
                <div className="flex items-center space-x-2">
                  <RadioGroupItem value="Standard" id="privacy-standard" />
                  <div className="space-y-1">
                    <Label htmlFor="privacy-standard" className="font-medium">
                      Standard Privacy
                    </Label>
                    <p className="text-sm text-muted-foreground">
                      Normal operation with standard telemetry. Eligible for all task types.
                    </p>
                  </div>
                </div>
                
                <div className="flex items-center space-x-2">
                  <RadioGroupItem value="Enhanced" id="privacy-enhanced" />
                  <div className="space-y-1">
                    <Label htmlFor="privacy-enhanced" className="font-medium">
                      Enhanced Privacy
                    </Label>
                    <p className="text-sm text-muted-foreground">
                      Minimal data logging and telemetry. May exclude some specialized tasks.
                    </p>
                  </div>
                </div>
                
                <div className="flex items-center space-x-2">
                  <RadioGroupItem value="Maximum" id="privacy-maximum" />
                  <div className="space-y-1">
                    <Label htmlFor="privacy-maximum" className="font-medium">
                      Maximum Privacy
                    </Label>
                    <p className="text-sm text-muted-foreground">
                      Local processing only, no external calls. Significant task limitations.
                    </p>
                  </div>
                </div>
              </div>
            </RadioGroup>
          </CardContent>
        </Card>
      )}

      {/* Validation Results */}
      {validationResult && config.ai_serving_enabled && (
        <Card>
          <CardHeader>
            <CardTitle>Configuration Validation</CardTitle>
          </CardHeader>
          <CardContent>
            {validationResult.system_validation?.errors?.length > 0 && (
              <Alert variant="destructive" className="mb-4">
                <AlertDescription>
                  <strong>Configuration Issues:</strong>
                  <ul className="mt-2 list-disc list-inside space-y-1">
                    {validationResult.system_validation.errors.map((error: string, idx: number) => (
                      <li key={idx} className="text-sm">{error}</li>
                    ))}
                  </ul>
                </AlertDescription>
              </Alert>
            )}

            {validationResult.system_validation?.warnings?.length > 0 && (
              <Alert className="mb-4">
                <AlertDescription>
                  <strong>Performance Warnings:</strong>
                  <ul className="mt-2 list-disc list-inside space-y-1">
                    {validationResult.system_validation.warnings.map((warning: string, idx: number) => (
                      <li key={idx} className="text-sm">{warning}</li>
                    ))}
                  </ul>
                </AlertDescription>
              </Alert>
            )}

            <Badge variant={
              validationResult.compatibility_status === 'Optimal' ? 'default' :
              validationResult.compatibility_status === 'Acceptable' ? 'secondary' :
              validationResult.compatibility_status === 'Limited' ? 'outline' : 'destructive'
            }>
              {validationResult.compatibility_status} Compatibility
            </Badge>
          </CardContent>
        </Card>
      )}

      {/* Risk Acknowledgment for AI Setup with Validation Errors */}
      {config.ai_serving_enabled && validationResult?.system_validation?.errors?.length > 0 && (
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center space-x-2">
              <Checkbox
                id="acknowledge-risks"
                checked={acknowledgedRisks}
                onCheckedChange={(checked: boolean) => setAcknowledgedRisks(checked)}
              />
              <Label htmlFor="acknowledge-risks" className="text-sm">
                I understand the configuration issues above and want to proceed anyway. I can install Ollama later or skip AI serving.
              </Label>
            </div>
          </CardContent>
        </Card>
      )}

      <div className="flex justify-between">
        <Button variant="outline" onClick={onBack}>
          Back to System Check
        </Button>
        <Button
          onClick={handleSaveAndContinue}
          disabled={
            isSaving || 
            isValidating || 
            (config.ai_serving_enabled && validationResult?.system_validation?.errors?.length > 0 && !acknowledgedRisks)
          }
        >
          {isSaving ? "Saving..." : "Continue to Wallet Setup"}
        </Button>
      </div>
    </div>
  )
}