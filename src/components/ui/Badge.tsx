import React from 'react';
import { cn } from '@/lib/utils';

export interface BadgeProps extends React.HTMLAttributes<HTMLSpanElement> {
  variant?: 'primary' | 'secondary' | 'warning' | 'error' | 'neutral';
  size?: 'sm' | 'base' | 'lg';
  children: React.ReactNode;
}

const Badge = React.forwardRef<HTMLSpanElement, BadgeProps>(
  ({ className, variant = 'neutral', size = 'base', children, ...props }, ref) => {
    const baseClasses = 'badge';
    
    const variantClasses = {
      primary: 'badge-primary',
      secondary: 'badge-secondary',
      warning: 'badge-warning',
      error: 'badge-error',
      neutral: 'badge-neutral',
    };
    
    const sizeClasses = {
      sm: 'px-2 py-0.5 text-xs',
      base: 'px-2.5 py-0.5 text-xs',
      lg: 'px-3 py-1 text-sm',
    };

    return (
      <span
        ref={ref}
        className={cn(
          baseClasses,
          variantClasses[variant],
          sizeClasses[size],
          className
        )}
        {...props}
      >
        {children}
      </span>
    );
  }
);

Badge.displayName = 'Badge';

export { Badge };