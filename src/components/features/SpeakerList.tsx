import React, { useState, useMemo } from 'react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/Button';
import { Icon } from '@/components/ui/Icon';
import { Badge } from '@/components/ui/Badge';

export interface Speaker {
  id: string;
  displayName: string;
  confidence?: number;
  totalSpeechTime: number;
  color?: string;
  lastActive?: number;
}

export interface SpeakerListProps {
  speakers: Map<string, Speaker>;
  sortBy?: 'name' | 'speechTime' | 'confidence' | 'lastActive';
  sortOrder?: 'asc' | 'desc';
  onSortChange?: (sortBy: string, sortOrder: 'asc' | 'desc') => void;
  onSpeakerClick?: (speakerId: string) => void;
  onSpeakerEdit?: (speakerId: string) => void;
  className?: string;
}

type SortOption = 'name' | 'speechTime' | 'confidence' | 'lastActive';

export const SpeakerList: React.FC<SpeakerListProps> = ({
  speakers,
  sortBy = 'speechTime',
  sortOrder = 'desc',
  onSortChange,
  onSpeakerClick,
  onSpeakerEdit,
  className = '',
}) => {
  const [currentSortBy, setCurrentSortBy] = useState<SortOption>(sortBy);
  const [currentSortOrder, setCurrentSortOrder] = useState<'asc' | 'desc'>(sortOrder);

  const sortedSpeakers = useMemo(() => {
    const speakerArray = Array.from(speakers.entries()).map(([id, speaker]) => ({
      ...speaker,
      id,
    }));

    return speakerArray.sort((a, b) => {
      let comparison = 0;

      switch (currentSortBy) {
        case 'name':
          comparison = a.displayName.localeCompare(b.displayName);
          break;
        case 'speechTime':
          comparison = a.totalSpeechTime - b.totalSpeechTime;
          break;
        case 'confidence':
          comparison = (a.confidence || 0) - (b.confidence || 0);
          break;
        case 'lastActive':
          comparison = (a.lastActive || 0) - (b.lastActive || 0);
          break;
        default:
          comparison = 0;
      }

      return currentSortOrder === 'asc' ? comparison : -comparison;
    });
  }, [speakers, currentSortBy, currentSortOrder]);

  const handleSortClick = (newSortBy: SortOption) => {
    let newSortOrder: 'asc' | 'desc' = 'desc';

    if (currentSortBy === newSortBy) {
      // Toggle order if same column
      newSortOrder = currentSortOrder === 'asc' ? 'desc' : 'asc';
    } else {
      // Default order for different columns
      newSortOrder = newSortBy === 'name' ? 'asc' : 'desc';
    }

    setCurrentSortBy(newSortBy);
    setCurrentSortOrder(newSortOrder);
    
    if (onSortChange) {
      onSortChange(newSortBy, newSortOrder);
    }
  };

  const formatSpeechTime = (seconds: number): string => {
    const totalMinutes = Math.floor(seconds / 60);
    const remainingSeconds = Math.floor(seconds % 60);
    
    if (totalMinutes === 0) {
      return `${remainingSeconds}s`;
    }
    
    return `${totalMinutes}m ${remainingSeconds}s`;
  };

  const formatConfidence = (confidence?: number): string => {
    if (confidence === undefined) return 'N/A';
    return `${Math.round(confidence * 100)}%`;
  };

  return (
    <div className={cn('flex flex-col', className)}>
      {/* Sort Controls */}
      <div className="flex items-center gap-2 mb-4 flex-wrap">
        <span className="text-sm text-neutral-600 dark:text-neutral-400">Sort by:</span>
        
        <Button
          variant={currentSortBy === 'speechTime' ? 'primary' : 'ghost'}
          size="sm"
          onClick={() => handleSortClick('speechTime')}
        >
          Speech Time
          {currentSortBy === 'speechTime' && (
            <Icon 
              name={currentSortOrder === 'asc' ? 'arrow-up' : 'arrow-down'} 
              size="sm" 
              className="ml-1" 
            />
          )}
        </Button>
        
        <Button
          variant={currentSortBy === 'name' ? 'primary' : 'ghost'}
          size="sm"
          onClick={() => handleSortClick('name')}
        >
          Sort by Name
          {currentSortBy === 'name' && (
            <Icon 
              name={currentSortOrder === 'asc' ? 'arrow-up' : 'arrow-down'} 
              size="sm" 
              className="ml-1" 
            />
          )}
        </Button>
        
        {sortedSpeakers.some(s => s.confidence !== undefined) && (
          <Button
            variant={currentSortBy === 'confidence' ? 'primary' : 'ghost'}
            size="sm"
            onClick={() => handleSortClick('confidence')}
          >
            Confidence
            {currentSortBy === 'confidence' && (
              <Icon 
                name={currentSortOrder === 'asc' ? 'arrow-up' : 'arrow-down'} 
                size="sm" 
                className="ml-1" 
              />
            )}
          </Button>
        )}
      </div>

      {/* Speaker List */}
      <div className="space-y-2">
        {sortedSpeakers.map((speaker, index) => (
          <div
            key={speaker.id}
            data-testid={`speaker-list-item-${index}`}
            className={cn(
              'flex items-center justify-between p-3 rounded-lg border border-neutral-200 dark:border-neutral-700',
              'hover:bg-neutral-50 dark:hover:bg-neutral-800 transition-colors',
              onSpeakerClick && 'cursor-pointer'
            )}
            onClick={() => onSpeakerClick?.(speaker.id)}
          >
            {/* Speaker Info */}
            <div className="flex items-center gap-3">
              {/* Color indicator */}
              <div
                className="w-3 h-3 rounded-full border border-neutral-300 dark:border-neutral-600"
                style={{ backgroundColor: speaker.color || '#6B7280' }}
              />
              
              {/* Name and stats */}
              <div className="flex flex-col">
                <span className="font-medium text-neutral-900 dark:text-neutral-100">
                  {speaker.displayName}
                </span>
                
                <div className="flex items-center gap-3 text-sm text-neutral-600 dark:text-neutral-400">
                  <span>{formatSpeechTime(speaker.totalSpeechTime)}</span>
                  
                  {speaker.confidence !== undefined && (
                    <>
                      <span>•</span>
                      <Badge
                        variant={speaker.confidence > 0.8 ? "secondary" : speaker.confidence > 0.6 ? "warning" : "error"}
                        size="sm"
                      >
                        {formatConfidence(speaker.confidence)}
                      </Badge>
                    </>
                  )}
                </div>
              </div>
            </div>

            {/* Actions */}
            {onSpeakerEdit && (
              <Button
                variant="ghost"
                size="sm"
                onClick={(e) => {
                  e.stopPropagation();
                  onSpeakerEdit(speaker.id);
                }}
                className="opacity-75 hover:opacity-100"
              >
                <Icon name="pencil" size="sm" />
              </Button>
            )}
          </div>
        ))}
      </div>

      {/* Empty state */}
      {sortedSpeakers.length === 0 && (
        <div className="text-center py-8 text-neutral-500 dark:text-neutral-400">
          <Icon name="users" size="lg" className="mx-auto mb-2 opacity-50" />
          <p>No speakers detected</p>
        </div>
      )}
    </div>
  );
};