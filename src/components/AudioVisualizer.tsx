/**
 * AudioVisualizer Component
 * 
 * Real-time audio visualization with waveform display, level meters, 
 * recording indicators, and VAD activity detection.
 */

import React, { useEffect, useRef, useState, useCallback, useMemo } from 'react';
import WaveSurfer from 'wavesurfer.js';

export interface AudioData {
  sampleRate: number;
  channels: 1 | 2;
  samples: Float32Array;
  timestamp: number;
  sourceChannel: 'microphone' | 'system' | 'mixed' | 'unknown';
  durationSeconds: number;
}

export interface AudioVisualizerProps {
  audioLevel: number; // 0-1
  isRecording: boolean;
  vadActivity: boolean;
  showWaveform: boolean;
  height?: number;
  width?: number;
  waveformColor?: string;
  progressColor?: string;
  audioData?: AudioData;
  showPeakIndicators?: boolean;
  onPlaybackToggle?: (playing: boolean) => void;
  onSeek?: (time: number) => void;
}

interface PeakLevel {
  level: number;
  timestamp: number;
}

export const AudioVisualizer: React.FC<AudioVisualizerProps> = ({
  audioLevel,
  isRecording,
  vadActivity,
  showWaveform,
  height = 100,
  width = 800,
  waveformColor = '#3b82f6',
  progressColor = '#1e40af',
  audioData,
  showPeakIndicators = false,
  onPlaybackToggle,
  onSeek,
}) => {
  const waveformRef = useRef<HTMLDivElement>(null);
  const wavesurfer = useRef<WaveSurfer | null>(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const [hasWaveformError, setHasWaveformError] = useState(false);
  const [peakHistory, setPeakHistory] = useState<PeakLevel[]>([]);
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [showClippingWarning, setShowClippingWarning] = useState(false);

  // Clamp audio level to valid range (0-1)
  const clampedLevel = useMemo(() => Math.max(0, Math.min(1, audioLevel)), [audioLevel]);
  
  // Detect clipping
  const isClipping = clampedLevel >= 0.9;

  // Update peak history for peak indicators
  useEffect(() => {
    if (showPeakIndicators && clampedLevel > 0) {
      const newPeak: PeakLevel = {
        level: clampedLevel,
        timestamp: Date.now(),
      };
      
      setPeakHistory(prev => {
        const updated = [...prev, newPeak];
        // Keep only recent peaks (last 2 seconds)
        const cutoff = Date.now() - 2000;
        return updated.filter(peak => peak.timestamp > cutoff);
      });
    }
  }, [clampedLevel, showPeakIndicators]);

  // Show clipping warning
  useEffect(() => {
    if (isClipping) {
      setShowClippingWarning(true);
      const timer = setTimeout(() => setShowClippingWarning(false), 2000);
      return () => clearTimeout(timer);
    }
  }, [isClipping]);

  // Initialize WaveSurfer
  useEffect(() => {
    if (showWaveform && waveformRef.current && !wavesurfer.current && !hasWaveformError) {
      try {
        wavesurfer.current = WaveSurfer.create({
          container: waveformRef.current,
          waveColor: waveformColor,
          progressColor: progressColor,
          height: height,
          normalize: true,
          interact: true,
        });

        // Set up event listeners
        wavesurfer.current.on('ready', () => {
          setDuration(wavesurfer.current?.getDuration() || 0);
        });

        wavesurfer.current.on('audioprocess', () => {
          setCurrentTime(wavesurfer.current?.getCurrentTime() || 0);
        });

        wavesurfer.current.on('play', () => {
          setIsPlaying(true);
        });

        wavesurfer.current.on('pause', () => {
          setIsPlaying(false);
        });

        wavesurfer.current.on('click', () => {
          if (onSeek) {
            onSeek(wavesurfer.current?.getCurrentTime() || 0);
          }
        });

      } catch (error) {
        console.error('Failed to initialize WaveSurfer:', error);
        setHasWaveformError(true);
      }
    }

    return () => {
      if (wavesurfer.current) {
        wavesurfer.current.destroy();
        wavesurfer.current = null;
      }
    };
  }, [showWaveform, waveformColor, progressColor, height, onSeek, hasWaveformError]);

  // Load audio data when provided
  useEffect(() => {
    if (wavesurfer.current && audioData) {
      try {
        // Convert AudioData to blob URL for WaveSurfer
        const audioBuffer = new ArrayBuffer(audioData.samples.length * 4);
        const view = new Float32Array(audioBuffer);
        view.set(audioData.samples);
        
        const blob = new Blob([audioBuffer], { type: 'audio/wav' });
        const url = URL.createObjectURL(blob);
        
        wavesurfer.current.load(url);
      } catch (error) {
        console.error('Failed to load audio data:', error);
      }
    }
  }, [audioData]);

  const handlePlaybackToggle = useCallback(() => {
    if (wavesurfer.current) {
      if (isPlaying) {
        wavesurfer.current.pause();
      } else {
        wavesurfer.current.play();
      }
      
      if (onPlaybackToggle) {
        onPlaybackToggle(!isPlaying);
      }
    }
  }, [isPlaying, onPlaybackToggle]);

  const formatTime = (time: number) => {
    return `${Math.floor(time)}s`;
  };

  const renderLevelMeters = () => {
    if (showWaveform) return null;

    return (
      <div 
        data-testid="level-meters"
        className="level-meters"
        style={{ 
          width: width, 
          height: height,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          backgroundColor: '#1a1a1a',
          borderRadius: '4px',
          position: 'relative'
        }}
      >
        <div className="level-meter-container" style={{ width: '80%', height: '60%' }}>
          <div
            data-testid="audio-level-meter"
            data-level={clampedLevel}
            className={`level-meter ${isClipping ? 'clipping-warning' : ''}`}
            style={{
              width: '100%',
              height: '100%',
              backgroundColor: '#333',
              borderRadius: '4px',
              overflow: 'hidden',
              position: 'relative'
            }}
          >
            <div
              data-testid="level-bar"
              className="level-bar"
              style={{
                width: `${clampedLevel * 100}%`,
                height: '100%',
                backgroundColor: isClipping ? '#ef4444' : '#10b981',
                transition: 'width 0.1s ease-out',
                borderRadius: '4px'
              }}
            />
          </div>
        </div>
        
        {showPeakIndicators && (
          <div data-testid="peak-indicators" className="peak-indicators">
            {peakHistory.slice(-10).map((peak, index) => (
              <div
                key={peak.timestamp}
                data-testid={`peak-bar-${index}`}
                className="peak-bar"
                style={{
                  position: 'absolute',
                  right: `${index * 4}px`,
                  top: '50%',
                  transform: 'translateY(-50%)',
                  width: '2px',
                  height: `${peak.level * 60}%`,
                  backgroundColor: '#6366f1',
                  opacity: Math.max(0.2, 1 - (index * 0.1))
                }}
              />
            ))}
          </div>
        )}
      </div>
    );
  };

  const renderWaveform = () => {
    if (!showWaveform) return null;

    if (hasWaveformError) {
      return (
        <div
          data-testid="waveform-error-fallback"
          className="waveform-error"
          style={{
            width: width,
            height: height,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            backgroundColor: '#1a1a1a',
            color: '#666',
            borderRadius: '4px'
          }}
        >
          <span>Waveform unavailable</span>
        </div>
      );
    }

    return (
      <div className="waveform-section">
        <div
          ref={waveformRef}
          data-testid="waveform-container"
          aria-label="Audio waveform visualization"
          style={{ width: width, height: height }}
          onClick={() => onSeek && wavesurfer.current && onSeek(wavesurfer.current.getCurrentTime())}
        />
        
        <div className="waveform-controls" style={{ marginTop: '8px', display: 'flex', alignItems: 'center', gap: '12px' }}>
          <button
            data-testid="waveform-play-button"
            aria-label={isPlaying ? 'Pause audio' : 'Play audio'}
            onClick={handlePlaybackToggle}
            style={{
              padding: '8px 12px',
              backgroundColor: '#3b82f6',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer'
            }}
          >
            {isPlaying ? '⏸️' : '▶️'}
          </button>
          
          <div data-testid="playback-progress" className="time-display">
            {formatTime(currentTime)} / {formatTime(duration)}
          </div>
          
          <div
            data-testid="progress-bar"
            data-progress={duration > 0 ? currentTime / duration : 0}
            style={{
              flex: 1,
              height: '4px',
              backgroundColor: '#333',
              borderRadius: '2px',
              overflow: 'hidden'
            }}
          >
            <div
              style={{
                width: `${duration > 0 ? (currentTime / duration) * 100 : 0}%`,
                height: '100%',
                backgroundColor: progressColor,
                transition: 'width 0.1s linear'
              }}
            />
          </div>
        </div>
      </div>
    );
  };

  const renderNoAudioState = () => {
    if (audioData !== undefined) return null;

    return (
      <div
        data-testid="no-audio-state"
        style={{
          width: width,
          height: height,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          backgroundColor: '#1a1a1a',
          color: '#666',
          borderRadius: '4px'
        }}
      >
        <span>No audio data available</span>
      </div>
    );
  };

  return (
    <div
      data-testid="audio-visualizer"
      data-width={width}
      data-height={height}
      className="audio-visualizer"
      style={{ position: 'relative' }}
    >
      {/* Recording Indicator */}
      <div
        data-testid="recording-indicator"
        className={`recording-indicator ${isRecording ? 'recording-active' : ''}`}
        aria-label={isRecording ? 'Recording active' : 'Recording inactive'}
        style={{
          position: 'absolute',
          top: '8px',
          left: '8px',
          width: '12px',
          height: '12px',
          borderRadius: '50%',
          backgroundColor: isRecording ? '#ef4444' : '#6b7280',
          zIndex: 10,
          animation: isRecording ? 'pulse 1s infinite' : 'none'
        }}
      />

      {/* VAD Activity Indicator */}
      <div
        data-testid="vad-indicator"
        className={`vad-indicator ${vadActivity ? 'vad-active' : ''}`}
        aria-label={vadActivity ? 'Speech detected' : 'No speech detected'}
        style={{
          position: 'absolute',
          top: '8px',
          right: '8px',
          width: '12px',
          height: '12px',
          borderRadius: '50%',
          backgroundColor: vadActivity ? '#10b981' : '#6b7280',
          zIndex: 10
        }}
      />

      {/* Audio Level Label */}
      <div
        aria-label={`Audio level ${Math.round(clampedLevel * 100)}%`}
        style={{ position: 'absolute', left: '-9999px' }}
      >
        Audio level: {Math.round(clampedLevel * 100)}%
      </div>

      {/* Clipping Warning */}
      {showClippingWarning && (
        <>
          <div
            data-testid="clipping-indicator"
            aria-label="Audio clipping detected"
            style={{
              position: 'absolute',
              top: '8px',
              left: '50%',
              transform: 'translateX(-50%)',
              padding: '4px 8px',
              backgroundColor: '#ef4444',
              color: 'white',
              borderRadius: '4px',
              fontSize: '12px',
              zIndex: 10
            }}
          >
            CLIPPING
          </div>
          <div role="alert" style={{ position: 'absolute', left: '-9999px' }}>
            Clipping detected - audio level too high
          </div>
        </>
      )}

      {/* Main Visualization Area */}
      <div style={{ marginTop: '32px' }}>
        {renderNoAudioState()}
        {renderLevelMeters()}
        {renderWaveform()}
      </div>

      {/* CSS Animation for recording pulse */}
      <style>{`
        @keyframes pulse {
          0%, 100% { opacity: 1; }
          50% { opacity: 0.5; }
        }
      `}</style>
    </div>
  );
};

export default AudioVisualizer;