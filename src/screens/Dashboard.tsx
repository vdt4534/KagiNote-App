import React, { useState } from 'react';
import { cn, formatDate, formatDuration } from '@/lib/utils';
// Using compatibility layer for smooth migration to shadcn/ui
import { 
  Button, 
  Input, 
  Card, 
  CardBody,
  Icon, 
  Badge,
  Select, 
  SelectContent, 
  SelectItem, 
  SelectTrigger, 
  SelectValue 
} from '@/components/ui/compat';

export interface MeetingFile {
  id: string;
  title: string;
  date: Date;
  duration: number;
  speakers: number;
  accuracy: number;
  language: string;
  quality: 'Standard' | 'High Accuracy' | 'Turbo';
  preview: string;
}

export interface DashboardProps {
  meetings?: MeetingFile[];
  onNewMeeting?: () => void;
  onImportFile?: () => void;
  onOpenMeeting?: (meetingId: string) => void;
  onDeleteMeeting?: (meetingId: string) => void;
  onSearch?: (query: string) => void;
  isLoading?: boolean;
  className?: string;
}

const mockMeetings: MeetingFile[] = [
  {
    id: '1',
    title: 'Team Standup Meeting',
    date: new Date('2024-01-20'),
    duration: 932, // 15:32 in seconds
    speakers: 3,
    accuracy: 95,
    language: 'English',
    quality: 'High Accuracy',
    preview: 'Discussed Q4 roadmap planning and sprint tasks...',
  },
  {
    id: '2',
    title: 'Client Presentation',
    date: new Date('2024-01-19'),
    duration: 2712, // 45:12 in seconds
    speakers: 5,
    accuracy: 92,
    language: 'English',
    quality: 'Standard',
    preview: 'Product demo and feedback session with stakeholders...',
  },
  {
    id: '3',
    title: 'Interview - Frontend Developer',
    date: new Date('2024-01-18'),
    duration: 1695, // 28:15 in seconds
    speakers: 2,
    accuracy: 98,
    language: 'English',
    quality: 'High Accuracy',
    preview: 'Technical interview covering React, TypeScript, and system design...',
  },
];

