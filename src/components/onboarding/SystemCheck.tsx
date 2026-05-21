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
          <p className="text-muted-foreground mb-8">Please wait while we analyze your hardware...</p>
          <div className="spinner mx-auto" />
        </div>
      </div>
    )
  }

  if (reqError) {
    return (
      <div className="text-center">
        <h2 className="text-2xl font-bold mb-4 text-red-600">Hardware Detection Failed</h2>
        <p className="text-muted-foreground mb-8">Cannot analyze system capabilities. Restart the application or check system permissions.</p>
        <Button onClick={() => window.location.reload()}>Restart System Check</Button>
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
        <h2 className="text-2xl font-bold mb-4">Hardware Validation</h2>
        <p className="text-muted-foreground">
          Verify your system meets requirements for stable provider operation and optimal earnings.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-8">
        {/* CPU Check */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-lg flex items-center">
              {requirements?.cpu_ok ? 'OK' : 'Warning'} CPU
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">{hardware?.cpu_name}</p>
            <p className="text-lg font-bold">{hardware?.cpu_cores} cores</p>
            <p className="text-xs mt-1 text-muted-foreground/80">
              {requirements?.cpu_ok ? 'Excellent' : 'Minimum 4 cores recommended'}
            </p>
          </CardContent>
        </Card>

        {/* RAM Check */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-lg flex items-center">
              {requirements?.ram_ok ? 'OK' : 'Error'} Memory
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-lg font-bold">{ramGB} GB RAM</p>
            <Progress 
              value={(ramGB / 32) * 100} 
              className="mt-2" 
            />
            <p className="text-xs mt-1 text-muted-foreground/80">
              {requirements?.ram_ok ? 'Sufficient' : 'Need 8GB+ for stable operation'}
            </p>
          </CardContent>
        </Card>

        {/* GPU Check */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-lg flex items-center">
              {requirements?.gpu_ok ? 'OK' : 'Error'} GPU
            </CardTitle>
          </CardHeader>
          <CardContent>
            {bestGpu ? (
              <>
                <p className="text-sm text-muted-foreground truncate">{bestGpu.name}</p>
                <p className="text-lg font-bold">{Math.round(bestGpu.memory / 1024)} GB VRAM</p>
                <p className="text-xs mt-1 text-muted-foreground/80">
                  {bestGpu.memory >= 4096 ? 'Ready for AI tasks' : 'Limited task capacity'}
                </p>
              </>
            ) : (
              <>
                <p className="text-sm text-muted-foreground">No supported GPU found</p>
                <p className="text-xs mt-1 text-destructive">
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
            <Card className="border-destructive/20 bg-destructive/5">
              <CardHeader>
                <CardTitle className="text-destructive">Critical Issues</CardTitle>
              </CardHeader>
              <CardContent>
                <ul className="list-disc list-inside space-y-1">
                  {requirements.errors.map((error, idx) => (
                    <li key={idx} className="text-destructive text-sm">{error}</li>
                  ))}
                </ul>
              </CardContent>
            </Card>
          )}
          
          {requirements.warnings.length > 0 && (
            <Card className="border-yellow-500/20 bg-yellow-500/5">
              <CardHeader>
                <CardTitle className="text-muted-foreground">Recommendations</CardTitle>
              </CardHeader>
              <CardContent>
                <ul className="list-disc list-inside space-y-1">
                  {requirements.warnings.map((warning, idx) => (
                    <li key={idx} className="text-yellow-500 text-sm">{warning}</li>
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
          ? 'border-green-500/20 bg-green-500/5' 
          : 'border-destructive/20 bg-destructive/5'
      }`}>
        <CardHeader>
          <CardTitle className={requirements?.meets_requirements ? 'text-green-800' : 'text-red-800'}>
            {requirements?.meets_requirements ? 'Hardware Ready' : 'Requirements Not Met'}
          </CardTitle>
          <CardDescription>
            {requirements?.meets_requirements 
              ? 'System meets all requirements for provider node operation.'
              : 'Hardware issues detected that will affect node performance or prevent operation.'
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
          {requirements?.meets_requirements ? 'Continue Setup' : 'Fix Hardware Issues'}
        </Button>
      </div>
    </div>
  )
}