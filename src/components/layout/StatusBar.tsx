import React from 'react';
import { cn } from '@/lib/utils';
import { Icon } from '@/components/ui/Icon';
import { Badge } from '@/components/ui/Badge';

export interface StatusBarProps {
  modelInfo?: {
    name: string;
    status: 'ready' | 'loading' | 'error';
  };
  recordingInfo?: {
    isRecording: boolean;
    duration?: string;
    status?: string;
  };
  systemInfo?: {
    privacy: boolean;
    cpu?: string;
    memory?: string;
  };
  className?: string;
}

const StatusBar: React.FC<StatusBarProps> = ({
  modelInfo,
  recordingInfo,
  systemInfo,
  className,
}) => {
  return (
    <div
      className={cn(
        'h-statusbar flex items-center justify-between px-4',
        'bg-neutral-50 dark:bg-neutral-800 border-t border-neutral-200 dark:border-neutral-700',
        'text-xs text-neutral-600 dark:text-neutral-400',
        className
      )}
      role="contentinfo"
      aria-label="Application status"
    >
      {/* Left side - Model and system info */}
      <div className="flex items-center gap-4">
        {modelInfo && (
          <div className="flex items-center gap-2">
            <span>Model:</span>
            <span className="font-medium text-neutral-900 dark:text-neutral-100">
              {modelInfo.name}
            </span>
            <Badge 
              variant={
                modelInfo.status === 'ready' ? 'secondary' : 
                modelInfo.status === 'loading' ? 'warning' : 'error'
              }
              size="sm"
            >
              {modelInfo.status === 'ready' && <Icon name="check-circle" size="sm" />}
              {modelInfo.status === 'loading' && <Icon name="clock" size="sm" />}
              {modelInfo.status === 'error' && <Icon name="x-circle" size="sm" />}
              {modelInfo.status}
            </Badge>
          </div>
        )}
        
        {systemInfo && (
          <div className="flex items-center gap-4">
            {systemInfo.privacy && (
              <div className="flex items-center gap-1">
                <Icon name="shield-check" size="sm" className="text-secondary-600" />
                <span>Local Processing</span>
              </div>
            )}
            
            {systemInfo.cpu && (
              <div className="flex items-center gap-1">
                <span>CPU:</span>
                <span className="font-mono">{systemInfo.cpu}</span>
              </div>
            )}
            
            {systemInfo.memory && (
              <div className="flex items-center gap-1">
                <span>RAM:</span>
                <span className="font-mono">{systemInfo.memory}</span>
              </div>
            )}
          </div>
        )}
      </div>
      
      {/* Right side - Recording info */}
      <div className="flex items-center gap-4">
        {recordingInfo && (
          <div className="flex items-center gap-2">
            {recordingInfo.isRecording && (
              <>
                <div className="flex items-center gap-1">
                  <div className="w-2 h-2 bg-error-500 rounded-full animate-pulse" />
                  <span className="font-medium text-error-600 dark:text-error-400">
                    Recording
                  </span>
                </div>
                
                {recordingInfo.duration && (
                  <span className="font-mono text-neutral-900 dark:text-neutral-100">
                    {recordingInfo.duration}
                  </span>
                )}
              </>
            )}
            
            {recordingInfo.status && !recordingInfo.isRecording && (
              <span className="text-neutral-500 dark:text-neutral-400">
                {recordingInfo.status}
              </span>
            )}
          </div>
        )}
        
        {/* Privacy reminder */}
        <div className="flex items-center gap-1 text-secondary-600 dark:text-secondary-400">
          <Icon name="eye-slash" size="sm" />
          <span>No Network Required</span>
        </div>
      </div>
    </div>
  );
};

StatusBar.displayName = 'StatusBar';

export { StatusBar };