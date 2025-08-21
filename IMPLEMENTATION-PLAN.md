# KagiNote Design Implementation Plan

## Overview

This implementation plan provides a comprehensive roadmap for integrating the new design system into KagiNote's existing Tauri v2 + React 19 + Tailwind CSS v3 architecture. The plan is organized into phases to ensure smooth development and maintains the existing technical foundation while enhancing the user experience.

## Current Technical Stack Integration

### Existing Architecture
- **Frontend**: React 19 with TypeScript
- **Styling**: Tailwind CSS v3.4.17 with PostCSS
- **Backend**: Tauri v2 with Rust
- **Build**: Vite with HMR
- **Development**: Port 1420 (frontend), 1421 (HMR)

### Design System Integration Points
- **Tailwind Config**: Extend existing configuration with design tokens
- **Component Library**: Build on existing React 19 patterns
- **Platform Detection**: Integrate with Tauri's platform APIs
- **Asset Management**: Leverage Vite's asset handling

## Phase 1: Foundation Setup (Week 1-2)

### 1.1 Tailwind Configuration Enhancement

Update `tailwind.config.js` to include design system tokens:

```javascript
// tailwind.config.js
/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'media', // Respect system preference
  theme: {
    extend: {
      // Color System
      colors: {
        // Primary - Trust Blue
        primary: {
          50: '#EFF6FF',
          100: '#DBEAFE',
          200: '#BFDBFE',
          300: '#93C5FD',
          400: '#60A5FA',
          500: '#3B82F6',
          600: '#2563EB',
          700: '#1D4ED8',
          800: '#1E40AF',
          900: '#1E3A8A',
        },
        // Secondary - Privacy Green
        secondary: {
          50: '#ECFDF5',
          100: '#D1FAE5',
          200: '#A7F3D0',
          300: '#6EE7B7',
          400: '#34D399',
          500: '#10B981',
          600: '#059669',
          700: '#047857',
          800: '#065F46',
          900: '#064E3B',
        },
        // Neutral - Professional Grays
        neutral: {
          50: '#F9FAFB',
          100: '#F3F4F6',
          200: '#E5E7EB',
          300: '#D1D5DB',
          400: '#9CA3AF',
          500: '#6B7280',
          600: '#4B5563',
          700: '#374151',
          800: '#1F2937',
          900: '#111827',
        },
        // Warning
        warning: {
          50: '#FFFBEB',
          100: '#FEF3C7',
          200: '#FDE68A',
          300: '#FCD34D',
          400: '#FBBF24',
          500: '#F59E0B',
          600: '#D97706',
          700: '#B45309',
          800: '#92400E',
          900: '#78350F',
        },
        // Error
        error: {
          50: '#FEF2F2',
          100: '#FEE2E2',
          200: '#FECACA',
          300: '#FCA5A5',
          400: '#F87171',
          500: '#EF4444',
          600: '#DC2626',
          700: '#B91C1C',
          800: '#991B1B',
          900: '#7F1D1D',
        }
      },
      
      // Typography
      fontFamily: {
        'sans': [
          'SF Pro Text',
          'SF Pro Display',
          'Segoe UI Variable Text',
          'Segoe UI Variable Display',
          'system-ui',
          '-apple-system',
          'sans-serif'
        ],
        'cjk': [
          'SF Pro Text',
          'Segoe UI Variable Text',
          'Noto Sans CJK JP',
          'Hiragino Kaku Gothic ProN',
          'Yu Gothic Medium',
          'system-ui',
          'sans-serif'
        ],
        'mono': [
          'SF Mono',
          'Monaco',
          'Cascadia Code',
          'Roboto Mono',
          'Courier New',
          'monospace'
        ]
      },
      
      // Spacing (4px base unit)
      spacing: {
        '18': '4.5rem',    // 72px
        '88': '22rem',     // 352px
        'sidebar': '17.5rem', // 280px
        'titlebar': '2.75rem', // 44px (macOS)
        'titlebar-win': '2rem', // 32px (Windows)
      },
      
      // Desktop-specific sizing
      width: {
        'sidebar': '17.5rem', // 280px
      },
      height: {
        'titlebar': '2.75rem', // 44px
        'titlebar-win': '2rem', // 32px
        'toolbar': '3.5rem',    // 56px
        'statusbar': '2rem',    // 32px
      },
      
      // Border radius
      borderRadius: {
        'sm': '0.125rem',  // 2px
        'base': '0.25rem', // 4px
        'md': '0.375rem',  // 6px
        'lg': '0.5rem',    // 8px
        'xl': '0.75rem',   // 12px
        '2xl': '1rem',     // 16px
      },
      
      // Shadows
      boxShadow: {
        'window': '0 8px 32px rgba(0, 0, 0, 0.12)',
        'modal': '0 16px 48px rgba(0, 0, 0, 0.24)',
      },
      
      // Backdrop blur (for platform effects)
      backdropBlur: {
        'macos': '20px',
        'windows': '40px',
      },
      
      // Animations
      animation: {
        'pulse-recording': 'pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'fade-in': 'fadeIn 0.2s ease-out',
        'slide-in': 'slideIn 0.3s ease-out',
      },
      
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        slideIn: {
          '0%': { transform: 'translateY(-10px)', opacity: '0' },
          '100%': { transform: 'translateY(0)', opacity: '1' },
        }
      },
      
      // Z-index scale
      zIndex: {
        'titlebar': '1000',
        'sidebar': '900',
        'modal': '2000',
        'tooltip': '3000',
      }
    },
  },
  plugins: [
    // Add any needed plugins
    require('@tailwindcss/forms'),
    require('@tailwindcss/typography'),
  ],
}
```

