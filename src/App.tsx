import { useState, useEffect } from "react";
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { AudioVisualizer } from "./components/AudioVisualizer";
import { TranscriptionController, FinalTranscriptionResult, TranscriptionError, TranscriptionUpdateEvent } from "./components/TranscriptionController";
import "./App.css";

interface AppState {
  isAppReady: boolean;
  audioLevel: number;
  isRecording: boolean;
  vadActivity: boolean;
  sessionResults: FinalTranscriptionResult[];
  errors: TranscriptionError[];
}

function App() {
  const [appState, setAppState] = useState<AppState>({
    isAppReady: false,
    audioLevel: 0,
    isRecording: false,
    vadActivity: false,
    sessionResults: [],
    errors: [],
  });

  // Initialize app and mark as ready
  useEffect(() => {
    const initializeApp = async () => {
      try {
        // Perform any necessary initialization
        await new Promise(resolve => setTimeout(resolve, 1000)); // Simulate initialization
        setAppState(prev => ({ ...prev, isAppReady: true }));
      } catch (error) {
        console.error('Failed to initialize app:', error);
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

  const handleSessionStart = (sessionId: string) => {
    console.log('Transcription session started:', sessionId);
    setAppState(prev => ({ 
      ...prev, 
      isRecording: true,
      audioLevel: 0 // Start with no audio level, wait for real data
    }));
  };

  const handleSessionEnd = (result: FinalTranscriptionResult) => {
    console.log('Transcription session ended:', result);
    setAppState(prev => ({ 
      ...prev, 
      isRecording: false,
      audioLevel: 0,
      vadActivity: false,
      sessionResults: [...prev.sessionResults, result]
    }));
  };

  const handleTranscriptionUpdate = (update: TranscriptionUpdateEvent) => {
    console.log('Transcription update:', update);
    // Handle real-time transcription updates
  };

  const handleError = (error: TranscriptionError) => {
    console.error('Transcription error:', error);
    setAppState(prev => ({ 
      ...prev, 
      errors: [...prev.errors, error],
      isRecording: error.severity === 'critical' ? false : prev.isRecording
    }));
  };

  const handlePlaybackToggle = (playing: boolean) => {
    console.log('Playback toggled:', playing);
  };

  const handleSeek = (time: number) => {
    console.log('Seek to time:', time);
  };

  if (!appState.isAppReady) {
    return (
      <div className="loading-container">
        <div>Loading KagiNote...</div>
        <div data-testid="system-check-running">Checking system requirements...</div>
      </div>
    );
  }

  return (
    <main className="container" data-testid="app-ready">
      <header className="app-header">
        <h1>KagiNote</h1>
        <p>Privacy-focused meeting transcription</p>
      </header>

      <div className="app-layout">
        {/* Audio Visualization Section */}
        <section className="audio-section">
          <h2>Audio Input</h2>
          <AudioVisualizer
            audioLevel={appState.audioLevel}
            isRecording={appState.isRecording}
            vadActivity={appState.vadActivity}
            showWaveform={false} // Start with level meters, can be toggled
            height={100}
            width={800}
            showPeakIndicators={true}
            onPlaybackToggle={handlePlaybackToggle}
            onSeek={handleSeek}
          />
        </section>

        {/* Transcription Control Section */}
        <section className="transcription-section">
          <TranscriptionController
            onSessionStart={handleSessionStart}
            onSessionEnd={handleSessionEnd}
            onError={handleError}
            onTranscriptionUpdate={handleTranscriptionUpdate}
            initialConfig={{
              qualityTier: 'standard',
              languages: ['en'],
              enableSpeakerDiarization: true,
              enableTwoPassRefinement: true,
            }}
          />
        </section>

        {/* Results Section */}
        {appState.sessionResults.length > 0 && (
          <section className="results-section">
            <h2>Session Results</h2>
            {appState.sessionResults.map((result, index) => (
              <div key={result.sessionId} className="session-result">
                <h3>Session {index + 1}</h3>
                <div>Duration: {result.totalDuration}s</div>
                <div>Segments: {result.segments.length}</div>
                <div>Processing Time: {result.processingTimeMs}ms</div>
              </div>
            ))}
          </section>
        )}

        {/* Error Display */}
        {appState.errors.length > 0 && (
          <section className="errors-section">
            <h2>System Messages</h2>
            {appState.errors.slice(-3).map((error, index) => (
              <div key={index} className={`error-item ${error.severity || 'error'}`}>
                <strong>{error.type}:</strong> {error.message}
              </div>
            ))}
          </section>
        )}
      </div>

      <style>{`
        .loading-container {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          height: 100vh;
          gap: 20px;
        }

        .app-header {
          text-align: center;
          margin-bottom: 30px;
          padding: 20px;
          border-bottom: 1px solid #e1e5e9;
        }

        .app-header h1 {
          color: #1a1a1a;
          margin: 0;
          font-size: 2.5rem;
        }

        .app-header p {
          color: #666;
          margin: 10px 0 0 0;
          font-size: 1.1rem;
        }

        .app-layout {
          display: flex;
          flex-direction: column;
          gap: 30px;
          max-width: 1200px;
          margin: 0 auto;
          padding: 0 20px;
        }

        .audio-section,
        .transcription-section,
        .results-section,
        .errors-section {
          background: white;
          border-radius: 8px;
          box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
          padding: 20px;
        }

        .audio-section h2,
        .transcription-section h2,
        .results-section h2,
        .errors-section h2 {
          margin: 0 0 20px 0;
          color: #1a1a1a;
          border-bottom: 2px solid #3b82f6;
          padding-bottom: 8px;
        }

        .session-result {
          padding: 15px;
          background: #f8f9fa;
          border-radius: 4px;
          margin-bottom: 10px;
        }

        .session-result h3 {
          margin: 0 0 10px 0;
          color: #1a1a1a;
        }

        .session-result div {
          margin: 5px 0;
          color: #666;
        }

        .error-item {
          padding: 10px;
          margin: 5px 0;
          border-radius: 4px;
          border-left: 4px solid;
        }

        .error-item.error {
          background: #ffebee;
          border-color: #c62828;
          color: #c62828;
        }

        .error-item.warning {
          background: #fff3e0;
          border-color: #ef6c00;
          color: #ef6c00;
        }

        .error-item.critical {
          background: #ffcdd2;
          border-color: #d32f2f;
          color: #d32f2f;
          font-weight: bold;
        }

        @media (max-width: 768px) {
          .app-layout {
            padding: 0 15px;
            gap: 20px;
          }

          .app-header h1 {
            font-size: 2rem;
          }
        }
      `}</style>
    </main>
  );
}

export default App;
