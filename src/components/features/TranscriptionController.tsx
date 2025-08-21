/**
 * TranscriptionController Component
 * 
 * Main orchestration component for transcription sessions, configuration,
 * real-time updates, and system monitoring.
 */

import React, { useState, useEffect, useRef, forwardRef, useImperativeHandle } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

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

export interface TranscriptionControllerProps {
  onSessionStart: (sessionId: string) => void;
  onSessionEnd: (result: FinalTranscriptionResult) => void;
  onError: (error: TranscriptionError) => void;
  onTranscriptionUpdate?: (update: TranscriptionUpdateEvent) => void;
  initialConfig?: Partial<TranscriptionConfig>;
}

export interface TranscriptionControllerRef {
  handleStartRecording: () => Promise<void>;
  handleStopRecording: () => Promise<void>;
  handleEmergencyStop: () => Promise<void>;
  isRecording: () => boolean;
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

export const TranscriptionController = forwardRef<TranscriptionControllerRef, TranscriptionControllerProps>(({
  onSessionStart,
  onSessionEnd,
  onError,
  onTranscriptionUpdate,
  initialConfig,
}, ref) => {
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
  
  const unlistenRefs = useRef<UnlistenFn[]>([]);

  // Define methods that will be called later
  const handleStartRecording = async () => {
    const validation = validateConfiguration();
    if (validation) {
      setValidationError(validation);
      return;
    }

    setValidationError('');
    setIsStarting(true);
    setError(null);

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
      onSessionStart(sessionId);
    } catch (err: any) {
      const error: TranscriptionError = {
        type: 'transcription_start_failed',
        message: err.message || 'Failed to start transcription',
        timestamp: Date.now(),
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
      await invoke('emergency_stop_all');
      setCurrentSession(null);
      setLatestTranscription('');
      setError(null);
    } catch (err: any) {
      const error: TranscriptionError = {
        type: 'emergency_stop_failed',
        message: err.message || 'Failed to stop all sessions',
        timestamp: Date.now(),
        severity: 'critical',
      };
      
      setError(error);
      onError(error);
    }
  };

  // Expose methods to parent component
  useImperativeHandle(ref, () => ({
    handleStartRecording,
    handleStopRecording,
    handleEmergencyStop,
    isRecording: () => currentSession?.status === 'active',
  }), [currentSession, config]);

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
            type: status.errorType || 'model_initialization_failed',
            message: status.message || 'Failed to initialize transcription model',
            timestamp: Date.now(),
            severity: 'critical',
            recoveryOptions: status.recoveryOptions || [
              'Check internet connectivity for model download',
              'Ensure sufficient disk space (2GB+)',
              'Try restarting the application',
              'Contact support if issue persists'
            ]
          });
        });
        
        // Fallback error for build issues
        const unlistenBuildError = await listen<any>('build-error', (event) => {
          const status = event.payload;
          console.error('Build error:', status);
          setError({
            type: 'whisper_build_failed',
            message: status.message || 'Whisper transcription engine failed to build',
            timestamp: Date.now(),
            severity: 'critical',
            recoveryOptions: [
              'Install cmake: brew install cmake',
              'Install Xcode Command Line Tools: xcode-select --install',
              'Restart terminal and try again'
            ]
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

        unlistenRefs.current = [
          unlistenTranscription, 
          unlistenStatus, 
          unlistenError, 
          unlistenModelStatus,
          unlistenModelReady,
          unlistenModelError,
          unlistenBuildError,
          unlistenModelProgress
        ];
      } catch (err) {
        console.error('Failed to set up event listeners:', err);
      }
    };

    setupEventListeners();

    return () => {
      unlistenRefs.current.forEach(unlisten => unlisten());
    };
  }, [currentSession, onTranscriptionUpdate, onError]);

  const validateConfiguration = (): string => {
    if (config.languages.length === 0) {
      return 'At least one language must be selected';
    }
    
    if (!config.audioSources.microphone && !config.audioSources.systemAudio) {
      return 'At least one audio source must be enabled';
    }

    return '';
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
    const isSystemError = error.type.includes('system');
    const isWhisperBuildError = error.message.includes('whisper-rs') || error.message.includes('cmake') || error.message.includes('build');
    
    return (
      <div>
        <div data-testid="error-message" className="error">
          <strong>Error:</strong> {error.message}
          {error.timestamp && (
            <div className="error-timestamp">
              {new Date(error.timestamp).toLocaleTimeString()}
            </div>
          )}
        </div>

        {/* Whisper Build Error - Special handling */}
        {isWhisperBuildError && (
          <div data-testid="whisper-build-error-dialog" className="error-dialog critical">
            <h4>🔨 Build System Error Detected</h4>
            <p>The Whisper transcription engine failed to compile. This is a build-time issue, not a runtime problem.</p>
            
            <div className="build-error-details">
              <h5>Most likely causes:</h5>
              <ul>
                <li>Missing cmake build tool</li>
                <li>Incompatible Xcode Command Line Tools</li>
                <li>Missing Metal SDK (macOS)</li>
                <li>Outdated clang/gcc compiler</li>
              </ul>
              
              <h5>Recommended fixes:</h5>
              <ol>
                <li><code>brew install cmake</code></li>
                <li><code>xcode-select --install</code></li>
                <li>Restart your terminal/IDE</li>
                <li>Run <code>npm run tauri dev</code> again</li>
              </ol>
            </div>
            
            <button 
              data-testid="retry-build-button"
              onClick={() => {
                setError(null);
                window.location.reload();
              }}
            >
              Retry After Fixes
            </button>
          </div>
        )}

        {/* Model Errors */}
        {isModelError && !isWhisperBuildError && (
          <div data-testid="model-error-dialog" className="error-dialog">
            <h4>🤖 Model Issue</h4>
            <div>{error.message}</div>
            
            {error.recoveryOptions && (
              <div className="recovery-options">
                <h5>Recovery Options:</h5>
                {error.recoveryOptions.map((option, index) => (
                  <div key={index} className="recovery-option">
                    • {option}
                  </div>
                ))}
              </div>
            )}
            
            <button data-testid="download-model-button">Retry Model Download</button>
          </div>
        )}

        {/* Audio Errors */}
        {isAudioError && (
          <div data-testid="audio-error-dialog" className="error-dialog">
            <h4>🎤 Audio System Issue</h4>
            <div>{error.message}</div>
            
            {error.recoveryOptions && (
              <div className="recovery-options">
                <h5>Try these solutions:</h5>
                {error.recoveryOptions.map((option, index) => (
                  <button
                    key={index}
                    data-testid={`recovery-${option.replace(/\s+/g, '-').toLowerCase()}`}
                    className="recovery-button"
                    onClick={() => {
                      if (option.includes('Emergency Stop')) {
                        handleEmergencyStop();
                      }
                    }}
                  >
                    {option.replace('_', ' ')}
                  </button>
                ))}
              </div>
            )}
          </div>
        )}

        {/* System Errors */}
        {isSystemError && (
          <div data-testid="system-error-dialog" className="error-dialog">
            <h4>💻 System Resource Issue</h4>
            <div>{error.message}</div>
            
            {systemCapabilities && (
              <div className="system-info">
                <p><strong>System Info:</strong></p>
                <p>CPU: {systemCapabilities.cpuCores} cores</p>
                <p>RAM: {systemCapabilities.availableMemoryGB.toFixed(1)}GB</p>
                <p>GPU: {systemCapabilities.hasGPU ? 'Available' : 'Not detected'}</p>
              </div>
            )}
            
            {error.recoveryOptions && (
              <div className="recovery-options">
                {error.recoveryOptions.map((option, index) => (
                  <div key={index} className="recovery-option">
                    • {option}
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {/* Critical Errors */}
        {error.severity === 'critical' && (
          <div data-testid="critical-error-dialog" className="critical-error">
            <h4>🚨 Critical Error</h4>
            <div>{error.message}</div>
            <button onClick={handleEmergencyStop}>Emergency Stop & Reset</button>
          </div>
        )}

        {/* Performance Warnings */}
        {error.severity === 'warning' && error.type === 'thermal_throttle' && (
          <div data-testid="system-warning" className="warning">
            <h4>🌡️ Thermal Warning</h4>
            <div>{error.message}</div>
            {error.message.includes('reducing quality') && (
              <div>Quality automatically reduced due to thermal constraints</div>
            )}
            <div data-testid="suggested-action">
              Consider reducing quality settings
            </div>
          </div>
        )}

        {/* Processing Warnings */}
        {error.type === 'processing_queue_full' && (
          <div data-testid="processing-warning" className="warning">
            <h4>⚠️ Processing Queue Full</h4>
            <div>{error.message}</div>
          </div>
        )}
        
        {/* Generic Error Actions */}
        <div className="error-actions">
          <button 
            onClick={() => setError(null)} 
            className="dismiss-error-btn"
          >
            Dismiss
          </button>
        </div>
      </div>
    );
  };

  // This is a headless component that only handles backend logic
  // All UI is rendered by RecordingScreen component
  return null;
});

TranscriptionController.displayName = 'TranscriptionController';

export default TranscriptionController;