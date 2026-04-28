import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import WalletSetup from '../components/onboarding/WalletSetup'
import { invoke } from '@tauri-apps/api/core'

const createTestQueryClient = () =>
  new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  })

describe('WalletSetup - Copy Clarity', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('should display clear explanation of wallet purpose', () => {
    const onNext = vi.fn()
    const onBack = vi.fn()
    const queryClient = createTestQueryClient()

    render(
      <QueryClientProvider client={queryClient}>
        <WalletSetup onNext={onNext} onBack={onBack} />
      </QueryClientProvider>
    )

    expect(
      screen.getByText(
        /Your Starknet wallet address is where you'll receive earnings/i
      )
    ).toBeInTheDocument()
    expect(
      screen.getByText(/We'll securely store your private key locally/i)
    ).toBeInTheDocument()
  })

  it('should clarify encryption password is optional in generate mode', () => {
    const onNext = vi.fn()
    const onBack = vi.fn()
    const queryClient = createTestQueryClient()

    render(
      <QueryClientProvider client={queryClient}>
        <WalletSetup onNext={onNext} onBack={onBack} />
      </QueryClientProvider>
    )

    expect(screen.getByText('Encryption Password (Optional)')).toBeInTheDocument()
    expect(
      screen.getByText(
        /Optional: Encrypt your locally-stored private key with a password/i
      )
    ).toBeInTheDocument()
  })

  it('should clarify encryption password is optional in import mode', async () => {
    const user = userEvent.setup()
    const onNext = vi.fn()
    const onBack = vi.fn()
    const queryClient = createTestQueryClient()

    render(
      <QueryClientProvider client={queryClient}>
        <WalletSetup onNext={onNext} onBack={onBack} />
      </QueryClientProvider>
    )

    // Switch to import mode
    const importButton = screen.getByText('Import Existing Wallet')
    await user.click(importButton)

    expect(screen.getByText('Encryption Password (Optional)')).toBeInTheDocument()
    expect(
      screen.getByText(
        /Optional: Encrypt your imported private key with a password/i
      )
    ).toBeInTheDocument()
  })

  it('should clarify private key field in import mode', async () => {
    const user = userEvent.setup()
    const onNext = vi.fn()
    const onBack = vi.fn()
    const queryClient = createTestQueryClient()

    render(
      <QueryClientProvider client={queryClient}>
        <WalletSetup onNext={onNext} onBack={onBack} />
      </QueryClientProvider>
    )

    // Switch to import mode
    const importButton = screen.getByText('Import Existing Wallet')
    await user.click(importButton)

    expect(screen.getByText('Your Starknet Private Key')).toBeInTheDocument()
    expect(
      screen.getByText(
        /Paste your existing Starknet private key \(66 hex characters starting with 0x\)/i
      )
    ).toBeInTheDocument()
  })

  it('should validate private key format and show helpful error', async () => {
    const user = userEvent.setup()
    const onNext = vi.fn()
    const onBack = vi.fn()
    const mockInvoke = vi.mocked(invoke)
    const queryClient = createTestQueryClient()

    render(
      <QueryClientProvider client={queryClient}>
        <WalletSetup onNext={onNext} onBack={onBack} />
      </QueryClientProvider>
    )

    // Switch to import mode
    const importButton = screen.getByText('Import Existing Wallet')
    await user.click(importButton)

    // Try to import with invalid key
    const privateKeyInput = screen.getByPlaceholderText('0x1234567890abcdef...')
    await user.type(privateKeyInput, 'invalid-key')

    const importWalletButton = screen.getByText('Import Wallet')
    
    // Button should be enabled (we allow attempting import to show validation)
    expect(importWalletButton).toBeEnabled()
    
    await user.click(importWalletButton)

    // Verify that import was not called with invalid key since validation should prevent it
    expect(mockInvoke).not.toHaveBeenCalledWith('import_wallet', expect.any(Object))
  })

  it('should successfully import wallet with valid private key', async () => {
    const user = userEvent.setup()
    const onNext = vi.fn()
    const onBack = vi.fn()
    const mockInvoke = vi.mocked(invoke)
    const queryClient = createTestQueryClient()

    const mockWallet = {
      address: '0x1234567890abcdef1234567890abcdef12345678',
      public_key: '0xabcdef1234567890abcdef1234567890abcdef12',
      created_at: new Date().toISOString(),
      encrypted: false,
    }

    mockInvoke.mockResolvedValueOnce(mockWallet)

    render(
      <QueryClientProvider client={queryClient}>
        <WalletSetup onNext={onNext} onBack={onBack} />
      </QueryClientProvider>
    )

    // Switch to import mode
    const importButton = screen.getByText('Import Existing Wallet')
    await user.click(importButton)

    // Enter valid private key
    const privateKeyInput = screen.getByPlaceholderText('0x1234567890abcdef...')
    const validKey = '0x' + 'a'.repeat(64)
    await user.type(privateKeyInput, validKey)

    const importWalletButton = screen.getByText('Import Wallet')
    await user.click(importWalletButton)

    // Verify import was called with correct parameters
    expect(mockInvoke).toHaveBeenCalledWith('import_wallet', {
      privateKey: validKey,
      password: undefined,
    })
  })

  it('should import wallet with encryption password', async () => {
    const user = userEvent.setup()
    const onNext = vi.fn()
    const onBack = vi.fn()
    const mockInvoke = vi.mocked(invoke)
    const queryClient = createTestQueryClient()

    const mockWallet = {
      address: '0x1234567890abcdef1234567890abcdef12345678',
      public_key: '0xabcdef1234567890abcdef1234567890abcdef12',
      created_at: new Date().toISOString(),
      encrypted: true,
    }

    mockInvoke.mockResolvedValueOnce(mockWallet)

    render(
      <QueryClientProvider client={queryClient}>
        <WalletSetup onNext={onNext} onBack={onBack} />
      </QueryClientProvider>
    )

    // Switch to import mode
    const importButton = screen.getByText('Import Existing Wallet')
    await user.click(importButton)

    // Enter valid private key
    const privateKeyInput = screen.getByPlaceholderText('0x1234567890abcdef...')
    const validKey = '0x' + 'b'.repeat(64)
    await user.type(privateKeyInput, validKey)

    // Enter password (this will make confirm password field appear)
    const passwordInput = screen.getByPlaceholderText('Enter password (optional)...')
    await user.type(passwordInput, 'securePassword123')

    // Now the confirm password field should appear
    const confirmPasswordInput = screen.getByPlaceholderText('Confirm password...')
    await user.type(confirmPasswordInput, 'securePassword123')

    const importWalletButton = screen.getByText('Import Wallet')
    await user.click(importWalletButton)

    // Verify import was called with password
    expect(mockInvoke).toHaveBeenCalledWith('import_wallet', {
      privateKey: validKey,
      password: 'securePassword123',
    })
  })

  it('should render success modal after wallet generation without crashing', async () => {
    const user = userEvent.setup()
    const onNext = vi.fn()
    const onBack = vi.fn()
    const mockInvoke = vi.mocked(invoke)
    const queryClient = createTestQueryClient()

    const mockGeneratedWallet = {
      address: '0x0742d13378f69a4b0c9f5e3e66e14c8d9b7dbaf56f9e3e66e14c8d9b7dbaf56f',
      public_key: '0x03e4c8f1234567890abcdef1234567890abcdef1234567890abcdef123456789',
      private_key: '0x[REDACTED_IN_PRODUCTION]',
      created_at: new Date().toISOString(),
      encrypted: false,
    }

    mockInvoke.mockResolvedValueOnce(mockGeneratedWallet)

    render(
      <QueryClientProvider client={queryClient}>
        <WalletSetup onNext={onNext} onBack={onBack} />
      </QueryClientProvider>
    )

    // Component starts in generate mode by default, click the actual generate button
    const generateButton = screen.getByRole('button', { name: 'Generate Wallet' })
    await user.click(generateButton)

    // Verify private key modal appears without crashing
    await waitFor(() => {
      expect(screen.getByText(/This is the ONLY time you will see your private key/i)).toBeInTheDocument()
    })

    // Verify critical security warning is displayed
    expect(screen.getByText(/Anyone with this key can access your wallet/i)).toBeInTheDocument()

    // Verify wallet address is set (check for the confirmation button)
    const continueButton = screen.getByText('I Have Saved My Key - Continue')
    expect(continueButton).toBeInTheDocument()
    expect(continueButton).toBeDisabled() // Should be disabled until user confirms

    // Verify onNext is NOT called before user confirms
    expect(onNext).not.toHaveBeenCalled()

    // Verify invoke was called correctly
    expect(mockInvoke).toHaveBeenCalledWith('generate_wallet', {
      password: undefined,
    })
  })
})
