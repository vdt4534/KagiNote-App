import React, { useState } from 'react';
import { cn } from '@/lib/utils';
import { Badge } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';
import { Icon } from '@/components/ui/Icon';
import { useSpeakerColors } from '@/hooks/useSpeakerColors';

export interface SpeakerSegment {
  startTime: number;
  endTime: number;
  speakerId: string;
  confidence: number;
  text: string;
}

export interface SpeakerProfile {
  id: string;
  displayName: string;
  voiceCharacteristics?: {
    pitch: number;
    formantF1: number;
    formantF2: number;
    speakingRate: number;
  };
  embeddings?: any[];
  totalSpeechTime: number;
  lastActive: number;
  confidence: number;
  color?: string;
}

export interface SpeakerDisplayProps {
  speakers: Map<string, SpeakerProfile>;
  segments: SpeakerSegment[];
  onSpeakerRename?: (speakerId: string, newName: string) => void;
  onSpeakerColorChange?: (speakerId: string, newColor: string) => void;
  className?: string;
}

export const SpeakerDisplay: React.FC<SpeakerDisplayProps> = ({
  speakers,
  segments,
  onSpeakerRename,
  onSpeakerColorChange,
  className = '',
}) => {
  const [selectedSpeaker, setSelectedSpeaker] = useState<string | null>(null);
  const [showRenameDialog, setShowRenameDialog] = useState(false);
  const [showColorPicker, setShowColorPicker] = useState(false);

  const speakerIds = Array.from(speakers.keys());
  const { getSpeakerColor } = useSpeakerColors(speakerIds);

  // Determine if mobile or desktop view based on window width
  const isMobile = typeof window !== 'undefined' && window.innerWidth < 768;

  const handleSpeakerNameClick = (speakerId: string) => {
    setSelectedSpeaker(speakerId);
    setShowRenameDialog(true);
  };

  const handleColorButtonClick = (speakerId: string) => {
    setSelectedSpeaker(speakerId);
    setShowColorPicker(true);
  };

  const formatSpeechTime = (seconds: number): string => {
    if (seconds < 60) {
      return `${seconds.toFixed(1)}s`;
    }
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = Math.floor(seconds % 60);
    return `${minutes}m ${remainingSeconds}s`;
  };

  const formatConfidence = (confidence: number): string => {
    return `${Math.round(confidence * 100)}%`;
  };

  return (
    <div 
      className={cn('flex flex-col gap-4', className)}
      data-testid={isMobile ? 'speaker-display-mobile' : 'speaker-display-desktop'}
    >
      {/* Header */}
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold text-neutral-900 dark:text-neutral-100">
          {speakers.size} Speakers Detected
        </h3>
      </div>

      {/* Speaker Cards */}
      <div className={cn(
        'grid gap-3',
        isMobile ? 'grid-cols-1' : 'grid-cols-2 lg:grid-cols-3'
      )}>
        {Array.from(speakers.entries()).map(([speakerId, speaker]) => {
          const colorConfig = getSpeakerColor(speakerId);
          const speechTime = formatSpeechTime(speaker.totalSpeechTime);
          const confidence = formatConfidence(speaker.confidence);

          return (
            <div
              key={speakerId}
              data-testid={`speaker-card-${speakerId}`}
              className="p-4 rounded-lg border-2 bg-white dark:bg-neutral-900 hover:shadow-sm transition-shadow"
              style={{ borderColor: speaker.color || colorConfig.color }}
            >
              {/* Speaker Name */}
              <div className="flex items-center justify-between mb-2">
                <button
                  data-testid={`speaker-name-button-${speakerId}`}
                  onClick={() => handleSpeakerNameClick(speakerId)}
                  className="font-medium text-neutral-900 dark:text-neutral-100 hover:text-primary-600 dark:hover:text-primary-400 transition-colors"
                >
                  {speaker.displayName}
                </button>
                
                <button
                  data-testid={`speaker-color-button-${speakerId}`}
                  onClick={() => handleColorButtonClick(speakerId)}
                  className="w-4 h-4 rounded-full border border-neutral-300 dark:border-neutral-600"
                  style={{ backgroundColor: speaker.color || colorConfig.color }}
                  aria-label={`Change color for ${speaker.displayName}`}
                />
              </div>

              {/* Statistics */}
              <div className="flex items-center gap-4 text-sm text-neutral-600 dark:text-neutral-400">
                <span>{speechTime}</span>
                <span>{confidence}</span>
              </div>
            </div>
          );
        })}
      </div>

      {/* Rename Dialog */}
      {showRenameDialog && selectedSpeaker && (
        <div 
          data-testid="speaker-rename-dialog"
          className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
        >
          <div className="bg-white dark:bg-neutral-900 p-6 rounded-lg shadow-xl max-w-md w-full mx-4">
            <h3 className="text-lg font-semibold mb-4">Rename Speaker</h3>
            
            <input
              type="text"
              defaultValue={speakers.get(selectedSpeaker)?.displayName || ''}
              className="w-full p-2 border border-neutral-300 dark:border-neutral-600 rounded-md mb-4"
              autoFocus
            />
            
            <div className="flex gap-2 justify-end">
              <Button
                variant="ghost"
                onClick={() => {
                  setShowRenameDialog(false);
                  setSelectedSpeaker(null);
                }}
              >
                Cancel
              </Button>
              <Button
                onClick={() => {
                  // Handle save logic here
                  setShowRenameDialog(false);
                  setSelectedSpeaker(null);
                }}
              >
                Save
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* Color Picker */}
      {showColorPicker && selectedSpeaker && (
        <div 
          data-testid="speaker-color-picker"
          className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
        >
          <div className="bg-white dark:bg-neutral-900 p-6 rounded-lg shadow-xl">
            <h3 className="text-lg font-semibold mb-4">Choose Color</h3>
            
            <div className="grid grid-cols-5 gap-2">
              <button
                data-testid="color-option-red"
                className="w-8 h-8 rounded-full border-2"
                style={{ backgroundColor: '#DC2626' }}
                onClick={() => {
                  onSpeakerColorChange?.(selectedSpeaker, '#DC2626');
                  setShowColorPicker(false);
                  setSelectedSpeaker(null);
                }}
              />
              {/* Add more color options as needed */}
            </div>
            
            <div className="flex gap-2 justify-end mt-4">
              <Button
                variant="ghost"
                onClick={() => {
                  setShowColorPicker(false);
                  setSelectedSpeaker(null);
                }}
              >
                Cancel
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};