### 1.2 Global Styles Setup

Create/update `src/styles/globals.css`:

```css
/* globals.css */
@tailwind base;
@tailwind components;
@tailwind utilities;

/* CSS Custom Properties for Platform Detection */
:root {
  --platform: 'web';
  --titlebar-height: 44px;
}

.platform-macos {
  --platform: 'macos';
  --titlebar-height: 44px;
}

.platform-windows {
  --platform: 'windows';
  --titlebar-height: 32px;
}

/* Base styles */
@layer base {
  html {
    font-family: theme('fontFamily.sans');
    font-size: 16px;
    line-height: 1.5;
  }
  
  body {
    @apply bg-neutral-50 text-neutral-900 antialiased;
    margin: 0;
    padding: 0;
    overflow: hidden; /* Desktop app - no scroll */
  }
  
  /* CJK character support */
  .text-cjk {
    font-family: theme('fontFamily.cjk');
    line-height: 1.75;
  }
  
  /* Accessibility improvements */
  .focus-ring {
    @apply ring-2 ring-primary-500 ring-offset-2 outline-none;
  }
  
  /* Reduced motion support */
  @media (prefers-reduced-motion: reduce) {
    * {
      animation-duration: 0.01ms !important;
      animation-iteration-count: 1 !important;
      transition-duration: 0.01ms !important;
    }
  }
}

/* Component base styles */
@layer components {
  /* Platform-specific styles */
  .titlebar-drag {
    -webkit-app-region: drag;
    user-select: none;
  }
  
  .titlebar-no-drag {
    -webkit-app-region: no-drag;
  }
  
  /* macOS-specific styles */
  .macos-blur {
    background: rgba(255, 255, 255, 0.72);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
  }
  
  .macos-dark-blur {
    background: rgba(30, 30, 30, 0.72);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
  }
  
  /* Windows-specific styles */
  .windows-mica {
    background: rgba(255, 255, 255, 0.78);
    backdrop-filter: blur(40px);
    -webkit-backdrop-filter: blur(40px);
  }
  
  /* Button components */
  .btn {
    @apply px-4 py-3 rounded-md font-medium transition-all duration-200 focus:focus-ring;
  }
  
  .btn-primary {
    @apply btn bg-primary-600 text-white hover:bg-primary-700 active:bg-primary-800;
  }
  
  .btn-secondary {
    @apply btn bg-transparent text-primary-600 border border-primary-600 hover:bg-primary-50;
  }
  
  .btn-ghost {
    @apply btn bg-transparent text-neutral-600 hover:bg-neutral-100;
  }
  
  /* Input components */
  .input {
    @apply w-full px-4 py-3 border border-neutral-300 rounded-md bg-white text-neutral-900 placeholder-neutral-400 transition-colors focus:border-primary-500 focus:ring-2 focus:ring-primary-100 focus:outline-none;
  }
  
  /* Card components */
  .card {
    @apply bg-white border border-neutral-200 rounded-lg shadow-sm;
  }
  
  .card-header {
    @apply p-6 border-b border-neutral-200;
  }
  
  .card-body {
    @apply p-6;
  }
}

/* Dark mode overrides */
@media (prefers-color-scheme: dark) {
  :root {
    color-scheme: dark;
  }
  
  body {
    @apply bg-neutral-900 text-neutral-100;
  }
  
  .card {
    @apply bg-neutral-800 border-neutral-700;
  }
  
  .input {
    @apply bg-neutral-800 border-neutral-600 text-neutral-100 placeholder-neutral-400;
  }
  
  .macos-blur {
    background: rgba(30, 30, 30, 0.72);
  }
  
  .windows-mica {
    background: rgba(30, 30, 30, 0.78);
  }
}
```

