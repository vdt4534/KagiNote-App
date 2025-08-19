/**
 * Transcription Controller Component Tests
 * 
 * These tests are written BEFORE implementation exists (TDD).
 * Tests define the contract for the main transcription orchestration component.
 * All tests should FAIL initially - this is correct TDD behavior.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor, fireEvent, act } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { AudioTestFactory } from '../../factories/AudioTestFactory';
import { TranscriptionTestFactory } from '../../factories/TranscriptionTestFactory';

// These imports WILL FAIL because components don't exist yet
import { TranscriptionController } from '@/components/TranscriptionController';
import type { 
  TranscriptionControllerProps,
  TranscriptionConfig,
  TranscriptionSession,
  TranscriptionError 
} from '@/components/TranscriptionController';

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
  emit: vi.fn(),
}));

describe('TranscriptionController Component', () => {
  let defaultProps: TranscriptionControllerProps;
  let mockInvoke: any;
  let mockListen: any;

  beforeEach(() => {
    defaultProps = {
      onSessionStart: vi.fn(),
      onSessionEnd: vi.fn(),
      onError: vi.fn(),
      onTranscriptionUpdate: vi.fn(),
      initialConfig: {
        qualityTier: 'standard',
        languages: ['en'],
        enableSpeakerDiarization: true,
        enableTwoPassRefinement: true,
        audioSources: {
          microphone: true,
          systemAudio: false,
        },
        vadThreshold: 0.5,
      },
    };

    mockInvoke = vi.mocked(invoke);
    mockListen = vi.mocked(listen);

    // Mock successful transcription session
    mockInvoke.mockImplementation((command: string, args?: any) => {
      switch (command) {
        case 'start_transcription':
          return Promise.resolve('session-123');
        case 'stop_transcription':
          return Promise.resolve({
            sessionId: 'session-123',
            totalDuration: 300,
            segments: TranscriptionTestFactory.createBasicTranscriptionSegments(),
            speakers: TranscriptionTestFactory.createSpeakerProfiles(),
            qualityMetrics: TranscriptionTestFactory.createQualityMetrics(),
            processingTimeMs: 2500,
          });
        case 'get_real_time_results':
          return Promise.resolve(TranscriptionTestFactory.createBasicTranscriptionSegments());
        case 'get_system_info':
          return Promise.resolve({
            recommendedTier: 'standard',
            availableMemoryGB: 16,
            hasGPU: false,
            cpuCores: 8,
          });
        default:
          return Promise.resolve();
      }
    });

    mockListen.mockImplementation((event: string, callback: Function) => {
      return Promise.resolve(() => {}); // Unsubscribe function
    });
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('Component Initialization', () => {
    it('should render transcription controls correctly', () => {
      // ACT - This WILL FAIL because TranscriptionController doesn't exist
      render(<TranscriptionController {...defaultProps} />);

      // ASSERT - Define what component must provide
      expect(screen.getByTestId('transcription-controller')).toBeInTheDocument();
      expect(screen.getByTestId('start-recording-button')).toBeInTheDocument();
      expect(screen.getByTestId('transcription-settings')).toBeInTheDocument();
      expect(screen.getByTestId('system-status')).toBeInTheDocument();
    });

    it('should initialize with system capability detection', async () => {
      // ACT
      render(<TranscriptionController {...defaultProps} />);

      // ASSERT - Should detect system capabilities
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('get_system_info');
      });

      expect(screen.getByTestId('system-capabilities')).toBeInTheDocument();
      expect(screen.getByTestId('recommended-tier')).toHaveTextContent('standard');
    });

    it('should apply initial configuration correctly', () => {
      // ARRANGE
      const customConfig = {
        qualityTier: 'high-accuracy' as const,
        languages: ['en', 'ja'],
        enableSpeakerDiarization: false,
      };

      // ACT
      render(
        <TranscriptionController 
          {...defaultProps} 
          initialConfig={customConfig}
        />
      );

      // ASSERT - Configuration should be reflected in UI
      expect(screen.getByTestId('quality-tier-selector')).toHaveValue('high-accuracy');
      expect(screen.getByTestId('language-en')).toBeChecked();
      expect(screen.getByTestId('language-ja')).toBeChecked();
      expect(screen.getByTestId('speaker-diarization-toggle')).not.toBeChecked();
    });

    it('should show hardware compatibility warnings when needed', async () => {
      // ARRANGE - Mock insufficient hardware
      mockInvoke.mockImplementation((command: string) => {
        if (command === 'get_system_info') {
          return Promise.resolve({
            recommendedTier: 'standard',
            availableMemoryGB: 4, // Low memory
            hasGPU: false,
            cpuCores: 4,
            warnings: ['insufficient_memory_for_high_accuracy'],
          });
        }
        return Promise.resolve();
      });

      // ACT
      render(<TranscriptionController {...defaultProps} />);

      // ASSERT - Should show compatibility warnings
      await waitFor(() => {
        expect(screen.getByTestId('compatibility-warning')).toBeInTheDocument();
        expect(screen.getByText(/insufficient memory/i)).toBeInTheDocument();
      });
    });
  });

  describe('Transcription Session Management', () => {
    it('should start transcription session correctly', async () => {
      // ARRANGE
      const user = userEvent.setup();
      render(<TranscriptionController {...defaultProps} />);

      // ACT - Click start recording
      const startButton = screen.getByTestId('start-recording-button');
      await user.click(startButton);

      // ASSERT - Should start transcription session
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('start_transcription', {
          config: expect.objectContaining({
            qualityTier: 'standard',
            languages: ['en'],
            enableSpeakerDiarization: true,
          }),
        });
      });

      expect(defaultProps.onSessionStart).toHaveBeenCalledWith('session-123');
      expect(screen.getByTestId('recording-active-indicator')).toBeInTheDocument();
      expect(startButton).toBeDisabled();
      expect(screen.getByTestId('stop-recording-button')).toBeEnabled();
    });

    it('should handle transcription start failures gracefully', async () => {
      // ARRANGE
      mockInvoke.mockImplementation((command: string) => {
        if (command === 'start_transcription') {
          return Promise.reject(new Error('Audio capture failed'));
        }
        return Promise.resolve();
      });

      const user = userEvent.setup();
      render(<TranscriptionController {...defaultProps} />);

      // ACT
      const startButton = screen.getByTestId('start-recording-button');
      await user.click(startButton);

      // ASSERT - Should handle error gracefully
      await waitFor(() => {
        expect(defaultProps.onError).toHaveBeenCalledWith(
          expect.objectContaining({
            message: 'Audio capture failed',
            type: 'transcription_start_failed',
          })
        );
      });

      expect(screen.getByTestId('error-message')).toBeInTheDocument();
      expect(screen.getByText(/audio capture failed/i)).toBeInTheDocument();
      expect(startButton).toBeEnabled(); // Should re-enable start button
    });

    it('should stop transcription session and return results', async () => {
      // ARRANGE
      const user = userEvent.setup();
      render(<TranscriptionController {...defaultProps} />);

      // Start session first
      await user.click(screen.getByTestId('start-recording-button'));
      await waitFor(() => {
        expect(screen.getByTestId('recording-active-indicator')).toBeInTheDocument();
      });

      // ACT - Stop recording
      const stopButton = screen.getByTestId('stop-recording-button');
      await user.click(stopButton);

      // ASSERT - Should stop session and return results
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('stop_transcription', {
          sessionId: 'session-123',
        });
      });

      expect(defaultProps.onSessionEnd).toHaveBeenCalledWith(
        expect.objectContaining({
          sessionId: 'session-123',
          segments: expect.any(Array),
          qualityMetrics: expect.any(Object),
        })
      );

      expect(screen.queryByTestId('recording-active-indicator')).not.toBeInTheDocument();
      expect(screen.getByTestId('start-recording-button')).toBeEnabled();
      expect(stopButton).toBeDisabled();
    });

    it('should prevent multiple concurrent sessions', async () => {
      // ARRANGE
      const user = userEvent.setup();
      render(<TranscriptionController {...defaultProps} />);

      // Start first session
      await user.click(screen.getByTestId('start-recording-button'));
      await waitFor(() => {
        expect(screen.getByTestId('recording-active-indicator')).toBeInTheDocument();
      });

      // ACT - Try to start another session
      const startButton = screen.getByTestId('start-recording-button');
      expect(startButton).toBeDisabled();

      // ASSERT - Button should remain disabled during active session
      await user.click(startButton); // Should not trigger anything
      expect(mockInvoke).toHaveBeenCalledTimes(2); // Only initial start + system info calls
    });
  });

  describe('Real-time Updates', () => {
    it('should listen for real-time transcription updates', async () => {
      // ARRANGE
      const user = userEvent.setup();
      render(<TranscriptionController {...defaultProps} />);

      // Start session to enable listening
      await user.click(screen.getByTestId('start-recording-button'));

      // ASSERT - Should set up event listeners
      await waitFor(() => {
        expect(mockListen).toHaveBeenCalledWith(
          'transcription-update',
          expect.any(Function)
        );
        expect(mockListen).toHaveBeenCalledWith(
          'system-status',
          expect.any(Function)
        );
        expect(mockListen).toHaveBeenCalledWith(
          'transcription-error',
          expect.any(Function)
        );
      });
    });

    it('should handle real-time transcription updates', async () => {
      // ARRANGE
      let transcriptionCallback: Function;
      mockListen.mockImplementation((event: string, callback: Function) => {
        if (event === 'transcription-update') {
          transcriptionCallback = callback;
        }
        return Promise.resolve(() => {});
      });

      const user = userEvent.setup();
      render(<TranscriptionController {...defaultProps} />);
      await user.click(screen.getByTestId('start-recording-button'));

      // ACT - Simulate real-time update
      const updateEvent = {
        sessionId: 'session-123',
        segment: TranscriptionTestFactory.createBasicTranscriptionSegments()[0],
        updateType: 'new',
        processingPass: 1,
      };

      act(() => {
        transcriptionCallback(updateEvent);
      });

      // ASSERT - Should handle update and notify parent
      await waitFor(() => {
        expect(defaultProps.onTranscriptionUpdate).toHaveBeenCalledWith(updateEvent);
      });

      expect(screen.getByTestId('latest-transcription')).toHaveTextContent(
        'Good morning everyone'
      );
    });

    it('should display system status updates', async () => {
      // ARRANGE
      let statusCallback: Function;
      mockListen.mockImplementation((event: string, callback: Function) => {
        if (event === 'system-status') {
          statusCallback = callback;
        }
        return Promise.resolve(() => {});
      });

      const user = userEvent.setup();
      render(<TranscriptionController {...defaultProps} />);
      await user.click(screen.getByTestId('start-recording-button'));

      // ACT - Simulate system status update
      const statusEvent = {
        thermalStatus: {
          temperature: 75.0,
          riskLevel: 'medium',
        },
        memoryUsage: {
          used: 6442450944, // 6GB
          available: 10737418240, // 10GB
          percentage: 60,
        },
        processingMetrics: {
          realTimeFactor: 0.8,
          averageLatency: 1200,
          queuedSegments: 2,
        },
      };

      act(() => {
        statusCallback(statusEvent);
      });

      // ASSERT - Should display system metrics
      await waitFor(() => {
        expect(screen.getByTestId('thermal-status')).toHaveTextContent('75°C');
        expect(screen.getByTestId('memory-usage')).toHaveTextContent('60%');
        expect(screen.getByTestId('real-time-factor')).toHaveTextContent('0.8');
      });
    });

    it('should handle system warnings and errors', async () => {
      // ARRANGE
      let errorCallback: Function;
      mockListen.mockImplementation((event: string, callback: Function) => {
        if (event === 'transcription-error') {
          errorCallback = callback;
        }
        return Promise.resolve(() => {});
      });

      const user = userEvent.setup();
      render(<TranscriptionController {...defaultProps} />);
      await user.click(screen.getByTestId('start-recording-button'));

      // ACT - Simulate thermal throttling warning
      const errorEvent = {
        sessionId: 'session-123',
        errorType: 'thermal_throttle',
        severity: 'warning',
        message: 'System temperature high, reducing quality to Standard tier',
        suggestedAction: 'Consider reducing quality settings',
        timestamp: Date.now(),
      };

      act(() => {
        errorCallback(errorEvent);
      });

      // ASSERT - Should show warning and suggested action
      await waitFor(() => {
        expect(screen.getByTestId('system-warning')).toBeInTheDocument();
        expect(screen.getByText(/temperature high/i)).toBeInTheDocument();
        expect(screen.getByText(/reducing quality/i)).toBeInTheDocument();
      });

      expect(screen.getByTestId('suggested-action')).toHaveTextContent(
        'Consider reducing quality settings'
      );
    });
  });

  describe('Configuration Management', () => {
    it('should allow quality tier changes when not recording', async () => {
      // ARRANGE
      const user = userEvent.setup();
      render(<TranscriptionController {...defaultProps} />);

      // ACT - Change quality tier
      const qualitySelector = screen.getByTestId('quality-tier-selector');
      await user.selectOptions(qualitySelector, 'high-accuracy');

      // ASSERT - Should update configuration
      expect(qualitySelector).toHaveValue('high-accuracy');
      expect(screen.getByTestId('estimated-requirements')).toHaveTextContent(
        /high accuracy.*16GB RAM/i
      );
    });

    it('should prevent configuration changes during recording', async () => {
      // ARRANGE
      const user = userEvent.setup();
      render(<TranscriptionController {...defaultProps} />);

      // Start recording
      await user.click(screen.getByTestId('start-recording-button'));
      await waitFor(() => {
        expect(screen.getByTestId('recording-active-indicator')).toBeInTheDocument();
      });

      // ACT - Try to change configuration
      const qualitySelector = screen.getByTestId('quality-tier-selector');
      const languageCheckbox = screen.getByTestId('language-ja');

      // ASSERT - Configuration should be disabled
      expect(qualitySelector).toBeDisabled();
      expect(languageCheckbox).toBeDisabled();
      expect(screen.getByTestId('config-locked-message')).toHaveTextContent(
        /settings locked during recording/i
      );
    });

    it('should validate configuration before starting session', async () => {
      // ARRANGE
      const user = userEvent.setup();
      render(<TranscriptionController {...defaultProps} />);

      // Set invalid configuration (no languages selected)
      await user.click(screen.getByTestId('language-en'));
      expect(screen.getByTestId('language-en')).not.toBeChecked();

      // ACT - Try to start recording
      const startButton = screen.getByTestId('start-recording-button');
      await user.click(startButton);

      // ASSERT - Should prevent start and show validation error
      expect(mockInvoke).not.toHaveBeenCalledWith('start_transcription', expect.anything());
      expect(screen.getByTestId('validation-error')).toHaveTextContent(
        /at least one language must be selected/i
      );
      expect(startButton).toBeEnabled();
    });

    it('should save configuration changes for next session', async () => {
      // ARRANGE
      const user = userEvent.setup();
      render(<TranscriptionController {...defaultProps} />);

      // ACT - Change configuration
      await user.selectOptions(screen.getByTestId('quality-tier-selector'), 'turbo');
      await user.click(screen.getByTestId('language-ja'));
      await user.click(screen.getByTestId('speaker-diarization-toggle'));

      // ASSERT - Configuration should be persisted
      // This would typically save to localStorage or similar
      const savedConfig = localStorage.getItem('transcription-config');
      expect(savedConfig).toBeTruthy();
      
      const config = JSON.parse(savedConfig!);
      expect(config.qualityTier).toBe('turbo');
      expect(config.languages).toContain('ja');
      expect(config.enableSpeakerDiarization).toBe(false);
    });
  });

  describe('Error Handling and Recovery', () => {
    it('should handle audio capture failures with recovery options', async () => {
      // ARRANGE
      mockInvoke.mockImplementation((command: string) => {
        if (command === 'start_transcription') {
          return Promise.reject({
            type: 'audio_capture_failed',
            message: 'Microphone access denied',
            recoveryOptions: ['request_permissions', 'use_system_audio'],
          });
        }
        return Promise.resolve();
      });

      const user = userEvent.setup();
      render(<TranscriptionController {...defaultProps} />);

      // ACT
      await user.click(screen.getByTestId('start-recording-button'));

      // ASSERT - Should show error with recovery options
      await waitFor(() => {
        expect(screen.getByTestId('error-dialog')).toBeInTheDocument();
        expect(screen.getByText(/microphone access denied/i)).toBeInTheDocument();
        expect(screen.getByTestId('recovery-request-permissions')).toBeInTheDocument();
        expect(screen.getByTestId('recovery-use-system-audio')).toBeInTheDocument();
      });
    });

    it('should handle model loading failures', async () => {
      // ARRANGE
      mockInvoke.mockImplementation((command: string) => {
        if (command === 'start_transcription') {
          return Promise.reject({
            type: 'model_load_failed',
            message: 'Whisper model not found',
            missingModel: 'whisper-medium',
          });
        }
        return Promise.resolve();
      });

      const user = userEvent.setup();
      render(<TranscriptionController {...defaultProps} />);

      // ACT
      await user.click(screen.getByTestId('start-recording-button'));

      // ASSERT - Should show model-specific error
      await waitFor(() => {
        expect(screen.getByTestId('model-error-dialog')).toBeInTheDocument();
        expect(screen.getByText(/whisper model not found/i)).toBeInTheDocument();
        expect(screen.getByTestId('download-model-button')).toBeInTheDocument();
      });
    });

    it('should recover from temporary processing errors', async () => {
      // ARRANGE
      let errorCallback: Function;
      mockListen.mockImplementation((event: string, callback: Function) => {
        if (event === 'transcription-error') {
          errorCallback = callback;
        }
        return Promise.resolve(() => {});
      });

      const user = userEvent.setup();
      render(<TranscriptionController {...defaultProps} />);
      await user.click(screen.getByTestId('start-recording-button'));

      // ACT - Simulate temporary processing error
      act(() => {
        errorCallback({
          errorType: 'processing_queue_full',
          severity: 'warning',
          message: 'Processing queue full, dropping segments',
          isRecoverable: true,
        });
      });

      // ASSERT - Should show warning but continue processing
      await waitFor(() => {
        expect(screen.getByTestId('processing-warning')).toBeInTheDocument();
        expect(screen.getByText(/processing queue full/i)).toBeInTheDocument();
      });

      // Should not stop the session
      expect(screen.getByTestId('recording-active-indicator')).toBeInTheDocument();
      expect(mockInvoke).not.toHaveBeenCalledWith('stop_transcription', expect.anything());
    });

    it('should handle critical errors by stopping session safely', async () => {
      // ARRANGE
      let errorCallback: Function;
      mockListen.mockImplementation((event: string, callback: Function) => {
        if (event === 'transcription-error') {
          errorCallback = callback;
        }
        return Promise.resolve(() => {});
      });

      const user = userEvent.setup();
      render(<TranscriptionController {...defaultProps} />);
      await user.click(screen.getByTestId('start-recording-button'));

      // ACT - Simulate critical error
      act(() => {
        errorCallback({
          errorType: 'model_crashed',
          severity: 'critical',
          message: 'ASR model encountered fatal error',
          isRecoverable: false,
        });
      });

      // ASSERT - Should stop session and show critical error
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('stop_transcription', {
          sessionId: 'session-123',
        });
      });

      expect(screen.getByTestId('critical-error-dialog')).toBeInTheDocument();
      expect(screen.getByText(/model encountered fatal error/i)).toBeInTheDocument();
      expect(screen.queryByTestId('recording-active-indicator')).not.toBeInTheDocument();
    });
  });

  describe('Performance Monitoring', () => {
    it('should display real-time performance metrics', async () => {
      // ARRANGE
      const user = userEvent.setup();
      render(<TranscriptionController {...defaultProps} />);
      await user.click(screen.getByTestId('start-recording-button'));

      // Simulate performance updates
      const statusCallback = vi.fn();
      mockListen.mockImplementation((event: string, callback: Function) => {
        if (event === 'system-status') {
          statusCallback.mockImplementation(callback);
        }
        return Promise.resolve(() => {});
      });

      // ACT - Send performance update
      act(() => {
        statusCallback({
          processingMetrics: {
            realTimeFactor: 0.75,
            averageLatency: 980,
            queuedSegments: 1,
            cpuUsage: 45.2,
            memoryUsage: 6.8,
          },
        });
      });

      // ASSERT - Should display metrics
      await waitFor(() => {
        expect(screen.getByTestId('rtf-display')).toHaveTextContent('0.75x');
        expect(screen.getByTestId('latency-display')).toHaveTextContent('980ms');
        expect(screen.getByTestId('cpu-usage')).toHaveTextContent('45.2%');
        expect(screen.getByTestId('memory-usage')).toHaveTextContent('6.8GB');
      });
    });

    it('should show performance warnings when thresholds exceeded', async () => {
      // ARRANGE
      const user = userEvent.setup();
      render(<TranscriptionController {...defaultProps} />);
      await user.click(screen.getByTestId('start-recording-button'));

      // ACT - Send high latency update
      const statusCallback = vi.fn();
      act(() => {
        statusCallback({
          processingMetrics: {
            realTimeFactor: 1.2, // Above real-time
            averageLatency: 3500, // High latency
            queuedSegments: 8,    // Queue backing up
          },
        });
      });

      // ASSERT - Should show performance warnings
      await waitFor(() => {
        expect(screen.getByTestId('performance-warning')).toBeInTheDocument();
        expect(screen.getByText(/processing slower than real-time/i)).toBeInTheDocument();
        expect(screen.getByText(/high latency detected/i)).toBeInTheDocument();
      });

      expect(screen.getByTestId('performance-suggestions')).toHaveTextContent(
        /consider reducing quality tier/i
      );
    });
  });
});

/*
COMPONENT CONTRACT DEFINITION:
============================

The TranscriptionController component must provide:

Required Props:
- onSessionStart: (sessionId: string) => void
- onSessionEnd: (result: FinalTranscriptionResult) => void  
- onError: (error: TranscriptionError) => void
- onTranscriptionUpdate?: (update: TranscriptionUpdateEvent) => void
- initialConfig?: Partial<TranscriptionConfig>

Component Features Required:

1. Session Management:
   - Start/stop transcription sessions
   - Prevent multiple concurrent sessions
   - Handle session failures gracefully
   - Clean session termination

2. Configuration Interface:
   - Quality tier selection (Standard/High Accuracy/Turbo)
   - Language selection (multiple languages supported)
   - Audio source configuration (mic/system audio)
   - Speaker diarization toggle
   - Two-pass refinement toggle
   - VAD threshold adjustment

3. Real-time Updates:
   - Listen for transcription updates
   - Display latest transcription text
   - System status monitoring
   - Performance metrics display

4. Error Handling:
   - Audio capture failures with recovery options
   - Model loading errors with solutions
   - Processing errors (temporary vs critical)
   - System resource warnings

5. System Integration:
   - Hardware capability detection
   - Performance monitoring and warnings
   - Thermal management notifications
   - Memory usage tracking

6. User Interface Elements:
   - Start/stop recording buttons
   - Configuration panels (locked during recording)
   - Real-time status displays
   - Error dialogs with recovery options
   - Performance metrics dashboard

7. State Management:
   - Configuration persistence
   - Session state tracking
   - Error state handling
   - Performance state monitoring

Tauri Commands Used:
- start_transcription(config) -> session_id
- stop_transcription(session_id) -> FinalResult
- get_system_info() -> SystemCapabilities

Event Listeners:
- transcription-update: Real-time transcription updates
- system-status: Performance and resource monitoring
- transcription-error: Error notifications and warnings

All tests should FAIL initially - this is correct TDD behavior.
*/