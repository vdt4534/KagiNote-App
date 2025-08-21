# KagiNote Brand Identity & Visual Design System

## Brand Identity

### Brand Personality
**Core Attributes:**
- **Trustworthy**: Reliable, consistent, dependable privacy protection
- **Professional**: Serious about business needs, quality-focused
- **Minimal**: Clean, uncluttered, focused on essentials
- **Intelligent**: AI-powered but not overwhelming, smart defaults
- **Local**: Privacy-first, on-device processing, user-controlled

### Brand Values
1. **Privacy First**: Your data stays on your device, always
2. **Quality**: Accurate transcription with professional reliability
3. **Simplicity**: Complex technology made simple to use
4. **Transparency**: Clear about what happens to your data
5. **Respect**: Respectful of user time, privacy, and intelligence

### Brand Positioning Statement
*"KagiNote is the privacy-first transcription tool for professionals who need accurate, reliable meeting transcription without compromising sensitive information."*

### Voice & Tone
- **Professional yet approachable**: Serious about privacy, friendly in presentation
- **Clear and direct**: No marketing fluff, honest about capabilities
- **Confident but humble**: Effective without being boastful
- **Respectful**: Acknowledges user intelligence and privacy concerns

## Visual Design System

### Color Palette

#### Primary Colors
```css
/* Trust Blue - Primary brand color */
--color-primary-50: #EFF6FF;   /* Lightest backgrounds */
--color-primary-100: #DBEAFE;  /* Light backgrounds */
--color-primary-200: #BFDBFE;  /* Borders, subtle accents */
--color-primary-300: #93C5FD;  /* Disabled states */
--color-primary-400: #60A5FA;  /* Hover states */
--color-primary-500: #3B82F6;  /* Primary buttons, links */
--color-primary-600: #2563EB;  /* Active states */
--color-primary-700: #1D4ED8;  /* Pressed states */
--color-primary-800: #1E40AF;  /* Dark mode primary */
--color-primary-900: #1E3A8A;  /* Darkest accents */
```

#### Secondary Colors
```css
/* Privacy Green - Success, local processing */
--color-secondary-50: #ECFDF5;
--color-secondary-100: #D1FAE5;
--color-secondary-200: #A7F3D0;
--color-secondary-300: #6EE7B7;
--color-secondary-400: #34D399;
--color-secondary-500: #10B981;
--color-secondary-600: #059669;
--color-secondary-700: #047857;
--color-secondary-800: #065F46;
--color-secondary-900: #064E3B;
```

#### Neutral Colors
```css
/* Professional Grays */
--color-neutral-50: #F9FAFB;   /* Background light */
--color-neutral-100: #F3F4F6;  /* Background */
--color-neutral-200: #E5E7EB;  /* Borders */
--color-neutral-300: #D1D5DB;  /* Borders dark */
--color-neutral-400: #9CA3AF;  /* Text muted */
--color-neutral-500: #6B7280;  /* Text secondary */
--color-neutral-600: #4B5563;  /* Text primary */
--color-neutral-700: #374151;  /* Text emphasis */
--color-neutral-800: #1F2937;  /* Text strong */
--color-neutral-900: #111827;  /* Text darkest */
```

#### Accent Colors
```css
/* Warning Orange */
--color-warning-50: #FFFBEB;
--color-warning-100: #FEF3C7;
--color-warning-200: #FDE68A;
--color-warning-300: #FCD34D;
--color-warning-400: #FBBF24;
--color-warning-500: #F59E0B;
--color-warning-600: #D97706;
--color-warning-700: #B45309;
--color-warning-800: #92400E;
--color-warning-900: #78350F;

/* Error Red */
--color-error-50: #FEF2F2;
--color-error-100: #FEE2E2;
--color-error-200: #FECACA;
--color-error-300: #FCA5A5;
--color-error-400: #F87171;
--color-error-500: #EF4444;
--color-error-600: #DC2626;
--color-error-700: #B91C1C;
--color-error-800: #991B1B;
--color-error-900: #7F1D1D;
```

### Typography System