### 1.3 Platform Detection Hook

Create `src/hooks/usePlatform.ts`:

```typescript
import { useEffect, useState } from 'react';
import { platform } from '@tauri-apps/api/os';

export type Platform = 'macos' | 'windows' | 'linux' | 'web';

export const usePlatform = () => {
  const [currentPlatform, setCurrentPlatform] = useState<Platform>('web');
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const detectPlatform = async () => {
      try {
        const platformName = await platform();
        setCurrentPlatform(platformName as Platform);
        
        // Add platform class to document
        document.documentElement.classList.add(`platform-${platformName}`);
      } catch (error) {
        // Fallback to web if Tauri API not available
        setCurrentPlatform('web');
        document.documentElement.classList.add('platform-web');
      } finally {
        setIsLoading(false);
      }
    };

    detectPlatform();
  }, []);

  return {
    platform: currentPlatform,
    isLoading,
    isMacOS: currentPlatform === 'macos',
    isWindows: currentPlatform === 'windows',
    isLinux: currentPlatform === 'linux',
    isWeb: currentPlatform === 'web',
  };
};
```

## Phase 2: Core Component Library (Week 2-3)

### 2.1 Base UI Components

Create component library structure:

```
src/components/
├── ui/                 # Base UI components
│   ├── Button.tsx
│   ├── Input.tsx
│   ├── Card.tsx
│   ├── Icon.tsx
│   ├── Badge.tsx
│   └── index.ts
├── layout/             # Layout components
│   ├── TitleBar.tsx
│   ├── Sidebar.tsx
│   ├── StatusBar.tsx
│   ├── AppLayout.tsx
│   └── index.ts
└── features/           # Feature-specific components
    ├── AudioVisualizer.tsx
    ├── RecordingControls.tsx
    ├── TranscriptView.tsx
    ├── SettingsPanel.tsx
    └── index.ts
```

### 2.2 Icon System Implementation

Create `src/components/ui/Icon.tsx`:

