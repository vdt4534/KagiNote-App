/**
 * TranscriptionController Component
 * 
 * Main orchestration component for transcription sessions, configuration,
 * real-time updates, and system monitoring.
 */

import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { DiarizationStatus } from '@/components/features/DiarizationStatusIndicator';
import { SpeakerActivity } from '@/components/features/SpeakerActivityDisplay';

export interface TranscriptionConfig {
  qualityTier: 'standard' | 'high-accuracy' | 'turbo';
  languages: string[];
  enableSpeakerDiarization: boolean;
  enableTwoPassRefinement: boolean;
  audioSources: {
    microphone: boolean;
    systemAudio: boolean;
  };
  vadThreshold: number;
}

export interface SystemCapabilities {
  recommendedTier: string;
  availableMemoryGB: number;
  hasGPU: boolean;
  cpuCores: number;
  warnings?: string[];
}

export interface TranscriptionSession {
  sessionId: string;
  config: TranscriptionConfig;
  startTime: number;
  status: 'active' | 'stopping' | 'stopped';
}

export interface TranscriptionError {
  type: string;
  message: string;
  severity?: 'warning' | 'error' | 'critical';
  recoveryOptions?: string[];
  timestamp: number;
  sessionId?: string;
}

export interface SystemStatus {
  thermalStatus?: {
    temperature: number;
    riskLevel: 'low' | 'medium' | 'high';
  };
  memoryUsage?: {
    used: number;
    available: number;
    percentage: number;
  };
  processingMetrics?: {
    realTimeFactor: number;
    averageLatency: number;
    queuedSegments: number;
    cpuUsage?: number;
    memoryUsage?: number;
  };
}

export interface TranscriptionUpdateEvent {
  sessionId: string;
  segment: any;
  updateType: 'new' | 'update' | 'correction';
  processingPass: 1 | 2;
}

export interface FinalTranscriptionResult {
  sessionId: string;
  totalDuration: number;
  segments: any[];
  speakers?: any[];
  qualityMetrics: any;
  processingTimeMs: number;
}

export interface SpeakerDetectionEvent {
  sessionId: string;
  speakerId: string;
  displayName: string;
  confidence: number;
  timestamp: number;
}

export interface SpeakerUpdateEvent {
  sessionId: string;
  speakers: SpeakerActivity[];
  currentSpeaker?: string;
  hasOverlappingSpeech: boolean;
}

export interface DiarizationWarningEvent {
  sessionId: string;
  message: string;
  type: 'low_confidence' | 'model_fallback' | 'processing_delay';
  recoveryHint?: string;
}

export interface DiarizationErrorEvent {
  sessionId: string;
  message: string;
  type: 'model_load_failed' | 'onnx_runtime_error' | 'insufficient_memory';
  severity: 'warning' | 'error' | 'critical';
  recoveryOptions?: string[];
}

export interface TranscriptionControllerProps {
  onSessionStart: (sessionId: string) => void;
  onSessionEnd: (result: FinalTranscriptionResult) => void;
  onError: (error: TranscriptionError) => void;
  onTranscriptionUpdate?: (update: TranscriptionUpdateEvent) => void;
  onSpeakerDetected?: (event: SpeakerDetectionEvent) => void;
  onSpeakerUpdate?: (event: SpeakerUpdateEvent) => void;
  onDiarizationWarning?: (event: DiarizationWarningEvent) => void;
  onDiarizationError?: (event: DiarizationErrorEvent) => void;
  initialConfig?: Partial<TranscriptionConfig>;
}

const DEFAULT_CONFIG: TranscriptionConfig = {
  qualityTier: 'standard',
  languages: ['en'],
  enableSpeakerDiarization: true,
  enableTwoPassRefinement: true,
  audioSources: {
    microphone: true,
    systemAudio: false,
  },
  vadThreshold: 0.5,
};

const AVAILABLE_LANGUAGES = [
  { code: 'en', name: 'English' },
  { code: 'ja', name: 'Japanese' },
  { code: 'es', name: 'Spanish' },
  { code: 'fr', name: 'French' },
  { code: 'de', name: 'German' },
];

