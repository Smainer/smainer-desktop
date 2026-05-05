import { useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../ui/card'
import { Button } from '../ui/button'
import { toast } from 'sonner'
import { invoke } from '@tauri-apps/api/core'
import { Download, FileText, Loader2 } from 'lucide-react'

interface DiagnosticsBundle {
  bundle_path: string;
  created_at: string;
  items_collected: string[];
  summary?: {
    provider_running: boolean;
    relayer_health_ok: boolean;
    ollama_api_ok: boolean;
    ai_enabled: boolean;
    node_id: string;
    relayer_url: string;
  };
}

export default function DiagnosticsExport() {
  const [isExporting, setIsExporting] = useState(false)
  const [lastBundle, setLastBundle] = useState<DiagnosticsBundle | null>(null)

  const handleExportDiagnostics = async () => {
    if (isExporting) return;
    
    setIsExporting(true)
    
    try {
      const bundle: DiagnosticsBundle = await invoke('export_diagnostics_bundle')
      
      setLastBundle(bundle)
      
      toast.success('Diagnostics bundle created successfully!', {
        description: `Bundle saved to: ${bundle.bundle_path}`,
        action: {
          label: 'Copy Path',
          onClick: () => {
            navigator.clipboard.writeText(bundle.bundle_path)
            toast.success('Path copied to clipboard')
          },
        },
        duration: 10000,
      })
    } catch (error) {
      console.error('Failed to export diagnostics:', error)
      toast.error('Failed to create diagnostics bundle', {
        description: error as string,
      })
    } finally {
      setIsExporting(false)
    }
  }

  const formatDate = (dateStr: string) => {
    try {
      return new Date(dateStr).toLocaleString()
    } catch {
      return dateStr
    }
  }

  return (
    <Card className="bg-gradient-to-br from-blue-50 to-indigo-50 border-blue-200">
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-blue-900">
          <FileText className="h-5 w-5" />
          Diagnostics Export
        </CardTitle>
        <CardDescription className="text-blue-700">
          Export a support bundle for troubleshooting relayer connectivity issues.
          All sensitive data (private keys) is automatically redacted for security.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="bg-blue-100 p-3 rounded-lg border border-blue-200">
          <h4 className="font-medium text-blue-900 mb-2">Bundle includes:</h4>
          <ul className="text-sm text-blue-800 space-y-1">
            <li>• Provider daemon logs (if available)</li>
            <li>• Startup configuration logs</li>  
            <li>• Network connectivity probes (DNS, HTTPS, WebSocket)</li>
            <li>• Wallet configuration (private key redacted)</li>
            <li>• System information and node status</li>
          </ul>
        </div>
        
        <div className="flex items-center gap-2">
          <Button 
            onClick={handleExportDiagnostics}
            disabled={isExporting}
            className="bg-blue-600 hover:bg-blue-700 text-white"
          >
            {isExporting ? (
              <>
                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                Creating Bundle...
              </>
            ) : (
              <>
                <Download className="h-4 w-4 mr-2" />
                Export Diagnostics Bundle
              </>
            )}
          </Button>
        </div>

        {lastBundle && (
          <div className="bg-green-50 p-3 rounded-lg border border-green-200">
            <h4 className="font-medium text-green-900 mb-2">Last export:</h4>
            <p className="text-sm text-green-800 mb-1">
              Created: {formatDate(lastBundle.created_at)}
            </p>
            <p className="text-sm text-green-800 mb-2">
              Items: {lastBundle.items_collected.join(', ')}
            </p>
            {lastBundle.summary && (
              <div className="text-xs text-green-800 mb-2 grid grid-cols-1 md:grid-cols-2 gap-1">
                <span>Provider: {lastBundle.summary.provider_running ? 'running' : 'stopped'}</span>
                <span>Relayer health: {lastBundle.summary.relayer_health_ok ? 'ok' : 'failed'}</span>
                <span>Ollama API: {lastBundle.summary.ollama_api_ok ? 'ok' : 'failed'}</span>
                <span>AI enabled: {lastBundle.summary.ai_enabled ? 'yes' : 'no'}</span>
              </div>
            )}
            <p className="text-xs text-green-700 font-mono break-all">
              {lastBundle.bundle_path}
            </p>
            <Button
              variant="outline"
              size="sm"
              className="mt-2 text-green-700 border-green-300 hover:bg-green-100"
              onClick={() => {
                navigator.clipboard.writeText(lastBundle.bundle_path)
                toast.success('Bundle path copied to clipboard')
              }}
            >
              Copy Path
            </Button>
          </div>
        )}

        <div className="text-xs text-gray-600 bg-gray-50 p-2 rounded border">
          <strong>Privacy note:</strong> This bundle is designed to be safe to share with Smainer support. 
          Private keys are automatically redacted (only first 6 + last 4 characters shown). 
          Please review the contents before sharing if you have concerns about other sensitive data.
        </div>
      </CardContent>
    </Card>
  )
}