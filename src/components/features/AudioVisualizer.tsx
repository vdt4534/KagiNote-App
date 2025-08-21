/**
 * AudioVisualizer Component
 * 
 * Real-time audio visualization with waveform display, level meters, 
 * recording indicators, and VAD activity detection.
 * Updated to use the new design system.
 */

import React, { useEffect, useRef, useState, useCallback, useMemo } from 'react';
import WaveSurfer from 'wavesurfer.js';
import { cn } from '@/lib/utils';
import { Icon } from '@/components/ui/Icon';
import { Badge } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';

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
  className?: string;
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
  className,
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

    // Ultra-compact horizontal meter design
    return (
      <div 
        data-testid="level-meters"
        className="level-meters relative flex items-center w-full"
        style={{ 
          height: Math.min(height, 32)
        }}
      >
        <div className="flex items-center gap-2 w-full">
          {/* Level meter bar */}
          <div className="relative flex-1">
            <div
              data-testid="audio-level-meter"
              data-level={clampedLevel}
              className={cn(
                "relative w-full h-5 bg-neutral-200 dark:bg-neutral-800 rounded-full overflow-hidden",
                isClipping && "ring-1 ring-error-500"
              )}
            >
              <div
                data-testid="level-bar"
                className={cn(
                  "absolute left-0 top-0 h-full transition-all duration-75 ease-out rounded-full",
                  isClipping 
                    ? "bg-error-500" 
                    : vadActivity 
                      ? "bg-secondary-500"
                      : "bg-neutral-400"
                )}
                style={{
                  width: `${clampedLevel * 100}%`
                }}
              />
              {/* Subtle grid markers */}
              {[25, 50, 75].map((percent) => (
                <div 
                  key={percent}
                  className="absolute h-full w-px bg-neutral-300 dark:bg-neutral-700 opacity-20"
                  style={{ left: `${percent}%` }}
                />
              ))}
            </div>
          </div>
          
          {/* Voice indicator and percentage */}
          <div className="flex items-center gap-2">
            <div className={cn(
              "w-1.5 h-1.5 rounded-full transition-colors",
              vadActivity ? "bg-secondary-500" : "bg-neutral-300 dark:bg-neutral-600"
            )} />
            <span className="text-xs font-mono text-neutral-500 dark:text-neutral-400 min-w-[3ch]">
              {Math.round(clampedLevel * 100)}%
            </span>
          </div>
        </div>
      </div>
    );
  };

  const renderWaveform = () => {
    if (!showWaveform) return null;

    if (hasWaveformError) {
      return (
        <div
          data-testid="waveform-error-fallback"
          className="flex items-center justify-center bg-neutral-50 dark:bg-neutral-900 text-neutral-500 dark:text-neutral-400 rounded-lg"
          style={{
            width: width,
            height: Math.min(height, 60)
          }}
        >
          <span className="text-sm">Waveform unavailable</span>
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
        className="flex items-center justify-center text-neutral-500 dark:text-neutral-400 w-full"
        style={{
          height: Math.min(height, 32)
        }}
      >
        <span className="text-xs">No audio data available</span>
      </div>
    );
  };

  return (
    <div
      data-testid="audio-visualizer"
      data-width={width}
      data-height={height}
      className={cn("audio-visualizer", className)}
    >
      {/* Audio Level Label for screen readers */}
      <div
        aria-label={`Audio level ${Math.round(clampedLevel * 100)}%`}
        className="sr-only"
      >
        Audio level: {Math.round(clampedLevel * 100)}%
      </div>

      {/* Main Visualization Area */}
      {renderNoAudioState()}
      {renderLevelMeters()}
      {renderWaveform()}

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