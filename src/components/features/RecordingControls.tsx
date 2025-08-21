import React from 'react';
import { cn, formatDuration } from '@/lib/utils';
import { Button } from '@/components/ui/Button';
import { Icon } from '@/components/ui/Icon';
import { Badge } from '@/components/ui/Badge';

export interface RecordingControlsProps {
  isRecording: boolean;
  isPaused: boolean;
  duration: number;
  onStart: () => void;
  onPause: () => void;
  onResume: () => void;
  onStop: () => void;
  disabled?: boolean;
  className?: string;
}

export const RecordingControls: React.FC<RecordingControlsProps> = ({
  isRecording,
  isPaused,
  duration,
  onStart,
  onPause,
  onResume,
  onStop,
  disabled = false,
  className = '',
}) => {
  const primaryAction = () => {
    if (disabled) return;
    
    if (!isRecording) return onStart();
    if (isPaused) return onResume();
    return onPause();
  };

  const primaryLabel = () => {
    if (!isRecording) return 'Start Recording';
    if (isPaused) return 'Resume';
    return 'Pause';
  };

  const primaryIcon = () => {
    if (!isRecording) return 'microphone';
    if (isPaused) return 'play';
    return 'pause';
  };

  const getRecordingStatus = () => {
    if (!isRecording) return null;
    if (isPaused) return 'Paused';
    return 'Recording';
  };

  const getStatusVariant = (): "primary" | "secondary" | "warning" | "error" | "neutral" => {
    if (!isRecording) return 'neutral';
    if (isPaused) return 'warning';
    return 'secondary';
  };

  return (
    <div
      className={cn(
        'flex items-center gap-4 p-6 bg-white rounded-xl border border-neutral-200 shadow-sm',
        'dark:bg-neutral-800 dark:border-neutral-700',
        className
      )}
      role="group"
      aria-label="Recording controls"
    >
      {/* Primary Action Button */}
      <Button
        onClick={primaryAction}
        variant={isRecording ? "secondary" : "primary"}
        size="lg"
        disabled={disabled}
        className={cn(
          "flex items-center gap-3 min-w-[160px]",
          isRecording && !isPaused && "animate-pulse-recording"
        )}
        aria-label={primaryLabel()}
      >
        <Icon name={primaryIcon()} size="base" />
        {primaryLabel()}
      </Button>

      {/* Stop Button (only when recording) */}
      {isRecording && (
        <Button
          onClick={onStop}
          variant="ghost"
          size="lg"
          disabled={disabled}
          className="flex items-center gap-3 text-error-600 hover:text-error-700 hover:bg-error-50 dark:hover:bg-error-900/20"
          aria-label="Stop recording"
        >
          <Icon name="stop" size="base" />
          Stop
        </Button>
      )}

      {/* Duration and Status Display */}
      <div className="flex items-center gap-3 ml-auto">
        <Badge 
          variant="neutral"
          size="lg"
          className="font-mono text-base min-w-[80px] justify-center"
        >
          {formatDuration(duration)}
        </Badge>
        
        {getRecordingStatus() && (
          <Badge 
            variant={getStatusVariant()}
            size="lg"
            className="flex items-center gap-1"
          >
            {!isPaused && isRecording && (
              <div className="w-2 h-2 bg-current rounded-full animate-pulse" />
            )}
            {getRecordingStatus()}
          </Badge>
        )}
      </div>
    </div>
  );
};