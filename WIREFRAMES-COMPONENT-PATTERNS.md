# KagiNote Wireframes & Component Patterns

## Application Architecture Overview

### Window Structure
```
┌─────────────────────────────────────────────────────────────────┐
│ Title Bar (macOS: Traffic Lights | Windows: Min/Max/Close)     │
├─────────────────────────────────────────────────────────────────┤
│ ┌─────────────┐ ┌─────────────────────────────────────────────┐ │
│ │             │ │                                             │ │
│ │   Sidebar   │ │            Main Content Area               │ │
│ │             │ │                                             │ │
│ │             │ │                                             │ │
│ └─────────────┘ └─────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│ Status Bar (Model info, Recording status, Timestamps)          │
└─────────────────────────────────────────────────────────────────┘
```

## Main Application Wireframes

### 1. Dashboard/Home View
```
┌─────────────────────────────────────────────────────────────────┐
│ ● ● ●                    KagiNote                              │ macOS
├─────────────────────────────────────────────────────────────────┤
│ ┌─────────────┐ ┌─────────────────────────────────────────────┐ │
│ │ [🏠] Home   │ │   Welcome to KagiNote                      │ │
│ │ [🎤] Record │ │   ┌─────────────────────────────────────┐   │ │
│ │ [📝] Files  │ │   │ Quick Start                         │   │ │
│ │ [⚙️] Settings│ │   │ ┌─────────────┐ ┌─────────────┐   │   │ │
│ │             │ │   │ │[🎤] Start   │ │[📁] Import  │   │   │ │
│ │ Privacy     │ │   │ │Recording    │ │File         │   │   │ │
│ │ [🔒] Local  │ │   │ └─────────────┘ └─────────────┘   │   │ │
│ │ [🚫] No Net │ │   └─────────────────────────────────────┘   │ │
│ │ [🏠] On-Dev │ │                                             │ │
│ │             │ │   Recent Sessions                           │ │
│ │             │ │   ┌─────────────────────────────────────┐   │ │
│ │             │ │   │ Meeting_2025-01-20.wav              │   │ │
│ │             │ │   │ 45:23 • English • High Quality      │   │ │
│ │             │ │   │ [▶️] [📝] [📤]                        │   │ │
│ │             │ │   └─────────────────────────────────────┘   │ │
│ │             │ │   ┌─────────────────────────────────────┐   │ │
│ │             │ │   │ Interview_2025-01-19.wav            │   │ │
│ │             │ │   │ 28:15 • Japanese • Standard         │   │ │
│ │             │ │   │ [▶️] [📝] [📤]                        │   │ │
│ │             │ │   └─────────────────────────────────────┘   │ │
│ └─────────────┘ └─────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│ Model: Standard • Status: Ready • Privacy: Local Processing    │
└─────────────────────────────────────────────────────────────────┘
```

### 2. Recording View
```
┌─────────────────────────────────────────────────────────────────┐
│ ● ● ●                    KagiNote                              │
├─────────────────────────────────────────────────────────────────┤
│ ┌─────────────┐ ┌─────────────────────────────────────────────┐ │
│ │ [🏠] Home   │ │   Live Recording                            │ │
│ │ [🎤] Record │ │   ┌─────────────────────────────────────┐   │ │
│ │ [📝] Files  │ │   │ Audio Visualization                 │   │ │
│ │ [⚙️] Settings│ │   │ ▁▂▃▅▇▆▄▂▃▅▇▆▄▂▁                     │   │ │
│ │             │ │   │ ▁▂▃▅▇▆▄▂▃▅▇▆▄▂▁                     │   │ │
│ │ Recording   │ │   └─────────────────────────────────────┘   │ │
│ │ [🔴] 15:32  │ │                                             │ │
│ │ [⏸️] Pause   │ │   Real-time Transcription                  │ │
│ │ [⏹️] Stop    │ │   ┌─────────────────────────────────────┐   │ │
│ │             │ │   │ Hello, and welcome to today's       │   │ │
│ │ Settings    │ │   │ meeting. We're going to be          │   │ │
│ │ Language:   │ │   │ discussing the quarterly results    │   │ │
│ │ [🇺🇸] English│ │   │ and our plans for the upcoming     │   │ │
│ │ Quality:    │ │   │ quarter. Let me start by...         │   │ │
│ │ [⭐] High   │ │   │                                     │   │ │
│ │             │ │   │ [Speaker detected: Voice 1]        │   │ │
│ │             │ │   │ [Confidence: 94%]                  │   │ │
│ │             │ │   └─────────────────────────────────────┘   │ │
│ └─────────────┘ └─────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│ Recording: 15:32 • Quality: High • Processing: Local          │
└─────────────────────────────────────────────────────────────────┘
```

