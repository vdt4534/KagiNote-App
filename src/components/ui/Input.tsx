import React from 'react';
import { cn } from '@/lib/utils';

export interface InputProps extends Omit<React.InputHTMLAttributes<HTMLInputElement>, 'size'> {
  size?: 'sm' | 'base' | 'lg';
  error?: boolean;
  helperText?: string;
  label?: string;
}

const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className, size = 'base', error = false, helperText, label, id, ...props }, ref) => {
    const inputId = id || React.useId();
    const helperTextId = helperText ? `${inputId}-helper` : undefined;
    
    const baseClasses = 'input';
    
    const sizeClasses = {
      sm: 'input-sm',
      base: '',
      lg: 'input-lg',
    };

    const errorClasses = error ? 'border-error-500 focus:border-error-500 focus:ring-error-100' : '';

    return (
      <div className="space-y-1">
        {label && (
          <label 
            htmlFor={inputId}
            className="block text-sm font-medium text-neutral-700 dark:text-neutral-300"
          >
            {label}
          </label>
        )}
        
        <input
          id={inputId}
          className={cn(
            baseClasses,
            sizeClasses[size],
            errorClasses,
            className
          )}
          ref={ref}
          aria-describedby={helperTextId}
          aria-invalid={error}
          {...props}
        />
        
        {helperText && (
          <p 
            id={helperTextId}
            className={cn(
              'text-sm',
              error ? 'text-error-600 dark:text-error-400' : 'text-neutral-500 dark:text-neutral-400'
            )}
          >
            {helperText}
          </p>
        )}
      </div>
    );
  }
);

Input.displayName = 'Input';

export { Input };