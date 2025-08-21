import React, { useState, useEffect } from 'react';
import { cn, formatDuration } from '@/lib/utils';
import { Card, CardHeader, CardBody } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Badge } from '@/components/ui/Badge';
import { Icon } from '@/components/ui/Icon';
import { AudioVisualizer } from '@/components/features/AudioVisualizer';
import { RecordingControls } from '@/components/features/RecordingControls';
import { TranscriptView, TranscriptSegment } from '@/components/features/TranscriptView';

export interface RecordingScreenProps {
  meetingTitle?: string;
  isRecording: boolean;
  isPaused: boolean;
  duration: number;
  audioLevel: number;
  vadActivity: boolean;
  transcriptSegments: TranscriptSegment[];
  currentModel?: string;
  language?: string;
  participants?: string[];
  onStart: () => void;
  onPause: () => void;
  onResume: () => void;
  onStop: () => void;
  onOpenSettings?: () => void;
  onEditSegment?: (segmentId: string, newText: string) => void;
  systemInfo?: {
    cpu: number;
    memory: number;
    rtf: number; // Real-time factor
  };
  className?: string;
}

export const RecordingScreen: React.FC<RecordingScreenProps> = ({
  meetingTitle = 'Live Recording',
  isRecording,
  isPaused,
  duration,
  audioLevel,
  vadActivity,
  transcriptSegments,
  currentModel = 'Standard',
  language = 'English',
  participants = [],
  onStart,
  onPause,
  onResume,
  onStop,
  onOpenSettings,
  onEditSegment,
  systemInfo = { cpu: 15, memory: 2.1, rtf: 0.8 },
  className = '',
}) => {
  const [showSystemInfo, setShowSystemInfo] = useState(false);

  // Auto-hide system info after 3 seconds
  useEffect(() => {
    if (showSystemInfo) {
      const timer = setTimeout(() => setShowSystemInfo(false), 3000);
      return () => clearTimeout(timer);
    }
  }, [showSystemInfo]);

  const getRecordingStatus = () => {
    if (!isRecording) return 'Ready to record';
    if (isPaused) return 'Recording paused';
    return 'Recording in progress';
  };

  const getStatusColor = (): "primary" | "secondary" | "warning" | "error" | "neutral" => {
    if (!isRecording) return 'neutral';
    if (isPaused) return 'warning';
    return 'secondary';
  };

  const formatSystemMetric = (value: number, unit: string) => {
    return `${value}${unit}`;
  };

  return (
    <div className={cn('flex flex-col h-full space-y-6', className)}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-neutral-900 dark:text-neutral-100">
            {meetingTitle}
          </h1>
          <div className="flex items-center gap-3 mt-1">
            <Badge variant={getStatusColor()} className="flex items-center gap-1">
              {isRecording && !isPaused && (
                <div className="w-2 h-2 bg-current rounded-full animate-pulse" />
              )}
              {getRecordingStatus()}
            </Badge>
            <span className="text-neutral-500 dark:text-neutral-400 text-sm">
              {formatDuration(duration)}
            </span>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setShowSystemInfo(!showSystemInfo)}
            className="flex items-center gap-1"
          >
            <Icon name="information-circle" size="sm" />
            System Info
          </Button>
          
          {onOpenSettings && (
            <Button
              variant="ghost"
              size="sm"
              onClick={onOpenSettings}
              className="flex items-center gap-1"
            >
              <Icon name="cog" size="sm" />
              Settings
            </Button>
          )}
        </div>
      </div>

      {/* System Info Panel (collapsible) */}
      {showSystemInfo && (
        <Card className="animate-slide-in">
          <CardBody className="p-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-6">
                <div className="flex items-center gap-2">
                  <Icon name="shield-check" size="sm" className="text-secondary-600" />
                  <span className="text-sm font-medium text-neutral-700 dark:text-neutral-300">
                    Processing locally
                  </span>
                </div>
                
                <div className="flex items-center gap-4 text-sm text-neutral-600 dark:text-neutral-400">
                  <span>CPU: {formatSystemMetric(systemInfo.cpu, '%')}</span>
                  <span>•</span>
                  <span>RAM: {formatSystemMetric(systemInfo.memory, 'GB')}</span>
                  <span>•</span>
                  <span>RTF: {formatSystemMetric(systemInfo.rtf, 'x')}</span>
                </div>
              </div>
              
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setShowSystemInfo(false)}
              >
                <Icon name="x" size="sm" />
              </Button>
            </div>
          </CardBody>
        </Card>
      )}

      {/* Main Content Grid */}
      <div className="flex-1 grid grid-cols-1 lg:grid-cols-2 gap-6 min-h-0">
        {/* Left Column - Audio Controls */}
        <div className="flex flex-col space-y-6">
          {/* Audio Visualization */}
          <Card>
            <CardHeader>
              <h3 className="text-lg font-semibold text-neutral-900 dark:text-neutral-100">
                Audio Levels
              </h3>
            </CardHeader>
            <CardBody>
              <AudioVisualizer
                audioLevel={audioLevel}
                isRecording={isRecording}
                vadActivity={vadActivity}
                showWaveform={false}
                height={120}
                className="mb-4"
              />
              
              <div className="flex items-center justify-between text-sm">
                <div className="flex items-center gap-2">
                  <div className={cn(
                    'w-3 h-3 rounded-full',
                    vadActivity ? 'bg-secondary-500 animate-pulse' : 'bg-neutral-400'
                  )} />
                  <span className="text-neutral-600 dark:text-neutral-400">
                    {vadActivity ? 'Voice Activity Detected' : 'No Voice Activity'}
                  </span>
                </div>
                
                <span className="text-neutral-500 dark:text-neutral-400 font-mono">
                  Level: {Math.round(audioLevel * 100)}%
                </span>
              </div>
            </CardBody>
          </Card>

          {/* Recording Controls */}
          <RecordingControls
            isRecording={isRecording}
            isPaused={isPaused}
            duration={duration}
            onStart={onStart}
            onPause={onPause}
            onResume={onResume}
            onStop={onStop}
          />

          {/* Meeting Info */}
          <Card>
            <CardHeader>
              <h3 className="text-lg font-semibold text-neutral-900 dark:text-neutral-100">
                Meeting Details
              </h3>
            </CardHeader>
            <CardBody className="space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-neutral-600 dark:text-neutral-400">Model:</span>
                <Badge variant="primary" size="sm">{currentModel}</Badge>
              </div>
              
              <div className="flex items-center justify-between">
                <span className="text-neutral-600 dark:text-neutral-400">Language:</span>
                <Badge variant="neutral" size="sm">{language}</Badge>
              </div>
              
              <div className="flex items-center justify-between">
                <span className="text-neutral-600 dark:text-neutral-400">Speakers:</span>
                <span className="text-neutral-900 dark:text-neutral-100 text-sm">
                  {participants.length > 0 ? participants.length : 'Auto-detecting'}
                </span>
              </div>
              
              <div className="flex items-center justify-between">
                <span className="text-neutral-600 dark:text-neutral-400">Duration:</span>
                <span className="text-neutral-900 dark:text-neutral-100 text-sm font-mono">
                  {formatDuration(duration)}
                </span>
              </div>
            </CardBody>
          </Card>
        </div>

        {/* Right Column - Live Transcript */}
        <div className="flex flex-col min-h-0">
          <Card className="flex-1 min-h-0">
            <CardHeader>
              <div className="flex items-center justify-between">
                <h3 className="text-lg font-semibold text-neutral-900 dark:text-neutral-100">
                  Live Transcript
                </h3>
                
                {isRecording && (
                  <div className="flex items-center gap-2">
                    <div className="w-2 h-2 bg-secondary-500 rounded-full animate-pulse" />
                    <span className="text-sm text-neutral-500 dark:text-neutral-400">
                      Real-time
                    </span>
                  </div>
                )}
              </div>
            </CardHeader>
            
            <CardBody className="flex-1 min-h-0 p-0">
              <TranscriptView
                segments={transcriptSegments}
                showTimestamps={true}
                showSpeakers={true}
                showConfidence={false}
                isLive={isRecording}
                onEditSegment={onEditSegment}
                className="h-full"
              />
            </CardBody>
          </Card>
        </div>
      </div>

      {/* Privacy Footer */}
      <div className="flex items-center justify-center py-4 border-t border-neutral-200 dark:border-neutral-700">
        <div className="flex items-center gap-4 text-sm text-neutral-500 dark:text-neutral-400">
          <div className="flex items-center gap-1">
            <Icon name="shield-check" size="sm" className="text-secondary-600" />
            <span>100% Local Processing</span>
          </div>
          <span>•</span>
          <div className="flex items-center gap-1">
            <Icon name="eye-slash" size="sm" className="text-secondary-600" />
            <span>No Network Required</span>
          </div>
          <span>•</span>
          <div className="flex items-center gap-1">
            <Icon name="lock-closed" size="sm" className="text-secondary-600" />
            <span>Privacy Protected</span>
          </div>
        </div>
      </div>
    </div>
  );
};