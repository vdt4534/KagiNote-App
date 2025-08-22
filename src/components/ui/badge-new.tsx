import * as React from "react"
import { cva, type VariantProps } from "class-variance-authority"
import { cn } from "@/lib/utils"

const badgeVariants = cva(
  "inline-flex items-center justify-center rounded-full px-2.5 py-0.5 text-xs font-medium whitespace-nowrap transition-colors",
  {
    variants: {
      variant: {
        // KagiNote's primary variant (Trust Blue)
        default:
          "bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400",
        // KagiNote's secondary variant
        secondary:
          "bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-300",
        // KagiNote's success variant (Privacy Green)
        success:
          "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400",
        // KagiNote's warning variant
        warning:
          "bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400",
        // KagiNote's error/destructive variant
        destructive:
          "bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400",
        // KagiNote's neutral variant
        outline:
          "border border-gray-300 text-gray-700 dark:border-gray-600 dark:text-gray-300",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  }
)

export interface BadgeProps
  extends React.HTMLAttributes<HTMLSpanElement>,
    Omit<VariantProps<typeof badgeVariants>, 'variant'> {
  // Support legacy KagiNote props
  children?: React.ReactNode
  // Support both legacy and new variant names
  variant?: 'default' | 'secondary' | 'success' | 'warning' | 'destructive' | 'outline' | 'primary' | 'error' | 'neutral' | null | undefined
  // Support legacy size prop
  size?: 'sm' | 'lg' | null | undefined
}

function Badge({ className, variant, size, ...props }: BadgeProps) {
  // Map legacy KagiNote variant names to new ones
  const mappedVariant = (
    variant === 'primary' ? 'default' : 
    variant === 'error' ? 'destructive' :
    variant === 'neutral' ? 'outline' :
    variant
  ) as 'default' | 'secondary' | 'success' | 'warning' | 'destructive' | 'outline' | null | undefined
  
  // Add size-based styles
  const sizeClasses = size === 'sm' ? 'text-xs px-2 py-0.5' : 
                     size === 'lg' ? 'text-sm px-3 py-1' : 
                     ''

  return (
    <span
      className={cn(badgeVariants({ variant: mappedVariant }), sizeClasses, className)}
      {...props}
    />
  )
}

export { Badge, badgeVariants }