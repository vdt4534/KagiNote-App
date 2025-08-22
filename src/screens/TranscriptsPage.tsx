import React, { useState, useMemo } from 'react';
import { cn, formatDuration } from '@/lib/utils';
import { Card, CardHeader, CardBody } from '@/components/ui/card-compat';
import { Button } from '@/components/ui/button-compat';
import { Badge } from '@/components/ui/badge-compat';
import { Input } from '@/components/ui/input-new';
import { Icon } from '@/components/ui/Icon';
import { 
  Select, 
  SelectContent, 
  SelectItem, 
  SelectTrigger, 
  SelectValue 
} from '@/components/ui/select-new';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { MeetingFile } from './Dashboard';

export interface TranscriptsPageProps {
  meetings: MeetingFile[];
  onOpenMeeting: (id: string) => void;
  onDeleteMeeting: (id: string) => void;
  onExportMeeting: (id: string, format: string) => void;
  className?: string;
}

type ViewMode = 'grid' | 'list';
type SortBy = 'date' | 'title' | 'duration' | 'speakers';
type FilterLanguage = 'all' | 'english' | 'japanese' | 'other';
type FilterQuality = 'all' | 'Standard' | 'High Accuracy' | 'Turbo';

export const TranscriptsPage: React.FC<TranscriptsPageProps> = ({
  meetings,
  onOpenMeeting,
  onDeleteMeeting,
  onExportMeeting,
  className,
}) => {
  const [viewMode, setViewMode] = useState<ViewMode>('grid');
  const [searchQuery, setSearchQuery] = useState('');
  const [sortBy, setSortBy] = useState<SortBy>('date');
  const [filterLanguage, setFilterLanguage] = useState<FilterLanguage>('all');
  const [filterQuality, setFilterQuality] = useState<FilterQuality>('all');
  const [selectedMeetings, setSelectedMeetings] = useState<Set<string>>(new Set());

  // Filter and sort meetings
  const filteredMeetings = useMemo(() => {
    let filtered = meetings.filter(meeting => {
      // Search filter
      const matchesSearch = searchQuery === '' || 
        meeting.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
        meeting.preview.toLowerCase().includes(searchQuery.toLowerCase());

      // Language filter
      const matchesLanguage = filterLanguage === 'all' ||
        (filterLanguage === 'english' && meeting.language.toLowerCase() === 'english') ||
        (filterLanguage === 'japanese' && meeting.language.toLowerCase() === 'japanese') ||
        (filterLanguage === 'other' && !['english', 'japanese'].includes(meeting.language.toLowerCase()));

      // Quality filter
      const matchesQuality = filterQuality === 'all' || meeting.quality === filterQuality;

      return matchesSearch && matchesLanguage && matchesQuality;
    });

    // Sort
    filtered.sort((a, b) => {
      switch (sortBy) {
        case 'title':
          return a.title.localeCompare(b.title);
        case 'duration':
          return b.duration - a.duration;
        case 'speakers':
          return b.speakers - a.speakers;
        case 'date':
        default:
          return b.date.getTime() - a.date.getTime();
      }
    });

    return filtered;
  }, [meetings, searchQuery, sortBy, filterLanguage, filterQuality]);

  // Calculate statistics
  const stats = useMemo(() => {
    const totalDuration = meetings.reduce((sum, m) => sum + m.duration, 0);
    const totalSpeakers = new Set(meetings.flatMap(m => Array(m.speakers).fill(0))).size;
    const avgAccuracy = meetings.reduce((sum, m) => sum + m.accuracy, 0) / (meetings.length || 1);
    
    return {
      totalMeetings: meetings.length,
      totalHours: Math.floor(totalDuration / 3600),
      totalSpeakers,
      avgAccuracy: Math.round(avgAccuracy),
    };
  }, [meetings]);

  const handleSelectMeeting = (id: string) => {
    const newSelection = new Set(selectedMeetings);
    if (newSelection.has(id)) {
      newSelection.delete(id);
    } else {
      newSelection.add(id);
    }
    setSelectedMeetings(newSelection);
  };

  const handleSelectAll = () => {
    if (selectedMeetings.size === filteredMeetings.length) {
      setSelectedMeetings(new Set());
    } else {
      setSelectedMeetings(new Set(filteredMeetings.map(m => m.id)));
    }
  };

  const handleBatchExport = (format: string) => {
    selectedMeetings.forEach(id => onExportMeeting(id, format));
    setSelectedMeetings(new Set());
  };

  const handleBatchDelete = () => {
    if (confirm(`Delete ${selectedMeetings.size} transcript(s)? This cannot be undone.`)) {
      selectedMeetings.forEach(id => onDeleteMeeting(id));
      setSelectedMeetings(new Set());
    }
  };

  const renderTranscriptCard = (meeting: MeetingFile) => {
    const isSelected = selectedMeetings.has(meeting.id);
    
    return (
      <Card 
        key={meeting.id}
        className={cn(
          'hover:shadow-lg transition-all cursor-pointer',
          isSelected && 'ring-2 ring-blue-500',
          className
        )}
      >
        <CardHeader className="pb-3">
          <div className="flex items-start justify-between">
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2 mb-1">
                <input
                  type="checkbox"
                  checked={isSelected}
                  onChange={() => handleSelectMeeting(meeting.id)}
                  className="rounded border-gray-300"
                  onClick={(e) => e.stopPropagation()}
                />
                <h3 
                  className="font-semibold text-gray-900 dark:text-gray-100 truncate cursor-pointer hover:text-blue-600"
                  onClick={() => onOpenMeeting(meeting.id)}
                >
                  {meeting.title}
                </h3>
              </div>
              <div className="flex items-center gap-2 text-sm text-gray-500">
                <Icon name="calendar" size="sm" />
                <span>{meeting.date.toLocaleDateString()}</span>
                <span>•</span>
                <Icon name="clock" size="sm" />
                <span>{formatDuration(meeting.duration)}</span>
              </div>
            </div>
            <div className="flex gap-1">
              <Button
                variant="ghost"
                size="sm"
                onClick={(e) => {
                  e.stopPropagation();
                  onExportMeeting(meeting.id, 'pdf');
                }}
                title="Export"
              >
                <Icon name="download" size="sm" />
              </Button>
              <Button
                variant="ghost"
                size="sm"
                onClick={(e) => {
                  e.stopPropagation();
                  onDeleteMeeting(meeting.id);
                }}
                title="Delete"
              >
                <Icon name="trash" size="sm" />
              </Button>
            </div>
          </div>
        </CardHeader>
        <CardBody className="pt-0">
          <p className="text-sm text-gray-600 dark:text-gray-400 line-clamp-2 mb-3">
            {meeting.preview}
          </p>
          <div className="flex items-center justify-between">
            <div className="flex gap-2">
              <Badge variant="secondary" size="sm">
                <Icon name="users" size="sm" className="mr-1" />
                {meeting.speakers} speakers
              </Badge>
              <Badge variant="outline" size="sm">
                {meeting.language}
              </Badge>
              <Badge 
                variant={
                  meeting.quality === 'High Accuracy' ? 'success' :
                  meeting.quality === 'Turbo' ? 'warning' : 'default'
                }
                size="sm"
              >
                {meeting.quality}
              </Badge>
            </div>
            <div className="text-sm text-gray-500">
              {meeting.accuracy}% accuracy
            </div>
          </div>
        </CardBody>
      </Card>
    );
  };

  const renderTranscriptRow = (meeting: MeetingFile) => {
    const isSelected = selectedMeetings.has(meeting.id);
    
    return (
      <tr 
        key={meeting.id}
        className={cn(
          'hover:bg-gray-50 dark:hover:bg-gray-800',
          isSelected && 'bg-blue-50 dark:bg-blue-900/20'
        )}
      >
        <td className="px-4 py-3">
          <input
            type="checkbox"
            checked={isSelected}
            onChange={() => handleSelectMeeting(meeting.id)}
            className="rounded border-gray-300"
          />
        </td>
        <td className="px-4 py-3">
          <button
            onClick={() => onOpenMeeting(meeting.id)}
            className="text-left hover:text-blue-600"
          >
            <div className="font-medium">{meeting.title}</div>
            <div className="text-sm text-gray-500 line-clamp-1">{meeting.preview}</div>
          </button>
        </td>
        <td className="px-4 py-3 text-sm">
          {meeting.date.toLocaleDateString()}
        </td>
        <td className="px-4 py-3 text-sm">
          {formatDuration(meeting.duration)}
        </td>
        <td className="px-4 py-3">
          <Badge variant="secondary" size="sm">
            {meeting.speakers}
          </Badge>
        </td>
        <td className="px-4 py-3">
          <Badge variant="outline" size="sm">
            {meeting.language}
          </Badge>
        </td>
        <td className="px-4 py-3">
          <Badge 
            variant={
              meeting.quality === 'High Accuracy' ? 'success' :
              meeting.quality === 'Turbo' ? 'warning' : 'default'
            }
            size="sm"
          >
            {meeting.quality}
          </Badge>
        </td>
        <td className="px-4 py-3 text-sm">
          {meeting.accuracy}%
        </td>
        <td className="px-4 py-3">
          <div className="flex gap-1">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => onExportMeeting(meeting.id, 'pdf')}
            >
              <Icon name="download" size="sm" />
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => onDeleteMeeting(meeting.id)}
            >
              <Icon name="trash" size="sm" />
            </Button>
          </div>
        </td>
      </tr>
    );
  };

  return (
    <div className={cn('h-full flex flex-col', className)}>
      {/* Header with search and filters */}
      <div className="flex-shrink-0 pb-6">
        <div className="flex items-center justify-between mb-4">
          <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
            Transcripts
          </h1>
          <div className="flex items-center gap-2">
            <Button
              variant={viewMode === 'grid' ? 'default' : 'ghost'}
              size="sm"
              onClick={() => setViewMode('grid')}
            >
              <Icon name="grid" size="sm" />
            </Button>
            <Button
              variant={viewMode === 'list' ? 'default' : 'ghost'}
              size="sm"
              onClick={() => setViewMode('list')}
            >
              <Icon name="list" size="sm" />
            </Button>
          </div>
        </div>

        {/* Search and filters */}
        <div className="flex flex-col sm:flex-row gap-3">
          <div className="flex-1">
            <div className="relative">
              <Icon 
                name="search" 
                size="sm" 
                className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400"
              />
              <Input
                type="text"
                placeholder="Search transcripts..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="pl-10"
              />
            </div>
          </div>
          <Select value={sortBy} onValueChange={(value) => setSortBy(value as SortBy)}>
            <SelectTrigger className="w-[140px]">
              <SelectValue placeholder="Sort by" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="date">Date</SelectItem>
              <SelectItem value="title">Title</SelectItem>
              <SelectItem value="duration">Duration</SelectItem>
              <SelectItem value="speakers">Speakers</SelectItem>
            </SelectContent>
          </Select>
          <Select value={filterLanguage} onValueChange={(value) => setFilterLanguage(value as FilterLanguage)}>
            <SelectTrigger className="w-[140px]">
              <SelectValue placeholder="Language" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Languages</SelectItem>
              <SelectItem value="english">English</SelectItem>
              <SelectItem value="japanese">Japanese</SelectItem>
              <SelectItem value="other">Other</SelectItem>
            </SelectContent>
          </Select>
          <Select value={filterQuality} onValueChange={(value) => setFilterQuality(value as FilterQuality)}>
            <SelectTrigger className="w-[140px]">
              <SelectValue placeholder="Quality" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Quality</SelectItem>
              <SelectItem value="Standard">Standard</SelectItem>
              <SelectItem value="High Accuracy">High Accuracy</SelectItem>
              <SelectItem value="Turbo">Turbo</SelectItem>
            </SelectContent>
          </Select>
        </div>

        {/* Batch operations */}
        {selectedMeetings.size > 0 && (
          <div className="mt-3 p-3 bg-blue-50 dark:bg-blue-900/20 rounded-lg flex items-center justify-between">
            <span className="text-sm text-blue-700 dark:text-blue-300">
              {selectedMeetings.size} transcript(s) selected
            </span>
            <div className="flex gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => handleBatchExport('pdf')}
              >
                <Icon name="download" size="sm" className="mr-1" />
                Export
              </Button>
              <Button
                variant="destructive"
                size="sm"
                onClick={handleBatchDelete}
              >
                <Icon name="trash" size="sm" className="mr-1" />
                Delete
              </Button>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setSelectedMeetings(new Set())}
              >
                Clear
              </Button>
            </div>
          </div>
        )}
      </div>

      {/* Stats bar */}
      <div className="flex-shrink-0 grid grid-cols-2 sm:grid-cols-4 gap-4 mb-6">
        <Card>
          <CardBody className="p-4">
            <div className="text-2xl font-bold text-gray-900 dark:text-gray-100">
              {stats.totalMeetings}
            </div>
            <div className="text-sm text-gray-500">Total Transcripts</div>
          </CardBody>
        </Card>
        <Card>
          <CardBody className="p-4">
            <div className="text-2xl font-bold text-gray-900 dark:text-gray-100">
              {stats.totalHours}h
            </div>
            <div className="text-sm text-gray-500">Total Hours</div>
          </CardBody>
        </Card>
        <Card>
          <CardBody className="p-4">
            <div className="text-2xl font-bold text-gray-900 dark:text-gray-100">
              {stats.totalSpeakers}
            </div>
            <div className="text-sm text-gray-500">Unique Speakers</div>
          </CardBody>
        </Card>
        <Card>
          <CardBody className="p-4">
            <div className="text-2xl font-bold text-gray-900 dark:text-gray-100">
              {stats.avgAccuracy}%
            </div>
            <div className="text-sm text-gray-500">Avg Accuracy</div>
          </CardBody>
        </Card>
      </div>

      {/* Content area */}
      <div className="flex-1 overflow-auto">
        {filteredMeetings.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-center">
            <Icon name="document-text" size="lg" className="text-gray-300 mb-4" />
            <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">
              No transcripts found
            </h3>
            <p className="text-sm text-gray-500">
              {searchQuery ? 'Try adjusting your search or filters' : 'Start recording to create your first transcript'}
            </p>
          </div>
        ) : viewMode === 'grid' ? (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {filteredMeetings.map(renderTranscriptCard)}
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead className="bg-gray-50 dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
                <tr>
                  <th className="px-4 py-3 text-left">
                    <input
                      type="checkbox"
                      checked={selectedMeetings.size === filteredMeetings.length && filteredMeetings.length > 0}
                      onChange={handleSelectAll}
                      className="rounded border-gray-300"
                    />
                  </th>
                  <th className="px-4 py-3 text-left text-sm font-medium text-gray-700 dark:text-gray-300">
                    Title
                  </th>
                  <th className="px-4 py-3 text-left text-sm font-medium text-gray-700 dark:text-gray-300">
                    Date
                  </th>
                  <th className="px-4 py-3 text-left text-sm font-medium text-gray-700 dark:text-gray-300">
                    Duration
                  </th>
                  <th className="px-4 py-3 text-left text-sm font-medium text-gray-700 dark:text-gray-300">
                    Speakers
                  </th>
                  <th className="px-4 py-3 text-left text-sm font-medium text-gray-700 dark:text-gray-300">
                    Language
                  </th>
                  <th className="px-4 py-3 text-left text-sm font-medium text-gray-700 dark:text-gray-300">
                    Quality
                  </th>
                  <th className="px-4 py-3 text-left text-sm font-medium text-gray-700 dark:text-gray-300">
                    Accuracy
                  </th>
                  <th className="px-4 py-3 text-left text-sm font-medium text-gray-700 dark:text-gray-300">
                    Actions
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-200 dark:divide-gray-700">
                {filteredMeetings.map(renderTranscriptRow)}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
};