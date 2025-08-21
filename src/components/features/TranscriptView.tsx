import React, { useState, useRef, useEffect } from 'react';
import { cn, formatDuration } from '@/lib/utils';
import { Icon } from '@/components/ui/Icon';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Badge } from '@/components/ui/Badge';

export interface TranscriptSegment {
  id: string;
  startTime: number;
  endTime: number;
  speaker: string;
  speakerId?: string; // Support new speaker ID format
  text: string;
  confidence: number;
  isEditing?: boolean;
}

export interface SpeakerInfo {
  id: string;
  displayName: string;
  color?: string;
}

export interface TranscriptViewProps {
  segments: TranscriptSegment[];
  speakers?: Map<string, SpeakerInfo>;
  currentTime?: number;
  onSeek?: (time: number) => void;
  onEditSegment?: (segmentId: string, newText: string) => void;
  showTimestamps?: boolean;
  showSpeakers?: boolean;
  showConfidence?: boolean;
  isLive?: boolean;
  searchQuery?: string;
  onSearch?: (query: string) => void;
  onSpeakerRename?: (speakerId: string, newName: string) => void;
  className?: string;
}

export const TranscriptView: React.FC<TranscriptViewProps> = ({
  segments,
  speakers = new Map(),
  currentTime = 0,
  onSeek,
  onEditSegment,
  showTimestamps = true,
  showSpeakers = true,
  showConfidence = false,
  isLive = false,
  searchQuery = '',
  onSearch,
  onSpeakerRename,
  className = '',
}) => {
  const [editingSegmentId, setEditingSegmentId] = useState<string | null>(null);
  const [editText, setEditText] = useState('');
  const containerRef = useRef<HTMLDivElement>(null);
  const [searchInput, setSearchInput] = useState(searchQuery);

  // Auto-scroll to current segment in live mode
  useEffect(() => {
    if (isLive && containerRef.current) {
      const currentSegment = segments.find(
        seg => currentTime >= seg.startTime && currentTime <= seg.endTime
      );
      
      if (currentSegment) {
        const segmentElement = document.querySelector(`[data-segment-id="${currentSegment.id}"]`);
        if (segmentElement) {
          segmentElement.scrollIntoView({ behavior: 'smooth', block: 'center' });
        }
      }
    }
  }, [currentTime, segments, isLive]);

  const handleEditStart = (segment: TranscriptSegment) => {
    setEditingSegmentId(segment.id);
    setEditText(segment.text);
  };

  const handleEditSave = (segmentId: string) => {
    if (onEditSegment && editText.trim()) {
      onEditSegment(segmentId, editText.trim());
    }
    setEditingSegmentId(null);
    setEditText('');
  };

  const handleEditCancel = () => {
    setEditingSegmentId(null);
    setEditText('');
  };

  const handleSeek = (time: number) => {
    if (onSeek) {
      onSeek(time);
    }
  };

  const handleSearchSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (onSearch) {
      onSearch(searchInput);
    }
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

  const getSpeakerColor = (segment: TranscriptSegment): string => {
    const speakerId = segment.speakerId || segment.speaker;
    const speakerInfo = speakers.get(speakerId);
    
    if (speakerInfo?.color) {
      return speakerInfo.color;
    }
    
    // Fallback to hash-based color generation
    const hash = speakerId.split('').reduce((a, b) => {
      a = ((a << 5) - a) + b.charCodeAt(0);
      return a & a;
    }, 0);
    
    const colors = [
      '#3B82F6', // Blue
      '#10B981', // Green  
      '#8B5CF6', // Purple
      '#F97316', // Orange
      '#06B6D4', // Teal
    ];
    
    return colors[Math.abs(hash) % colors.length];
  };

  const getSpeakerDisplayName = (segment: TranscriptSegment): string => {
    const speakerId = segment.speakerId || segment.speaker;
    const speakerInfo = speakers.get(speakerId);
    return speakerInfo?.displayName || segment.speaker;
  };

  const isCurrentSegment = (segment: TranscriptSegment): boolean => {
    return currentTime >= segment.startTime && currentTime <= segment.endTime;
  };

  return (
    <div className={cn('flex flex-col h-full', className)}>
      {/* Search Bar */}
      {onSearch && (
        <div className="p-4 border-b border-neutral-200 dark:border-neutral-700">
          <form onSubmit={handleSearchSubmit} className="flex gap-2">
            <div className="flex-1">
              <Input
                value={searchInput}
                onChange={(e) => setSearchInput(e.target.value)}
                placeholder="Search in transcript..."
                className="w-full"
              />
            </div>
            <Button type="submit" variant="ghost" size="base">
              <Icon name="search" size="base" />
            </Button>
          </form>
        </div>
      )}

      {/* Transcript Content */}
      <div 
        ref={containerRef}
        className="flex-1 overflow-y-auto p-4 space-y-4 scrollbar-thin"
        role="log"
        aria-label="Transcript content"
        aria-live={isLive ? "polite" : "off"}
      >
        {segments.length === 0 ? (
          <div className="flex items-center justify-center h-full text-neutral-500 dark:text-neutral-400">
            <div className="text-center space-y-2">
              <Icon name="document-text" size="lg" className="mx-auto opacity-50" />
              <p>
                {isLive ? 'Waiting for transcription...' : 'No transcript available'}
              </p>
            </div>
          </div>
        ) : (
          segments.map((segment) => (
            <div
              key={segment.id}
              data-segment-id={segment.id}
              data-testid={`transcript-segment-${segment.speakerId || segment.speaker}`}
              data-speaker-id={segment.speakerId || segment.speaker}
              className={cn(
                'group p-4 rounded-lg border-l-4 border-r border-y transition-all duration-200',
                'max-h-[120px] overflow-hidden flex flex-col',
                isCurrentSegment(segment) 
                  ? 'border-r-primary-300 border-y-primary-300 bg-primary-50 dark:border-r-primary-700 dark:border-y-primary-700 dark:bg-primary-900/20'
                  : 'border-r-neutral-200 border-y-neutral-200 hover:border-r-neutral-300 hover:border-y-neutral-300 dark:border-r-neutral-700 dark:border-y-neutral-700 dark:hover:border-r-neutral-600 dark:hover:border-y-neutral-600',
                'hover:shadow-sm'
              )}
              style={{ 
                borderLeftColor: getSpeakerColor(segment)
              }}
            >
              {/* Segment Header */}
              <div className="flex items-center justify-between mb-2 flex-shrink-0">
                <div className="flex items-center gap-3">
                  {showSpeakers && (
                    <Badge 
                      variant="neutral" 
                      className="font-medium text-white"
                      style={{ 
                        backgroundColor: getSpeakerColor(segment),
                        borderColor: getSpeakerColor(segment)
                      }}
                    >
                      {getSpeakerDisplayName(segment)}
                    </Badge>
                  )}
                  
                  {showTimestamps && (
                    <button
                      onClick={() => handleSeek(segment.startTime)}
                      className="text-sm text-neutral-500 dark:text-neutral-400 hover:text-primary-600 dark:hover:text-primary-400 font-mono transition-colors"
                      aria-label={`Seek to ${formatDuration(segment.startTime)}`}
                    >
                      {formatDuration(segment.startTime)}
                    </button>
                  )}
                  
                  {showConfidence && (
                    <Badge 
                      variant={segment.confidence > 0.8 ? "secondary" : segment.confidence > 0.6 ? "warning" : "error"}
                      size="sm"
                    >
                      {Math.round(segment.confidence * 100)}%
                    </Badge>
                  )}
                </div>
                
                {/* Edit Button */}
                {onEditSegment && !isLive && (
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleEditStart(segment)}
                    className="opacity-0 group-hover:opacity-100 transition-opacity"
                    aria-label="Edit segment"
                  >
                    <Icon name="pencil" size="sm" />
                  </Button>
                )}
              </div>

              {/* Segment Content */}
              <div className="flex-1 overflow-y-auto min-h-0">
                {editingSegmentId === segment.id ? (
                  <div className="space-y-2">
                    <textarea
                      value={editText}
                      onChange={(e) => setEditText(e.target.value)}
                      className="w-full p-2 border border-neutral-300 rounded-md resize-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500 dark:bg-neutral-800 dark:border-neutral-600"
                      rows={3}
                      autoFocus
                    />
                    <div className="flex gap-2">
                      <Button
                        size="sm"
                        onClick={() => handleEditSave(segment.id)}
                        disabled={!editText.trim()}
                      >
                        Save
                      </Button>
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={handleEditCancel}
                      >
                        Cancel
                      </Button>
                    </div>
                  </div>
                ) : (
                  <p 
                    className={cn(
                      'text-neutral-900 dark:text-neutral-100 leading-relaxed',
                      isCurrentSegment(segment) && 'font-medium',
                      'pr-2' // Add padding for scrollbar
                    )}
                    style={{ opacity: segment.confidence }}
                  >
                    {highlightSearchText(segment.text, searchQuery)}
                  </p>
                )}
              </div>
            </div>
          ))
        )}
        
        {/* Live indicator */}
        {isLive && segments.length > 0 && (
          <div className="flex items-center justify-center py-4">
            <div className="flex items-center gap-2 text-sm text-neutral-500 dark:text-neutral-400">
              <div className="w-2 h-2 bg-secondary-500 rounded-full animate-pulse" />
              <span>Live transcription</span>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};