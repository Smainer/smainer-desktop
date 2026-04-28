import React, { useState } from 'react'
import { useMutation } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { Button } from '../ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../ui/card'
import { Alert, AlertDescription } from '../ui/alert'
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
  private_key?: string // BUG FIX: Added optional private_key field (only returned on generation)
}

export default function WalletSetup({ onNext, onBack }: WalletSetupProps) {
  const [password, setPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [importKey, setImportKey] = useState('')
  const [mode, setMode] = useState<'generate' | 'import'>('generate')
  const [showPassword, setShowPassword] = useState(false)
  // BUG FIX: State for showing private key modal
  const [generatedPrivateKey, setGeneratedPrivateKey] = useState<string | null>(null)
  const [privateKeySaved, setPrivateKeySaved] = useState(false)
  const [walletAddress, setWalletAddress] = useState<string | null>(null)

  const generateWallet = useMutation({
    mutationFn: (password?: string) => 
      invoke<WalletInfo>('generate_wallet', { password }),
    onSuccess: (wallet) => {
      // BUG FIX: Show private key modal instead of immediately proceeding
      if (wallet.private_key) {
        setGeneratedPrivateKey(wallet.private_key)
        setWalletAddress(wallet.address)
        setPrivateKeySaved(false)
      } else {
        toast.success('Wallet generated successfully!')
        onNext(wallet.address)
      }
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

  const handlePrivateKeyConfirmation = () => {
    if (!privateKeySaved) {
      toast.error('Please confirm you have saved your private key')
      return
    }
    if (walletAddress) {
      toast.success('Wallet generated successfully!')
      onNext(walletAddress)
    }
  }

  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text)
      toast.success('Copied to clipboard!')
    } catch (error) {
      toast.error('Failed to copy to clipboard')
    }
  }

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

  // BUG FIX: Private key display modal
  if (generatedPrivateKey) {
    return (
      <div className="space-y-6">
        <div className="text-center mb-8">
          <h2 className="text-2xl font-bold mb-4 text-destructive">⚠️ Save Your Private Key</h2>
          <p className="text-muted-foreground">
            This is the ONLY time you will see your private key. Save it securely now!
          </p>
        </div>

        <Card className="border-destructive">
          <CardHeader>
            <CardTitle className="text-destructive">Your Private Key</CardTitle>
            <CardDescription>
              Store this in a secure password manager. Never share it with anyone.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="bg-muted p-4 rounded-lg border-2 border-destructive">
              <div className="font-mono text-sm break-all select-all">
                {generatedPrivateKey}
              </div>
            </div>

            <Button
              onClick={() => copyToClipboard(generatedPrivateKey)}
              variant="outline"
              className="w-full"
            >
              Copy Private Key
            </Button>

            <Alert variant="destructive">
              <AlertDescription>
                <strong>CRITICAL WARNING:</strong>
                <ul className="mt-2 list-disc list-inside space-y-1 text-sm">
                  <li>Anyone with this key can access your wallet and steal your funds</li>
                  <li>If you lose this key, you will lose access to your wallet forever</li>
                  <li>Smainer support will NEVER ask for your private key</li>
                  <li>Store it offline in multiple secure locations</li>
                </ul>
              </AlertDescription>
            </Alert>

            <div className="flex items-center space-x-2 p-4 bg-muted rounded-lg">
              <input
                type="checkbox"
                id="confirmSaved"
                checked={privateKeySaved}
                onChange={(e) => setPrivateKeySaved(e.target.checked)}
                className="rounded"
              />
              <label htmlFor="confirmSaved" className="text-sm font-medium">
                I have securely saved my private key and understand that I cannot recover it if lost
              </label>
            </div>

            <div className="flex justify-between pt-4">
              <Button
                variant="outline"
                onClick={() => {
                  setGeneratedPrivateKey(null)
                  setWalletAddress(null)
                  setPrivateKeySaved(false)
                }}
              >
                Cancel
              </Button>
              <Button
                onClick={handlePrivateKeyConfirmation}
                disabled={!privateKeySaved}
                className="px-8"
              >
                I Have Saved My Key - Continue
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h2 className="text-2xl font-bold mb-4">Wallet Setup</h2>
        <p className="text-muted-foreground">
          Your Starknet wallet address is where you'll receive earnings. We'll securely store your private key locally.
        </p>
      </div>

      {/* Mode Selection */}
      <div className="flex space-x-4 mb-6">
        <Button
          variant={mode === 'generate' ? 'default' : 'outline'}
          onClick={() => setMode('generate')}
          className="flex-1"
        >
          Generate New Wallet
        </Button>
        <Button
          variant={mode === 'import' ? 'default' : 'outline'}
          onClick={() => setMode('import')}
          className="flex-1"
        >
          Import Existing Wallet
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
            <div className="bg-muted/50 border border-border rounded-lg p-4">
              <div className="flex items-start space-x-2">
                <span className="text-destructive mt-0.5">Warning</span>
                <div className="text-sm text-muted-foreground">
                  <strong>Important:</strong> Make sure to backup your private key after generation. 
                  Store it securely - if you lose it, you'll lose access to your wallet and earnings.
                </div>
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium mb-2">
                Encryption Password (Optional)
              </label>
              <input
                type={showPassword ? 'text' : 'password'}
                placeholder="Enter password (optional)..."
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="w-full px-3 py-2 border border-border rounded-lg focus:ring-2 focus:ring-ring focus:border-ring"
              />
              <p className="text-xs text-muted-foreground mt-1">
                Optional: Encrypt your locally-stored private key with a password for extra security
              </p>
            </div>

            {password && (
              <div>
                <input
                  type="password"
                  placeholder="Confirm password..."
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  className="w-full px-3 py-2 border border-border rounded-lg focus:ring-2 focus:ring-ring focus:border-ring"
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
              <label htmlFor="showPassword" className="text-sm text-muted-foreground">
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
                Your Starknet Private Key
              </label>
              <textarea
                placeholder="0x1234567890abcdef..."
                value={importKey}
                onChange={(e) => setImportKey(e.target.value.trim())}
                className="w-full h-24 px-3 py-2 border border-border rounded-lg focus:ring-2 focus:ring-ring focus:border-ring font-mono text-sm resize-none"
              />
              <p className="text-xs text-muted-foreground mt-1">
                Paste your existing Starknet private key (66 hex characters starting with 0x)
              </p>
            </div>

            <div>
              <label className="block text-sm font-medium mb-2">
                Encryption Password (Optional)
              </label>
              <input
                type={showPassword ? 'text' : 'password'}
                placeholder="Enter password (optional)..."
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="w-full px-3 py-2 border border-border rounded-lg focus:ring-2 focus:ring-ring focus:border-ring"
              />
              <p className="text-xs text-muted-foreground mt-1">
                Optional: Encrypt your imported private key with a password for extra security
              </p>
            </div>

            {password && (
              <div>
                <input
                  type="password"
                  placeholder="Confirm password..."
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  className="w-full px-3 py-2 border border-border rounded-lg focus:ring-2 focus:ring-ring focus:border-ring"
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
              <label htmlFor="showImportPassword" className="text-sm text-muted-foreground">
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
              <div className="w-4 h-4 border-2 border-primary border-t-transparent rounded-full animate-spin" />
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