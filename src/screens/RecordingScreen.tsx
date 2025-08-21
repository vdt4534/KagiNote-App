import React, { useState } from 'react';
import { cn } from '@/lib/utils';
import { Card, CardHeader, CardBody } from '@/components/ui/Card';
import { Icon } from '@/components/ui/Icon';
import { TranscriptView, TranscriptSegment } from '@/components/features/TranscriptView';
import { ControlBar } from '@/components/features/ControlBar';
import { MeetingDetailsPanel } from '@/components/features/MeetingDetailsPanel';

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
  const [showDetails, setShowDetails] = useState(false);

  return (
    <div className={cn('flex flex-col h-full bg-neutral-50 dark:bg-neutral-950', className)}>
      {/* Compact Control Bar - 64px height */}
      <ControlBar
        meetingTitle={meetingTitle}
        isRecording={isRecording}
        isPaused={isPaused}
        duration={duration}
        audioLevel={audioLevel}
        vadActivity={vadActivity}
        onStart={onStart}
        onPause={onPause}
        onResume={onResume}
        onStop={onStop}
        onToggleDetails={() => setShowDetails(!showDetails)}
        showDetailsButton={true}
      />

      {/* Main Transcript Area - Takes remaining height */}
      <div className="flex-1 min-h-0 p-2 sm:p-4">
        <Card className="h-full flex flex-col">
          <CardHeader className="flex-shrink-0 pb-2 sm:pb-3">
            <div className="flex items-center justify-between">
              <h2 className="text-lg sm:text-xl font-semibold text-neutral-900 dark:text-neutral-100">
                Live Transcript
              </h2>
              
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

      {/* Minimal Status Bar - 40px height */}
      <div className="h-10 px-2 sm:px-4 flex items-center justify-center border-t border-neutral-200 dark:border-neutral-700 bg-white dark:bg-neutral-900">
        <div className="flex items-center gap-3 sm:gap-6 text-[10px] sm:text-xs text-neutral-500 dark:text-neutral-400">
          <div className="flex items-center gap-1">
            <Icon name="shield-check" size="sm" className="text-secondary-600" />
            <span className="hidden sm:inline">100% Local Processing</span>
            <span className="sm:hidden">Local</span>
          </div>
          <span className="text-neutral-300 dark:text-neutral-700">•</span>
          <div className="flex items-center gap-1">
            <Icon name="eye-slash" size="sm" className="text-secondary-600" />
            <span className="hidden sm:inline">No Network Required</span>
            <span className="sm:hidden">Private</span>
          </div>
        </div>
      </div>

      {/* Collapsible Details Panel */}
      <MeetingDetailsPanel
        isOpen={showDetails}
        onClose={() => setShowDetails(false)}
        currentModel={currentModel}
        language={language}
        participants={participants}
        duration={duration}
        systemInfo={systemInfo}
        onOpenSettings={onOpenSettings}
      />
    </div>
  );
};