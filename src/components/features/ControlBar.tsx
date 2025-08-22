import React from 'react';
import { cn, formatDuration } from '@/lib/utils';
import { Button } from '@/components/ui/button-compat';
import { Icon } from '@/components/ui/Icon';
import { Badge } from '@/components/ui/badge-compat';
import { AudioVisualizer } from '@/components/features/AudioVisualizer';

export interface ControlBarProps {
  meetingTitle: string;
  isRecording: boolean;
  isPaused: boolean;
  duration: number;
  audioLevel: number;
  vadActivity: boolean;
  onStart: () => void;
  onPause: () => void;
  onResume: () => void;
  onStop: () => void;
  onToggleDetails?: () => void;
  showDetailsButton?: boolean;
  className?: string;
}

export const ControlBar: React.FC<ControlBarProps> = ({
  meetingTitle,
  isRecording,
  isPaused,
  duration,
  audioLevel,
  vadActivity,
  onStart,
  onPause,
  onResume,
  onStop,
  onToggleDetails,
  showDetailsButton = true,
  className = '',
}) => {
  const getRecordingStatus = () => {
    if (!isRecording) return 'Ready';
    if (isPaused) return 'Paused';
    return 'Recording';
  };

  const getStatusColor = (): "primary" | "secondary" | "warning" | "error" | "neutral" => {
    if (!isRecording) return 'neutral';
    if (isPaused) return 'warning';
    return 'error';
  };

  return (
    <div className={cn(
      'h-16 px-4 flex items-center gap-2 sm:gap-4 bg-white dark:bg-neutral-900 border-b border-neutral-200 dark:border-neutral-700',
      className
    )}>
      {/* Left: Title and Status */}
      <div className="flex items-center gap-2 sm:gap-3 min-w-0 flex-1">
        <h1 className="text-base sm:text-lg font-semibold text-neutral-900 dark:text-neutral-100 truncate">
          {meetingTitle}
        </h1>
        <Badge variant={getStatusColor()} size="sm" className="flex items-center gap-1 flex-shrink-0 hidden sm:flex">
          {isRecording && !isPaused && (
            <div className="w-1.5 h-1.5 bg-current rounded-full animate-pulse" />
          )}
          {getRecordingStatus()}
        </Badge>
      </div>

      {/* Center: Controls and Audio */}
      <div className="flex items-center gap-2 sm:gap-3">
        {/* Recording Controls */}
        <div className="flex items-center gap-1 sm:gap-2">
          {!isRecording ? (
            <Button
              onClick={onStart}
              variant="primary"
              size="sm"
              className="flex items-center gap-1.5"
            >
              <Icon name="play" size="sm" />
              <span className="hidden sm:inline">Start</span>
            </Button>
          ) : (
            <>
              {isPaused ? (
                <Button
                  onClick={onResume}
                  variant="primary"
                  size="sm"
                  className="flex items-center gap-1.5"
                >
                  <Icon name="play" size="sm" />
                  <span className="hidden sm:inline">Resume</span>
                </Button>
              ) : (
                <Button
                  onClick={onPause}
                  variant="ghost"
                  size="sm"
                  className="flex items-center gap-1.5"
                >
                  <Icon name="pause" size="sm" />
                  <span className="hidden sm:inline">Pause</span>
                </Button>
              )}
              <Button
                onClick={onStop}
                variant="danger"
                size="sm"
                className="flex items-center gap-1.5"
              >
                <Icon name="stop" size="sm" />
                <span className="hidden sm:inline">Stop</span>
              </Button>
            </>
          )}
        </div>

        {/* Duration */}
        <div className="text-xs sm:text-sm font-mono text-neutral-600 dark:text-neutral-400 min-w-[60px] sm:min-w-[80px] text-center">
          {formatDuration(duration)}
        </div>

        {/* Audio Visualizer - Hidden on mobile */}
        <div className="hidden sm:flex items-center gap-2">
          {isRecording && (
            <div className="w-1.5 h-1.5 bg-error-500 rounded-full animate-pulse" />
          )}
          <div className="w-32">
            <AudioVisualizer
              audioLevel={audioLevel}
              isRecording={isRecording}
              vadActivity={vadActivity}
              showWaveform={false}
              height={24}
              className="w-full"
            />
          </div>
        </div>
      </div>

      {/* Right: Details Toggle */}
      {showDetailsButton && (
        <Button
          variant="ghost"
          size="sm"
          onClick={onToggleDetails}
          className="flex items-center gap-1.5 ml-auto"
        >
          <Icon name="information-circle" size="sm" />
          <span className="hidden sm:inline">Details</span>
        </Button>
      )}
    </div>
  );
};