```tsx
import React from 'react';
import { cn } from '@/lib/utils';

// Icon size mapping
const sizeClasses = {
  sm: 'w-4 h-4',
  base: 'w-6 h-6',
  lg: 'w-8 h-8',
  xl: 'w-10 h-10',
} as const;

export interface IconProps {
  name: string;
  size?: keyof typeof sizeClasses;
  className?: string;
  'aria-label'?: string;
}

// Icon components (using heroicons or similar)
const icons = {
  microphone: (props: React.SVGProps<SVGSVGElement>) => (
    <svg fill="none" stroke="currentColor" strokeWidth={2} {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z" />
    </svg>
  ),
  'microphone-slash': (props: React.SVGProps<SVGSVGElement>) => (
    <svg fill="none" stroke="currentColor" strokeWidth={2} {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M5.586 5.586A2 2 0 017 5v6a3 3 0 005.395 1.74M9 9v3a3 3 0 002.08 2.86l1.92-1.92M15 9V5a3 3 0 10-6 0v1m6 8a7.001 7.001 0 01-11.04-2.05M21 21l-18-18" />
    </svg>
  ),
  'shield-check': (props: React.SVGProps<SVGSVGElement>) => (
    <svg fill="none" stroke="currentColor" strokeWidth={2} {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
    </svg>
  ),
  home: (props: React.SVGProps<SVGSVGElement>) => (
    <svg fill="none" stroke="currentColor" strokeWidth={2} {...props}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
    </svg>
  ),
  // Add more icons as needed
} as const;

export const Icon: React.FC<IconProps> = ({
  name,
  size = 'base',
  className = '',
  'aria-label': ariaLabel,
}) => {
  const IconComponent = icons[name as keyof typeof icons];
  
  if (!IconComponent) {
    console.warn(`Icon "${name}" not found`);
    return null;
  }

  return (
    <IconComponent
      className={cn(sizeClasses[size], className)}
      aria-label={ariaLabel}
      role="img"
    />
  );
};
```

### 2.3 Layout Components

Create `src/components/layout/AppLayout.tsx`:

```tsx
import React from 'react';
import { TitleBar } from './TitleBar';
import { Sidebar } from './Sidebar';
import { StatusBar } from './StatusBar';
import { usePlatform } from '@/hooks/usePlatform';
import { cn } from '@/lib/utils';

interface AppLayoutProps {
  children: React.ReactNode;
}

export const AppLayout: React.FC<AppLayoutProps> = ({ children }) => {
  const { platform, isLoading } = usePlatform();

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-screen bg-neutral-50">
        <div className="animate-spin rounded-full h-8 w-8 border-2 border-primary-500 border-t-transparent" />
      </div>
    );
  }

  return (
    <div className={cn(
      "flex flex-col h-screen bg-neutral-50 text-neutral-900",
      "dark:bg-neutral-900 dark:text-neutral-100"
    )}>
      <TitleBar platform={platform} />
      
      <div className="flex flex-1 overflow-hidden">
        <Sidebar />
        
        <main className="flex-1 flex flex-col overflow-hidden">
          <div className="flex-1 overflow-auto p-6">
            {children}
          </div>
        </main>
      </div>
      
      <StatusBar />
    </div>
  );
};
```

## Phase 3: Feature Components (Week 3-4)

### 3.1 Audio Visualizer Component

Create `src/components/features/AudioVisualizer.tsx`:

```tsx
import React, { useEffect, useRef, useState } from 'react';
import { cn } from '@/lib/utils';

interface AudioVisualizerProps {
  audioData?: Float32Array;
  isRecording: boolean;
  isPlaying: boolean;
  height?: number;
  className?: string;
}

export const AudioVisualizer: React.FC<AudioVisualizerProps> = ({
  audioData,
  isRecording,
  isPlaying,
  height = 80,
  className = '',
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animationFrameRef = useRef<number>();
  const [bars, setBars] = useState<number[]>(Array(64).fill(0));

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const render = () => {
      // Clear canvas
      ctx.clearRect(0, 0, canvas.width, canvas.height);

      // Set up bar properties
      const barWidth = canvas.width / bars.length;
      const barMaxHeight = canvas.height - 4;

      bars.forEach((barHeight, index) => {
        const x = index * barWidth;
        const y = canvas.height - barHeight * barMaxHeight;
        const actualHeight = barHeight * barMaxHeight;

        // Color based on state
        let color = '#6B7280'; // neutral-500 (idle)
        if (isRecording) color = '#10B981'; // secondary-500 (recording)
        else if (isPlaying) color = '#3B82F6'; // primary-500 (playing)

        ctx.fillStyle = color;
        ctx.fillRect(x + 1, y, barWidth - 2, actualHeight);
      });

      if (isRecording || isPlaying) {
        animationFrameRef.current = requestAnimationFrame(render);
      }
    };

    render();

    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
  }, [bars, isRecording, isPlaying]);

  // Update bars based on audio data
  useEffect(() => {
    if (!audioData) return;

    // Process audio data into bar heights
    const newBars = Array.from({ length: 64 }, (_, i) => {
      const slice = audioData.slice(
        Math.floor((i * audioData.length) / 64),
        Math.floor(((i + 1) * audioData.length) / 64)
      );
      const avg = slice.reduce((sum, val) => sum + Math.abs(val), 0) / slice.length;
      return Math.min(avg * 5, 1); // Scale and clamp to 0-1
    });

    setBars(newBars);
  }, [audioData]);

  return (
    <div className={cn("flex items-center justify-center", className)}>
      <canvas
        ref={canvasRef}
        width={400}
        height={height}
        className="w-full h-full rounded-md bg-neutral-100 dark:bg-neutral-800"
      />
    </div>
  );
};
```

