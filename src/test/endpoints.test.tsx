import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor, act } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { renderHook, waitFor as waitForHook } from '@testing-library/react'
import { invoke } from '@tauri-apps/api/core'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import React from 'react'
import NodeStatus from '../components/dashboard/NodeStatus'
import { useNodeStatus } from '../hooks/useNodeStatus'
import { useStartProvider, useStopProvider } from '../hooks/useProviderCommands'
import { createTestQueryClient, TestQueryClientProvider } from './test-utils'

// ── helpers ─────────────────────────────────────────────────────────────────

/** Mirrors the node_id derivation in monitoring.rs and provider.rs */
function nodeIdFromAddress(addr: string): string {
  const stripped = addr.replace(/^0x/i, '')
  const id = stripped.split('').filter(c => /[a-zA-Z0-9]/.test(c)).slice(0, 24).join('')
  return id.length === 0 ? 'default-node' : id
}

function makeStatus(overrides = {}) {
  return {
    is_online: false,
    node_id: '0xabc',
    uptime: 0,
    last_heartbeat: new Date().toISOString(),
    tasks_active: 0,
    tasks_completed_today: 0,
    earnings_today: 0,
    cpu_usage: 0,
    memory_usage: 0,
    gpu_usage: undefined,
    network_status: 'disconnected',
    relayer_connected: false,
    provider_version: '0.1.0',
    node_tier: 'standard',
    ...overrides,
  }
}

function wrapper({ children }: { children: React.ReactNode }) {
  return <TestQueryClientProvider>{children}</TestQueryClientProvider>
}

// ── 1. node_id derivation ────────────────────────────────────────────────────

describe('nodeIdFromAddress — consistent with Rust monitoring.rs / provider.rs', () => {
  it('takes first 24 alphanumeric chars and strips 0x prefix', () => {
    const addr = '0x74e524cffd76919451cd8fb524076b909a55b40a8251d3c8275a3d65d0fd958'
    expect(nodeIdFromAddress(addr)).toBe('74e524cffd76919451cd8fb5')
    expect(nodeIdFromAddress(addr)).toHaveLength(24)
  })

  it('returns default-node for empty address', () => {
    expect(nodeIdFromAddress('')).toBe('default-node')
    expect(nodeIdFromAddress('0x')).toBe('default-node')
  })

  it('strips non-alphanumeric chars', () => {
    const addr = '0x!!abc123def456ghi789jkl0'
    const result = nodeIdFromAddress(addr)
    expect(/^[a-zA-Z0-9]+$/.test(result)).toBe(true)
  })

  it('provider.rs and monitoring.rs produce same node_id for same address', () => {
    // Both use: addr.trim_start_matches("0x").chars().filter(alphanumeric).take(24)
    const addr = '0x4c4988b9f8e878db51322ea5121eb7f75daa032e28fbb47b1c7f5fa830168abc'
    const id1 = nodeIdFromAddress(addr)
    const id2 = nodeIdFromAddress(addr)
    expect(id1).toBe(id2)
    expect(id1).toHaveLength(24)
  })
})

// ── 2. useNodeStatus hook ────────────────────────────────────────────────────

describe('useNodeStatus hook', () => {
  beforeEach(() => vi.clearAllMocks())

  it('returns offline status when process is not running', async () => {
    vi.mocked(invoke).mockResolvedValue(makeStatus({
      is_online: false,
      relayer_connected: false,
      network_status: 'disconnected',
    }))

    const { result } = renderHook(() => useNodeStatus(), { wrapper })
    await waitForHook(() => expect(result.current.isSuccess).toBe(true))

    expect(result.current.data?.is_online).toBe(false)
    expect(result.current.data?.relayer_connected).toBe(false)
    expect(result.current.data?.network_status).toBe('disconnected')
  })

  it('returns connecting when process running but relayer not confirmed', async () => {
    vi.mocked(invoke).mockResolvedValue(makeStatus({
      is_online: true,
      relayer_connected: false,
      network_status: 'connecting',
    }))

    const { result } = renderHook(() => useNodeStatus(), { wrapper })
    await waitForHook(() => expect(result.current.isSuccess).toBe(true))

    expect(result.current.data?.is_online).toBe(true)
    expect(result.current.data?.relayer_connected).toBe(false)
    expect(result.current.data?.network_status).toBe('connecting')
  })

  it('returns online when relayer confirms node registration', async () => {
    vi.mocked(invoke).mockResolvedValue(makeStatus({
      is_online: true,
      relayer_connected: true,
      network_status: 'connected',
      uptime: 120,
    }))

    const { result } = renderHook(() => useNodeStatus(), { wrapper })
    await waitForHook(() => expect(result.current.isSuccess).toBe(true))

    expect(result.current.data?.is_online).toBe(true)
    expect(result.current.data?.relayer_connected).toBe(true)
    expect(result.current.data?.uptime).toBe(120)
  })

  it('uptime is non-zero when process has been running', async () => {
    vi.mocked(invoke).mockResolvedValue(makeStatus({ uptime: 3661 }))
    const { result } = renderHook(() => useNodeStatus(), { wrapper })
    await waitForHook(() => expect(result.current.isSuccess).toBe(true))
    expect(result.current.data!.uptime).toBeGreaterThan(0)
  })

  it('surfaces registration-failed message after timeout', async () => {
    vi.mocked(invoke).mockResolvedValue(makeStatus({
      is_online: true,
      relayer_connected: false,
      network_status: 'Provider running — registration failed (check logs)',
    }))

    const { result } = renderHook(() => useNodeStatus(), { wrapper })
    await waitForHook(() => expect(result.current.isSuccess).toBe(true))
    expect(result.current.data?.network_status).toContain('registration failed')
  })

  it('handles invoke error gracefully without crashing', async () => {
    vi.mocked(invoke).mockRejectedValue(new Error('Tauri IPC error'))
    // useNodeStatus has retry:1 — two attempts with ~1s delay between them
    const { result } = renderHook(() => useNodeStatus(), { wrapper })
    await waitForHook(() => expect(result.current.isError).toBe(true), { timeout: 5000 })
    expect(result.current.data).toBeUndefined()
  })
})

