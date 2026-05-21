import { cn } from '../../lib/utils'

interface ProgressProps {
  value?: number
  className?: string
}

export function Progress({ value = 0, className }: ProgressProps) {
  return (
    <div className={cn(
      'relative h-4 w-full overflow-hidden rounded-full bg-secondary',
      className
    )}>
      <div
        className="h-full w-full flex-1 bg-primary transition-all"
        style={{ transform: `translateX(-${100 - (value || 0)}%)` }}
      />
    </div>
  )
}