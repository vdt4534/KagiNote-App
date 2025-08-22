/**
 * DiarizationAwareTranscriptionController Component
 * 
 * Enhanced TranscriptionController with full diarization event handling,
 * toast notifications, and user-friendly error reporting.
 */

import React from 'react';
import { 
  TranscriptionController, 
  type TranscriptionControllerProps,
  type SpeakerDetectionEvent,
  type SpeakerUpdateEvent,
  type DiarizationWarningEvent,
  type DiarizationErrorEvent
} from '../TranscriptionController';
import { useToast, createDiarizationToasts } from '@/components/ui/ToastContainer';

interface DiarizationAwareTranscriptionControllerProps extends TranscriptionControllerProps {
  showToastNotifications?: boolean;
  onDiarizationStatusChange?: (status: 'ready' | 'initializing' | 'error' | 'disabled') => void;
}

export const DiarizationAwareTranscriptionController: React.FC<DiarizationAwareTranscriptionControllerProps> = ({
  showToastNotifications = true,
  onDiarizationStatusChange,
  ...props
}) => {
  const toastContext = useToast();
  const diarizationToasts = createDiarizationToasts(() => toastContext);

  const handleSpeakerDetected = (event: SpeakerDetectionEvent) => {
    console.log('Speaker detected:', event);
    
    if (showToastNotifications) {
      diarizationToasts.showSpeakerDetected(
        event.displayName,
        event.confidence
      );
    }
    
    onDiarizationStatusChange?.('ready');
    props.onSpeakerDetected?.(event);
  };

  const handleSpeakerUpdate = (event: SpeakerUpdateEvent) => {
    console.log('Speaker update:', event);
    props.onSpeakerUpdate?.(event);
  };

  const handleDiarizationWarning = (event: DiarizationWarningEvent) => {
    console.warn('Diarization warning:', event);
    
    if (showToastNotifications) {
      diarizationToasts.showDiarizationWarning(
        event.message,
        event.recoveryHint
      );
    }
    
    props.onDiarizationWarning?.(event);
  };

  const handleDiarizationError = (event: DiarizationErrorEvent) => {
    console.error('Diarization error:', event);
    
    if (showToastNotifications) {
      const recoveryOptions = event.severity === 'critical' 
        ? undefined 
        : ['Retry initialization', 'Disable speaker detection'];
      
      diarizationToasts.showDiarizationError(
        event.message,
        recoveryOptions
      );
    }
    
    onDiarizationStatusChange?.(event.severity === 'critical' ? 'error' : 'ready');
    props.onDiarizationError?.(event);
  };

  const handleSessionStart = (sessionId: string) => {
    if (props.initialConfig?.enableSpeakerDiarization && showToastNotifications) {
      diarizationToasts.showDiarizationInfo(
        'Initializing speaker detection...'
      );
      onDiarizationStatusChange?.('initializing');
    }
    
    props.onSessionStart(sessionId);
  };

  return (
    <TranscriptionController
      {...props}
      onSessionStart={handleSessionStart}
      onSpeakerDetected={handleSpeakerDetected}
      onSpeakerUpdate={handleSpeakerUpdate}
      onDiarizationWarning={handleDiarizationWarning}
      onDiarizationError={handleDiarizationError}
    />
  );
};

DiarizationAwareTranscriptionController.displayName = 'DiarizationAwareTranscriptionController';

export default DiarizationAwareTranscriptionController;