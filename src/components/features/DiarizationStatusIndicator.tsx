/**
 * DiarizationStatusIndicator Component
 * 
 * Shows real-time status of speaker diarization service including:
 * - Service health (ready, loading, error)
 * - Model loading status
 * - Speaker detection capability
 * - Error messages with recovery guidance
 */

import React from 'react';
import { cn } from '@/lib/utils';
import { Icon } from '@/components/ui/Icon';
import { Badge } from '@/components/ui/Badge';

export interface DiarizationStatus {
  serviceHealth: 'ready' | 'initializing' | 'error' | 'disabled';
  modelStatus: 'loaded' | 'loading' | 'failed' | 'not_available';
  speakerCount?: number;
  lastError?: {
    message: string;
    type: 'warning' | 'error' | 'critical';
    recoveryHint?: string;
  };
  confidence?: number; // 0-1 confidence in current speaker detection
}

export interface DiarizationStatusIndicatorProps {
  status: DiarizationStatus;
  className?: string;
  showDetails?: boolean;
  onToggleDetails?: () => void;
}

const DiarizationStatusIndicator: React.FC<DiarizationStatusIndicatorProps> = ({
  status,
  className,
  showDetails = false,
  onToggleDetails,
}) => {
  const getStatusColor = () => {
    switch (status.serviceHealth) {
      case 'ready':
        return 'text-secondary-600 dark:text-secondary-400';
      case 'initializing':
        return 'text-warning-600 dark:text-warning-400';
      case 'error':
        return 'text-error-600 dark:text-error-400';
      case 'disabled':
        return 'text-neutral-500 dark:text-neutral-400';
      default:
        return 'text-neutral-500 dark:text-neutral-400';
    }
  };

  const getStatusIcon = () => {
    switch (status.serviceHealth) {
      case 'ready':
        return 'users';
      case 'initializing':
        return 'clock';
      case 'error':
        return 'x-circle';
      case 'disabled':
        return 'users-x';
      default:
        return 'users';
    }
  };

  const getStatusText = () => {
    switch (status.serviceHealth) {
      case 'ready':
        return status.speakerCount !== undefined 
          ? `${status.speakerCount} speaker${status.speakerCount !== 1 ? 's' : ''} detected`
          : 'Speaker detection ready';
      case 'initializing':
        return status.modelStatus === 'loading' 
          ? 'Loading speaker models...'
          : 'Initializing speaker detection...';
      case 'error':
        return status.lastError?.message || 'Speaker detection unavailable';
      case 'disabled':
        return 'Speaker detection disabled';
      default:
        return 'Unknown status';
    }
  };

  const getBadgeVariant = () => {
    switch (status.serviceHealth) {
      case 'ready':
        return 'secondary';
      case 'initializing':
        return 'warning';
      case 'error':
        return 'error';
      case 'disabled':
        return 'neutral';
      default:
        return 'neutral';
    }
  };

  return (
    <div 
      className={cn(
        'flex items-center gap-2',
        className
      )}
      data-testid="diarization-status-indicator"
    >
      {/* Main Status Display */}
      <div className="flex items-center gap-2">
        <Icon 
          name={getStatusIcon()} 
          size="sm" 
          className={cn(
            getStatusColor(),
            status.serviceHealth === 'initializing' && 'animate-pulse'
          )}
        />
        
        <span className={cn(
          'text-sm font-medium',
          getStatusColor()
        )}>
          {getStatusText()}
        </span>
        
        <Badge 
          variant={getBadgeVariant()}
          size="sm"
          data-testid="diarization-status-badge"
        >
          {status.serviceHealth}
        </Badge>
      </div>

      {/* Confidence Indicator */}
      {status.serviceHealth === 'ready' && status.confidence !== undefined && (
        <div className="flex items-center gap-1">
          <div className="w-12 h-1.5 bg-neutral-200 dark:bg-neutral-700 rounded-full overflow-hidden">
            <div 
              className="h-full bg-secondary-500 transition-all duration-300"
              style={{ width: `${status.confidence * 100}%` }}
            />
          </div>
          <span className="text-xs text-neutral-500 dark:text-neutral-400">
            {Math.round(status.confidence * 100)}%
          </span>
        </div>
      )}

      {/* Details Toggle */}
      {onToggleDetails && (
        <button
          onClick={onToggleDetails}
          className="p-1 rounded hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
          data-testid="diarization-details-toggle"
        >
          <Icon 
            name={showDetails ? 'chevron-up' : 'chevron-down'} 
            size="sm" 
            className="text-neutral-500 dark:text-neutral-400"
          />
        </button>
      )}

      {/* Error Details */}
      {showDetails && status.lastError && (
        <div className={cn(
          'absolute top-full left-0 mt-2 p-3 rounded-lg shadow-lg z-10',
          'bg-white dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700',
          'min-w-64 max-w-80'
        )}>
          <div className="flex items-start gap-2 mb-2">
            <Icon 
              name={status.lastError.type === 'critical' ? 'x-circle' : 'alert-triangle'} 
              size="sm" 
              className={cn(
                status.lastError.type === 'critical' ? 'text-error-500' : 'text-warning-500'
              )}
            />
            <div className="flex-1">
              <p className="text-sm font-medium text-neutral-900 dark:text-neutral-100">
                {status.lastError.message}
              </p>
              {status.lastError.recoveryHint && (
                <p className="text-xs text-neutral-600 dark:text-neutral-400 mt-1">
                  {status.lastError.recoveryHint}
                </p>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

DiarizationStatusIndicator.displayName = 'DiarizationStatusIndicator';

export { DiarizationStatusIndicator };