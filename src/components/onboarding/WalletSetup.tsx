import React, { useState } from 'react'
import { useMutation } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { Button } from '../ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../ui/card'
import { toast } from 'sonner'

interface WalletSetupProps {
  onNext: (address: string) => void
  onBack: () => void
}

interface WalletInfo {
  address: string
  public_key: string
  created_at: string
  encrypted: boolean
}

export default function WalletSetup({ onNext, onBack }: WalletSetupProps) {
  const [password, setPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [importKey, setImportKey] = useState('')
  const [mode, setMode] = useState<'generate' | 'import'>('generate')
  const [showPassword, setShowPassword] = useState(false)

  const generateWallet = useMutation({
    mutationFn: (password?: string) => 
      invoke<WalletInfo>('generate_wallet', { password }),
    onSuccess: (wallet) => {
      toast.success('Wallet generated successfully!')
      onNext(wallet.address)
    },
    onError: (error: any) => {
      toast.error(`Failed to generate wallet: ${error}`)
    },
  })

  const importWallet = useMutation({
    mutationFn: ({ privateKey, password }: { privateKey: string, password?: string }) =>
      invoke<WalletInfo>('import_wallet', { privateKey, password }),
    onSuccess: (wallet) => {
      toast.success('Wallet imported successfully!')
      onNext(wallet.address)
    },
    onError: (error: any) => {
      toast.error(`Failed to import wallet: ${error}`)
    },
  })

  const handleGenerate = () => {
    if (password && password !== confirmPassword) {
      toast.error('Passwords do not match')
      return
    }
    
    if (password && password.length < 8) {
      toast.error('Password must be at least 8 characters')
      return
    }

    generateWallet.mutate(password || undefined)
  }

  const handleImport = () => {
    if (!importKey || importKey.length !== 66 || !importKey.startsWith('0x')) {
      toast.error('Please enter a valid private key (64 hex characters with 0x prefix)')
      return
    }

    if (password && password !== confirmPassword) {
      toast.error('Passwords do not match')
      return
    }

    if (password && password.length < 8) {
      toast.error('Password must be at least 8 characters')
      return
    }

    importWallet.mutate({ 
      privateKey: importKey, 
      password: password || undefined 
    })
  }

  const isLoading = generateWallet.isPending || importWallet.isPending

  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h2 className="text-2xl font-bold mb-4">Wallet Setup</h2>
        <p className="text-gray-600">
          Create or import a wallet to receive earnings from your provider node.
        </p>
      </div>

      {/* Mode Selection */}
      <div className="flex space-x-4 mb-6">
        <Button
          variant={mode === 'generate' ? 'default' : 'outline'}
          onClick={() => setMode('generate')}
          className="flex-1"
        >
          🎲 Generate New Wallet
        </Button>
        <Button
          variant={mode === 'import' ? 'default' : 'outline'}
          onClick={() => setMode('import')}
          className="flex-1"
        >
          📥 Import Existing Wallet
        </Button>
      </div>

      {mode === 'generate' && (
        <Card>
          <CardHeader>
            <CardTitle>Generate New Wallet</CardTitle>
            <CardDescription>
              Creates a new Starknet wallet with a secure private key. You can optionally encrypt it with a password.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
              <div className="flex items-start space-x-2">
                <span className="text-yellow-600 mt-0.5">⚠️</span>
                <div className="text-sm text-yellow-800">
                  <strong>Important:</strong> Make sure to backup your private key after generation. 
                  Store it securely - if you lose it, you'll lose access to your wallet and earnings.
                </div>
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium mb-2">
                Optional Password Protection
              </label>
              <input
                type={showPassword ? 'text' : 'password'}
                placeholder="Enter password (optional)..."
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              />
              <p className="text-xs text-gray-500 mt-1">
                Leave empty to skip password protection
              </p>
            </div>

            {password && (
              <div>
                <input
                  type="password"
                  placeholder="Confirm password..."
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                />
              </div>
            )}

            <div className="flex items-center space-x-2">
              <input
                type="checkbox"
                id="showPassword"
                checked={showPassword}
                onChange={(e) => setShowPassword(e.target.checked)}
                className="rounded"
              />
              <label htmlFor="showPassword" className="text-sm text-gray-600">
                Show password
              </label>
            </div>
          </CardContent>
        </Card>
      )}

      {mode === 'import' && (
        <Card>
          <CardHeader>
            <CardTitle>Import Existing Wallet</CardTitle>
            <CardDescription>
              Import an existing Starknet wallet using your private key.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div>
              <label className="block text-sm font-medium mb-2">
                Private Key
              </label>
              <textarea
                placeholder="0x1234567890abcdef..."
                value={importKey}
                onChange={(e) => setImportKey(e.target.value.trim())}
                className="w-full h-24 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 font-mono text-sm resize-none"
              />
              <p className="text-xs text-gray-500 mt-1">
                Enter your 64-character hex private key starting with '0x'
              </p>
            </div>

            <div>
              <label className="block text-sm font-medium mb-2">
                Password Protection (Optional)
              </label>
              <input
                type={showPassword ? 'text' : 'password'}
                placeholder="Enter password (optional)..."
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              />
            </div>

            {password && (
              <div>
                <input
                  type="password"
                  placeholder="Confirm password..."
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                />
              </div>
            )}

            <div className="flex items-center space-x-2">
              <input
                type="checkbox"
                id="showImportPassword"
                checked={showPassword}
                onChange={(e) => setShowPassword(e.target.checked)}
                className="rounded"
              />
              <label htmlFor="showImportPassword" className="text-sm text-gray-600">
                Show password
              </label>
            </div>
          </CardContent>
        </Card>
      )}

      <div className="flex justify-between">
        <Button variant="outline" onClick={onBack} disabled={isLoading}>
          Back
        </Button>
        <Button
          onClick={mode === 'generate' ? handleGenerate : handleImport}
          disabled={isLoading || (mode === 'import' && !importKey)}
          className="px-8"
        >
          {isLoading ? (
            <div className="flex items-center space-x-2">
              <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
              <span>{mode === 'generate' ? 'Generating...' : 'Importing...'}</span>
            </div>
          ) : (
            mode === 'generate' ? 'Generate Wallet' : 'Import Wallet'
          )}
        </Button>
      </div>
    </div>
  )
}