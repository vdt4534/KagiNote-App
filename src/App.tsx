import { useState, useEffect, useRef } from "react";
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { AppLayout } from "./components/layout/AppLayout";
import { Dashboard } from "./screens/Dashboard";
import { NewMeetingModal, MeetingConfig } from "./screens/NewMeetingModal";
import { RecordingScreen } from "./screens/RecordingScreen";
import { TranscriptSegment } from "./components/features/TranscriptView";
import { TranscriptionController, FinalTranscriptionResult, TranscriptionError, TranscriptionUpdateEvent, TranscriptionConfig, TranscriptionControllerRef } from "./components/features/TranscriptionController";
import { MeetingFile } from "./screens/Dashboard";
import './styles/globals.css';

type AppScreen = 'dashboard' | 'recording' | 'meeting-review';

interface AppState {
  isAppReady: boolean;
  currentScreen: AppScreen;
  audioLevel: number;
  isRecording: boolean;
  isPaused: boolean;
  vadActivity: boolean;
  recordingDuration: number;
  currentMeeting: MeetingConfig | null;
  transcriptSegments: TranscriptSegment[];
  sessionResults: FinalTranscriptionResult[];
  errors: TranscriptionError[];
  showNewMeetingModal: boolean;
  meetings: MeetingFile[];
}

