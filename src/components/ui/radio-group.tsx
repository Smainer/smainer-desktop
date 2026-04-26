import React, { createContext, useContext } from "react"
import { cn } from "@/lib/utils"

interface RadioGroupContextType {
  value?: string
  onValueChange?: (value: string) => void
  name: string
}

const RadioGroupContext = createContext<RadioGroupContextType | undefined>(undefined)

export interface RadioGroupProps {
  value?: string
  onValueChange?: (value: string) => void
  children: React.ReactNode
  className?: string
  name?: string
}

const RadioGroup = React.forwardRef<HTMLDivElement, RadioGroupProps>(
  ({ className, value, onValueChange, children, name = `radio-group-${Math.random()}`, ...props }, ref) => {
    return (
      <RadioGroupContext.Provider value={{ value, onValueChange, name }}>
        <div
          ref={ref}
          className={cn("grid gap-2", className)}
          {...props}
        >
          {children}
        </div>
      </RadioGroupContext.Provider>
    )
  }
)
RadioGroup.displayName = "RadioGroup"

export interface RadioGroupItemProps extends Omit<React.InputHTMLAttributes<HTMLInputElement>, 'type'> {
  value: string
}

const RadioGroupItem = React.forwardRef<HTMLInputElement, RadioGroupItemProps>(
  ({ className, value: itemValue, ...props }, ref) => {
    const context = useContext(RadioGroupContext)
    
    if (!context) {
      throw new Error("RadioGroupItem must be used within RadioGroup")
    }
    
    const { value, onValueChange, name } = context
    
    return (
      <input
        ref={ref}
        type="radio"
        name={name}
        value={itemValue}
        checked={value === itemValue}
        onChange={() => onValueChange?.(itemValue)}
        className={cn(
          "aspect-square h-4 w-4 rounded-full border border-primary text-primary ring-offset-background focus:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50",
          className
        )}
        {...props}
      />
    )
  }
)
RadioGroupItem.displayName = "RadioGroupItem"

export { RadioGroup, RadioGroupItem }