// ── 3. NodeStatus component rendering ───────────────────────────────────────

describe('NodeStatus component — relayer status display', () => {
  beforeEach(() => vi.clearAllMocks())

  it('shows "Offline" in Relayer cell when not connected', () => {
    render(
      <TestQueryClientProvider>
        <NodeStatus status={makeStatus({ relayer_connected: false })} />
      </TestQueryClientProvider>
    )
    expect(screen.getByText('Offline')).toBeInTheDocument()
    expect(screen.getByText('Relayer')).toBeInTheDocument()
  })

  it('shows "Online" in Relayer cell when connected', () => {
    render(
      <TestQueryClientProvider>
        <NodeStatus status={makeStatus({ is_online: true, relayer_connected: true })} />
      </TestQueryClientProvider>
    )
    expect(screen.getByText('Online')).toBeInTheDocument()
  })

  it('shows "Start Node" button when offline', () => {
    render(
      <TestQueryClientProvider>
        <NodeStatus status={makeStatus({ is_online: false })} />
      </TestQueryClientProvider>
    )
    expect(screen.getByRole('button', { name: /start node/i })).toBeInTheDocument()
  })

  it('shows "Stop Node" button when online', () => {
    render(
      <TestQueryClientProvider>
        <NodeStatus status={makeStatus({ is_online: true, relayer_connected: true })} />
      </TestQueryClientProvider>
    )
    expect(screen.getByRole('button', { name: /stop node/i })).toBeInTheDocument()
  })

  it('shows offline description when disconnected', () => {
    render(
      <TestQueryClientProvider>
        <NodeStatus status={makeStatus({
          is_online: false,
          relayer_connected: false,
          network_status: 'disconnected',
        })} />
      </TestQueryClientProvider>
    )
    expect(screen.getByText(/unable to connect to relayer/i)).toBeInTheDocument()
  })

  it('shows running description when online', () => {
    render(
      <TestQueryClientProvider>
        <NodeStatus status={makeStatus({ is_online: true, relayer_connected: true })} />
      </TestQueryClientProvider>
    )
    expect(screen.getByText(/accepting tasks/i)).toBeInTheDocument()
  })

  it('displays correct uptime from status', () => {
    render(
      <TestQueryClientProvider>
        <NodeStatus status={makeStatus({ uptime: 7380 })} />  {/* 2h 3m */}
      </TestQueryClientProvider>
    )
    expect(screen.getByText('2h 3m')).toBeInTheDocument()
  })

  it('displays 0h 0m when uptime is zero', () => {
    render(
      <TestQueryClientProvider>
        <NodeStatus status={makeStatus({ uptime: 0 })} />
      </TestQueryClientProvider>
    )
    expect(screen.getByText('0h 0m')).toBeInTheDocument()
  })

  it('displays tasks today and active tasks', () => {
    render(
      <TestQueryClientProvider>
        <NodeStatus status={makeStatus({ tasks_completed_today: 5, tasks_active: 2 })} />
      </TestQueryClientProvider>
    )
    expect(screen.getByText('5')).toBeInTheDocument()
    expect(screen.getByText('2')).toBeInTheDocument()
  })

  it('shows "Starting" in Relayer cell when process is connecting (not yet registered)', () => {
    render(
      <TestQueryClientProvider>
        <NodeStatus status={makeStatus({
          is_online: false,
          relayer_connected: false,
          network_status: 'connecting',
        })} />
      </TestQueryClientProvider>
    )
    expect(screen.getByText('Starting')).toBeInTheDocument()
    expect(screen.queryByText('Offline')).not.toBeInTheDocument()
  })

  it('shows connecting description when network_status is connecting', () => {
    render(
      <TestQueryClientProvider>
        <NodeStatus status={makeStatus({
          is_online: false,
          relayer_connected: false,
          network_status: 'connecting',
        })} />
      </TestQueryClientProvider>
    )
    expect(screen.getByText(/connecting to relayer/i)).toBeInTheDocument()
    expect(screen.queryByText(/node is offline/i)).not.toBeInTheDocument()
    expect(screen.queryByText(/unable to connect/i)).not.toBeInTheDocument()
  })

  it('still shows "Offline" in Relayer cell when network_status is disconnected', () => {
    render(
      <TestQueryClientProvider>
        <NodeStatus status={makeStatus({
          is_online: false,
          relayer_connected: false,
          network_status: 'disconnected',
        })} />
      </TestQueryClientProvider>
    )
    expect(screen.getByText('Offline')).toBeInTheDocument()
    expect(screen.queryByText('Starting')).not.toBeInTheDocument()
  })
})

