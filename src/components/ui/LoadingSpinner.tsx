import React from 'react';
import { cn } from '@/lib/utils';

export interface LoadingSpinnerProps {
  size?: 'sm' | 'base' | 'lg' | 'xl';
  className?: string;
  'aria-label'?: string;
}

const LoadingSpinner: React.FC<LoadingSpinnerProps> = ({
  size = 'base',
  className,
  'aria-label': ariaLabel = 'Loading...',
}) => {
  const sizeClasses = {
    sm: 'h-4 w-4 border-2',
    base: 'h-6 w-6 border-2',
    lg: 'h-8 w-8 border-2',
    xl: 'h-12 w-12 border-4',
  };

  return (
    <div
      className={cn(
        'animate-spin rounded-full border-primary-500 border-t-transparent',
        sizeClasses[size],
        className
      )}
      role="status"
      aria-label={ariaLabel}
    >
      <span className="sr-only">{ariaLabel}</span>
    </div>
  );
};

LoadingSpinner.displayName = 'LoadingSpinner';

export { LoadingSpinner };