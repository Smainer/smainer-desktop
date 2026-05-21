import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import { QueryClientProvider } from '@tanstack/react-query'
import App from '../App'
import { invoke } from '@tauri-apps/api/core'
import { createTestQueryClient } from './test-utils'

// Mock child components
vi.mock('../components/onboarding/SystemCheck', () => ({
  default: ({ onNext }: any) => (
    <div data-testid="system-check">
      <button onClick={onNext}>Next</button>
    </div>
  ),
}))

vi.mock('../components/onboarding/AISetup', () => ({
  default: ({ onNext, onBack }: any) => (
    <div data-testid="ai-setup">
      <button onClick={onBack}>Back</button>
      <button onClick={onNext}>Next</button>
    </div>
  ),
}))

vi.mock('../components/onboarding/WalletSetup', () => ({
  default: ({ onNext, onBack }: any) => (
    <div data-testid="wallet-setup">
      <button onClick={onBack}>Back</button>
      <button onClick={() => onNext('0x123')}>Next</button>
    </div>
  ),
}))

vi.mock('../components/onboarding/NodeRegistration', () => ({
  default: ({ onComplete, onBack }: any) => (
    <div data-testid="node-registration">
      <button onClick={onBack}>Back</button>
      <button onClick={() => onComplete('0x123', 'node-1')}>Complete</button>
    </div>
  ),
}))

vi.mock('../components/dashboard/NodeStatus', () => ({
  default: () => <div data-testid="node-status">Node Status</div>,
}))

vi.mock('../components/dashboard/EarningsCard', () => ({
  default: () => <div data-testid="earnings-card">Earnings</div>,
}))

vi.mock('../components/dashboard/TaskHistory', () => ({
  default: () => <div data-testid="task-history">Tasks</div>,
}))

vi.mock('../components/settings/HardwareConfig', () => ({
  default: () => <div data-testid="hardware-config">Hardware</div>,
}))

vi.mock('../components/settings/ServiceOptions', () => ({
  default: () => <div data-testid="service-options">Service</div>,
}))

vi.mock('../hooks/useNodeStatus', () => ({
  useNodeStatus: () => ({ data: { node_id: 'test-node', is_running: false } }),
}))

vi.mock('../hooks/useHardwareInfo', () => ({
  useHardwareInfo: () => ({ data: { total_ram: 8589934592, gpus: [] } }),
}))

describe('App - Onboarding Step Stability', () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset()
    localStorage.clear()
  })

  it('should not reset onboarding step when nodeStatus polling updates', async () => {
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValue(null) // No wallet initially
    
    const queryClient = createTestQueryClient()
    
    render(
      <QueryClientProvider client={queryClient}>
        <App />
      </QueryClientProvider>
    )

    // Should start at step 0 (SystemCheck)
    await waitFor(() => {
      expect(screen.getByTestId('system-check')).toBeInTheDocument()
    })

    // Simulate user advancing to step 1
    const nextButton = screen.getByText('Next')
    nextButton.click()

    await waitFor(() => {
      expect(screen.getByTestId('ai-setup')).toBeInTheDocument()
    })

    // Verify checkOnboardingStatus was only called once during mount
    // It should NOT be called again when nodeStatus updates
    expect(mockInvoke).toHaveBeenCalledTimes(1)
    expect(mockInvoke).toHaveBeenCalledWith('get_wallet_address')
  })

  it('should preserve current step across nodeStatus updates', async () => {
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValue(null)
    
    const queryClient = createTestQueryClient()
    
    const { rerender } = render(
      <QueryClientProvider client={queryClient}>
        <App />
      </QueryClientProvider>
    )

    await waitFor(() => {
      expect(screen.getByTestId('system-check')).toBeInTheDocument()
    })

    // Clear the initial mount calls
    vi.clearAllMocks()

    // Simulate nodeStatus update by re-rendering
    rerender(
      <QueryClientProvider client={queryClient}>
        <App />
      </QueryClientProvider>
    )

    // Should still be on SystemCheck step, no additional invoke calls
    expect(screen.getByTestId('system-check')).toBeInTheDocument()
    expect(mockInvoke).not.toHaveBeenCalled()
  })

  it('should resume from node registration when wallet exists but provider is not running', async () => {
    const mockInvoke = vi.mocked(invoke)
    mockInvoke
      .mockResolvedValueOnce('0x1234567890abcdef') // wallet exists
      .mockResolvedValueOnce({ is_running: false, relayer_connected: false }) // provider not running

    localStorage.setItem('smainer_provider_config', JSON.stringify({
      wallet_address: '0x1234567890abcdef',
      relayer_url: 'https://api.smainer.io',
      port: 8080,
      max_tasks: 1,
      gpu_enabled: true,
      auto_start: false,
    }))
    
    const queryClient = createTestQueryClient()
    
    render(
      <QueryClientProvider client={queryClient}>
        <App />
      </QueryClientProvider>
    )

    // Wallet already exists, so the next actionable step is provider registration/startup.
    await waitFor(() => {
      expect(screen.getByTestId('node-registration')).toBeInTheDocument()
    })

    expect(mockInvoke).toHaveBeenCalledWith('get_wallet_address')
    expect(mockInvoke).toHaveBeenCalledWith('get_provider_status')
  })
})