### 3. Transcript View
```
┌─────────────────────────────────────────────────────────────────┐
│ ● ● ●                    KagiNote                              │
├─────────────────────────────────────────────────────────────────┤
│ ┌─────────────┐ ┌─────────────────────────────────────────────┐ │
│ │ [🏠] Home   │ │   Meeting_2025-01-20.wav                   │ │
│ │ [🎤] Record │ │   ┌─────────────────────────────────────┐   │ │
│ │ [📝] Files  │ │   │ [⏮️] [▶️] [⏭️]   00:15:32 / 45:23    │   │ │
│ │ [⚙️] Settings│ │   │ ▁▂▃▅▇▆▄▂▃▅▇▆▄▂▁                     │   │ │
│ │             │ │   │ [🔊] ████████░░ [⚙️] [📤]             │   │ │
│ │ Actions     │ │   └─────────────────────────────────────┘   │ │
│ │ [📤] Export │ │                                             │ │
│ │ [✏️] Edit    │ │   Transcript                                │ │
│ │ [🔍] Search │ │   ┌─────────────────────────────────────┐   │ │
│ │ [👥] Speakers│ │   │ [00:00] Speaker 1:                  │   │ │
│ │             │ │   │ Hello and welcome to today's        │   │ │
│ │ Export      │ │   │ meeting. We're going to be          │   │ │
│ │ [📄] TXT    │ │   │ discussing the quarterly results.   │   │ │
│ │ [📊] JSON   │ │   │                                     │   │ │
│ │ [📝] DOCX   │ │   │ [02:15] Speaker 2:                  │   │ │
│ │ [📋] SRT    │ │   │ Thank you for that introduction.    │   │ │
│ │             │ │   │ I'd like to start with our key      │   │ │
│ │             │ │   │ metrics for this quarter...         │   │ │
│ │             │ │   │                                     │   │ │
│ │             │ │   │ [05:42] Speaker 1:                  │   │ │
│ │             │ │   │ That's excellent progress. What     │   │ │
│ │             │ │   │ about our expansion plans?          │   │ │
│ └─────────────┘ └─────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│ 45:23 duration • 2 Speakers • 98% accuracy • Exported: Never  │
└─────────────────────────────────────────────────────────────────┘
```