#### Font Stack
```css
/* Primary - System UI optimized for all platforms */
--font-family-primary: 
  "SF Pro Text", 
  "SF Pro Display", 
  "Segoe UI Variable Text", 
  "Segoe UI Variable Display",
  system-ui, 
  -apple-system, 
  sans-serif;

/* CJK Support for Japanese */
--font-family-cjk: 
  "SF Pro Text", 
  "Segoe UI Variable Text",
  "Noto Sans CJK JP", 
  "Hiragino Kaku Gothic ProN", 
  "Yu Gothic Medium",
  system-ui, 
  sans-serif;

/* Monospace for timestamps and code */
--font-family-mono: 
  "SF Mono", 
  "Monaco", 
  "Cascadia Code", 
  "Roboto Mono", 
  "Courier New", 
  monospace;
```

#### Type Scale
```css
/* Desktop-optimized sizes */
--text-xs: 0.75rem;     /* 12px - Captions, timestamps */
--text-sm: 0.875rem;    /* 14px - Small labels */
--text-base: 1rem;      /* 16px - Body text */
--text-lg: 1.125rem;    /* 18px - Large body text */
--text-xl: 1.25rem;     /* 20px - Subheadings */
--text-2xl: 1.5rem;     /* 24px - Section headings */
--text-3xl: 1.875rem;   /* 30px - Page headings */
--text-4xl: 2.25rem;    /* 36px - Display headings */
--text-5xl: 3rem;       /* 48px - Hero headings */

/* Line Heights */
--leading-tight: 1.25;   /* Headings */
--leading-normal: 1.5;   /* Body text */
--leading-relaxed: 1.75; /* CJK characters */
--leading-loose: 2;      /* Transcription text */

/* Font Weights */
--font-thin: 100;
--font-light: 300;
--font-normal: 400;
--font-medium: 500;
--font-semibold: 600;
--font-bold: 700;
```

### Spacing System

#### Base Unit: 4px
```css
/* Spacing scale based on 4px units */
--spacing-0: 0;
--spacing-1: 0.25rem;   /* 4px */
--spacing-2: 0.5rem;    /* 8px */
--spacing-3: 0.75rem;   /* 12px */
--spacing-4: 1rem;      /* 16px */
--spacing-5: 1.25rem;   /* 20px */
--spacing-6: 1.5rem;    /* 24px */
--spacing-8: 2rem;      /* 32px */
--spacing-10: 2.5rem;   /* 40px */
--spacing-12: 3rem;     /* 48px */
--spacing-16: 4rem;     /* 64px */
--spacing-20: 5rem;     /* 80px */
--spacing-24: 6rem;     /* 96px */
--spacing-32: 8rem;     /* 128px */
```

#### Layout Spacing
```css
/* Component-specific spacing */
--window-padding: var(--spacing-6);     /* 24px */
--section-gap: var(--spacing-8);        /* 32px */
--component-gap: var(--spacing-4);      /* 16px */
--element-gap: var(--spacing-2);        /* 8px */

/* Desktop-specific */
--sidebar-width: 280px;
--titlebar-height: 44px;    /* macOS standard */
--toolbar-height: 56px;
--statusbar-height: 32px;
```

### Border Radius System

```css
--radius-none: 0;
--radius-sm: 0.125rem;    /* 2px */
--radius-base: 0.25rem;   /* 4px */
--radius-md: 0.375rem;    /* 6px */
--radius-lg: 0.5rem;      /* 8px */
--radius-xl: 0.75rem;     /* 12px */
--radius-2xl: 1rem;       /* 16px */
--radius-full: 9999px;    /* Circular */
```

### Shadow System

```css
/* Elevation system */
--shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
--shadow-base: 0 1px 3px 0 rgba(0, 0, 0, 0.1), 0 1px 2px 0 rgba(0, 0, 0, 0.06);
--shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
--shadow-lg: 0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05);
--shadow-xl: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
--shadow-2xl: 0 25px 50px -12px rgba(0, 0, 0, 0.25);

/* Window shadows */
--shadow-window: 0 8px 32px rgba(0, 0, 0, 0.12);
--shadow-modal: 0 16px 48px rgba(0, 0, 0, 0.24);
```

### Animation System

