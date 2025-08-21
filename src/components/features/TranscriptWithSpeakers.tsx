import React, { useState, useMemo } from 'react';
import { cn, formatDuration } from '@/lib/utils';
import { Button } from '@/components/ui/Button';
import { Icon } from '@/components/ui/Icon';
import { Badge } from '@/components/ui/Badge';
import { useSpeakerColors } from '@/hooks/useSpeakerColors';

export interface TranscriptSegmentWithSpeaker {
  startTime: number;
  endTime: number;
  speakerId: string;
  confidence: number;
  text: string;
}

export interface Speaker {
  id: string;
  displayName: string;
  color?: string;
}

export interface TranscriptWithSpeakersProps {
  segments: TranscriptSegmentWithSpeaker[];
  speakers: Map<string, Speaker>;
  showTimestamps?: boolean;
  showConfidence?: boolean;
  enableFiltering?: boolean;
  onExport?: (options: ExportOptions) => void;
  currentTime?: number;
  onSeek?: (time: number) => void;
  className?: string;
}

export interface ExportOptions {
  format: 'txt' | 'json' | 'srt';
  includeSpeakers: boolean;
  includeTimestamps: boolean;
  includeConfidence: boolean;
}

export const TranscriptWithSpeakers: React.FC<TranscriptWithSpeakersProps> = ({
  segments,
  speakers,
  showTimestamps = true,
  showConfidence = false,
  enableFiltering = false,
  onExport,
  currentTime = 0,
  onSeek,
  className = '',
}) => {
  const [filteredSpeakers, setFilteredSpeakers] = useState<Set<string>>(new Set());
  const [searchQuery, setSearchQuery] = useState('');

  const speakerIds = Array.from(speakers.keys());
  const { getSpeakerColor } = useSpeakerColors(speakerIds);

  const filteredSegments = useMemo(() => {
    return segments.filter(segment => {
      // Filter by speaker if any speakers are selected
      if (filteredSpeakers.size > 0 && !filteredSpeakers.has(segment.speakerId)) {
        return false;
      }

      // Filter by search query
      if (searchQuery && !segment.text.toLowerCase().includes(searchQuery.toLowerCase())) {
        return false;
      }

      return true;
    });
  }, [segments, filteredSpeakers, searchQuery]);

  const toggleSpeakerFilter = (speakerId: string) => {
    const newFiltered = new Set(filteredSpeakers);
    if (newFiltered.has(speakerId)) {
      newFiltered.delete(speakerId);
    } else {
      newFiltered.add(speakerId);
    }
    setFilteredSpeakers(newFiltered);
  };

  const clearAllFilters = () => {
    setFilteredSpeakers(new Set());
    setSearchQuery('');
  };

  const handleExport = () => {
    if (onExport) {
      onExport({
        format: 'txt',
        includeSpeakers: true,
        includeTimestamps: true,
        includeConfidence: false,
      });
    }
  };

  const handleSeek = (time: number) => {
    if (onSeek) {
      onSeek(time);
    }
  };

  const getSpeakerDisplayName = (speakerId: string): string => {
    return speakers.get(speakerId)?.displayName || speakerId;
  };

  const getSpeakerColorStyle = (speakerId: string): React.CSSProperties => {
    const speaker = speakers.get(speakerId);
    const colorConfig = getSpeakerColor(speakerId);
    const color = speaker?.color || colorConfig.color;
    
    return {
      borderLeft: `4px solid ${color}`,
    };
  };

  const formatTimestamp = (seconds: number): string => {
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = Math.floor(seconds % 60);
    return `${minutes.toString().padStart(2, '0')}:${remainingSeconds.toString().padStart(2, '0')}`;
  };

  const highlightSearchText = (text: string, query: string): React.ReactNode => {
    if (!query.trim()) return text;
    
    const parts = text.split(new RegExp(`(${query})`, 'gi'));
    return parts.map((part, index) => 
      part.toLowerCase() === query.toLowerCase() ? (
        <mark key={index} className="bg-warning-200 dark:bg-warning-800 rounded px-1">
          {part}
        </mark>
      ) : part
    );
  };

  return (
    <div className={cn('flex flex-col', className)}>
      {/* Controls */}
      <div className="flex flex-col gap-4 mb-6">
        {/* Search */}
        <div className="flex items-center gap-2">
          <div className="flex-1">
            <input
              type="text"
              placeholder="Search transcript..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full p-2 border border-neutral-300 dark:border-neutral-600 rounded-md bg-white dark:bg-neutral-800"
            />
          </div>
          
          {onExport && (
            <Button onClick={handleExport} variant="secondary" size="sm">
              Export with Speakers
            </Button>
          )}
        </div>

        {/* Speaker Filters */}
        {enableFiltering && (
          <div className="flex flex-wrap items-center gap-2">
            <span className="text-sm text-neutral-600 dark:text-neutral-400">Filter by speaker:</span>
            
            {Array.from(speakers.entries()).map(([speakerId, speaker]) => {
              const isFiltered = filteredSpeakers.has(speakerId);
              const colorConfig = getSpeakerColor(speakerId);
              const color = speaker.color || colorConfig.color;

              return (
                <button
                  key={speakerId}
                  data-testid={`speaker-filter-${speakerId}`}
                  onClick={() => toggleSpeakerFilter(speakerId)}
                  className={cn(
                    'px-3 py-1 rounded-full text-sm font-medium transition-all duration-200',
                    'border-2',
                    isFiltered
                      ? 'bg-white dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100'
                      : 'bg-transparent text-neutral-600 dark:text-neutral-400 hover:bg-neutral-50 dark:hover:bg-neutral-800'
                  )}
                  style={{
                    borderColor: color,
                    ...(isFiltered && { backgroundColor: color + '20' })
                  }}
                >
                  {speaker.displayName}
                </button>
              );
            })}
            
            {(filteredSpeakers.size > 0 || searchQuery) && (
              <Button
                onClick={clearAllFilters}
                variant="ghost"
                size="sm"
              >
                Clear Filters
              </Button>
            )}
          </div>
        )}
      </div>

      {/* Transcript Segments */}
      <div className="space-y-4">
        {filteredSegments.length === 0 ? (
          <div className="text-center py-8 text-neutral-500 dark:text-neutral-400">
            <Icon name="document-text" size="lg" className="mx-auto mb-2 opacity-50" />
            <p>No transcript segments match your filters</p>
          </div>
        ) : (
          filteredSegments.map((segment, index) => {
            const speaker = speakers.get(segment.speakerId);
            const isCurrentSegment = currentTime >= segment.startTime && currentTime <= segment.endTime;

            return (
              <div
                key={`${segment.speakerId}-${segment.startTime}`}
                data-testid={`transcript-segment-${segment.speakerId}-${index}`}
                data-speaker-id={segment.speakerId}
                className={cn(
                  'p-4 rounded-lg bg-white dark:bg-neutral-900 transition-all duration-200',
                  'border border-neutral-200 dark:border-neutral-700',
                  isCurrentSegment && 'ring-2 ring-primary-300 dark:ring-primary-700'
                )}
                style={getSpeakerColorStyle(segment.speakerId)}
              >
                {/* Header */}
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-3">
                    {/* Speaker Name */}
                    <span className="font-medium text-neutral-900 dark:text-neutral-100">
                      {getSpeakerDisplayName(segment.speakerId)}:
                    </span>
                    
                    {/* Timestamp */}
                    {showTimestamps && (
                      <button
                        onClick={() => handleSeek(segment.startTime)}
                        className="text-sm text-neutral-500 dark:text-neutral-400 hover:text-primary-600 dark:hover:text-primary-400 font-mono transition-colors"
                        title={`Seek to ${formatTimestamp(segment.startTime)}`}
                      >
                        {formatTimestamp(segment.startTime)}
                      </button>
                    )}
                  </div>

                  {/* Confidence Badge */}
                  {showConfidence && (
                    <Badge
                      variant={segment.confidence > 0.8 ? "secondary" : segment.confidence > 0.6 ? "warning" : "error"}
                      size="sm"
                    >
                      {Math.round(segment.confidence * 100)}%
                    </Badge>
                  )}
                </div>

                {/* Text Content */}
                <p className="text-neutral-900 dark:text-neutral-100 leading-relaxed">
                  {highlightSearchText(segment.text, searchQuery)}
                </p>
              </div>
            );
          })
        )}
      </div>

      {/* Summary */}
      {segments.length > 0 && (
        <div className="mt-6 p-4 bg-neutral-50 dark:bg-neutral-800 rounded-lg">
          <div className="flex items-center justify-between text-sm text-neutral-600 dark:text-neutral-400">
            <span>
              {filteredSegments.length} of {segments.length} segments
              {filteredSpeakers.size > 0 && ` • ${filteredSpeakers.size} speakers selected`}
            </span>
            
            <div className="flex items-center gap-4">
              <span>
                Total duration: {formatTimestamp(Math.max(...segments.map(s => s.endTime)))}
              </span>
              <span>
                {speakers.size} speakers detected
              </span>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};