### 4. Settings View
```
┌─────────────────────────────────────────────────────────────────┐
│ ● ● ●                    KagiNote                              │
├─────────────────────────────────────────────────────────────────┤
│ ┌─────────────┐ ┌─────────────────────────────────────────────┐ │
│ │ [🏠] Home   │ │   Settings                                  │ │
│ │ [🎤] Record │ │                                             │ │
│ │ [📝] Files  │ │   Audio Settings                            │ │
│ │ [⚙️] Settings│ │   ┌─────────────────────────────────────┐   │ │
│ │             │ │   │ Input Device                        │   │ │
│ │ Categories  │ │   │ [🎤] MacBook Pro Microphone ▼       │   │ │
│ │ [🎤] Audio  │ │   │                                     │   │ │
│ │ [🌐] Language│ │   │ Sample Rate: 48kHz → 16kHz         │   │ │
│ │ [🤖] Models │ │   │ Quality: [●●●●○] Very High          │   │ │
│ │ [🔒] Privacy│ │   │ Auto Gain: [✓] Enabled             │   │ │
│ │ [📁] Storage│ │   │ Noise Reduction: [✓] Enabled       │   │ │
│ │ [🎨] Interface│ │   └─────────────────────────────────────┘   │ │
│ │             │ │                                             │ │
│ │             │ │   Language & Models                         │ │
│ │             │ │   ┌─────────────────────────────────────┐   │ │
│ │             │ │   │ Primary Language                    │   │ │
│ │             │ │   │ [🇺🇸] English ▼                     │   │ │
│ │             │ │   │                                     │   │ │
│ │             │ │   │ Model Quality                       │   │ │
│ │             │ │   │ ○ Standard (1.5GB)                  │   │ │
│ │             │ │   │ ● High Accuracy (2.4GB)             │   │ │
│ │             │ │   │ ○ Turbo (1.2GB)                     │   │ │
│ │             │ │   │                                     │ │
│ │             │ │   │ Status: [✓] Downloaded              │   │ │
│ │             │ │   │ Location: ~/.../KagiNote/models/    │   │ │
│ └─────────────┘ └─────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│ Model: High Accuracy • Privacy: All Local • Updates: None     │
└─────────────────────────────────────────────────────────────────┘
```

## Component Patterns

### 1. Audio Visualizer Component
```tsx
interface AudioVisualizerProps {
  audioData: Float32Array;
  isRecording: boolean;
  height?: number;
  className?: string;
}

const AudioVisualizer: React.FC<AudioVisualizerProps> = ({
  audioData,
  isRecording,
  height = 80,
  className = ''
}) => {
  // Render waveform visualization
  // Real-time bars showing audio levels
  // Different colors for recording vs. playback
  // Smooth animations for level changes
};
```

**Visual Design:**
- Bars: 4px width, 2px gap
- Colors: Green when recording, Blue when playing, Gray when idle
- Heights: Responsive to audio levels (0-100%)
- Animation: 60fps smooth transitions

### 2. Recording Controls Component
```tsx
interface RecordingControlsProps {
  isRecording: boolean;
  isPaused: boolean;
  duration: number;
  onStart: () => void;
  onPause: () => void;
  onStop: () => void;
  onResume: () => void;
}

const RecordingControls: React.FC<RecordingControlsProps> = ({
  isRecording,
  isPaused,
  duration,
  onStart,
  onPause,
  onStop,
  onResume
}) => {
  // Primary action button (Start/Pause/Resume)
  // Secondary stop button
  // Duration display with live timer
  // Status indicators
};
```

**Layout:**
```
┌─────────────────────────────────────┐
│ [🔴] Start Recording    [00:00:00]  │
│ [⏸️] Pause              [⏹️] Stop    │
└─────────────────────────────────────┘
```

### 3. Transcript Display Component
```tsx
interface TranscriptDisplayProps {
  segments: TranscriptSegment[];
  currentTime: number;
  onSeek: (time: number) => void;
  onEdit: (segmentId: string, text: string) => void;
  showTimestamps: boolean;
  showSpeakers: boolean;
}

interface TranscriptSegment {
  id: string;
  startTime: number;
  endTime: number;
  speaker: string;
  text: string;
  confidence: number;
}
```

**Visual Features:**
- Click timestamps to seek audio
- Inline editing of transcript text
- Speaker identification and coloring
- Confidence indicators (subtle opacity)
- Current playback position highlighting

### 4. Model Status Component
```tsx
interface ModelStatusProps {
  model: ModelInfo;
  downloadProgress?: number;
  isLoading: boolean;
  onDownload: () => void;
  onSwitch: (modelId: string) => void;
}

interface ModelInfo {
  id: string;
  name: string;
  size: string;
  quality: 'Standard' | 'High' | 'Turbo';
  isDownloaded: boolean;
  languages: string[];
}
```

**Status Indicators:**
- ✅ Downloaded and ready
- ⬇️ Downloading (with progress)
- ⚠️ Not downloaded
- 🔄 Loading/switching
- ❌ Error state

### 5. Privacy Indicator Component
```tsx
interface PrivacyIndicatorProps {
  isLocal: boolean;
  isOffline: boolean;
  dataLocation: string;
  className?: string;
}
```