function App() {
  const transcriptionControllerRef = useRef<TranscriptionControllerRef>(null);

  const [appState, setAppState] = useState<AppState>({
    isAppReady: false,
    currentScreen: 'dashboard',
    audioLevel: 0,
    isRecording: false,
    isPaused: false,
    vadActivity: false,
    recordingDuration: 0,
    currentMeeting: null,
    transcriptSegments: [],
    sessionResults: [],
    errors: [],
    showNewMeetingModal: false,
    meetings: [],
  });

  // Timer for recording duration
  useEffect(() => {
    let interval: NodeJS.Timeout;
    
    if (appState.isRecording && !appState.isPaused) {
      interval = setInterval(() => {
        setAppState(prev => ({ 
          ...prev, 
          recordingDuration: prev.recordingDuration + 1 
        }));
      }, 1000);
    }
    
    return () => {
      if (interval) clearInterval(interval);
    };
  }, [appState.isRecording, appState.isPaused]);

  // Initialize app and load existing meetings
  useEffect(() => {
    const initializeApp = async () => {
      try {
        // Load existing meetings from localStorage
        const loadMeetings = () => {
          try {
            const imported = JSON.parse(localStorage.getItem('imported-meetings') || '[]');
            const recorded = JSON.parse(localStorage.getItem('recorded-meetings') || '[]');
            return [...imported, ...recorded];
          } catch (error) {
            console.error('Failed to load meetings from localStorage:', error);
            return [];
          }
        };

        const meetings = loadMeetings();
        
        // Perform any necessary initialization
        await new Promise(resolve => setTimeout(resolve, 500));
        setAppState(prev => ({ ...prev, isAppReady: true, meetings }));
      } catch (error) {
        console.error('Failed to initialize app:', error);
        setAppState(prev => ({ ...prev, isAppReady: true }));
      }
    };

    initializeApp();
  }, []);

  // Listen for real audio level updates from backend
  useEffect(() => {
    if (!appState.isRecording) return;

    let unlistenAudioLevel: UnlistenFn | null = null;

    const setupAudioLevelListener = async () => {
      unlistenAudioLevel = await listen<{
        level: number;
        vadActivity: boolean;
        sessionId: string;
        timestamp: number;
      }>('audio-level', (event) => {
        const { level, vadActivity } = event.payload;
        setAppState(prev => ({ 
          ...prev, 
          audioLevel: level,
          vadActivity
        }));
      });
    };

    setupAudioLevelListener();

    return () => {
      if (unlistenAudioLevel) {
        unlistenAudioLevel();
      }
    };
  }, [appState.isRecording]);

  // Meeting Management Functions
  const saveMeetingToStorage = (meeting: MeetingFile, type: 'imported' | 'recorded') => {
    try {
      const storageKey = `${type}-meetings`;
      const existing = JSON.parse(localStorage.getItem(storageKey) || '[]');
      existing.push(meeting);
      localStorage.setItem(storageKey, JSON.stringify(existing));
      
      // Update app state with new meeting
      setAppState(prev => ({
        ...prev,
        meetings: [...prev.meetings, meeting]
      }));
    } catch (error) {
      console.error('Failed to save meeting to localStorage:', error);
    }
  };

  const deleteMeetingFromStorage = (meetingId: string) => {
    try {
      // Remove from both storage types
      ['imported-meetings', 'recorded-meetings'].forEach(storageKey => {
        const existing = JSON.parse(localStorage.getItem(storageKey) || '[]');
        const filtered = existing.filter((m: any) => m.id !== meetingId);
        localStorage.setItem(storageKey, JSON.stringify(filtered));
      });
      
      // Update app state
      setAppState(prev => ({
        ...prev,
        meetings: prev.meetings.filter(m => m.id !== meetingId)
      }));
    } catch (error) {
      console.error('Failed to delete meeting from localStorage:', error);
    }
  };

  const convertSessionToMeeting = (sessionResult: FinalTranscriptionResult, config: MeetingConfig, duration: number): MeetingFile => {
    const totalText = sessionResult.segments?.map((s: any) => s.text).join(' ') || '';
    const averageConfidence = sessionResult.segments?.reduce((acc: number, s: any) => acc + (s.confidence || 0), 0) / (sessionResult.segments?.length || 1) || 0;
    
    return {
      id: `recording-${Date.now()}`,
      title: config.title,
      date: new Date(),
      duration: duration,
      speakers: sessionResult.segments?.reduce((acc: string[], s: any) => {
        const speaker = s.speaker || 'Speaker 1';
        return acc.includes(speaker) ? acc : [...acc, speaker];
      }, [] as string[]).length || 1,
      accuracy: Math.round(averageConfidence * 100),
      language: config.language === 'en' ? 'English' : config.language,
      quality: config.modelId === 'high-accuracy' ? 'High Accuracy' : 
               config.modelId === 'turbo' ? 'Turbo' : 'Standard',
      preview: totalText.substring(0, 100) + (totalText.length > 100 ? '...' : ''),
    };
  };

  // App Navigation Handlers
  const handleNewMeeting = () => {
    setAppState(prev => ({ ...prev, showNewMeetingModal: true }));
  };

  const handleStartRecording = async (config: MeetingConfig) => {
    setAppState(prev => ({
      ...prev,
      currentMeeting: config,
      currentScreen: 'recording',
      showNewMeetingModal: false,
      isRecording: true,
      isPaused: false,
      recordingDuration: 0,
      transcriptSegments: [],
      audioLevel: 0,
    }));

    // Start actual backend transcription
    if (transcriptionControllerRef.current?.handleStartRecording) {
      try {
        await transcriptionControllerRef.current.handleStartRecording();
      } catch (error) {
        console.error('Failed to start transcription:', error);
        // Revert UI state on error
        setAppState(prev => ({
          ...prev,
          isRecording: false,
          currentScreen: 'dashboard',
        }));
      }
    }
  };

  const handleStopRecording = async () => {
    // Stop actual backend transcription first
    if (transcriptionControllerRef.current?.handleStopRecording) {
      try {
        await transcriptionControllerRef.current.handleStopRecording();
      } catch (error) {
        console.error('Failed to stop transcription:', error);
      }
    }

    setAppState(prev => ({
      ...prev,
      isRecording: false,
      isPaused: false,
      audioLevel: 0,
      vadActivity: false,
      currentScreen: 'dashboard',
    }));
  };

  const handlePauseRecording = () => {
    setAppState(prev => ({ ...prev, isPaused: true }));
  };

  const handleResumeRecording = () => {
    setAppState(prev => ({ ...prev, isPaused: false }));
  };

  const handleEditSegment = (segmentId: string, newText: string) => {
    setAppState(prev => ({
      ...prev,
      transcriptSegments: prev.transcriptSegments.map(segment =>
        segment.id === segmentId ? { ...segment, text: newText } : segment
      )
    }));
  };

  const handleImportFile = async () => {
    try {
      // Create a file input element
      const input = document.createElement('input');
      input.type = 'file';
      input.accept = '.wav,.mp3,.m4a,.webm';
      input.style.display = 'none';
      
      // Create a promise to handle the file selection
      const filePromise = new Promise<File | null>((resolve) => {
        input.onchange = () => {
          const file = input.files?.[0] || null;
          resolve(file);
          document.body.removeChild(input);
        };
        input.oncancel = () => {
          resolve(null);
          document.body.removeChild(input);
        };
      });

      document.body.appendChild(input);
      input.click();
      
      const selectedFile = await filePromise;
      if (selectedFile) {
        console.log('Selected file:', selectedFile.name);

        // For now, we'll just show a placeholder since we need the actual file path
        // In a real implementation, we'd need to save the file to a temporary location
        alert(`File selected: ${selectedFile.name}. Full file processing will be implemented when the file dialog backend command is added.`);
        
        // TODO: This would work with actual file paths from Tauri file dialog
        const mockFilePath = `/tmp/${selectedFile.name}`;
        console.log('Mock file path:', mockFilePath);

        // Create transcription config based on current meeting config or defaults
        const config: TranscriptionConfig = {
          qualityTier: 'standard',
          languages: ['en'],
          enableSpeakerDiarization: true,
          enableTwoPassRefinement: true,
          audioSources: {
            microphone: false,
            systemAudio: false,
          },
          vadThreshold: 0.5,
        };

        // TODO: Call backend to transcribe the file (when file dialog is implemented)
        // const transcriptionResult = await invoke('transcribe_audio_file', {
        //   request: {
        //     filePath: mockFilePath,
        //     config: config
        //   }
        // });

        // For now, create a mock transcription result
        const transcriptionResult = {
          text: `Mock transcription for ${selectedFile.name}. This would contain the actual transcribed text from the audio file.`,
          confidence: 0.95,
          language: 'en'
        };

        console.log('Mock transcription result:', transcriptionResult);

        // Create a meeting record for the imported file
        const fileName = selectedFile.name.replace(/\.[^/.]+$/, "") || 'Imported Audio';
        const newSegment: TranscriptSegment = {
          id: `import-${Date.now()}`,
          startTime: 0,
          endTime: 0,
          speaker: 'Speaker 1',
          text: (transcriptionResult as any).text || 'Transcription completed',
          confidence: (transcriptionResult as any).confidence || 0.95,
        };

        // Add to transcript segments to display
        setAppState(prev => ({
          ...prev,
          transcriptSegments: [newSegment],
          currentScreen: 'meeting-review', // Switch to show the result
        }));

        // Save as a meeting in the dashboard format
        const meetingData: MeetingFile = {
          id: `import-${Date.now()}`,
          title: fileName,
          date: new Date(),
          duration: 0, // Unknown duration for imported files
          speakers: 1, // Default to 1 speaker for imported files
          accuracy: Math.round(((transcriptionResult as any).confidence || 0.95) * 100),
          language: 'English', // Default language, could be detected
          quality: 'Standard', // Default quality for imported files
          preview: (transcriptionResult as any).text?.substring(0, 100) + '...' || 'Transcription completed',
        };
        
        // Save using the new storage function
        saveMeetingToStorage(meetingData, 'imported');

      }
    } catch (error) {
      console.error('Failed to import audio file:', error);
      // TODO: Show error toast/notification
      alert(`Failed to import audio file: ${error}`);
    }
  };

  // Transcription Event Handlers
  const handleSessionStart = (sessionId: string) => {
    console.log('Transcription session started:', sessionId);
  };

  const handleSessionEnd = (result: FinalTranscriptionResult) => {
    console.log('Transcription session ended:', result);
    
    // Save recording session as a meeting if we have a current meeting config
    if (appState.currentMeeting) {
      const meeting = convertSessionToMeeting(result, appState.currentMeeting, appState.recordingDuration);
      saveMeetingToStorage(meeting, 'recorded');
    }
    
    setAppState(prev => ({ 
      ...prev, 
      sessionResults: [...prev.sessionResults, result]
    }));
  };

  const handleTranscriptionUpdate = (update: TranscriptionUpdateEvent) => {
    console.log('Transcription update:', update);
    // Convert update to transcript segment and add to live transcript
    const newSegment: TranscriptSegment = {
      id: `${Date.now()}-${Math.random()}`,
      startTime: 0, // Would come from update
      endTime: 0, // Would come from update  
      speaker: 'Speaker 1', // Would come from update
      text: (update as any).text || 'Live transcription text...',
      confidence: 0.95 // Would come from update
    };
    
    setAppState(prev => ({
      ...prev,
      transcriptSegments: [...prev.transcriptSegments, newSegment]
    }));
  };

  const handleError = (error: TranscriptionError) => {
    console.error('Transcription error:', error);
    setAppState(prev => ({ 
      ...prev, 
      errors: [...prev.errors, error],
      isRecording: error.severity === 'critical' ? false : prev.isRecording
    }));
  };

  // Get current screen subtitle
  const getScreenSubtitle = () => {
    switch (appState.currentScreen) {
      case 'recording':
        return appState.currentMeeting?.title || 'Live Recording';
      case 'meeting-review':
        return 'Meeting Review';
      default:
        return '100% Local Privacy • No Cloud Required';
    }
  };

  // Get model info for status bar
  const getModelInfo = () => {
    if (appState.currentMeeting) {
      return {
        name: appState.currentMeeting.modelId,
        status: 'ready' as const,
      };
    }
    return {
      name: 'Standard',
      status: 'ready' as const,
    };
  };

  // Get recording info for status bar
  const getRecordingInfo = () => {
    if (appState.isRecording) {
      return {
        isRecording: true,
        duration: `${Math.floor(appState.recordingDuration / 60)}:${(appState.recordingDuration % 60).toString().padStart(2, '0')}`,
        status: appState.isPaused ? 'Paused' : 'Recording',
      };
    }
    return {
      isRecording: false,
      status: 'Ready',
    };
  };

  // Render current screen
  const renderCurrentScreen = () => {
    switch (appState.currentScreen) {
      case 'recording':
        return (
          <RecordingScreen
            meetingTitle={appState.currentMeeting?.title}
            isRecording={appState.isRecording}
            isPaused={appState.isPaused}
            duration={appState.recordingDuration}
            audioLevel={appState.audioLevel}
            vadActivity={appState.vadActivity}
            transcriptSegments={appState.transcriptSegments}
            currentModel={appState.currentMeeting?.modelId}
            language={appState.currentMeeting?.language}
            onStart={() => {/* handled by controls */}}
            onPause={handlePauseRecording}
            onResume={handleResumeRecording}
            onStop={handleStopRecording}
            onEditSegment={handleEditSegment}
          />
        );
      default:
        return (
          <Dashboard
            meetings={appState.meetings}
            onNewMeeting={handleNewMeeting}
            onImportFile={handleImportFile}
            onOpenMeeting={(id) => {
              console.log('Open meeting', id);
              // TODO: Implement meeting review screen
              const meeting = appState.meetings.find(m => m.id === id);
              if (meeting) {
                console.log('Opening meeting:', meeting.title);
                // Could switch to meeting-review screen here
              }
            }}
            onDeleteMeeting={(id) => {
              console.log('Delete meeting', id);
              deleteMeetingFromStorage(id);
            }}
            onSearch={(query) => console.log('Search', query)}
          />
        );
    }
  };

  if (!appState.isAppReady) {
    return (
      <AppLayout loading={true} title="KagiNote" subtitle="Initializing...">
        <div />
      </AppLayout>
    );
  }

  return (
    <>
      <AppLayout
        title="KagiNote"
        subtitle={getScreenSubtitle()}
        modelInfo={getModelInfo()}
        recordingInfo={getRecordingInfo()}
        systemInfo={{ privacy: true, cpu: '15%', memory: '2.1GB' }}
      >
        {renderCurrentScreen()}
      </AppLayout>

      {/* New Meeting Modal */}
      <NewMeetingModal
        isOpen={appState.showNewMeetingModal}
        onClose={() => setAppState(prev => ({ ...prev, showNewMeetingModal: false }))}
        onStartRecording={handleStartRecording}
      />

      {/* TranscriptionController for backend integration */}
      <TranscriptionController
        ref={transcriptionControllerRef}
        onSessionStart={handleSessionStart}
        onSessionEnd={handleSessionEnd}
        onError={handleError}
        onTranscriptionUpdate={handleTranscriptionUpdate}
        initialConfig={{
          qualityTier: appState.currentMeeting?.modelId as 'standard' | 'high-accuracy' | 'turbo' || 'standard',
          languages: [appState.currentMeeting?.language || 'en'],
          enableSpeakerDiarization: true,
          enableTwoPassRefinement: true,
          audioSources: {
            microphone: true,
            systemAudio: false,
          },
          vadThreshold: 0.5,
        }}
      />
    </>
  );
}

export default App;
