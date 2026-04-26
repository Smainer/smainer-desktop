import React from 'react'
import { useHardwareInfo, useSystemRequirements } from '../../hooks/useHardwareInfo'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../ui/card'
import { Button } from '../ui/button'
import { Progress } from '../ui/progress'

export default function HardwareConfig() {
  const { data: hardware, isLoading: hwLoading, refetch: refetchHw } = useHardwareInfo()
  const { data: requirements, isLoading: reqLoading, refetch: refetchReq } = useSystemRequirements()

  const isLoading = hwLoading || reqLoading

  const handleRefresh = () => {
    refetchHw()
    refetchReq()
  }

  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Hardware Configuration</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="animate-pulse space-y-4">
            <div className="h-4 bg-card rounded"></div>
            <div className="h-4 bg-card rounded"></div>
            <div className="h-4 bg-card rounded"></div>
          </div>
        </CardContent>
      </Card>
    )
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>Hardware Configuration</CardTitle>
            <CardDescription>
              System specifications and capabilities
            </CardDescription>
          </div>
          <Button variant="outline" size="sm" onClick={handleRefresh}>
            Refresh
          </Button>
        </div>
      </CardHeader>
      <CardContent className="space-y-6">
        
        {/* System Overview */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="space-y-2">
            <h3 className="font-medium text-muted-foreground">System Information</h3>
            <div className="text-sm space-y-1">
              <div>OS: {hardware?.os} {hardware?.os_version}</div>
              <div>CPU: {hardware?.cpu_name}</div>
              <div>Cores: {hardware?.cpu_cores}</div>
              <div>RAM: {hardware ? Math.round(hardware.total_ram / (1024 * 1024 * 1024)) : 0} GB</div>
            </div>
          </div>
          
          <div className="space-y-2">
            <h3 className="font-medium text-muted-foreground">Capacity Status</h3>
            <div className="space-y-3">
              <div>
                <div className="flex justify-between text-sm mb-1">
                  <span>Memory Usage</span>
                  <span>
                    {hardware ? Math.round((hardware.total_ram - hardware.available_ram) / hardware.total_ram * 100) : 0}%
                  </span>
                </div>
                <Progress 
                  value={hardware ? (hardware.total_ram - hardware.available_ram) / hardware.total_ram * 100 : 0} 
                  className="h-2" 
                />
              </div>
              
              <div className={`text-sm p-2 rounded border ${
                requirements?.meets_requirements 
                  ? 'bg-primary/10 border-primary/20 text-primary'
                  : 'bg-destructive/10 border-destructive/20 text-destructive'
              }`}>
                {requirements?.meets_requirements 
                  ? 'System meets all requirements'
                  : 'System has compatibility issues'
                }
              </div>
            </div>
          </div>
        </div>

        {/* GPU Information */}
        <div>
          <h3 className="font-medium text-muted-foreground mb-3">Graphics Cards</h3>
          {hardware?.gpus && hardware.gpus.length > 0 ? (
            <div className="space-y-3">
              {hardware.gpus.map((gpu, idx) => (
                <div 
                  key={idx} 
                  className={`p-4 border rounded-lg ${
                    gpu.is_supported 
                      ? 'border-primary/20 bg-primary/10'
                      : 'border-border bg-muted'
                  }`}
                >
                  <div className="flex items-start justify-between">
                    <div>
                      <div className="font-medium">{gpu.name}</div>
                      <div className="text-sm text-muted-foreground">Vendor: {gpu.vendor}</div>
                      <div className="text-sm text-muted-foreground">
                        VRAM: {Math.round(gpu.memory / 1024).toFixed(1)} GB
                      </div>
                      <div className="text-sm text-muted-foreground">
                        Driver: {gpu.driver_version}
                      </div>
                    </div>
                    <div className={`px-2 py-1 rounded text-xs font-medium ${
                      gpu.is_supported 
                        ? 'bg-primary/20 text-primary'
                        : 'bg-muted text-muted-foreground'
                    }`}>
                      {gpu.is_supported ? 'Supported' : 'Limited Support'}
                    </div>
                  </div>
                  
                  {!gpu.is_supported && (
                    <div className="mt-2 text-xs text-muted-foreground">
                      This GPU may have limited task compatibility or performance
                    </div>
                  )}
                </div>
              ))}
              
              <div className="text-xs text-muted-foreground">
                Total GPU Memory: {Math.round(hardware.gpus.reduce((sum, gpu) => sum + gpu.memory, 0) / 1024)} GB
              </div>
            </div>
          ) : (
            <div className="p-4 border border-destructive/20 bg-destructive/10 rounded-lg text-destructive">
              <div className="font-medium">No GPUs detected</div>
              <div className="text-sm mt-1">
                GPU acceleration is recommended for optimal earnings. Your node will be limited to CPU-only tasks.
              </div>
            </div>
          )}
        </div>

        {/* Warnings and Errors */}
        {requirements && (requirements.warnings.length > 0 || requirements.errors.length > 0) && (
          <div className="space-y-3">
            {requirements.errors.length > 0 && (
              <div className="p-4 border border-destructive/20 bg-destructive/10 rounded-lg">
                <div className="font-medium text-destructive mb-2">Critical Issues</div>
                <ul className="list-disc list-inside space-y-1 text-sm text-destructive">
                  {requirements.errors.map((error, idx) => (
                    <li key={idx}>{error}</li>
                  ))}
                </ul>
              </div>
            )}
            
            {requirements.warnings.length > 0 && (
              <div className="p-4 border border-border bg-muted rounded-lg">
                <div className="font-medium text-muted-foreground mb-2">Recommendations</div>
                <ul className="list-disc list-inside space-y-1 text-sm text-muted-foreground">
                  {requirements.warnings.map((warning, idx) => (
                    <li key={idx}>{warning}</li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        )}

        {/* Hardware Optimization Tips */}
        <div className="p-4 border border-primary/20 bg-primary/10 rounded-lg">
          <div className="font-medium text-primary mb-2">Optimization Tips</div>
          <ul className="list-disc list-inside space-y-1 text-sm text-muted-foreground">
            <li>Close unnecessary applications to free up RAM and GPU resources</li>
            <li>Ensure GPU drivers are up to date for best compatibility</li>
            <li>Consider upgrading to a GPU with 8GB+ VRAM for higher-value tasks</li>
            <li>Monitor temperatures to prevent thermal throttling during intensive tasks</li>
          </ul>
        </div>
      </CardContent>
    </Card>
  )
}