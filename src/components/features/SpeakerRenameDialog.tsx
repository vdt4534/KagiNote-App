import React, { useState, useEffect } from 'react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Badge } from '@/components/ui/Badge';

export interface SpeakerRenameDialogProps {
  speaker: {
    id: string;
    displayName: string;
    confidence?: number;
  };
  onRename: (speakerId: string, newName: string) => void;
  onClose: () => void;
  isOpen: boolean;
  existingNames?: string[];
  className?: string;
}

export const SpeakerRenameDialog: React.FC<SpeakerRenameDialogProps> = ({
  speaker,
  onRename,
  onClose,
  isOpen,
  existingNames = [],
  className = '',
}) => {
  const [name, setName] = useState(speaker.displayName);
  const [error, setError] = useState('');
  const [touched, setTouched] = useState(false);

  useEffect(() => {
    if (isOpen) {
      setName(speaker.displayName);
      setError('');
      setTouched(false);
    }
  }, [isOpen, speaker.displayName]);

  const validateName = (newName: string): string => {
    const trimmedName = newName.trim();
    
    if (!trimmedName) {
      return 'Name cannot be empty';
    }
    
    if (trimmedName.length > 100) {
      return 'Name must be 100 characters or less';
    }
    
    if (trimmedName === speaker.displayName) {
      return ''; // Same name is okay
    }
    
    if (existingNames.includes(trimmedName)) {
      return 'This name is already in use';
    }
    
    return '';
  };

  const handleNameChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newName = e.target.value;
    setName(newName);
    setTouched(true);
    
    const validationError = validateName(newName);
    setError(validationError);
  };

  const handleSave = () => {
    const trimmedName = name.trim();
    const validationError = validateName(trimmedName);
    
    if (validationError) {
      setError(validationError);
      setTouched(true);
      return;
    }
    
    onRename(speaker.id, trimmedName);
    onClose();
  };

  const handleCancel = () => {
    setName(speaker.displayName);
    setError('');
    setTouched(false);
    onClose();
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !error && name.trim()) {
      handleSave();
    } else if (e.key === 'Escape') {
      handleCancel();
    }
  };

  if (!isOpen) {
    return null;
  }

  const hasError = touched && error;
  const canSave = name.trim() && !error;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div 
        data-testid="speaker-rename-dialog"
        className={cn(
          'bg-white dark:bg-neutral-900 p-6 rounded-lg shadow-xl max-w-md w-full mx-4',
          className
        )}
      >
        {/* Header */}
        <h3 className="text-lg font-semibold mb-4 text-neutral-900 dark:text-neutral-100">
          Rename Speaker
        </h3>

        {/* Speaker Info */}
        <div className="mb-4 p-3 bg-neutral-50 dark:bg-neutral-800 rounded-md">
          <div className="flex items-center justify-between">
            <span className="text-sm text-neutral-600 dark:text-neutral-400">
              Current name:
            </span>
            <span className="font-medium text-neutral-900 dark:text-neutral-100">
              {speaker.displayName}
            </span>
          </div>
          
          {speaker.confidence !== undefined && (
            <div className="flex items-center justify-between mt-2">
              <span className="text-sm text-neutral-600 dark:text-neutral-400">
                Confidence:
              </span>
              <Badge
                variant={speaker.confidence > 0.8 ? "secondary" : speaker.confidence > 0.6 ? "warning" : "error"}
                size="sm"
              >
                {Math.round(speaker.confidence * 100)}%
              </Badge>
            </div>
          )}
        </div>

        {/* Name Input */}
        <div className="mb-4">
          <label 
            htmlFor="speaker-name-input"
            className="block text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-2"
          >
            New name:
          </label>
          
          <Input
            id="speaker-name-input"
            type="text"
            value={name}
            onChange={handleNameChange}
            onKeyDown={handleKeyDown}
            placeholder="Enter speaker name"
            className={cn(
              hasError && 'border-red-500 focus:border-red-500 focus:ring-red-500'
            )}
            autoFocus
            maxLength={100}
          />
          
          {/* Character count */}
          <div className="flex justify-between items-center mt-1">
            <div>
              {hasError && (
                <p className="text-sm text-red-600 dark:text-red-400">
                  {error}
                </p>
              )}
            </div>
            
            <span className="text-xs text-neutral-500 dark:text-neutral-400">
              {name.length}/100
            </span>
          </div>
        </div>

        {/* Suggestions (if any) */}
        {speaker.displayName.startsWith('Speaker ') && (
          <div className="mb-4">
            <p className="text-sm text-neutral-600 dark:text-neutral-400 mb-2">
              Suggestions:
            </p>
            <div className="flex flex-wrap gap-2">
              {['Host', 'Presenter', 'Guest', 'Interviewer', 'Participant'].map((suggestion) => (
                <button
                  key={suggestion}
                  onClick={() => {
                    setName(suggestion);
                    setTouched(true);
                    setError(validateName(suggestion));
                  }}
                  className="px-2 py-1 text-xs bg-neutral-100 dark:bg-neutral-800 hover:bg-neutral-200 dark:hover:bg-neutral-700 rounded transition-colors"
                >
                  {suggestion}
                </button>
              ))}
            </div>
          </div>
        )}

        {/* Actions */}
        <div className="flex gap-2 justify-end">
          <Button
            variant="ghost"
            onClick={handleCancel}
          >
            Cancel
          </Button>
          <Button
            onClick={handleSave}
            disabled={!canSave}
          >
            Save
          </Button>
        </div>
      </div>
    </div>
  );
};