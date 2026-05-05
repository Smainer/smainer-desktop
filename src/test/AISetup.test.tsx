import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClientProvider } from '@tanstack/react-query'
import AISetup from '../components/onboarding/AISetup'
import { invoke } from '@tauri-apps/api/core'
import { createTestQueryClient } from './test-utils'

// Mock fetch for Ollama check
globalThis.fetch = vi.fn() as any

vi.mock('../hooks/useHardwareInfo', () => ({
  useHardwareInfo: () => ({
    data: {
      total_ram: 17179869184, // 16GB
      gpus: [
        {
          name: 'NVIDIA RTX 3060',
          memory: 12288, // 12GB VRAM
          is_supported: true,
        },
      ],
    },
  }),
}))

describe('AISetup - Continue Button Behavior', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('should allow continue when AI serving is disabled', async () => {
    const onNext = vi.fn()
    const onBack = vi.fn()
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValue({
      schema_version: '1.0.0',
      contract_version: '2024.1',
      ai_serving_enabled: false,
      model_preferences: [],
      privacy_mode: 'Standard',
      resources: { max_cpu_percent: 80, max_ram_gb: 8 },
    })

    const queryClient = createTestQueryClient()

    render(
      <QueryClientProvider client={queryClient}>
        <AISetup onNext={onNext} onBack={onBack} />
      </QueryClientProvider>
    )

    await waitFor(() => {
      const continueButton = screen.getByText('Continue to Wallet Setup')
      expect(continueButton).not.toBeDisabled()
    })
  })

  it('should allow continue when Ollama auto-install is checked without requiring risk acknowledgment', async () => {
    const user = userEvent.setup()
    const onNext = vi.fn()
    const onBack = vi.fn()
    const mockInvoke = vi.mocked(invoke)
    
    // Mock Ollama not available
    vi.mocked(fetch).mockRejectedValue(new Error('Not available'))

    // Mock load_ai_config
    mockInvoke.mockImplementation((cmd: string, _args?: any) => {
      if (cmd === 'load_ai_config') {
        return Promise.resolve({
          schema_version: '1.0.0',
          contract_version: '2024.1',
          ai_serving_enabled: false,
          model_preferences: [],
          privacy_mode: 'Standard',
          resources: { max_cpu_percent: 80, max_ram_gb: 8 },
        })
      }
      if (cmd === 'validate_ai_capabilities') {
        return Promise.resolve({
          system_validation: {
            errors: ['Ollama runtime not available'],
            warnings: [],
          },
          compatibility_status: 'Incompatible',
        })
      }
      if (cmd === 'check_ollama_installed') return Promise.resolve(false)
      if (cmd === 'save_ai_config') {
        return Promise.resolve(undefined)
      }
      return Promise.reject(new Error('Unknown command'))
    })

    const queryClient = createTestQueryClient()

    render(
      <QueryClientProvider client={queryClient}>
        <AISetup onNext={onNext} onBack={onBack} />
      </QueryClientProvider>
    )

    // Enable AI serving
    const enableCheckbox = await screen.findByLabelText(/Enable AI inference serving/)
    await user.click(enableCheckbox)

    // Wait for validation
    await waitFor(() => {
      expect(screen.getAllByText(/Configuration Issues/i).length).toBeGreaterThan(0)
    })

    // Auto-install checkbox should be checked by default (Ollama not available)
    const autoInstallCheckbox = await screen.findByLabelText(/Auto-install Ollama runtime/i)
    expect(autoInstallCheckbox).toBeChecked()

    // Continue button should be enabled WITHOUT acknowledgment
    const continueButton = screen.getByText('Continue to Wallet Setup')
    await waitFor(() => {
      expect(continueButton).not.toBeDisabled()
    })

    // Verify auto-install info alert is shown
    expect(screen.getByText(/Auto-install was selected/i)).toBeInTheDocument()

    // Click continue
    await user.click(continueButton)

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('save_ai_config', expect.any(Object))
      expect(onNext).toHaveBeenCalled()
    })
  })

  it('should require acknowledgment when auto-install is unchecked', async () => {
    const user = userEvent.setup()
    const onNext = vi.fn()
    const onBack = vi.fn()
    const mockInvoke = vi.mocked(invoke)
    
    // Mock Ollama not available
    vi.mocked(fetch).mockRejectedValue(new Error('Not available'))

    mockInvoke.mockImplementation((cmd: string, _args?: any) => {
      if (cmd === 'load_ai_config') {
        return Promise.resolve({
          schema_version: '1.0.0',
          contract_version: '2024.1',
          ai_serving_enabled: false,
          model_preferences: [],
          privacy_mode: 'Standard',
          resources: { max_cpu_percent: 80, max_ram_gb: 8 },
        })
      }
      if (cmd === 'validate_ai_capabilities') {
        return Promise.resolve({
          system_validation: {
            errors: ['Ollama runtime not available'],
            warnings: [],
          },
          compatibility_status: 'Incompatible',
        })
      }
      if (cmd === 'check_ollama_installed') return Promise.resolve(false)
      if (cmd === 'save_ai_config') {
        return Promise.resolve(undefined)
      }
      return Promise.reject(new Error('Unknown command'))
    })

    const queryClient = createTestQueryClient()

    render(
      <QueryClientProvider client={queryClient}>
        <AISetup onNext={onNext} onBack={onBack} />
      </QueryClientProvider>
    )

    // Enable AI serving
    const enableCheckbox = await screen.findByLabelText(/Enable AI inference serving/)
    await user.click(enableCheckbox)

    // Wait for validation
    await waitFor(() => {
      expect(screen.getAllByText(/Configuration Issues/i).length).toBeGreaterThan(0)
    })

    // Uncheck auto-install
    const autoInstallCheckbox = await screen.findByLabelText(/Auto-install Ollama runtime/i)
    await user.click(autoInstallCheckbox)

    // Continue button should be disabled without acknowledgment
    const continueButton = screen.getByText('Continue to Wallet Setup')
    await waitFor(() => {
      expect(continueButton).toBeDisabled()
    })

    // Acknowledge risks
    const acknowledgeCheckbox = await screen.findByLabelText(
      /I understand the configuration issues/i
    )
    await user.click(acknowledgeCheckbox)

    // Now continue button should be enabled
    await waitFor(() => {
      expect(continueButton).not.toBeDisabled()
    })

    // Click continue
    await user.click(continueButton)

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('save_ai_config', expect.any(Object))
      expect(onNext).toHaveBeenCalled()
    })
  })

  it('should show long-running install message during Ollama auto-install save', async () => {
    const user = userEvent.setup()
    const onNext = vi.fn()
    const onBack = vi.fn()
    const mockInvoke = vi.mocked(invoke)
    
    // Mock Ollama not available
    vi.mocked(fetch).mockRejectedValue(new Error('Not available'))

    // Mock a slow install_ollama to keep isSaving true
    let resolveSave: any
    const savePromise = new Promise((resolve) => { resolveSave = resolve })

    mockInvoke.mockImplementation((cmd: string, args?: any) => {
      if (cmd === 'load_ai_config') {
        return Promise.resolve({
          schema_version: '1.0.0',
          contract_version: '2024.1',
          ai_serving_enabled: false,
          model_preferences: [],
          privacy_mode: 'Standard',
          resources: { max_cpu_percent: 80, max_ram_gb: 8 },
        })
      }
      if (cmd === 'validate_ai_capabilities') {
        return Promise.resolve({
          system_validation: {
            errors: ['Ollama runtime not available'],
            warnings: [],
          },
          compatibility_status: 'Incompatible',
        })
      }
      if (cmd === 'save_ai_config') {
        return savePromise
      }
      if (cmd === 'install_ollama') {
        return savePromise
      }
      if (cmd === 'check_ollama_installed') return Promise.resolve(false)
      return Promise.reject(new Error('Unknown command'))
    })

    const queryClient = createTestQueryClient()

    render(
      <QueryClientProvider client={queryClient}>
        <AISetup onNext={onNext} onBack={onBack} />
      </QueryClientProvider>
    )

    // Enable AI serving
    const enableCheckbox = await screen.findByLabelText(/Enable AI inference serving/)
    await user.click(enableCheckbox)

    // Wait for validation
    await waitFor(() => {
      expect(screen.getAllByText(/Configuration Issues/i).length).toBeGreaterThan(0)
    })

    // Auto-install should be checked by default
    const autoInstallCheckbox = await screen.findByLabelText(/Auto-install Ollama runtime/i)
    expect(autoInstallCheckbox).toBeChecked()

    // Click continue to trigger save with install
    const continueButton = screen.getByText('Continue to Wallet Setup')
    await user.click(continueButton)

    // Verify long-running message appears
    await waitFor(() => {
      expect(screen.getByText(/This might take a few minutes/i)).toBeInTheDocument()
    })

    // Clean up - resolve the promise
    resolveSave(undefined)
  })

  it('should not fire install_ollama when ollamaAvailable is null (availability still loading)', async () => {
    const user = userEvent.setup()
    const onNext = vi.fn()
    const onBack = vi.fn()
    const mockInvoke = vi.mocked(invoke)

    // fetch never resolves — ollamaAvailable stays null
    vi.mocked(fetch).mockReturnValue(new Promise(() => {}))

    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'load_ai_config') {
        return Promise.resolve({
          schema_version: '1.0.0',
          contract_version: '2024.1',
          ai_serving_enabled: false,
          model_preferences: [],
          privacy_mode: 'Standard',
          resources: { max_cpu_percent: 80, max_ram_gb: 8 },
        })
      }
      if (cmd === 'check_ollama_installed') {
        // also never resolves — ollamaInstalled stays null
        return new Promise(() => {})
      }
      if (cmd === 'validate_ai_capabilities') {
        return Promise.resolve({
          system_validation: { errors: [], warnings: [] },
          compatibility_status: 'Compatible',
        })
      }
      if (cmd === 'save_ai_config') return Promise.resolve(undefined)
      return Promise.reject(new Error('Unknown command'))
    })

    const queryClient = createTestQueryClient()
    render(
      <QueryClientProvider client={queryClient}>
        <AISetup onNext={onNext} onBack={onBack} />
      </QueryClientProvider>
    )

    // Enable AI serving (ollamaAvailable is still null here)
    const enableCheckbox = await screen.findByLabelText(/Enable AI inference serving/)
    await user.click(enableCheckbox)

    const continueButton = screen.getByText('Continue to Wallet Setup')
    await user.click(continueButton)

    await waitFor(() => {
      expect(onNext).toHaveBeenCalled()
    })

    // install_ollama must NOT have been called — availability was null, not false
    expect(mockInvoke).not.toHaveBeenCalledWith('install_ollama', expect.anything())
    expect(mockInvoke).not.toHaveBeenCalledWith('install_ollama')
  })

  it('should not fire install_ollama when ollamaAvailable resolves true even if saved config has install_requested=true', async () => {
    const user = userEvent.setup()
    const onNext = vi.fn()
    const onBack = vi.fn()
    const mockInvoke = vi.mocked(invoke)

    // Ollama is present
    vi.mocked(fetch).mockResolvedValue({ ok: true } as Response)

    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'load_ai_config') {
        // Saved config from a previous session where Ollama was absent
        return Promise.resolve({
          schema_version: '1.0.0',
          contract_version: '2024.1',
          ai_serving_enabled: true,
          ollama_config: {
            install_requested: true, // stale true from prior session
            api_endpoint: 'http://localhost:11434',
            models_to_install: ['llama3.1:8b'],
            auto_update: false,
          },
          model_preferences: [],
          privacy_mode: 'Standard',
          resources: { max_cpu_percent: 80, max_ram_gb: 8 },
        })
      }
      if (cmd === 'validate_ai_capabilities') {
        return Promise.resolve({
          system_validation: { errors: [], warnings: [] },
          compatibility_status: 'Compatible',
        })
      }
      if (cmd === 'check_ollama_installed') return Promise.resolve(true)
      if (cmd === 'save_ai_config') return Promise.resolve(undefined)
      return Promise.reject(new Error('Unknown command'))
    })

    const queryClient = createTestQueryClient()
    render(
      <QueryClientProvider client={queryClient}>
        <AISetup onNext={onNext} onBack={onBack} />
      </QueryClientProvider>
    )

    const continueButton = await screen.findByText('Continue to Wallet Setup')

    // Wait for Ollama availability to resolve (fetch resolved ok=true)
    await waitFor(() => {
      expect(screen.getByText(/Ollama runtime detected/i)).toBeInTheDocument()
    })

    await user.click(continueButton)

    await waitFor(() => {
      expect(onNext).toHaveBeenCalled()
    })

    // install_ollama must NOT have been called — Ollama is already available
    expect(mockInvoke).not.toHaveBeenCalledWith('install_ollama', expect.anything())
    expect(mockInvoke).not.toHaveBeenCalledWith('install_ollama')
  })
})
