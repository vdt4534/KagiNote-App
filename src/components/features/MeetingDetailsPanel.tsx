import React from 'react';
import { cn, formatDuration } from '@/lib/utils';
import { Badge } from '@/components/ui/Badge';
import { Icon } from '@/components/ui/Icon';
import { Button } from '@/components/ui/Button';

export interface MeetingDetailsPanelProps {
  isOpen: boolean;
  onClose: () => void;
  currentModel?: string;
  language?: string;
  participants?: string[];
  duration: number;
  systemInfo?: {
    cpu: number;
    memory: number;
    rtf: number;
  };
  onOpenSettings?: () => void;
  className?: string;
}

export const MeetingDetailsPanel: React.FC<MeetingDetailsPanelProps> = ({
  isOpen,
  onClose,
  currentModel = 'Standard',
  language = 'English',
  participants = [],
  duration,
  systemInfo = { cpu: 15, memory: 2.1, rtf: 0.8 },
  onOpenSettings,
  className = '',
}) => {
  if (!isOpen) return null;

  const formatSystemMetric = (value: number, unit: string) => {
    return `${value}${unit}`;
  };

  return (
    <>
      {/* Backdrop */}
      <div 
        className="fixed inset-0 bg-black/20 dark:bg-black/40 z-40 animate-in fade-in duration-200"
        onClick={onClose}
      />
      
      {/* Panel */}
      <div className={cn(
        'fixed right-0 top-16 bottom-0 w-full sm:w-80 bg-white dark:bg-neutral-900 shadow-xl z-50',
        'border-l border-neutral-200 dark:border-neutral-700',
        'animate-in slide-in-from-right duration-200',
        className
      )}>
        <div className="flex flex-col h-full">
          {/* Header */}
          <div className="flex items-center justify-between p-4 border-b border-neutral-200 dark:border-neutral-700">
            <h2 className="text-lg font-semibold text-neutral-900 dark:text-neutral-100">
              Meeting Details
            </h2>
            <Button
              variant="ghost"
              size="sm"
              onClick={onClose}
              className="p-1"
            >
              <Icon name="x" size="base" />
            </Button>
          </div>

          {/* Content */}
          <div className="flex-1 overflow-y-auto p-4 space-y-6">
            {/* Recording Info */}
            <div className="space-y-3">
              <h3 className="text-sm font-medium text-neutral-600 dark:text-neutral-400 uppercase tracking-wider">
                Recording Info
              </h3>
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <span className="text-sm text-neutral-600 dark:text-neutral-400">Duration:</span>
                  <span className="text-sm font-mono text-neutral-900 dark:text-neutral-100">
                    {formatDuration(duration)}
                  </span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sm text-neutral-600 dark:text-neutral-400">Model:</span>
                  <Badge variant="primary" size="sm">{currentModel}</Badge>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sm text-neutral-600 dark:text-neutral-400">Language:</span>
                  <Badge variant="neutral" size="sm">{language}</Badge>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sm text-neutral-600 dark:text-neutral-400">Speakers:</span>
                  <span className="text-sm text-neutral-900 dark:text-neutral-100">
                    {participants.length > 0 ? participants.length : 'Auto-detecting'}
                  </span>
                </div>
              </div>
            </div>

            {/* System Performance */}
            <div className="space-y-3">
              <h3 className="text-sm font-medium text-neutral-600 dark:text-neutral-400 uppercase tracking-wider">
                System Performance
              </h3>
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <span className="text-sm text-neutral-600 dark:text-neutral-400">CPU Usage:</span>
                  <span className="text-sm font-mono text-neutral-900 dark:text-neutral-100">
                    {formatSystemMetric(systemInfo.cpu, '%')}
                  </span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sm text-neutral-600 dark:text-neutral-400">Memory:</span>
                  <span className="text-sm font-mono text-neutral-900 dark:text-neutral-100">
                    {formatSystemMetric(systemInfo.memory, 'GB')}
                  </span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sm text-neutral-600 dark:text-neutral-400">RTF:</span>
                  <span className="text-sm font-mono text-neutral-900 dark:text-neutral-100">
                    {formatSystemMetric(systemInfo.rtf, 'x')}
                  </span>
                </div>
              </div>
            </div>

            {/* Privacy Status */}
            <div className="space-y-3">
              <h3 className="text-sm font-medium text-neutral-600 dark:text-neutral-400 uppercase tracking-wider">
                Privacy
              </h3>
              <div className="space-y-2">
                <div className="flex items-center gap-2">
                  <Icon name="shield-check" size="sm" className="text-secondary-600" />
                  <span className="text-sm text-neutral-700 dark:text-neutral-300">
                    100% Local Processing
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  <Icon name="eye-slash" size="sm" className="text-secondary-600" />
                  <span className="text-sm text-neutral-700 dark:text-neutral-300">
                    No Network Required
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  <Icon name="lock-closed" size="sm" className="text-secondary-600" />
                  <span className="text-sm text-neutral-700 dark:text-neutral-300">
                    End-to-End Encrypted
                  </span>
                </div>
              </div>
            </div>
          </div>

          {/* Footer */}
          {onOpenSettings && (
            <div className="p-4 border-t border-neutral-200 dark:border-neutral-700">
              <Button
                variant="ghost"
                size="base"
                onClick={onOpenSettings}
                className="w-full flex items-center justify-center gap-2"
              >
                <Icon name="cog" size="base" />
                <span>Settings</span>
              </Button>
            </div>
          )}
        </div>
      </div>
    </>
  );
};