**Visual Elements:**
```
┌─────────────────────────┐
│ 🔒 Private & Local      │
│ 🏠 Processed on device  │
│ 🚫 No network required  │
└─────────────────────────┘
```

### 6. File Management Component
```tsx
interface FileManagerProps {
  files: TranscriptFile[];
  onImport: () => void;
  onExport: (fileId: string, format: ExportFormat) => void;
  onDelete: (fileId: string) => void;
  onRename: (fileId: string, newName: string) => void;
}

interface TranscriptFile {
  id: string;
  name: string;
  duration: number;
  createdAt: Date;
  language: string;
  quality: string;
  speakers: number;
  accuracy: number;
}

type ExportFormat = 'txt' | 'json' | 'docx' | 'srt' | 'vtt';
```

## Interaction Patterns

### 1. Recording Workflow
```
Idle → Click Record → Recording → (Pause/Resume) → Stop → Processing → Transcript Ready
```

**States:**
- **Idle**: Ready to record, show start button
- **Recording**: Red indicator, live timer, pause/stop options
- **Paused**: Paused indicator, resume/stop options
- **Processing**: Loading indicator, progress if available
- **Complete**: Transcript available, export options

### 2. File Import Workflow
```
Drag & Drop / Click Import → File Validation → Processing → Transcript Display
```

**Feedback:**
- Drag overlay with upload zone
- File format validation messages
- Processing progress indicator
- Error handling for unsupported formats

### 3. Model Management Workflow
```
Settings → Model Selection → Download (if needed) → Switch Active Model
```

**Progress Indicators:**
- Download progress bars
- Speed and remaining time estimates
- Disk space requirements
- Installation success/failure feedback

## Responsive Design Patterns

### Minimum Window Size: 800x600px
- Sidebar: 240px minimum, collapsible to icons
- Main content: Minimum 400px width
- Status bar: Fixed height 32px

### Breakpoints
- **Compact**: 800-1000px width (sidebar collapses)
- **Standard**: 1000-1400px width (full sidebar)
- **Large**: 1400px+ width (expanded layout)

### Sidebar Behavior
```css
/* Compact: Icon-only sidebar */
@media (max-width: 1000px) {
  .sidebar {
    width: 64px;
  }
  .sidebar-label {
    display: none;
  }
}

/* Standard: Full sidebar */
@media (min-width: 1000px) {
  .sidebar {
    width: 240px;
  }
}
```

## Accessibility Patterns

### Keyboard Navigation
- **Tab Order**: Logical flow through interface
- **Shortcuts**: Spacebar for play/pause, R for record, S for stop
- **Focus Management**: Clear focus indicators, skip links

### Screen Reader Support
```tsx
// Example ARIA labels
<button 
  aria-label={`${isRecording ? 'Stop' : 'Start'} recording`}
  aria-pressed={isRecording}
>
  {isRecording ? <StopIcon /> : <RecordIcon />}
</button>

<div 
  role="region" 
  aria-label="Live transcript"
  aria-live="polite"
>
  {transcriptText}
</div>
```

### Color & Contrast
- All text meets WCAG AA standards (4.5:1 contrast)
- Interactive elements have focus indicators
- Color is not the only indicator of state

## Animation & Micro-interactions

### Recording State Transitions
```css
.record-button {
  transition: all 200ms ease-out;
}

.record-button.recording {
  background-color: var(--color-error-500);
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.7; }
}
```

### Audio Visualization Animation
```css
.audio-bar {
  transition: height 100ms ease-out;
  transform-origin: bottom;
}

.audio-bars.recording .audio-bar {
  animation: audioPulse 150ms ease-in-out infinite alternate;
}
```

### Loading States
```tsx
const LoadingSpinner = () => (
  <div className="animate-spin rounded-full h-6 w-6 border-2 border-primary-500 border-t-transparent" />
);
```

This comprehensive wireframe and component pattern guide provides the foundation for implementing KagiNote's desktop interface with consistent, accessible, and professional design patterns.