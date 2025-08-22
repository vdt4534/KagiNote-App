import React from 'react';
import { cn } from '@/lib/utils';
import { Icon } from '@/components/ui/Icon';
import { Badge } from '@/components/ui/Badge';
import { DiarizationStatus } from '@/components/features/DiarizationStatusIndicator';

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
  diarizationStatus?: DiarizationStatus;
  className?: string;
}

const StatusBar: React.FC<StatusBarProps> = ({
  modelInfo,
  recordingInfo,
  systemInfo,
  diarizationStatus,
  className,
}) => {
  return (
    <div
      className={cn(
        'h-statusbar flex items-center justify-between px-2 sm:px-4',
        'bg-neutral-50 dark:bg-neutral-800 border-t border-neutral-200 dark:border-neutral-700',
        'text-xs text-neutral-600 dark:text-neutral-400',
        className
      )}
      role="contentinfo"
      aria-label="Application status"
    >
      {/* Left side - Model and system info */}
      <div className="flex items-center gap-2 sm:gap-4 min-w-0">
        {modelInfo && (
          <div className="flex items-center gap-1 sm:gap-2 min-w-0">
            <span className="hidden sm:inline">Model:</span>
            <span className="font-medium text-neutral-900 dark:text-neutral-100 truncate text-xs sm:text-xs">
              {modelInfo.name}
            </span>
            <Badge 
              variant={
                modelInfo.status === 'ready' ? 'secondary' : 
                modelInfo.status === 'loading' ? 'warning' : 'error'
              }
              size="sm"
              className="shrink-0"
            >
              {modelInfo.status === 'ready' && <Icon name="check-circle" size="sm" />}
              {modelInfo.status === 'loading' && <Icon name="clock" size="sm" />}
              {modelInfo.status === 'error' && <Icon name="x-circle" size="sm" />}
              <span className="hidden sm:inline">{modelInfo.status}</span>
            </Badge>
          </div>
        )}
        
        {systemInfo && (
          <div className="flex items-center gap-2 sm:gap-4">
            {systemInfo.privacy && (
              <div className="flex items-center gap-1 hidden sm:flex">
                <Icon name="shield-check" size="sm" className="text-secondary-600" />
                <span>Local</span>
              </div>
            )}
            
            {systemInfo.cpu && (
              <div className="hidden md:flex items-center gap-1">
                <span>CPU:</span>
                <span className="font-mono text-xs">{systemInfo.cpu}</span>
              </div>
            )}
            
            {systemInfo.memory && (
              <div className="hidden md:flex items-center gap-1">
                <span>RAM:</span>
                <span className="font-mono text-xs">{systemInfo.memory}</span>
              </div>
            )}
          </div>
        )}
        
        {/* Diarization Status */}
        {diarizationStatus && diarizationStatus.serviceHealth !== 'disabled' && (
          <div className="flex items-center gap-1 sm:gap-2 shrink-0">
            <Icon 
              name={diarizationStatus.serviceHealth === 'ready' ? 'users' : 
                    diarizationStatus.serviceHealth === 'initializing' ? 'clock' : 'users-x'} 
              size="sm" 
              className={cn(
                'shrink-0',
                diarizationStatus.serviceHealth === 'ready' ? 'text-secondary-600' :
                diarizationStatus.serviceHealth === 'initializing' ? 'text-warning-600' : 'text-error-600'
              )}
            />
            <span className="text-xs hidden sm:inline">
              {diarizationStatus.serviceHealth === 'ready' && diarizationStatus.speakerCount !== undefined
                ? `${diarizationStatus.speakerCount} speakers`
                : diarizationStatus.serviceHealth === 'initializing'
                ? 'Loading...'
                : 'Error'
              }
            </span>
            <span className="text-xs sm:hidden">
              {diarizationStatus.serviceHealth === 'ready' && diarizationStatus.speakerCount !== undefined
                ? `${diarizationStatus.speakerCount}`
                : diarizationStatus.serviceHealth === 'initializing'
                ? '...'
                : '!'
              }
            </span>
            {diarizationStatus.serviceHealth === 'error' && (
              <Badge variant="error" size="sm" className="hidden sm:inline-flex">
                <Icon name="x-circle" size="sm" />
                Error
              </Badge>
            )}
          </div>
        )}
      </div>
      
      {/* Center - Diarization Status (if recording) - Hidden on mobile to save space */}
      {recordingInfo?.isRecording && diarizationStatus && diarizationStatus.serviceHealth !== 'disabled' && (
        <div className="hidden lg:flex items-center gap-2">
          <div 
            className={cn(
              'w-2 h-2 rounded-full shrink-0',
              diarizationStatus.serviceHealth === 'ready' ? 'bg-secondary-500' :
              diarizationStatus.serviceHealth === 'initializing' ? 'bg-warning-500 animate-pulse' :
              'bg-error-500'
            )}
          />
          <span className="text-xs font-medium whitespace-nowrap">
            {diarizationStatus.serviceHealth === 'ready' && diarizationStatus.speakerCount !== undefined
              ? `${diarizationStatus.speakerCount} speaker${diarizationStatus.speakerCount !== 1 ? 's' : ''}`
              : diarizationStatus.serviceHealth === 'initializing'
              ? 'Detecting speakers'
              : 'Speaker error'
            }
          </span>
        </div>
      )}
      
      {/* Right side - Recording info */}
      <div className="flex items-center gap-2 sm:gap-4 shrink-0">
        {recordingInfo && (
          <div className="flex items-center gap-1 sm:gap-2">
            {recordingInfo.isRecording && (
              <>
                <div className="flex items-center gap-1">
                  <div className="w-2 h-2 bg-error-500 rounded-full animate-pulse shrink-0" />
                  <span className="font-medium text-error-600 dark:text-error-400 text-xs sm:text-xs">
                    <span className="hidden sm:inline">Recording</span>
                    <span className="sm:hidden">REC</span>
                  </span>
                </div>
                
                {recordingInfo.duration && (
                  <span className="font-mono text-neutral-900 dark:text-neutral-100 text-xs sm:text-xs whitespace-nowrap">
                    {recordingInfo.duration}
                  </span>
                )}
              </>
            )}
            
            {recordingInfo.status && !recordingInfo.isRecording && (
              <span className="text-neutral-500 dark:text-neutral-400 text-xs truncate">
                {recordingInfo.status}
              </span>
            )}
          </div>
        )}
        
        {/* Privacy reminder */}
        <div className="flex items-center gap-1 text-secondary-600 dark:text-secondary-400 shrink-0">
          <Icon name="eye-slash" size="sm" className="shrink-0" />
          <span className="hidden sm:inline text-xs">No Network Required</span>
          <span className="sm:hidden text-xs">Private</span>
        </div>
      </div>
    </div>
  );
};

StatusBar.displayName = 'StatusBar';

export { StatusBar };