```css
/* Timing Functions */
--ease-out: cubic-bezier(0, 0, 0.2, 1);
--ease-in: cubic-bezier(0.4, 0, 1, 1);
--ease-in-out: cubic-bezier(0.4, 0, 0.2, 1);
--ease-native: cubic-bezier(0.4, 0, 0.2, 1);

/* Durations */
--duration-fast: 150ms;
--duration-normal: 200ms;
--duration-slow: 300ms;
--duration-slower: 500ms;

/* Common animations */
--transition-colors: color var(--duration-fast) var(--ease-out),
                    background-color var(--duration-fast) var(--ease-out),
                    border-color var(--duration-fast) var(--ease-out);

--transition-transform: transform var(--duration-normal) var(--ease-out);
--transition-opacity: opacity var(--duration-normal) var(--ease-out);
--transition-all: all var(--duration-fast) var(--ease-out);
```

## Icon System

### Style Guidelines
- **Size**: 24x24px base with 16px and 32px variants
- **Stroke Width**: 2px consistent stroke
- **Style**: Outlined style, minimal fills
- **Corners**: Rounded end caps, consistent with border radius
- **Grid**: Aligned to 4px grid system

### Icon Categories

#### Core Navigation
- `home` - Home/dashboard
- `microphone` - Audio recording
- `document-text` - Transcriptions
- `cog` - Settings
- `folder` - File management

#### Audio States
- `microphone-solid` - Recording active
- `microphone-slash` - Muted/disabled
- `volume-up` - Audio playing
- `volume-off` - Audio muted
- `waveform` - Audio visualization

#### Privacy & Security
- `shield-check` - Protected/secure
- `lock-closed` - Encrypted
- `home-modern` - Local processing
- `wifi-off` - Offline capable
- `eye-slash` - Privacy protected

#### File Operations
- `upload` - Import files
- `download` - Export files
- `save` - Save transcript
- `share` - Share content
- `trash` - Delete items

#### Status Indicators
- `check-circle` - Complete/success
- `clock` - Processing/waiting
- `exclamation-triangle` - Warning
- `x-circle` - Error/failed
- `information-circle` - Information

### Implementation

```tsx
// Icon component structure
interface IconProps {
  name: string;
  size?: 'sm' | 'base' | 'lg'; // 16px, 24px, 32px
  className?: string;
  'aria-label'?: string;
}

const Icon: React.FC<IconProps> = ({ 
  name, 
  size = 'base', 
  className = '', 
  'aria-label': ariaLabel 
}) => {
  const sizeClasses = {
    sm: 'w-4 h-4',
    base: 'w-6 h-6', 
    lg: 'w-8 h-8'
  };
  
  return (
    <svg 
      className={`${sizeClasses[size]} ${className}`}
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-label={ariaLabel}
      role="img"
    >
      {/* SVG content */}
    </svg>
  );
};
```

## Component Design Patterns

### Button System

#### Primary Button
```css
.button-primary {
  background-color: var(--color-primary-600);
  color: white;
  border: 1px solid var(--color-primary-600);
  border-radius: var(--radius-md);
  padding: var(--spacing-3) var(--spacing-4);
  font-weight: var(--font-medium);
  transition: var(--transition-colors);
}

.button-primary:hover {
  background-color: var(--color-primary-700);
  border-color: var(--color-primary-700);
}

.button-primary:active {
  background-color: var(--color-primary-800);
}
```

#### Secondary Button
```css
.button-secondary {
  background-color: transparent;
  color: var(--color-primary-600);
  border: 1px solid var(--color-primary-600);
  border-radius: var(--radius-md);
  padding: var(--spacing-3) var(--spacing-4);
  font-weight: var(--font-medium);
  transition: var(--transition-colors);
}

.button-secondary:hover {
  background-color: var(--color-primary-50);
}
```

### Input System

#### Text Input
```css
.input-text {
  background-color: white;
  border: 1px solid var(--color-neutral-300);
  border-radius: var(--radius-md);
  padding: var(--spacing-3) var(--spacing-4);
  font-size: var(--text-base);
  transition: var(--transition-colors);
}

.input-text:focus {
  outline: none;
  border-color: var(--color-primary-500);
  box-shadow: 0 0 0 3px var(--color-primary-100);
}
```

