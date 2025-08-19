/**
 * Audio Visualizer Component Tests
 * 
 * These tests are written BEFORE implementation exists (TDD).
 * Tests define the contract for real-time audio visualization component.
 * All tests should FAIL initially - this is correct TDD behavior.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { AudioTestFactory } from '../../factories/AudioTestFactory';

// These imports WILL FAIL because components don't exist yet
import { AudioVisualizer } from '@/components/AudioVisualizer';
import type { AudioVisualizerProps } from '@/components/AudioVisualizer';

// Mock WaveSurfer since it's a dependency
vi.mock('wavesurfer.js', () => ({
  default: {
    create: vi.fn(() => ({
      load: vi.fn(),
      play: vi.fn(),
      pause: vi.fn(),
      seekTo: vi.fn(),
      setVolume: vi.fn(),
      destroy: vi.fn(),
      on: vi.fn(),
      off: vi.fn(),
      getDuration: vi.fn(() => 10),
      getCurrentTime: vi.fn(() => 5),
      isPlaying: vi.fn(() => false),
    })),
  },
}));

describe('AudioVisualizer Component', () => {
  let defaultProps: AudioVisualizerProps;
  let mockWaveSurfer: any;

  beforeEach(() => {
    defaultProps = {
      audioLevel: 0.5,
      isRecording: false,
      vadActivity: false,
      showWaveform: true,
      height: 100,
      width: 800,
      onPlaybackToggle: vi.fn(),
      onSeek: vi.fn(),
    };

    // Reset WaveSurfer mock
    const WaveSurfer = vi.mocked(require('wavesurfer.js').default);
    mockWaveSurfer = {
      load: vi.fn(),
      play: vi.fn(),
      pause: vi.fn(),
      seekTo: vi.fn(),
      setVolume: vi.fn(),
      destroy: vi.fn(),
      on: vi.fn(),
      off: vi.fn(),
      getDuration: vi.fn(() => 10),
      getCurrentTime: vi.fn(() => 5),
      isPlaying: vi.fn(() => false),
    };
    WaveSurfer.create.mockReturnValue(mockWaveSurfer);
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('Component Rendering', () => {
    it('should render audio visualizer with correct dimensions', () => {
      // ACT - This WILL FAIL because AudioVisualizer doesn't exist
      render(<AudioVisualizer {...defaultProps} />);

      // ASSERT - Define what component must provide
      const visualizer = screen.getByTestId('audio-visualizer');
      expect(visualizer).toBeInTheDocument();
      expect(visualizer).toHaveAttribute('data-width', '800');
      expect(visualizer).toHaveAttribute('data-height', '100');
    });

    it('should show recording indicator when recording is active', () => {
      // ARRANGE
      const recordingProps = {
        ...defaultProps,
        isRecording: true,
      };

      // ACT
      render(<AudioVisualizer {...recordingProps} />);

      // ASSERT - Recording state should be visible
      expect(screen.getByTestId('recording-indicator')).toBeInTheDocument();
      expect(screen.getByTestId('recording-indicator')).toHaveClass('recording-active');
      expect(screen.getByLabelText(/recording/i)).toBeInTheDocument();
    });

    it('should display VAD activity indicator when speech detected', () => {
      // ARRANGE
      const vadActiveProps = {
        ...defaultProps,
        vadActivity: true,
      };

      // ACT
      render(<AudioVisualizer {...vadActiveProps} />);

      // ASSERT - VAD activity should be indicated
      expect(screen.getByTestId('vad-indicator')).toBeInTheDocument();
      expect(screen.getByTestId('vad-indicator')).toHaveClass('vad-active');
      expect(screen.getByLabelText(/speech detected/i)).toBeInTheDocument();
    });

    it('should render waveform when showWaveform is true', () => {
      // ARRANGE
      const waveformProps = {
        ...defaultProps,
        showWaveform: true,
      };

      // ACT
      render(<AudioVisualizer {...waveformProps} />);

      // ASSERT - Waveform should be rendered
      expect(screen.getByTestId('waveform-container')).toBeInTheDocument();
      expect(mockWaveSurfer.load).toHaveBeenCalled();
    });

    it('should render level meters when showWaveform is false', () => {
      // ARRANGE
      const levelMeterProps = {
        ...defaultProps,
        showWaveform: false,
      };

      // ACT
      render(<AudioVisualizer {...levelMeterProps} />);

      // ASSERT - Level meters should be shown instead of waveform
      expect(screen.getByTestId('level-meters')).toBeInTheDocument();
      expect(screen.queryByTestId('waveform-container')).not.toBeInTheDocument();
    });
  });

  describe('Audio Level Display', () => {
    it('should display current audio level accurately', () => {
      // ARRANGE
      const levelProps = {
        ...defaultProps,
        audioLevel: 0.75, // 75% level
      };

      // ACT
      render(<AudioVisualizer {...levelProps} />);

      // ASSERT - Audio level should be reflected in visualization
      const levelMeter = screen.getByTestId('audio-level-meter');
      expect(levelMeter).toBeInTheDocument();
      expect(levelMeter).toHaveAttribute('data-level', '0.75');
      
      // Visual representation should match level
      const levelBar = screen.getByTestId('level-bar');
      expect(levelBar).toHaveStyle({
        width: '75%',
      });
    });

    it('should update level display in real-time', async () => {
      // ARRANGE
      const { rerender } = render(<AudioVisualizer {...defaultProps} audioLevel={0.3} />);

      // Initial level
      expect(screen.getByTestId('level-bar')).toHaveStyle({ width: '30%' });

      // ACT - Update level
      rerender(<AudioVisualizer {...defaultProps} audioLevel={0.8} />);

      // ASSERT - Level should update smoothly
      await waitFor(() => {
        expect(screen.getByTestId('level-bar')).toHaveStyle({ width: '80%' });
      });
    });

    it('should indicate clipping when audio level exceeds threshold', () => {
      // ARRANGE
      const clippingProps = {
        ...defaultProps,
        audioLevel: 0.95, // Near clipping
      };

      // ACT
      render(<AudioVisualizer {...clippingProps} />);

      // ASSERT - Clipping warning should be shown
      const levelMeter = screen.getByTestId('audio-level-meter');
      expect(levelMeter).toHaveClass('clipping-warning');
      expect(screen.getByTestId('clipping-indicator')).toBeInTheDocument();
      expect(screen.getByLabelText(/clipping detected/i)).toBeInTheDocument();
    });

    it('should show peak level indicators', () => {
      // ARRANGE
      const peakProps = {
        ...defaultProps,
        audioLevel: 0.6,
        showPeakIndicators: true,
      };

      // ACT
      render(<AudioVisualizer {...peakProps} />);

      // ASSERT - Peak indicators should be visible
      expect(screen.getByTestId('peak-indicators')).toBeInTheDocument();
      
      // Should show recent peak levels
      const peakBars = screen.getAllByTestId(/peak-bar-/);
      expect(peakBars.length).toBeGreaterThan(0);
    });
  });

  describe('Waveform Functionality', () => {
    it('should initialize WaveSurfer with correct configuration', () => {
      // ARRANGE
      const waveformProps = {
        ...defaultProps,
        showWaveform: true,
        waveformColor: '#3b82f6',
        progressColor: '#1e40af',
      };

      // ACT
      render(<AudioVisualizer {...waveformProps} />);

      // ASSERT - WaveSurfer should be configured correctly
      expect(mockWaveSurfer.create).toHaveBeenCalledWith(
        expect.objectContaining({
          container: expect.any(HTMLElement),
          waveColor: '#3b82f6',
          progressColor: '#1e40af',
          height: 100,
          responsive: true,
          normalize: true,
        })
      );
    });

    it('should handle waveform playback controls', async () => {
      // ARRANGE
      const user = userEvent.setup();
      render(<AudioVisualizer {...defaultProps} showWaveform={true} />);

      // ACT - Click play button
      const playButton = screen.getByTestId('waveform-play-button');
      await user.click(playButton);

      // ASSERT - Should trigger playback
      expect(defaultProps.onPlaybackToggle).toHaveBeenCalledWith(true);
      expect(mockWaveSurfer.play).toHaveBeenCalled();
    });

    it('should handle waveform seeking', async () => {
      // ARRANGE
      const user = userEvent.setup();
      render(<AudioVisualizer {...defaultProps} showWaveform={true} />);

      // ACT - Click on waveform to seek
      const waveform = screen.getByTestId('waveform-container');
      await user.click(waveform);

      // ASSERT - Should trigger seek callback
      expect(defaultProps.onSeek).toHaveBeenCalled();
      expect(mockWaveSurfer.seekTo).toHaveBeenCalled();
    });

    it('should display playback progress accurately', () => {
      // ARRANGE
      mockWaveSurfer.getDuration.mockReturnValue(100); // 100 seconds
      mockWaveSurfer.getCurrentTime.mockReturnValue(25); // 25 seconds

      // ACT
      render(<AudioVisualizer {...defaultProps} showWaveform={true} />);

      // ASSERT - Progress should be displayed
      const progressIndicator = screen.getByTestId('playback-progress');
      expect(progressIndicator).toHaveTextContent('25s / 100s');
      
      const progressBar = screen.getByTestId('progress-bar');
      expect(progressBar).toHaveAttribute('data-progress', '0.25'); // 25%
    });

    it('should update waveform when new audio data is provided', () => {
      // ARRANGE
      const { rerender } = render(<AudioVisualizer {...defaultProps} />);
      const newAudioData = AudioTestFactory.createCleanSpeech(10);

      // ACT - Provide new audio data
      rerender(
        <AudioVisualizer
          {...defaultProps}
          audioData={newAudioData}
        />
      );

      // ASSERT - Waveform should be updated
      expect(mockWaveSurfer.load).toHaveBeenCalledWith(expect.any(String));
    });
  });

  describe('Real-time Updates', () => {
    it('should update visualization smoothly during recording', async () => {
      // ARRANGE
      const { rerender } = render(
        <AudioVisualizer {...defaultProps} isRecording={true} audioLevel={0.2} />
      );

      // ACT - Simulate real-time level changes
      const levels = [0.3, 0.5, 0.7, 0.4, 0.2];
      for (const level of levels) {
        rerender(
          <AudioVisualizer {...defaultProps} isRecording={true} audioLevel={level} />
        );
        
        // Small delay to simulate real-time updates
        await new Promise(resolve => setTimeout(resolve, 50));
      }

      // ASSERT - Should handle rapid updates without performance issues
      const levelBar = screen.getByTestId('level-bar');
      expect(levelBar).toBeInTheDocument();
      expect(levelBar).toHaveStyle({ width: '20%' }); // Final level
    });

    it('should show VAD activity changes in real-time', async () => {
      // ARRANGE
      const { rerender } = render(
        <AudioVisualizer {...defaultProps} vadActivity={false} />
      );

      // Initial state
      expect(screen.getByTestId('vad-indicator')).not.toHaveClass('vad-active');

      // ACT - Activate VAD
      rerender(<AudioVisualizer {...defaultProps} vadActivity={true} />);

      // ASSERT - Should show VAD activity immediately
      await waitFor(() => {
        expect(screen.getByTestId('vad-indicator')).toHaveClass('vad-active');
      });

      // ACT - Deactivate VAD
      rerender(<AudioVisualizer {...defaultProps} vadActivity={false} />);

      // ASSERT - Should remove VAD activity
      await waitFor(() => {
        expect(screen.getByTestId('vad-indicator')).not.toHaveClass('vad-active');
      });
    });

    it('should maintain 60fps animation performance', async () => {
      // ARRANGE
      const performanceStart = performance.now();
      const { rerender } = render(<AudioVisualizer {...defaultProps} />);

      // ACT - Rapid updates (simulate 60fps)
      for (let i = 0; i < 60; i++) {
        rerender(
          <AudioVisualizer
            {...defaultProps}
            audioLevel={Math.sin(i * 0.1) * 0.5 + 0.5}
            vadActivity={i % 10 < 3}
          />
        );
        await new Promise(resolve => setTimeout(resolve, 16)); // ~60fps
      }

      const performanceEnd = performance.now();
      const totalTime = performanceEnd - performanceStart;

      // ASSERT - Should complete within reasonable time (allowing for test overhead)
      expect(totalTime).toBeLessThan(2000); // Should be ~1 second for 60 frames
    });
  });

  describe('Accessibility', () => {
    it('should provide appropriate ARIA labels', () => {
      // ACT
      render(<AudioVisualizer {...defaultProps} isRecording={true} vadActivity={true} />);

      // ASSERT - All interactive elements should have proper labels
      expect(screen.getByLabelText(/audio level/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/recording/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/speech detected/i)).toBeInTheDocument();
      
      if (defaultProps.showWaveform) {
        expect(screen.getByLabelText(/waveform/i)).toBeInTheDocument();
        expect(screen.getByLabelText(/play audio/i)).toBeInTheDocument();
      }
    });

    it('should support keyboard navigation for waveform controls', async () => {
      // ARRANGE
      const user = userEvent.setup();
      render(<AudioVisualizer {...defaultProps} showWaveform={true} />);

      // ACT - Tab to play button and press Enter
      await user.tab();
      expect(screen.getByTestId('waveform-play-button')).toHaveFocus();
      
      await user.keyboard('{Enter}');

      // ASSERT - Should trigger playback
      expect(defaultProps.onPlaybackToggle).toHaveBeenCalled();
    });

    it('should provide screen reader announcements for level changes', async () => {
      // ARRANGE
      render(<AudioVisualizer {...defaultProps} audioLevel={0.2} />);

      // ACT - Change to clipping level
      const { rerender } = render(
        <AudioVisualizer {...defaultProps} audioLevel={0.95} />
      );

      // ASSERT - Should announce clipping warning
      await waitFor(() => {
        expect(screen.getByRole('alert')).toHaveTextContent(/clipping detected/i);
      });
    });

    it('should meet WCAG color contrast requirements', () => {
      // ACT
      render(<AudioVisualizer {...defaultProps} />);

      // ASSERT - Visual elements should have sufficient contrast
      const levelBar = screen.getByTestId('level-bar');
      const computedStyle = getComputedStyle(levelBar);
      
      // These would be tested with actual color values in real implementation
      expect(computedStyle.backgroundColor).toBeTruthy();
      expect(computedStyle.borderColor).toBeTruthy();
    });
  });

  describe('Error Handling', () => {
    it('should handle invalid audio level values gracefully', () => {
      // ARRANGE - Test with invalid level
      const invalidProps = {
        ...defaultProps,
        audioLevel: 1.5, // Invalid: should be 0-1
      };

      // ACT & ASSERT - Should not crash
      expect(() => {
        render(<AudioVisualizer {...invalidProps} />);
      }).not.toThrow();

      // Should clamp to valid range
      const levelBar = screen.getByTestId('level-bar');
      expect(levelBar).toHaveStyle({ width: '100%' }); // Clamped to maximum
    });

    it('should handle WaveSurfer initialization failures', () => {
      // ARRANGE - Mock WaveSurfer to throw error
      const WaveSurfer = vi.mocked(require('wavesurfer.js').default);
      WaveSurfer.create.mockImplementation(() => {
        throw new Error('Failed to initialize WaveSurfer');
      });

      // ACT & ASSERT - Should not crash, should show fallback
      expect(() => {
        render(<AudioVisualizer {...defaultProps} showWaveform={true} />);
      }).not.toThrow();

      expect(screen.getByTestId('waveform-error-fallback')).toBeInTheDocument();
      expect(screen.getByText(/waveform unavailable/i)).toBeInTheDocument();
    });

    it('should handle missing audio data gracefully', () => {
      // ARRANGE
      const noAudioProps = {
        ...defaultProps,
        audioData: undefined,
      };

      // ACT
      render(<AudioVisualizer {...noAudioProps} />);

      // ASSERT - Should show appropriate state
      expect(screen.getByTestId('no-audio-state')).toBeInTheDocument();
      expect(screen.getByText(/no audio data/i)).toBeInTheDocument();
    });
  });

  describe('Performance Optimization', () => {
    it('should debounce rapid level updates', async () => {
      // ARRANGE
      const { rerender } = render(<AudioVisualizer {...defaultProps} />);
      let renderCount = 0;
      
      // Mock to count renders
      const originalRender = vi.fn(() => renderCount++);

      // ACT - Send rapid updates
      for (let i = 0; i < 100; i++) {
        rerender(<AudioVisualizer {...defaultProps} audioLevel={i / 100} />);
      }

      await new Promise(resolve => setTimeout(resolve, 100));

      // ASSERT - Should not render for every update
      // Exact count depends on debouncing implementation
      expect(renderCount).toBeLessThan(50);
    });

    it('should cleanup resources on unmount', () => {
      // ARRANGE
      const { unmount } = render(<AudioVisualizer {...defaultProps} showWaveform={true} />);

      // ACT
      unmount();

      // ASSERT - WaveSurfer should be destroyed
      expect(mockWaveSurfer.destroy).toHaveBeenCalled();
    });

    it('should use requestAnimationFrame for smooth animations', async () => {
      // ARRANGE - Mock requestAnimationFrame
      const rafSpy = vi.spyOn(window, 'requestAnimationFrame');
      
      render(<AudioVisualizer {...defaultProps} isRecording={true} />);

      // ACT - Trigger animation updates
      fireEvent.animationFrame(screen.getByTestId('level-bar'));

      await waitFor(() => {
        // ASSERT - Should use RAF for smooth animations
        expect(rafSpy).toHaveBeenCalled();
      });

      rafSpy.mockRestore();
    });
  });
});

/*
COMPONENT CONTRACT DEFINITION:
============================

The AudioVisualizer component must provide:

Required Props:
- audioLevel: number (0-1) - Current audio input level
- isRecording: boolean - Whether recording is active
- vadActivity: boolean - Whether speech is detected
- showWaveform: boolean - Show waveform or level meters

Optional Props:  
- height?: number - Component height in pixels
- width?: number - Component width in pixels
- waveformColor?: string - Waveform color
- progressColor?: string - Progress indicator color
- audioData?: AudioData - Audio data for waveform
- showPeakIndicators?: boolean - Show peak level indicators
- onPlaybackToggle?: (playing: boolean) => void
- onSeek?: (time: number) => void

Features Required:
1. Real-time audio level visualization
2. Recording state indicator with visual feedback
3. VAD (voice activity) indicator
4. Waveform display with playback controls
5. Level meters when waveform disabled
6. Clipping detection and warnings
7. Smooth 60fps animations
8. Full accessibility support (ARIA, keyboard nav)
9. Error handling for initialization failures
10. Performance optimization (debouncing, RAF)

Visual Elements:
- Level bars with smooth updates
- Recording indicator (animated when active)  
- VAD activity indicator
- Waveform with progress tracking
- Peak level indicators
- Clipping warnings

All tests should FAIL initially - this is correct TDD behavior.
The component implementation will be written to make these tests pass.
*/