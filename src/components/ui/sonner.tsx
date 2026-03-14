import { toast, Toaster as SonnerToaster } from 'sonner'

export { toast }

export function Toaster() {
  return (
    <SonnerToaster
      theme="light"
      position="bottom-right"
      toastOptions={{
        style: {
          background: 'white',
          border: '1px solid #e2e8f0',
          color: '#0f172a',
        },
      }}
    />
  )
}