### 3.2 Recording Controls Component

Create `src/components/features/RecordingControls.tsx`:

```tsx
import React from 'react';
import { Button } from '@/components/ui/Button';
import { Icon } from '@/components/ui/Icon';
import { Badge } from '@/components/ui/Badge';
import { cn } from '@/lib/utils';

interface RecordingControlsProps {
  isRecording: boolean;
  isPaused: boolean;
  duration: number;
  onStart: () => void;
  onPause: () => void;
  onResume: () => void;
  onStop: () => void;
  className?: string;
}

const formatDuration = (seconds: number): string => {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);
  
  if (hours > 0) {
    return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  }
  return `${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
};

export const RecordingControls: React.FC<RecordingControlsProps> = ({
  isRecording,
  isPaused,
  duration,
  onStart,
  onPause,
  onResume,
  onStop,
  className = '',
}) => {
  const primaryAction = () => {
    if (!isRecording) return onStart();
    if (isPaused) return onResume();
    return onPause();
  };

  const primaryLabel = () => {
    if (!isRecording) return 'Start Recording';
    if (isPaused) return 'Resume';
    return 'Pause';
  };

  const primaryIcon = () => {
    if (!isRecording) return 'microphone';
    if (isPaused) return 'play';
    return 'pause';
  };

  return (
    <div className={cn(
      "flex items-center gap-4 p-4 bg-white rounded-lg border border-neutral-200 shadow-sm",
      "dark:bg-neutral-800 dark:border-neutral-700",
      className
    )}>
      {/* Primary Action Button */}
      <Button
        onClick={primaryAction}
        variant={isRecording ? "secondary" : "primary"}
        className={cn(
          "flex items-center gap-2",
          isRecording && !isPaused && "animate-pulse-recording"
        )}
      >
        <Icon name={primaryIcon()} size="base" />
        {primaryLabel()}
      </Button>

      {/* Stop Button (only when recording) */}
      {isRecording && (
        <Button
          onClick={onStop}
          variant="ghost"
          className="flex items-center gap-2 text-error-600 hover:text-error-700 hover:bg-error-50"
        >
          <Icon name="stop" size="base" />
          Stop
        </Button>
      )}

      {/* Duration Display */}
      <div className="flex items-center gap-2 ml-auto">
        <Badge 
          variant={isRecording ? (isPaused ? "warning" : "success") : "secondary"}
          className="font-mono"
        >
          {formatDuration(duration)}
        </Badge>
        
        {isRecording && (
          <Badge variant={isPaused ? "warning" : "success"}>
            {isPaused ? 'Paused' : 'Recording'}
          </Badge>
        )}
      </div>
    </div>
  );
};
```

## Phase 4: Integration & Refinement (Week 4-5)

### 4.1 Integration with Existing Tauri Commands

Update existing components to use new design system:

```tsx
// Update existing App.tsx
import React from 'react';
import { AppLayout } from '@/components/layout/AppLayout';
import { TranscriptionController } from '@/components/TranscriptionController';
import { AudioVisualizer } from '@/components/features/AudioVisualizer';