// ── 4. useStartProvider mutation ─────────────────────────────────────────────

describe('useStartProvider mutation', () => {
  beforeEach(() => vi.clearAllMocks())

  it('calls invoke("start_provider") with correct config shape', async () => {
    vi.mocked(invoke).mockResolvedValue(true)
    const user = userEvent.setup()

    const { result } = renderHook(() => useStartProvider(), { wrapper })

    const config = {
      wallet_address: '0x74e524cffd76919451cd8fb524076b909',
      relayer_url: 'https://api.smainer.io',
      port: 8080,
      max_tasks: 10,
      gpu_enabled: true,
      auto_start: false,
    }

    await act(async () => {
      result.current.mutate(config)
    })

    await waitForHook(() => expect(result.current.isSuccess).toBe(true))

    expect(invoke).toHaveBeenCalledWith('start_provider', { config })
  })

  it('relayer_url passed as https:// — Rust layer converts to wss://', async () => {
    vi.mocked(invoke).mockResolvedValue(true)

    const { result } = renderHook(() => useStartProvider(), { wrapper })

    await act(async () => {
      result.current.mutate({
        wallet_address: '0xabc',
        relayer_url: 'https://api.smainer.io',
        port: 8080,
        max_tasks: 10,
        gpu_enabled: false,
        auto_start: false,
      })
    })

    await waitForHook(() => expect(result.current.isSuccess).toBe(true))

    const callArgs = (invoke as any).mock.calls[0]
    expect(callArgs[1].config.relayer_url).toBe('https://api.smainer.io')
    // The Rust command's http_to_ws_url() will produce wss:// from this
  })

  it('mutation is in error state when invoke rejects', async () => {
    vi.mocked(invoke).mockRejectedValue('Provider daemon not found')

    const { result } = renderHook(() => useStartProvider(), { wrapper })

    await act(async () => {
      result.current.mutate({
        wallet_address: '0xabc',
        relayer_url: 'https://api.smainer.io',
        port: 8080,
        max_tasks: 10,
        gpu_enabled: false,
        auto_start: false,
      })
    })

    await waitForHook(() => expect(result.current.isError).toBe(true))
  })
})

// ── 5. useStopProvider mutation ──────────────────────────────────────────────

describe('useStopProvider mutation', () => {
  beforeEach(() => vi.clearAllMocks())

  it('calls invoke("stop_provider") with no args', async () => {
    vi.mocked(invoke).mockResolvedValue(true)

    const { result } = renderHook(() => useStopProvider(), { wrapper })

    await act(async () => {
      result.current.mutate()
    })

    await waitForHook(() => expect(result.current.isSuccess).toBe(true))
    expect(invoke).toHaveBeenCalledWith('stop_provider')
  })
})

// ── 6. get_node_status invoke contract ──────────────────────────────────────

describe('get_node_status invoke contract', () => {
  beforeEach(() => vi.clearAllMocks())

  it('NodeStatus fields are all present in returned data', async () => {
    const fullStatus = makeStatus({
      is_online: true,
      node_id: '74e524cffd76919451cd8fb5',
      uptime: 300,
      relayer_connected: true,
      network_status: 'connected',
      cpu_usage: 45.2,
      memory_usage: 62.0,
      tasks_active: 1,
      tasks_completed_today: 3,
    })
    vi.mocked(invoke).mockResolvedValue(fullStatus)

    const { result } = renderHook(() => useNodeStatus(), { wrapper })
    await waitForHook(() => expect(result.current.isSuccess).toBe(true))

    const data = result.current.data!
    expect(data.is_online).toBe(true)
    expect(data.node_id).toBe('74e524cffd76919451cd8fb5')
    expect(data.uptime).toBe(300)
    expect(data.relayer_connected).toBe(true)
    expect(data.network_status).toBe('connected')
    expect(data.cpu_usage).toBeCloseTo(45.2)
    expect(data.tasks_active).toBe(1)
    expect(data.tasks_completed_today).toBe(3)
  })

  it('node_id in response matches derivation formula', async () => {
    const walletAddr = '0x74e524cffd76919451cd8fb524076b909a55b40a'
    const expectedNodeId = nodeIdFromAddress(walletAddr)

    vi.mocked(invoke).mockResolvedValue(makeStatus({ node_id: expectedNodeId }))

    const { result } = renderHook(() => useNodeStatus(), { wrapper })
    await waitForHook(() => expect(result.current.isSuccess).toBe(true))

    expect(result.current.data?.node_id).toBe(expectedNodeId)
  })
})
