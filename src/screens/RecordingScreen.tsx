import React, { useState } from 'react';
import { cn } from '@/lib/utils';
import { Card, CardHeader, CardBody } from '@/components/ui/card-compat';
import { Icon } from '@/components/ui/Icon';
import { TranscriptView, TranscriptSegment } from '@/components/features/TranscriptView';
import { ControlBar } from '@/components/features/ControlBar';
import { MeetingDetailsPanel } from '@/components/features/MeetingDetailsPanel';
import { DiarizationStatusIndicator, DiarizationStatus } from '@/components/features/DiarizationStatusIndicator';
import { SpeakerActivityDisplay, SpeakerActivity } from '@/components/features/SpeakerActivityDisplay';

export interface SpeakerInfo {
  id: string;
  displayName: string;
  color?: string;
}

export interface RecordingScreenProps {
  meetingTitle?: string;
  isRecording: boolean;
  isPaused: boolean;
  duration: number;
  audioLevel: number;
  vadActivity: boolean;
  transcriptSegments: TranscriptSegment[];
  speakers?: Map<string, SpeakerInfo>;
  currentSpeaker?: string;
  currentModel?: string;
  language?: string;
  participants?: string[];
  diarizationStatus?: DiarizationStatus;
  speakerActivities?: SpeakerActivity[];
  hasOverlappingSpeech?: boolean;
  onStart: () => void;
  onPause: () => void;
  onResume: () => void;
  onStop: () => void;
  onOpenSettings?: () => void;
  onEditSegment?: (segmentId: string, newText: string) => void;
  onSpeakerRename?: (speakerId: string, newName: string) => void;
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
  speakers = new Map(),
  currentSpeaker,
  currentModel = 'Standard',
  language = 'English',
  participants = [],
  diarizationStatus,
  speakerActivities = [],
  hasOverlappingSpeech = false,
  onStart,
  onPause,
  onResume,
  onStop,
  onOpenSettings,
  onEditSegment,
  onSpeakerRename,
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

      {/* Speaker Activity Panel (Mobile) */}
      {speakerActivities.length > 0 && (
        <div className="sm:hidden px-2 py-1 bg-white dark:bg-neutral-900 border-b border-neutral-200 dark:border-neutral-700">
          <SpeakerActivityDisplay
            speakers={speakerActivities}
            currentSpeaker={currentSpeaker}
            isProcessing={diarizationStatus?.serviceHealth === 'initializing'}
            hasOverlappingSpeech={hasOverlappingSpeech}
            layout="horizontal"
          />
        </div>
      )}
      
      {/* Main Transcript Area - Takes remaining height */}
      <div className="flex-1 min-h-0 p-2 sm:p-4">
        <Card className="h-full flex flex-col">
          <CardHeader className="flex-shrink-0 pb-2 sm:pb-3">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <h2 className="text-lg sm:text-xl font-semibold text-neutral-900 dark:text-neutral-100">
                  Live Transcript
                </h2>
                
                {/* Diarization Status Indicator */}
                {diarizationStatus && (
                  <DiarizationStatusIndicator 
                    status={diarizationStatus}
                    className="hidden sm:flex"
                  />
                )}
                
                {/* Speaker Count (fallback for mobile) */}
                {!diarizationStatus && speakers.size > 0 && (
                  <div className="flex items-center gap-1 px-2 py-1 rounded-full bg-neutral-100 dark:bg-neutral-800">
                    <Icon name="users" size="sm" className="text-neutral-600 dark:text-neutral-400" />
                    <span className="text-sm font-medium text-neutral-600 dark:text-neutral-400">
                      {speakers.size}
                    </span>
                  </div>
                )}
              </div>
              
              <div className="flex items-center gap-3">
                {/* Speaker Activity Display */}
                {speakerActivities.length > 0 && (
                  <SpeakerActivityDisplay
                    speakers={speakerActivities}
                    currentSpeaker={currentSpeaker}
                    isProcessing={diarizationStatus?.serviceHealth === 'initializing'}
                    hasOverlappingSpeech={hasOverlappingSpeech}
                    layout="compact"
                    className="hidden sm:flex"
                  />
                )}
                
                {/* Current Speaker Indicator (fallback) */}
                {(!speakerActivities.length || !diarizationStatus) && isRecording && currentSpeaker && speakers.get(currentSpeaker) && (
                  <div className="flex items-center gap-2">
                    <div 
                      className="w-3 h-3 rounded-full"
                      style={{ 
                        backgroundColor: speakers.get(currentSpeaker)?.color || '#6B7280'
                      }}
                    />
                    <span className="text-sm font-medium text-neutral-700 dark:text-neutral-300">
                      {speakers.get(currentSpeaker)?.displayName}
                    </span>
                  </div>
                )}
                
                {/* Recording Indicator */}
                {isRecording && (
                  <div className="flex items-center gap-2">
                    <div className="w-2 h-2 bg-secondary-500 rounded-full animate-pulse" />
                    <span className="text-sm text-neutral-500 dark:text-neutral-400">
                      Real-time
                    </span>
                  </div>
                )}
              </div>
            </div>
          </CardHeader>
          
          <CardBody className="flex-1 min-h-0 p-0">
            <TranscriptView
              segments={transcriptSegments}
              speakers={speakers}
              showTimestamps={true}
              showSpeakers={true}
              showConfidence={false}
              isLive={isRecording}
              onEditSegment={onEditSegment}
              onSpeakerRename={onSpeakerRename}
              className="h-full"
            />
          </CardBody>
        </Card>
      </div>

      {/* Minimal Status Bar - 40px height */}
      <div className="h-10 px-2 sm:px-4 flex items-center justify-between border-t border-neutral-200 dark:border-neutral-700 bg-white dark:bg-neutral-900">
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
        
        {/* Diarization Status in Status Bar */}
        {diarizationStatus && diarizationStatus.serviceHealth !== 'disabled' && (
          <div className="flex items-center gap-2 text-[10px] sm:text-xs">
            <div 
              className={cn(
                'w-1.5 h-1.5 rounded-full',
                diarizationStatus.serviceHealth === 'ready' ? 'bg-secondary-500' :
                diarizationStatus.serviceHealth === 'initializing' ? 'bg-warning-500 animate-pulse' :
                'bg-error-500'
              )}
            />
            <span className={cn(
              diarizationStatus.serviceHealth === 'ready' ? 'text-secondary-600 dark:text-secondary-400' :
              diarizationStatus.serviceHealth === 'initializing' ? 'text-warning-600 dark:text-warning-400' :
              'text-error-600 dark:text-error-400'
            )}>
              {diarizationStatus.serviceHealth === 'ready' && diarizationStatus.speakerCount !== undefined
                ? `${diarizationStatus.speakerCount} speakers`
                : diarizationStatus.serviceHealth === 'initializing'
                ? 'Detecting'
                : 'Speaker error'
              }
            </span>
          </div>
        )}
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