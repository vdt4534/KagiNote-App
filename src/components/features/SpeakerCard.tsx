import React from 'react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/Button';
import { Icon } from '@/components/ui/Icon';
import { Badge } from '@/components/ui/Badge';

export interface SpeakerCardProps {
  speaker: {
    id: string;
    displayName: string;
    confidence: number;
    totalSpeechTime: number;
    color?: string;
    voiceCharacteristics?: {
      pitch: number;
      formantF1?: number;
      formantF2?: number;
      speakingRate?: number;
    };
  };
  onEdit?: (speakerId: string) => void;
  onColorChange?: (speakerId: string, color: string) => void;
  className?: string;
}

export const SpeakerCard: React.FC<SpeakerCardProps> = ({
  speaker,
  onEdit,
  onColorChange,
  className = '',
}) => {
  const formatSpeechTime = (seconds: number): string => {
    const totalMinutes = Math.floor(seconds / 60);
    const remainingSeconds = Math.floor(seconds % 60);
    
    if (totalMinutes === 0) {
      return `${remainingSeconds}s`;
    }
    
    return `${totalMinutes}m ${remainingSeconds}s`;
  };

  const formatConfidence = (confidence: number): string => {
    return `${Math.round(confidence * 100)}%`;
  };

  const handleEditClick = () => {
    if (onEdit) {
      onEdit(speaker.id);
    }
  };

  const handleColorClick = () => {
    if (onColorChange) {
      // For now, cycle through a few colors
      const colors = ['#3B82F6', '#10B981', '#F59E0B', '#EF4444', '#8B5CF6'];
      const currentIndex = colors.findIndex(c => c === speaker.color);
      const nextColor = colors[(currentIndex + 1) % colors.length];
      onColorChange(speaker.id, nextColor);
    }
  };

  return (
    <div
      data-testid={`speaker-card-${speaker.id}`}
      className={cn(
        'p-4 rounded-lg border-2 bg-white dark:bg-neutral-900 hover:shadow-sm transition-all duration-200',
        className
      )}
      style={{ borderColor: speaker.color || '#6B7280' }}
    >
      {/* Header with name and controls */}
      <div className="flex items-center justify-between mb-3">
        <h4 className="font-semibold text-neutral-900 dark:text-neutral-100">
          {speaker.displayName}
        </h4>
        
        <div className="flex items-center gap-2">
          {/* Color indicator button */}
          <button
            onClick={handleColorClick}
            className="w-4 h-4 rounded-full border border-neutral-300 dark:border-neutral-600 hover:scale-110 transition-transform"
            style={{ backgroundColor: speaker.color || '#6B7280' }}
            aria-label={`Change color for ${speaker.displayName}`}
          />
          
          {/* Edit button */}
          {onEdit && (
            <Button
              data-testid="speaker-edit-button"
              variant="ghost"
              size="sm"
              onClick={handleEditClick}
              className="opacity-75 hover:opacity-100"
            >
              <Icon name="pencil" size="sm" />
            </Button>
          )}
        </div>
      </div>

      {/* Statistics */}
      <div className="space-y-2">
        <div className="flex items-center justify-between text-sm">
          <span className="text-neutral-600 dark:text-neutral-400">Speech Time:</span>
          <span className="font-medium text-neutral-900 dark:text-neutral-100">
            {formatSpeechTime(speaker.totalSpeechTime)}
          </span>
        </div>
        
        <div className="flex items-center justify-between text-sm">
          <span className="text-neutral-600 dark:text-neutral-400">Confidence:</span>
          <Badge
            variant={speaker.confidence > 0.8 ? "secondary" : speaker.confidence > 0.6 ? "warning" : "error"}
            size="sm"
          >
            {formatConfidence(speaker.confidence)}
          </Badge>
        </div>

        {/* Voice characteristics if available */}
        {speaker.voiceCharacteristics && (
          <div className="pt-2 border-t border-neutral-200 dark:border-neutral-700">
            <div className="flex items-center justify-between text-xs text-neutral-500 dark:text-neutral-400">
              <span>Pitch:</span>
              <span>{Math.round(speaker.voiceCharacteristics.pitch)}Hz</span>
            </div>
            
            {speaker.voiceCharacteristics.speakingRate && (
              <div className="flex items-center justify-between text-xs text-neutral-500 dark:text-neutral-400">
                <span>Rate:</span>
                <span>{Math.round(speaker.voiceCharacteristics.speakingRate)} WPM</span>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};