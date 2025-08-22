/**
 * SpeakerActivityDisplay Component
 * 
 * Real-time display of speaker activity including:
 * - Current active speaker
 * - Speaker activity levels
 * - Speaking duration indicators
 * - Overlapping speech detection
 */

import React from 'react';
import { cn } from '@/lib/utils';
import { Icon } from '@/components/ui/Icon';

export interface SpeakerActivity {
  speakerId: string;
  displayName: string;
  color?: string;
  isActive: boolean;
  activityLevel: number; // 0-1 intensity
  confidenceScore: number; // 0-1 confidence in speaker identification
  speakingDuration?: number; // seconds currently speaking
}

export interface SpeakerActivityDisplayProps {
  speakers: SpeakerActivity[];
  currentSpeaker?: string;
  isProcessing?: boolean;
  hasOverlappingSpeech?: boolean;
  className?: string;
  layout?: 'horizontal' | 'vertical' | 'compact';
}

const SpeakerActivityDisplay: React.FC<SpeakerActivityDisplayProps> = ({
  speakers = [],
  currentSpeaker,
  isProcessing = false,
  hasOverlappingSpeech = false,
  className,
  layout = 'horizontal',
}) => {
  const activeSpeakers = speakers.filter(s => s.isActive);
  const inactiveSpeakers = speakers.filter(s => !s.isActive);

  const formatDuration = (seconds: number) => {
    if (seconds < 60) return `${Math.round(seconds)}s`;
    const mins = Math.floor(seconds / 60);
    const secs = Math.round(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const SpeakerItem: React.FC<{ speaker: SpeakerActivity }> = ({ speaker }) => (
    <div
      className={cn(
        'flex items-center gap-2 p-2 rounded-lg transition-all duration-200',
        speaker.isActive ? 'bg-neutral-100 dark:bg-neutral-800' : 'bg-neutral-50 dark:bg-neutral-900',
        speaker.speakerId === currentSpeaker && 'ring-2 ring-secondary-500',
        layout === 'compact' && 'p-1'
      )}
      data-testid={`speaker-activity-${speaker.speakerId}`}
    >
      {/* Speaker Color Indicator */}
      <div 
        className={cn(
          'w-3 h-3 rounded-full flex-shrink-0',
          speaker.isActive && 'animate-pulse'
        )}
        style={{ 
          backgroundColor: speaker.color || '#6B7280',
          opacity: speaker.activityLevel * 0.8 + 0.2
        }}
      />
      
      {/* Speaker Name */}
      <span className={cn(
        'text-sm font-medium flex-1 truncate',
        speaker.isActive 
          ? 'text-neutral-900 dark:text-neutral-100' 
          : 'text-neutral-600 dark:text-neutral-400'
      )}>
        {speaker.displayName}
      </span>
      
      {/* Activity Level Bar */}
      {layout !== 'compact' && (
        <div className="w-8 h-1.5 bg-neutral-200 dark:bg-neutral-700 rounded-full overflow-hidden">
          <div 
            className={cn(
              'h-full transition-all duration-100',
              speaker.isActive ? 'bg-secondary-500' : 'bg-neutral-400'
            )}
            style={{ width: `${speaker.activityLevel * 100}%` }}
          />
        </div>
      )}
      
      {/* Speaking Duration */}
      {speaker.isActive && speaker.speakingDuration !== undefined && (
        <span className="text-xs text-neutral-500 dark:text-neutral-400 font-mono">
          {formatDuration(speaker.speakingDuration)}
        </span>
      )}
      
      {/* Confidence Score */}
      {layout !== 'compact' && speaker.confidenceScore < 0.7 && (
        <Icon 
          name="alert-triangle" 
          size="sm" 
          className="text-warning-500 flex-shrink-0"
          title={`Low confidence: ${Math.round(speaker.confidenceScore * 100)}%`}
        />
      )}
    </div>
  );

  if (speakers.length === 0) {
    return (
      <div 
        className={cn(
          'flex items-center gap-2 text-neutral-500 dark:text-neutral-400',
          className
        )}
        data-testid="no-speakers-detected"
      >
        <Icon name="users" size="sm" />
        <span className="text-sm">
          {isProcessing ? 'Detecting speakers...' : 'No speakers detected'}
        </span>
        {isProcessing && <Icon name="clock" size="sm" className="animate-pulse" />}
      </div>
    );
  }

  return (
    <div 
      className={cn(
        'speaker-activity-display',
        layout === 'horizontal' && 'flex items-center gap-3',
        layout === 'vertical' && 'flex flex-col gap-2',
        layout === 'compact' && 'flex flex-wrap items-center gap-1',
        className
      )}
      data-testid="speaker-activity-display"
    >
      {/* Processing Indicator */}
      {isProcessing && (
        <div className="flex items-center gap-1 text-xs text-warning-600 dark:text-warning-400">
          <Icon name="brain" size="sm" className="animate-pulse" />
          <span>Analyzing...</span>
        </div>
      )}
      
      {/* Overlapping Speech Warning */}
      {hasOverlappingSpeech && (
        <div className="flex items-center gap-1 text-xs text-warning-600 dark:text-warning-400">
          <Icon name="volume-x" size="sm" />
          <span>Multiple speakers</span>
        </div>
      )}
      
      {/* Active Speakers */}
      {activeSpeakers.length > 0 && (
        <div className={cn(
          layout === 'horizontal' && 'flex items-center gap-2',
          layout === 'vertical' && 'space-y-1',
          layout === 'compact' && 'flex flex-wrap gap-1'
        )}>
          {layout !== 'compact' && activeSpeakers.length > 0 && (
            <span className="text-xs font-medium text-neutral-700 dark:text-neutral-300">
              Active:
            </span>
          )}
          {activeSpeakers.map(speaker => (
            <SpeakerItem key={speaker.speakerId} speaker={speaker} />
          ))}
        </div>
      )}
      
      {/* Inactive Speakers (only in vertical layout) */}
      {layout === 'vertical' && inactiveSpeakers.length > 0 && (
        <div className="space-y-1">
          <span className="text-xs font-medium text-neutral-500 dark:text-neutral-400">
            Detected:
          </span>
          {inactiveSpeakers.map(speaker => (
            <SpeakerItem key={speaker.speakerId} speaker={speaker} />
          ))}
        </div>
      )}
      
      {/* Summary for compact/horizontal layouts */}
      {layout !== 'vertical' && speakers.length > activeSpeakers.length && (
        <span className="text-xs text-neutral-500 dark:text-neutral-400">
          +{speakers.length - activeSpeakers.length} more
        </span>
      )}
    </div>
  );
};

SpeakerActivityDisplay.displayName = 'SpeakerActivityDisplay';

export { SpeakerActivityDisplay };