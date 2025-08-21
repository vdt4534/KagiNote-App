import React, { useEffect } from 'react';
import { cn } from '@/lib/utils';
import { FocusUtils } from '@/lib/utils';
import { Icon } from './Icon';

export interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title?: string;
  children: React.ReactNode;
  size?: 'sm' | 'base' | 'lg' | 'xl';
  closeOnOverlayClick?: boolean;
  showCloseButton?: boolean;
  className?: string;
}

const Modal: React.FC<ModalProps> = ({
  isOpen,
  onClose,
  title,
  children,
  size = 'base',
  closeOnOverlayClick = true,
  showCloseButton = true,
  className,
}) => {
  const modalRef = React.useRef<HTMLDivElement>(null);
  
  const sizeClasses = {
    sm: 'max-w-md',
    base: 'max-w-lg',
    lg: 'max-w-2xl',
    xl: 'max-w-4xl',
  };

  // Handle escape key and focus management
  useEffect(() => {
    if (!isOpen) return;
    
    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        onClose();
      }
    };
    
    const handleTabKey = (event: KeyboardEvent) => {
      if (modalRef.current) {
        FocusUtils.trapFocus(modalRef.current, event);
      }
    };
    
    document.addEventListener('keydown', handleEscape);
    document.addEventListener('keydown', handleTabKey);
    
    // Focus the first focusable element
    if (modalRef.current) {
      FocusUtils.focusFirst(modalRef.current);
    }
    
    // Prevent body scroll
    document.body.style.overflow = 'hidden';
    
    return () => {
      document.removeEventListener('keydown', handleEscape);
      document.removeEventListener('keydown', handleTabKey);
      document.body.style.overflow = 'unset';
    };
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  const handleOverlayClick = (event: React.MouseEvent) => {
    if (closeOnOverlayClick && event.target === event.currentTarget) {
      onClose();
    }
  };

  return (
    <div className="modal-overlay" onClick={handleOverlayClick}>
      <div 
        ref={modalRef}
        className={cn(
          'modal-content',
          sizeClasses[size],
          className
        )}
        role="dialog"
        aria-modal="true"
        aria-labelledby={title ? 'modal-title' : undefined}
      >
        {(title || showCloseButton) && (
          <div className="flex items-center justify-between p-6 border-b border-neutral-200 dark:border-neutral-700">
            {title && (
              <h2 
                id="modal-title"
                className="text-lg font-semibold text-neutral-900 dark:text-neutral-100"
              >
                {title}
              </h2>
            )}
            
            {showCloseButton && (
              <button
                onClick={onClose}
                className="p-2 text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-300 rounded-md hover:bg-neutral-100 dark:hover:bg-neutral-700 transition-colors"
                aria-label="Close modal"
              >
                <Icon name="x" size="base" />
              </button>
            )}
          </div>
        )}
        
        <div className="p-6">
          {children}
        </div>
      </div>
    </div>
  );
};

Modal.displayName = 'Modal';

export { Modal };