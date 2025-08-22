/**
 * ToastContainer Component
 * 
 * Manages and displays multiple toast notifications in a stack.
 * Provides context for creating and dismissing toasts throughout the app.
 */

import React, { createContext, useContext, useState, useCallback } from 'react';
import { Toast, ToastProps } from './Toast';

interface ToastData {
  id: string;
  title: string;
  message: string;
  type: 'info' | 'success' | 'warning' | 'error';
  duration?: number;
  actions?: Array<{
    label: string;
    onClick: () => void;
    variant?: 'primary' | 'secondary';
  }>;
}

interface ToastContextType {
  showToast: (toast: Omit<ToastData, 'id'>) => string;
  dismissToast: (id: string) => void;
  clearAllToasts: () => void;
}

const ToastContext = createContext<ToastContextType | null>(null);

export const useToast = () => {
  const context = useContext(ToastContext);
  if (!context) {
    throw new Error('useToast must be used within a ToastProvider');
  }
  return context;
};

interface ToastProviderProps {
  children: React.ReactNode;
  maxToasts?: number;
}

export const ToastProvider: React.FC<ToastProviderProps> = ({
  children,
  maxToasts = 5,
}) => {
  const [toasts, setToasts] = useState<ToastData[]>([]);

  const showToast = useCallback((toastData: Omit<ToastData, 'id'>) => {
    const id = Math.random().toString(36).substr(2, 9);
    const newToast: ToastData = { ...toastData, id };

    setToasts(prev => {
      const updated = [newToast, ...prev];
      // Keep only the most recent toasts if we exceed the limit
      return updated.slice(0, maxToasts);
    });

    return id;
  }, [maxToasts]);

  const dismissToast = useCallback((id: string) => {
    setToasts(prev => prev.filter(toast => toast.id !== id));
  }, []);

  const clearAllToasts = useCallback(() => {
    setToasts([]);
  }, []);

  const contextValue: ToastContextType = {
    showToast,
    dismissToast,
    clearAllToasts,
  };

  return (
    <ToastContext.Provider value={contextValue}>
      {children}
      <ToastContainer toasts={toasts} onDismiss={dismissToast} />
    </ToastContext.Provider>
  );
};

interface ToastContainerProps {
  toasts: ToastData[];
  onDismiss: (id: string) => void;
}

const ToastContainer: React.FC<ToastContainerProps> = ({ toasts, onDismiss }) => {
  if (toasts.length === 0) return null;

  return (
    <div
      className="fixed top-4 right-4 z-50 space-y-2 pointer-events-none"
      data-testid="toast-container"
    >
      {toasts.map(toast => (
        <Toast
          key={toast.id}
          {...toast}
          onDismiss={onDismiss}
        />
      ))}
    </div>
  );
};

// Convenience functions for common toast types
export const createDiarizationToasts = (useToast: () => ToastContextType) => {
  const { showToast } = useToast();

  return {
    showDiarizationWarning: (message: string, recoveryHint?: string) => {
      return showToast({
        title: 'Speaker Detection Warning',
        message,
        type: 'warning',
        duration: 8000,
        actions: recoveryHint ? [{
          label: 'Learn More',
          onClick: () => console.log('Show recovery guidance:', recoveryHint),
          variant: 'secondary'
        }] : undefined
      });
    },

    showDiarizationError: (message: string, recoveryOptions?: string[]) => {
      return showToast({
        title: 'Speaker Detection Error',
        message,
        type: 'error',
        duration: 0, // Don't auto-dismiss errors
        actions: recoveryOptions ? recoveryOptions.slice(0, 2).map(option => ({
          label: option.replace('_', ' '),
          onClick: () => console.log('Execute recovery option:', option),
          variant: 'primary' as const
        })) : undefined
      });
    },

    showDiarizationInfo: (message: string) => {
      return showToast({
        title: 'Speaker Detection',
        message,
        type: 'info',
        duration: 4000
      });
    },

    showSpeakerDetected: (speakerName: string, confidence: number) => {
      return showToast({
        title: 'New Speaker Detected',
        message: `${speakerName} (${Math.round(confidence * 100)}% confidence)`,
        type: 'success',
        duration: 3000
      });
    },

    showModelLoadingInfo: (modelName: string) => {
      return showToast({
        title: 'Loading Speaker Models',
        message: `Initializing ${modelName} for speaker detection...`,
        type: 'info',
        duration: 6000
      });
    }
  };
};

export default ToastContainer;