import React from 'react'
import { useSystemRequirements, useHardwareInfo } from '../../hooks/useHardwareInfo'
import { Button } from '../ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../ui/card'
import { Progress } from '../ui/progress'

interface SystemCheckProps {
  onNext: () => void
}

export default function SystemCheck({ onNext }: SystemCheckProps) {
  const { data: requirements, isLoading: reqLoading, error: reqError } = useSystemRequirements()
  const { data: hardware, isLoading: hwLoading } = useHardwareInfo()

  const isLoading = reqLoading || hwLoading

  if (isLoading) {
    return (
      <div className="space-y-6">
        <div className="text-center">
          <h2 className="text-2xl font-bold mb-4">Checking Your System</h2>
          <p className="text-gray-600 mb-8">Please wait while we analyze your hardware...</p>
          <div className="spinner mx-auto" />
        </div>
      </div>
    )
  }

  if (reqError) {
    return (
      <div className="text-center">
        <h2 className="text-2xl font-bold mb-4 text-red-600">System Check Failed</h2>
        <p className="text-gray-600 mb-8">Unable to analyze your system. Please try again.</p>
        <Button onClick={() => window.location.reload()}>Retry</Button>
      </div>
    )
  }

  const ramGB = hardware ? Math.round(hardware.total_ram / (1024 * 1024 * 1024)) : 0
  const supportedGpus = hardware?.gpus.filter(gpu => gpu.is_supported) || []
  const bestGpu = supportedGpus.reduce((best, current) => 
    current.memory > (best?.memory || 0) ? current : best, null as any
  )

  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h2 className="text-2xl font-bold mb-4">System Requirements Check</h2>
        <p className="text-gray-600">
          Let's make sure your system can run a Smainer provider node effectively.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-8">
        {/* CPU Check */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-lg flex items-center">
              {requirements?.cpu_ok ? '✅' : '⚠️'} CPU
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-gray-600">{hardware?.cpu_name}</p>
            <p className="text-lg font-bold">{hardware?.cpu_cores} cores</p>
            <p className="text-xs mt-1 text-gray-500">
              {requirements?.cpu_ok ? 'Excellent' : 'Minimum 4 cores recommended'}
            </p>
          </CardContent>
        </Card>

        {/* RAM Check */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-lg flex items-center">
              {requirements?.ram_ok ? '✅' : '❌'} Memory
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-lg font-bold">{ramGB} GB RAM</p>
            <Progress 
              value={(ramGB / 32) * 100} 
              className="mt-2" 
            />
            <p className="text-xs mt-1 text-gray-500">
              {requirements?.ram_ok ? 'Sufficient' : 'Need 8GB+ for stable operation'}
            </p>
          </CardContent>
        </Card>

        {/* GPU Check */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-lg flex items-center">
              {requirements?.gpu_ok ? '✅' : '❌'} GPU
            </CardTitle>
          </CardHeader>
          <CardContent>
            {bestGpu ? (
              <>
                <p className="text-sm text-gray-600 truncate">{bestGpu.name}</p>
                <p className="text-lg font-bold">{Math.round(bestGpu.memory / 1024)} GB VRAM</p>
                <p className="text-xs mt-1 text-gray-500">
                  {bestGpu.memory >= 4096 ? 'Ready for AI tasks' : 'Limited task capacity'}
                </p>
              </>
            ) : (
              <>
                <p className="text-sm text-gray-600">No supported GPU found</p>
                <p className="text-xs mt-1 text-red-500">
                  GPU required for optimal earnings
                </p>
              </>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Warnings and Errors */}
      {requirements && (requirements.warnings.length > 0 || requirements.errors.length > 0) && (
        <div className="space-y-4">
          {requirements.errors.length > 0 && (
            <Card className="border-red-200 bg-red-50">
              <CardHeader>
                <CardTitle className="text-red-800">❌ Critical Issues</CardTitle>
              </CardHeader>
              <CardContent>
                <ul className="list-disc list-inside space-y-1">
                  {requirements.errors.map((error, idx) => (
                    <li key={idx} className="text-red-700 text-sm">{error}</li>
                  ))}
                </ul>
              </CardContent>
            </Card>
          )}
          
          {requirements.warnings.length > 0 && (
            <Card className="border-yellow-200 bg-yellow-50">
              <CardHeader>
                <CardTitle className="text-yellow-800">⚠️ Recommendations</CardTitle>
              </CardHeader>
              <CardContent>
                <ul className="list-disc list-inside space-y-1">
                  {requirements.warnings.map((warning, idx) => (
                    <li key={idx} className="text-yellow-700 text-sm">{warning}</li>
                  ))}
                </ul>
              </CardContent>
            </Card>
          )}
        </div>
      )}

      {/* Overall Status */}
      <Card className={`${
        requirements?.meets_requirements 
          ? 'border-green-200 bg-green-50' 
          : 'border-red-200 bg-red-50'
      }`}>
        <CardHeader>
          <CardTitle className={requirements?.meets_requirements ? 'text-green-800' : 'text-red-800'}>
            {requirements?.meets_requirements ? '🎉 System Ready!' : '⚠️ System Issues Detected'}
          </CardTitle>
          <CardDescription>
            {requirements?.meets_requirements 
              ? 'Your system meets all requirements for running a Smainer provider node.'
              : 'Some issues were found that may affect performance or prevent operation.'
            }
          </CardDescription>
        </CardHeader>
      </Card>

      <div className="flex justify-end">
        <Button 
          onClick={onNext}
          disabled={!requirements?.meets_requirements}
          className="px-8"
        >
          {requirements?.meets_requirements ? 'Continue to Wallet Setup' : 'Fix Issues First'}
        </Button>
      </div>
    </div>
  )
}