### Card System

#### Basic Card
```css
.card {
  background-color: white;
  border: 1px solid var(--color-neutral-200);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-sm);
  padding: var(--spacing-6);
}

.card-header {
  margin-bottom: var(--spacing-4);
  padding-bottom: var(--spacing-4);
  border-bottom: 1px solid var(--color-neutral-200);
}
```

## Dark Mode System

### Dark Color Palette
```css
/* Dark mode overrides */
@media (prefers-color-scheme: dark) {
  :root {
    --color-background: var(--color-neutral-900);
    --color-surface: var(--color-neutral-800);
    --color-border: var(--color-neutral-700);
    --color-text-primary: var(--color-neutral-100);
    --color-text-secondary: var(--color-neutral-300);
    --color-text-muted: var(--color-neutral-400);
  }
}
```

### Dark Mode Components
- Adjust shadow opacity: reduce to 20% for dark backgrounds
- Increase border opacity for better definition
- Use lighter primary colors for better contrast
- Maintain accessibility contrast ratios

## Platform-Specific Adaptations

### macOS Specific
```css
/* macOS title bar integration */
.titlebar-macos {
  height: 44px;
  background: rgba(246, 246, 246, 0.72);
  backdrop-filter: blur(20px);
  border-bottom: 1px solid var(--color-neutral-200);
  padding-left: 80px; /* Space for traffic lights */
}

/* macOS sidebar blur */
.sidebar-macos {
  background: rgba(255, 255, 255, 0.72);
  backdrop-filter: blur(20px);
  border-right: 1px solid var(--color-neutral-200);
}
```

### Windows Specific
```css
/* Windows title bar */
.titlebar-windows {
  height: 32px;
  background: var(--color-neutral-50);
  border-bottom: 1px solid var(--color-neutral-200);
}

/* Windows Mica/Acrylic effects */
.surface-windows {
  background: rgba(255, 255, 255, 0.78);
  backdrop-filter: blur(40px);
}
```

## Accessibility Guidelines

### Color Contrast
- **AAA Standard**: 7:1 contrast ratio for normal text
- **AA Standard**: 4.5:1 minimum for all interactive elements
- **Large Text**: 3:1 minimum for 18px+ or 14px+ bold text

### Focus Management
```css
.focus-ring {
  outline: 2px solid var(--color-primary-500);
  outline-offset: 2px;
  border-radius: var(--radius-md);
}

/* High contrast mode support */
@media (prefers-contrast: high) {
  .focus-ring {
    outline-width: 3px;
    outline-color: HighlightText;
  }
}
```

### Motion Preferences
```css
/* Respect reduced motion preferences */
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

## Implementation Guidelines

### CSS Custom Properties Structure
```css
:root {
  /* Color system */
  --color-*: [values];
  
  /* Typography */
  --font-*: [values];
  --text-*: [values];
  --leading-*: [values];
  
  /* Spacing */
  --spacing-*: [values];
  
  /* Elevation */
  --shadow-*: [values];
  
  /* Motion */
  --transition-*: [values];
  --duration-*: [values];
  
  /* Layout */
  --radius-*: [values];
  --z-*: [values];
}
```

### Component Library Structure
```
src/
├── components/
│   ├── ui/                 # Basic UI components
│   │   ├── Button.tsx
│   │   ├── Input.tsx
│   │   ├── Card.tsx
│   │   └── Icon.tsx
│   ├── layout/             # Layout components
│   │   ├── TitleBar.tsx
│   │   ├── Sidebar.tsx
│   │   └── StatusBar.tsx
│   └── features/           # Feature-specific components
│       ├── AudioVisualizer.tsx
│       ├── TranscriptView.tsx
│       └── SettingsPanel.tsx
├── styles/
│   ├── globals.css         # CSS custom properties
│   ├── components.css      # Component styles
│   └── utilities.css       # Utility classes
└── assets/
    ├── icons/              # SVG icon files
    └── images/             # Brand images
```

This design system provides a comprehensive foundation for building KagiNote's desktop application with consistent, accessible, and platform-appropriate visual design.