function App() {
  return (
    <AppLayout>
      <div className="flex flex-col gap-6">
        <div className="card">
          <div className="card-header">
            <h1 className="text-2xl font-semibold text-neutral-900 dark:text-neutral-100">
              KagiNote
            </h1>
            <p className="text-neutral-600 dark:text-neutral-400">
              Privacy-first meeting transcription
            </p>
          </div>
          
          <div className="card-body">
            <TranscriptionController />
          </div>
        </div>
      </div>
    </AppLayout>
  );
}

export default App;
```

### 4.2 Dark Mode Implementation

Ensure all components support dark mode:

```tsx
// Update component patterns to use dark mode classes
const Component = () => (
  <div className="bg-white dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100">
    {/* Component content */}
  </div>
);
```

### 4.3 Performance Optimization

Implement performance optimizations:

```tsx
// Lazy load heavy components
const AudioVisualizer = React.lazy(() => import('@/components/features/AudioVisualizer'));
const SettingsPanel = React.lazy(() => import('@/components/features/SettingsPanel'));

// Memoize expensive components
const MemoizedAudioVisualizer = React.memo(AudioVisualizer);
```

## Phase 5: Platform-Specific Enhancements (Week 5-6)

### 5.1 macOS Integration

Implement macOS-specific features:

```tsx
// src/components/layout/TitleBar.tsx
import React from 'react';
import { usePlatform } from '@/hooks/usePlatform';
import { cn } from '@/lib/utils';

export const TitleBar: React.FC = () => {
  const { isMacOS, isWindows } = usePlatform();

  if (isMacOS) {
    return (
      <div className={cn(
        "h-titlebar flex items-center px-20 titlebar-drag",
        "macos-blur border-b border-neutral-200 dark:border-neutral-700"
      )}>
        <div className="titlebar-no-drag">
          <h1 className="text-sm font-medium text-neutral-900 dark:text-neutral-100">
            KagiNote
          </h1>
        </div>
      </div>
    );
  }

  if (isWindows) {
    return (
      <div className={cn(
        "h-titlebar-win flex items-center justify-between px-4 titlebar-drag",
        "windows-mica border-b border-neutral-200 dark:border-neutral-700"
      )}>
        <h1 className="text-sm font-medium text-neutral-900 dark:text-neutral-100">
          KagiNote
        </h1>
        
        <div className="titlebar-no-drag flex">
          {/* Windows window controls would go here */}
        </div>
      </div>
    );
  }

  return null;
};
```

### 5.2 Windows Integration

Implement Windows-specific features:

```css
/* Windows-specific styles in globals.css */
.platform-windows .app-window {
  border-radius: 8px; /* Windows 11 rounded corners */
}

.platform-windows .sidebar {
  background: rgba(255, 255, 255, 0.78);
  backdrop-filter: blur(40px);
}
```

## Testing & Quality Assurance

### Accessibility Testing
```bash
# Install accessibility testing tools
npm install --save-dev @axe-core/react eslint-plugin-jsx-a11y
```

### Performance Testing
```bash
# Add lighthouse CI for performance monitoring
npm install --save-dev @lhci/cli
```

### Cross-platform Testing
- Test on macOS Intel and Apple Silicon
- Test on Windows 10 and 11
- Verify native integrations work correctly

## Deployment Checklist

### Pre-deployment
- [ ] All components implement dark mode
- [ ] Accessibility standards met (WCAG 2.1 AA)
- [ ] Platform-specific styles working
- [ ] Performance targets achieved
- [ ] Integration tests passing

### Post-deployment
- [ ] User feedback collection
- [ ] Performance monitoring
- [ ] Accessibility audit
- [ ] Platform-specific testing

## Maintenance & Updates

### Design System Evolution
- Component versioning strategy
- Design token updates process
- Breaking change management
- Documentation maintenance

### Performance Monitoring
- Bundle size tracking
- Runtime performance metrics
- User experience metrics
- Platform-specific optimization

This implementation plan provides a comprehensive roadmap for integrating the new design system while maintaining the existing technical architecture and ensuring cross-platform compatibility.