export const TranscriptionController: React.FC<TranscriptionControllerProps> = ({
  onSessionStart,
  onSessionEnd,
  onError,
  onTranscriptionUpdate,
  onSpeakerDetected,
  onSpeakerUpdate,
  onDiarizationWarning,
  onDiarizationError,
  initialConfig,
}) => {
  const [config, setConfig] = useState<TranscriptionConfig>({
    ...DEFAULT_CONFIG,
    ...initialConfig,
  });
  const [systemCapabilities, setSystemCapabilities] = useState<SystemCapabilities | null>(null);
  const [currentSession, setCurrentSession] = useState<TranscriptionSession | null>(null);
  const [isStarting, setIsStarting] = useState(false);
  const [systemStatus, setSystemStatus] = useState<SystemStatus>({});
  const [latestTranscription, setLatestTranscription] = useState<string>('');
  const [error, setError] = useState<TranscriptionError | null>(null);
  const [validationError, setValidationError] = useState<string>('');
  const [diarizationStatus, setDiarizationStatus] = useState<DiarizationStatus>({
    serviceHealth: 'disabled',
    modelStatus: 'not_available'
  });
  const [speakerActivities, setSpeakerActivities] = useState<SpeakerActivity[]>([]);
  const [currentSpeaker, setCurrentSpeaker] = useState<string | undefined>();
  
  const unlistenRefs = useRef<UnlistenFn[]>([]);

  // Initialize system capabilities on mount
  useEffect(() => {
    const initializeSystem = async () => {
      try {
        const capabilities = await invoke<SystemCapabilities>('get_system_info');
        setSystemCapabilities(capabilities);
      } catch (err) {
        console.error('Failed to get system info:', err);
      }
    };

    initializeSystem();
  }, []);

  // Save configuration to localStorage when it changes
  useEffect(() => {
    localStorage.setItem('transcription-config', JSON.stringify(config));
  }, [config]);

  // Set up event listeners when session starts
  useEffect(() => {
    const setupEventListeners = async () => {
      if (!currentSession) return;

      try {
        // Clean up existing listeners
        unlistenRefs.current.forEach(unlisten => unlisten());
        unlistenRefs.current = [];

        // Transcription updates
        const unlistenTranscription = await listen<TranscriptionUpdateEvent>(
          'transcription-update',
          (event) => {
            const update = event.payload;
            if (update.sessionId === currentSession.sessionId) {
              setLatestTranscription(update.segment.text || '');
              if (onTranscriptionUpdate) {
                onTranscriptionUpdate(update);
              }
            }
          }
        );

        // System status updates
        const unlistenStatus = await listen<SystemStatus>('system-status', (event) => {
          setSystemStatus(event.payload);
        });

        // Error handling
        const unlistenError = await listen<TranscriptionError>('transcription-error', (event) => {
          const error = event.payload;
          setError(error);

          if (error.severity === 'critical') {
            // Stop session on critical errors
            handleStopRecording();
          }

          onError(error);
        });

        // Model status updates
        const unlistenModelStatus = await listen<any>('model-status', (event) => {
          const status = event.payload;
          console.log('Model status update:', status);
          
          if (status.status === 'downloading') {
            setLatestTranscription(status.message || 'Model downloading...');
          }
        });

        // Model ready notification
        const unlistenModelReady = await listen<any>('model-ready', (event) => {
          const status = event.payload;
          console.log('Model ready:', status);
          setLatestTranscription('Model loaded! Transcription starting...');
        });

        // Model error notification
        const unlistenModelError = await listen<any>('model-error', (event) => {
          const status = event.payload;
          console.error('Model error:', status);
          setError({
            type: 'model_initialization_failed',
            message: status.message || 'Failed to initialize transcription model',
            timestamp: Date.now(),
            severity: 'critical'
          });
        });

        // Model download progress
        const unlistenModelProgress = await listen<any>('model-progress', (event) => {
          const progress = event.payload;
          console.log('Model progress:', progress);
          
          if (progress.status === 'downloading') {
            setLatestTranscription(`${progress.message} (${progress.progress}%)`);
          } else if (progress.status === 'ready') {
            setLatestTranscription('Model ready! Listening for speech...');
          }
        });

        // Diarization event listeners
        const unlistenSpeakerDetected = await listen<SpeakerDetectionEvent>(
          'speaker-detected',
          (event) => {
            const speakerEvent = event.payload;
            if (speakerEvent.sessionId === currentSession.sessionId) {
              setDiarizationStatus(prev => ({
                ...prev,
                serviceHealth: 'ready',
                speakerCount: (prev.speakerCount || 0) + 1
              }));
              if (onSpeakerDetected) {
                onSpeakerDetected(speakerEvent);
              }
            }
          }
        );

        const unlistenSpeakerUpdate = await listen<SpeakerUpdateEvent>(
          'speaker-update',
          (event) => {
            const updateEvent = event.payload;
            if (updateEvent.sessionId === currentSession.sessionId) {
              setSpeakerActivities(updateEvent.speakers);
              setCurrentSpeaker(updateEvent.currentSpeaker);
              setDiarizationStatus(prev => ({
                ...prev,
                speakerCount: updateEvent.speakers.length,
                confidence: updateEvent.currentSpeaker 
                  ? updateEvent.speakers.find(s => s.speakerId === updateEvent.currentSpeaker)?.confidenceScore
                  : undefined
              }));
              if (onSpeakerUpdate) {
                onSpeakerUpdate(updateEvent);
              }
            }
          }
        );

        const unlistenDiarizationWarning = await listen<DiarizationWarningEvent>(
          'diarization-warning',
          (event) => {
            const warningEvent = event.payload;
            if (warningEvent.sessionId === currentSession.sessionId) {
              setDiarizationStatus(prev => ({
                ...prev,
                lastError: {
                  message: warningEvent.message,
                  type: 'warning',
                  recoveryHint: warningEvent.recoveryHint
                }
              }));
              if (onDiarizationWarning) {
                onDiarizationWarning(warningEvent);
              }
            }
          }
        );

        const unlistenDiarizationError = await listen<DiarizationErrorEvent>(
          'diarization-error',
          (event) => {
            const errorEvent = event.payload;
            if (errorEvent.sessionId === currentSession.sessionId) {
              setDiarizationStatus(prev => ({
                ...prev,
                serviceHealth: 'error',
                lastError: {
                  message: errorEvent.message,
                  type: errorEvent.severity,
                  recoveryHint: errorEvent.recoveryOptions?.join(', ')
                }
              }));
              if (onDiarizationError) {
                onDiarizationError(errorEvent);
              }
            }
          }
        );

        unlistenRefs.current = [
          unlistenTranscription, 
          unlistenStatus, 
          unlistenError, 
          unlistenModelStatus,
          unlistenModelReady,
          unlistenModelError,
          unlistenModelProgress,
          unlistenSpeakerDetected,
          unlistenSpeakerUpdate,
          unlistenDiarizationWarning,
          unlistenDiarizationError
        ];
      } catch (err) {
        console.error('Failed to set up event listeners:', err);
      }
    };

    setupEventListeners();

    return () => {
      unlistenRefs.current.forEach(unlisten => unlisten());
    };
  }, [currentSession, onTranscriptionUpdate, onError, onSpeakerDetected, onSpeakerUpdate, onDiarizationWarning, onDiarizationError]);

  const validateConfiguration = (): string => {
    if (config.languages.length === 0) {
      return 'At least one language must be selected';
    }
    
    if (!config.audioSources.microphone && !config.audioSources.systemAudio) {
      return 'At least one audio source must be enabled';
    }

    return '';
  };

  const handleStartRecording = async () => {
    const validation = validateConfiguration();
    if (validation) {
      setValidationError(validation);
      return;
    }

    setValidationError('');
    setIsStarting(true);
    setError(null);
    
    // Initialize diarization status based on config
    if (config.enableSpeakerDiarization) {
      setDiarizationStatus({
        serviceHealth: 'initializing',
        modelStatus: 'loading'
      });
    } else {
      setDiarizationStatus({
        serviceHealth: 'disabled',
        modelStatus: 'not_available'
      });
    }

    try {
      const sessionId = await invoke<string>('start_transcription', {
        config: config,
      });

      const session: TranscriptionSession = {
        sessionId,
        config: { ...config },
        startTime: Date.now(),
        status: 'active',
      };

      setCurrentSession(session);
      setLatestTranscription('');
      setSpeakerActivities([]);
      setCurrentSpeaker(undefined);
      onSessionStart(sessionId);
    } catch (err: any) {
      const error: TranscriptionError = {
        type: err.type || 'transcription_start_failed',
        message: err.message || 'Failed to start transcription',
        timestamp: Date.now(),
        recoveryOptions: err.recoveryOptions,
      };
      
      setError(error);
      onError(error);
    } finally {
      setIsStarting(false);
    }
  };

  const handleStopRecording = async () => {
    if (!currentSession) return;

    setCurrentSession(prev => prev ? { ...prev, status: 'stopping' } : null);

    try {
      const result = await invoke<FinalTranscriptionResult>('stop_transcription', {
        sessionId: currentSession.sessionId,
      });

      onSessionEnd(result);
      setCurrentSession(null);
      setLatestTranscription('');
      setSpeakerActivities([]);
      setCurrentSpeaker(undefined);
      setDiarizationStatus({
        serviceHealth: 'disabled',
        modelStatus: 'not_available'
      });
    } catch (err: any) {
      const error: TranscriptionError = {
        type: 'transcription_stop_failed',
        message: err.message || 'Failed to stop transcription',
        timestamp: Date.now(),
        sessionId: currentSession.sessionId,
      };
      
      setError(error);
      onError(error);
    }
  };

  const handleEmergencyStop = async () => {
    try {
      const result = await invoke<string>('emergency_stop_all');
      console.log('Emergency stop result:', result);
      setCurrentSession(null);
      setLatestTranscription('');
      setError(null);
      setSpeakerActivities([]);
      setCurrentSpeaker(undefined);
      setDiarizationStatus({
        serviceHealth: 'disabled',
        modelStatus: 'not_available'
      });
    } catch (err: any) {
      console.error('Emergency stop failed:', err);
    }
  };

  const handleConfigChange = <K extends keyof TranscriptionConfig>(
    key: K,
    value: TranscriptionConfig[K]
  ) => {
    if (currentSession) return; // Don't allow changes during recording
    
    setConfig(prev => ({ ...prev, [key]: value }));
  };

  const handleLanguageToggle = (languageCode: string) => {
    if (currentSession) return;

    setConfig(prev => {
      const languages = prev.languages.includes(languageCode)
        ? prev.languages.filter(lang => lang !== languageCode)
        : [...prev.languages, languageCode];
      
      return { ...prev, languages };
    });
  };

  const getEstimatedRequirements = (tier: string) => {
    switch (tier) {
      case 'turbo':
        return 'Turbo: 4GB RAM, basic CPU';
      case 'standard':
        return 'Standard: 8GB RAM, medium CPU';
      case 'high-accuracy':
        return 'High Accuracy: 16GB RAM, high-end CPU';
      default:
        return 'Unknown tier requirements';
    }
  };

  const formatTemperature = (temp: number) => `${temp.toFixed(1)}°C`;
  const formatMemoryUsage = (percentage: number) => `${percentage}%`;
  const formatRTF = (rtf: number) => `${rtf.toFixed(1)}x`;
  const formatLatency = (ms: number) => `${ms}ms`;

  const renderSystemCapabilities = () => {
    if (!systemCapabilities) return null;

    return (
      <div data-testid="system-capabilities" className="system-capabilities">
        <h3>System Information</h3>
        <div>CPU cores: {systemCapabilities.cpuCores}</div>
        <div>Available memory: {systemCapabilities.availableMemoryGB}GB</div>
        <div>GPU acceleration: {systemCapabilities.hasGPU ? 'Available' : 'Not available'}</div>
        <div data-testid="recommended-tier">
          Recommended tier: {systemCapabilities.recommendedTier}
        </div>
        
        {systemCapabilities.warnings && systemCapabilities.warnings.length > 0 && (
          <div data-testid="compatibility-warning" className="warning">
            <strong>Warnings:</strong>
            {systemCapabilities.warnings.map((warning, index) => (
              <div key={index}>{warning.replace('_', ' ')}</div>
            ))}
          </div>
        )}
      </div>
    );
  };

  const renderConfigurationPanel = () => {
    const isLocked = !!currentSession;
    
    return (
      <div data-testid="transcription-settings" className="configuration-panel">
        <h3>Configuration</h3>
        
        {isLocked && (
          <div data-testid="config-locked-message" className="locked-message">
            Settings locked during recording
          </div>
        )}
        
        {/* Quality Tier Selection */}
        <div className="setting-group">
          <label htmlFor="quality-tier">Quality Tier:</label>
          <select
            id="quality-tier"
            data-testid="quality-tier-selector"
            value={config.qualityTier}
            disabled={isLocked}
            onChange={(e) => handleConfigChange('qualityTier', e.target.value as any)}
          >
            <option value="turbo">Turbo</option>
            <option value="standard">Standard</option>
            <option value="high-accuracy">High Accuracy</option>
          </select>
          <div data-testid="estimated-requirements">
            {getEstimatedRequirements(config.qualityTier)}
          </div>
        </div>

        {/* Language Selection */}
        <div className="setting-group">
          <label>Languages:</label>
          {AVAILABLE_LANGUAGES.map(lang => (
            <label key={lang.code} className="checkbox-label">
              <input
                type="checkbox"
                data-testid={`language-${lang.code}`}
                checked={config.languages.includes(lang.code)}
                disabled={isLocked}
                onChange={() => handleLanguageToggle(lang.code)}
              />
              {lang.name}
            </label>
          ))}
        </div>

        {/* Speaker Diarization */}
        <div className="setting-group">
          <label className="checkbox-label">
            <input
              type="checkbox"
              data-testid="speaker-diarization-toggle"
              checked={config.enableSpeakerDiarization}
              disabled={isLocked}
              onChange={(e) => handleConfigChange('enableSpeakerDiarization', e.target.checked)}
            />
            Enable Speaker Diarization
          </label>
        </div>

        {/* Two-Pass Refinement */}
        <div className="setting-group">
          <label className="checkbox-label">
            <input
              type="checkbox"
              checked={config.enableTwoPassRefinement}
              disabled={isLocked}
              onChange={(e) => handleConfigChange('enableTwoPassRefinement', e.target.checked)}
            />
            Enable Two-Pass Refinement
          </label>
        </div>
      </div>
    );
  };

  const renderSystemStatus = () => {
    return (
      <div data-testid="system-status" className="system-status">
        <h3>System Status</h3>
        
        {systemStatus.thermalStatus && (
          <div data-testid="thermal-status">
            Temperature: {formatTemperature(systemStatus.thermalStatus.temperature)}
          </div>
        )}
        
        {systemStatus.memoryUsage && (
          <div data-testid="memory-usage">
            Memory: {formatMemoryUsage(systemStatus.memoryUsage.percentage)}
          </div>
        )}
        
        {systemStatus.processingMetrics && (
          <>
            <div data-testid="real-time-factor">
              RTF: {formatRTF(systemStatus.processingMetrics.realTimeFactor)}
            </div>
            <div data-testid="rtf-display">
              {formatRTF(systemStatus.processingMetrics.realTimeFactor)}
            </div>
            <div data-testid="latency-display">
              {formatLatency(systemStatus.processingMetrics.averageLatency)}
            </div>
            {systemStatus.processingMetrics.cpuUsage && (
              <div data-testid="cpu-usage">
                CPU: {systemStatus.processingMetrics.cpuUsage.toFixed(1)}%
              </div>
            )}
            {systemStatus.processingMetrics.memoryUsage && (
              <div data-testid="memory-usage">
                Memory: {systemStatus.processingMetrics.memoryUsage.toFixed(1)}GB
              </div>
            )}
          </>
        )}

        {/* Performance Warnings */}
        {(systemStatus.processingMetrics?.realTimeFactor ?? 0) > 1.0 && (
          <div data-testid="performance-warning" className="warning">
            <div>Processing slower than real-time detected</div>
            <div data-testid="performance-suggestions">
              Consider reducing quality tier or closing other applications
            </div>
          </div>
        )}

        {(systemStatus.processingMetrics?.averageLatency ?? 0) > 3000 && (
          <div className="warning">
            <div>High latency detected</div>
          </div>
        )}
      </div>
    );
  };

  const renderErrors = () => {
    if (!error) return null;

    const isModelError = error.type.includes('model');
    const isAudioError = error.type.includes('audio');

    return (
      <div>
        <div data-testid="error-message" className="error">
          {error.message}
        </div>

        {isModelError && (
          <div data-testid="model-error-dialog" className="error-dialog">
            <div>{error.message}</div>
            <button data-testid="download-model-button">Download Model</button>
          </div>
        )}

        {isAudioError && error.recoveryOptions && (
          <div data-testid="error-dialog" className="error-dialog">
            <div>{error.message}</div>
            {error.recoveryOptions.map(option => (
              <button
                key={option}
                data-testid={`recovery-${option.replace(/\s+/g, '-').toLowerCase()}`}
              >
                {option.replace('_', ' ')}
              </button>
            ))}
          </div>
        )}

        {error.severity === 'critical' && (
          <div data-testid="critical-error-dialog" className="critical-error">
            <div>{error.message}</div>
          </div>
        )}

        {error.severity === 'warning' && error.type === 'thermal_throttle' && (
          <div data-testid="system-warning" className="warning">
            <div>{error.message}</div>
            {error.message.includes('reducing quality') && (
              <div>Quality automatically reduced due to thermal constraints</div>
            )}
            <div data-testid="suggested-action">
              Consider reducing quality settings
            </div>
          </div>
        )}

        {error.type === 'processing_queue_full' && (
          <div data-testid="processing-warning" className="warning">
            <div>{error.message}</div>
          </div>
        )}
      </div>
    );
  };

  return (
    <div data-testid="transcription-controller" className="transcription-controller">
      {/* System Capabilities */}
      {renderSystemCapabilities()}
      
      {/* Configuration Panel */}
      {renderConfigurationPanel()}
      
      {/* Control Buttons */}
      <div className="control-buttons">
        {validationError && (
          <div data-testid="validation-error" className="validation-error">
            {validationError}
          </div>
        )}
        
        <button
          data-testid="start-recording-button"
          disabled={!!currentSession || isStarting}
          onClick={handleStartRecording}
        >
          {isStarting ? 'Starting...' : 'Start Recording'}
        </button>
        
        <button
          data-testid="stop-recording-button"
          disabled={!currentSession || currentSession.status === 'stopping'}
          onClick={handleStopRecording}
        >
          {currentSession?.status === 'stopping' ? 'Stopping...' : 'Stop Recording'}
        </button>
        
        <button
          data-testid="emergency-stop-button"
          onClick={handleEmergencyStop}
          style={{ backgroundColor: '#d32f2f', color: 'white' }}
        >
          Emergency Stop
        </button>
      </div>

      {/* Recording Status */}
      {currentSession && (
        <div data-testid="recording-active-indicator" className="recording-active">
          Recording in progress...
        </div>
      )}

      {/* Latest Transcription */}
      {latestTranscription && (
        <div data-testid="latest-transcription" className="latest-transcription">
          <h3>Live Transcription:</h3>
          <div data-testid="transcription-text">{latestTranscription}</div>
        </div>
      )}

      {/* System Status */}
      {renderSystemStatus()}
      
      {/* Error Display */}
      {renderErrors()}

      <style>{`
        .transcription-controller {
          padding: 20px;
          max-width: 800px;
        }
        
        .system-capabilities {
          margin-bottom: 20px;
          padding: 15px;
          border: 1px solid #ddd;
          border-radius: 4px;
        }
        
        .configuration-panel {
          margin-bottom: 20px;
          padding: 15px;
          border: 1px solid #ddd;
          border-radius: 4px;
        }
        
        .setting-group {
          margin-bottom: 15px;
        }
        
        .checkbox-label {
          display: block;
          margin: 5px 0;
        }
        
        .control-buttons {
          margin-bottom: 20px;
        }
        
        .control-buttons button {
          margin-right: 10px;
          padding: 10px 20px;
          border: none;
          border-radius: 4px;
          cursor: pointer;
        }
        
        .control-buttons button:disabled {
          opacity: 0.6;
          cursor: not-allowed;
        }
        
        .recording-active {
          padding: 10px;
          background: #e8f5e8;
          border: 1px solid #4caf50;
          border-radius: 4px;
          margin-bottom: 20px;
        }
        
        .latest-transcription {
          margin-bottom: 20px;
          padding: 15px;
          border: 1px solid #ddd;
          border-radius: 4px;
        }
        
        .system-status {
          margin-bottom: 20px;
          padding: 15px;
          border: 1px solid #ddd;
          border-radius: 4px;
        }
        
        .error, .warning, .validation-error {
          padding: 10px;
          margin: 10px 0;
          border-radius: 4px;
        }
        
        .error, .validation-error {
          background: #ffebee;
          color: #c62828;
          border: 1px solid #c62828;
        }
        
        .warning {
          background: #fff3e0;
          color: #ef6c00;
          border: 1px solid #ef6c00;
        }
        
        .critical-error {
          background: #ffcdd2;
          color: #d32f2f;
          border: 2px solid #d32f2f;
          font-weight: bold;
        }
        
        .error-dialog {
          padding: 15px;
          margin: 10px 0;
          border: 1px solid #ccc;
          border-radius: 4px;
          background: white;
        }
        
        .error-dialog button {
          margin: 5px;
          padding: 8px 16px;
          border: none;
          border-radius: 4px;
          background: #2196f3;
          color: white;
          cursor: pointer;
        }
        
        .locked-message {
          color: #666;
          font-style: italic;
          margin-bottom: 10px;
        }
      `}</style>
    </div>
  );
};

export default TranscriptionController;