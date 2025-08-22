import * as React from "react"
import { cn } from "@/lib/utils"

export interface InputProps
  extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string
  error?: string
  helperText?: string
}

const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className, type, label, error, helperText, id, ...props }, ref) => {
    const inputId = id || label?.toLowerCase().replace(/\s+/g, '-')
    
    if (label || error || helperText) {
      return (
        <div className="space-y-1">
          {label && (
            <label
              htmlFor={inputId}
              className="block text-sm font-medium text-gray-700 dark:text-gray-300"
            >
              {label}
            </label>
          )}
          <input
            type={type}
            id={inputId}
            ref={ref}
            className={cn(
              "flex h-9 w-full rounded-md border bg-white px-3 py-1 text-sm shadow-sm transition-colors",
              "placeholder:text-gray-400 dark:placeholder:text-gray-500",
              "dark:bg-gray-800",
              "file:border-0 file:bg-transparent file:text-sm file:font-medium",
              "focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent",
              "disabled:cursor-not-allowed disabled:opacity-50",
              error 
                ? "border-red-500 text-red-900 dark:border-red-400 dark:text-red-400"
                : "border-gray-300 dark:border-gray-600",
              className
            )}
            aria-invalid={!!error}
            aria-describedby={
              error ? `${inputId}-error` : helperText ? `${inputId}-helper` : undefined
            }
            {...props}
          />
          {error && (
            <p id={`${inputId}-error`} className="text-sm text-red-600 dark:text-red-400">
              {error}
            </p>
          )}
          {helperText && !error && (
            <p id={`${inputId}-helper`} className="text-sm text-gray-500 dark:text-gray-400">
              {helperText}
            </p>
          )}
        </div>
      )
    }
    
    return (
      <input
        type={type}
        ref={ref}
        className={cn(
          "flex h-9 w-full rounded-md border bg-white px-3 py-1 text-sm shadow-sm transition-colors",
          "placeholder:text-gray-400 dark:placeholder:text-gray-500",
          "dark:bg-gray-800",
          "file:border-0 file:bg-transparent file:text-sm file:font-medium",
          "focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent",
          "disabled:cursor-not-allowed disabled:opacity-50",
          "border-gray-300 dark:border-gray-600",
          className
        )}
        {...props}
      />
    )
  }
)
Input.displayName = "Input"

export { Input }