export const Dashboard: React.FC<DashboardProps> = ({
  meetings = mockMeetings,
  onNewMeeting,
  onImportFile,
  onOpenMeeting,
  onDeleteMeeting,
  onSearch,
  isLoading = false,
  className = '',
}) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [sortBy, setSortBy] = useState<'date' | 'title' | 'duration'>('date');

  const handleSearch = (query: string) => {
    setSearchQuery(query);
    if (onSearch) {
      onSearch(query);
    }
  };

  const filteredMeetings = meetings.filter(meeting =>
    meeting.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
    meeting.preview.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const sortedMeetings = [...filteredMeetings].sort((a, b) => {
    switch (sortBy) {
      case 'date':
        return b.date.getTime() - a.date.getTime();
      case 'title':
        return a.title.localeCompare(b.title);
      case 'duration':
        return b.duration - a.duration;
      default:
        return 0;
    }
  });

  const getQualityColor = (quality: string) => {
    switch (quality) {
      case 'High Accuracy':
        return 'success';
      case 'Turbo':
        return 'warning';
      default:
        return 'secondary';
    }
  };

  const getAccuracyColor = (accuracy: number) => {
    if (accuracy >= 95) return 'success';
    if (accuracy >= 90) return 'warning';
    return 'destructive';
  };

  return (
    <div className={cn('flex flex-col h-full space-y-6', className)}>
      {/* Header Section */}
      <div className="flex flex-col gap-4">
        <div>
          <h1 className="text-3xl font-bold text-neutral-900 dark:text-neutral-100">
            Welcome to KagiNote
          </h1>
          <p className="text-neutral-600 dark:text-neutral-400 mt-1">
            100% Local Privacy • No Cloud Required
          </p>
        </div>

        {/* Search Bar */}
        <div className="w-full">
          <Input
            value={searchQuery}
            onChange={(e) => handleSearch(e.target.value)}
            placeholder="Search across all meetings..."
            className="w-full sm:max-w-xl"
            disabled={isLoading}
          />
        </div>

        {/* Quick Actions - Responsive layout */}
        <div className="flex flex-col sm:flex-row gap-3">
          <Button
            onClick={onNewMeeting}
            size="default"
            disabled={isLoading}
            className="w-full sm:w-auto flex items-center justify-center gap-2"
          >
            <Icon name="plus" size="base" />
            New Meeting
          </Button>
          
          <Button
            onClick={onImportFile}
            variant="outline"
            size="default"
            disabled={isLoading}
            className="w-full sm:w-auto flex items-center justify-center gap-2"
          >
            <Icon name="upload" size="base" />
            Import Audio
          </Button>
          
          <Button
            variant="outline"
            size="default"
            disabled={isLoading}
            className="w-full sm:w-auto flex items-center justify-center gap-2"
          >
            <Icon name="cog" size="base" />
            Settings
          </Button>
        </div>
      </div>

      {/* Meetings Section */}
      <div className="flex-1 flex flex-col min-h-0">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between mb-4 gap-3">
          <h2 className="text-xl font-semibold text-neutral-900 dark:text-neutral-100">
            Your Meetings
          </h2>
          
          <div className="flex items-center gap-2">
            <span className="text-sm text-neutral-500 dark:text-neutral-400">Sort:</span>
            <Select value={sortBy} onValueChange={(value) => setSortBy(value as 'date' | 'title' | 'duration')} disabled={isLoading}>
              <SelectTrigger className="w-32">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="date">Recent</SelectItem>
                <SelectItem value="title">Title</SelectItem>
                <SelectItem value="duration">Duration</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

        {/* Meetings List */}
        <div className="flex-1 overflow-y-auto space-y-4 scrollbar-thin">
          {isLoading ? (
            <div className="flex items-center justify-center h-32">
              <div className="animate-spin rounded-full h-8 w-8 border-2 border-primary-500 border-t-transparent" />
            </div>
          ) : sortedMeetings.length === 0 ? (
            <Card className="border-2 border-dashed border-neutral-300 dark:border-neutral-700 bg-neutral-50 dark:bg-neutral-800/50">
              <CardBody>
                <div className="text-center py-12">
                  <div className="w-20 h-20 bg-blue-100 dark:bg-blue-900/30 rounded-full flex items-center justify-center mx-auto mb-6">
                    <Icon name="document-text" size="xl" className="text-blue-600 dark:text-blue-400" />
                  </div>
                  <h3 className="text-xl font-semibold text-neutral-900 dark:text-neutral-100 mb-3">
                    {searchQuery ? 'No meetings found' : 'No meetings yet'}
                  </h3>
                  <p className="text-neutral-600 dark:text-neutral-400 mb-6 max-w-sm mx-auto">
                    {searchQuery 
                      ? `No meetings match "${searchQuery}". Try a different search term.`
                      : 'Start by creating your first meeting or importing an audio file.'
                    }
                  </p>
                  {!searchQuery && (
                    <Button onClick={onNewMeeting} size="lg" className="flex items-center gap-2">
                      <Icon name="plus" size="base" />
                      Create First Meeting
                    </Button>
                  )}
                </div>
              </CardBody>
            </Card>
          ) : (
            sortedMeetings.map((meeting) => (
              <Card 
                key={meeting.id}
                className="hover:shadow-md transition-shadow cursor-pointer group"
                onClick={() => onOpenMeeting?.(meeting.id)}
              >
                <CardBody>
                  <div className="flex items-start gap-4">
                    {/* Meeting Icon */}
                    <div className="flex-shrink-0 w-12 h-12 bg-primary-100 dark:bg-primary-900 rounded-lg flex items-center justify-center">
                      <Icon name="document-text" size="lg" className="text-primary-600 dark:text-primary-400" />
                    </div>

                    {/* Meeting Info */}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-start justify-between mb-2">
                        <h3 className="text-lg font-semibold text-neutral-900 dark:text-neutral-100 truncate">
                          {meeting.title}
                        </h3>
                        
                        <div className="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={(e) => {
                              e.stopPropagation();
                              onOpenMeeting?.(meeting.id);
                            }}
                            className="text-primary-600 hover:text-primary-700"
                          >
                            Open
                            <Icon name="chevron-right" size="sm" />
                          </Button>
                        </div>
                      </div>

                      <div className="flex items-center gap-4 text-sm text-neutral-500 dark:text-neutral-400 mb-2">
                        <span>{formatDate(meeting.date)}</span>
                        <span>•</span>
                        <span>{formatDuration(meeting.duration)}</span>
                        <span>•</span>
                        <span>{meeting.speakers} speakers</span>
                      </div>

                      <div className="flex items-center gap-2 mb-3">
                        <div className="flex-1 bg-neutral-200 dark:bg-neutral-700 rounded-full h-2">
                          <div 
                            className="bg-primary-500 h-2 rounded-full transition-all duration-300"
                            style={{ width: `${meeting.accuracy}%` }}
                          />
                        </div>
                        <Badge variant={getAccuracyColor(meeting.accuracy) as any}>
                          {meeting.accuracy}% accuracy
                        </Badge>
                      </div>

                      <p className="text-neutral-600 dark:text-neutral-400 text-sm mb-3 line-clamp-2">
                        {meeting.preview}
                      </p>

                      <div className="flex items-center gap-2">
                        <Badge variant={getQualityColor(meeting.quality) as any}>
                          {meeting.quality}
                        </Badge>
                        <Badge variant="outline">
                          {meeting.language}
                        </Badge>
                      </div>
                    </div>
                  </div>
                </CardBody>
              </Card>
            ))
          )}
        </div>
      </div>
    </div>
  );
};