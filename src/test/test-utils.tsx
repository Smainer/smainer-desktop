import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import React from 'react'

/**
 * Create a QueryClient configured for testing
 * - No retries for faster, deterministic tests
 * - Fresh instance per test to avoid state pollution
 */
export function createTestQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  })
}

/**
 * Wrapper component for tests that need React Query
 */
export function TestQueryClientProvider({ children }: { children: React.ReactNode }) {
  const queryClient = createTestQueryClient()
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
}
