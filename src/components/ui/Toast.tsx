/**
 * Toast Component
 * 
 * Provides toast notifications for warnings, errors, and informational messages.
 * Includes auto-dismiss functionality and action buttons for recovery options.
 */

import React, { useState, useEffect } from 'react';
import { cn } from '@/lib/utils';
import { Icon } from '@/components/ui/Icon';

export interface ToastProps {
  id: string;
  title: string;
  message: string;
  type: 'info' | 'success' | 'warning' | 'error';
  duration?: number; // milliseconds, 0 = permanent
  actions?: Array<{
    label: string;
    onClick: () => void;
    variant?: 'primary' | 'secondary';
  }>;
  onDismiss: (id: string) => void;
}

const Toast: React.FC<ToastProps> = ({
  id,
  title,
  message,
  type,
  duration = 5000,
  actions = [],
  onDismiss,
}) => {
  const [isVisible, setIsVisible] = useState(true);

  useEffect(() => {
    if (duration > 0) {
      const timer = setTimeout(() => {
        handleDismiss();
      }, duration);

      return () => clearTimeout(timer);
    }
  }, [duration, id]);

  const handleDismiss = () => {
    setIsVisible(false);
    setTimeout(() => onDismiss(id), 150); // Wait for animation
  };

  const getIcon = () => {
    switch (type) {
      case 'info':
        return 'info';
      case 'success':
        return 'check-circle';
      case 'warning':
        return 'alert-triangle';
      case 'error':
        return 'x-circle';
      default:
        return 'info';
    }
  };

  const getTypeStyles = () => {
    switch (type) {
      case 'info':
        return 'bg-blue-50 dark:bg-blue-950 border-blue-200 dark:border-blue-800 text-blue-900 dark:text-blue-100';
      case 'success':
        return 'bg-green-50 dark:bg-green-950 border-green-200 dark:border-green-800 text-green-900 dark:text-green-100';
      case 'warning':
        return 'bg-yellow-50 dark:bg-yellow-950 border-yellow-200 dark:border-yellow-800 text-yellow-900 dark:text-yellow-100';
      case 'error':
        return 'bg-red-50 dark:bg-red-950 border-red-200 dark:border-red-800 text-red-900 dark:text-red-100';
      default:
        return 'bg-neutral-50 dark:bg-neutral-950 border-neutral-200 dark:border-neutral-800 text-neutral-900 dark:text-neutral-100';
    }
  };

  const getIconColor = () => {
    switch (type) {
      case 'info':
        return 'text-blue-500';
      case 'success':
        return 'text-green-500';
      case 'warning':
        return 'text-yellow-500';
      case 'error':
        return 'text-red-500';
      default:
        return 'text-neutral-500';
    }
  };

  return (
    <div
      className={cn(
        'max-w-sm w-full pointer-events-auto border rounded-lg shadow-lg p-4 transition-all duration-150',
        getTypeStyles(),
        isVisible ? 'opacity-100 translate-x-0' : 'opacity-0 translate-x-full'
      )}
      data-testid={`toast-${type}`}
    >
      <div className="flex items-start gap-3">
        <Icon
          name={getIcon()}
          size="sm"
          className={cn('flex-shrink-0 mt-0.5', getIconColor())}
        />
        
        <div className="flex-1 min-w-0">
          <div className="flex items-start justify-between">
            <div className="flex-1">
              <h3 className="text-sm font-medium">{title}</h3>
              <p className="mt-1 text-sm opacity-90">{message}</p>
            </div>
            
            <button
              onClick={handleDismiss}
              className="flex-shrink-0 ml-2 p-1 rounded hover:bg-black/10 dark:hover:bg-white/10 transition-colors"
              data-testid="toast-dismiss"
            >
              <Icon name="x" size="sm" className="opacity-60" />
            </button>
          </div>
          
          {actions.length > 0 && (
            <div className="mt-3 flex gap-2">
              {actions.map((action, index) => (
                <button
                  key={index}
                  onClick={action.onClick}
                  className={cn(
                    'px-3 py-1.5 text-xs font-medium rounded transition-colors',
                    action.variant === 'primary'
                      ? 'bg-current text-white hover:opacity-90'
                      : 'border border-current hover:bg-current hover:text-white'
                  )}
                  data-testid={`toast-action-${index}`}
                >
                  {action.label}
                </button>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

Toast.displayName = 'Toast';

export { Toast };