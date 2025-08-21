# KagiNote Unified Design System

## Executive Summary

KagiNote is a privacy-first meeting transcription application built with Tauri v2, React 19, and TypeScript. This document consolidates all design specifications, methodologies, and implementation guidelines into a single comprehensive reference.

## Table of Contents

1. [Design Philosophy](#design-philosophy)
2. [Brand Identity](#brand-identity)
3. [Visual Design System](#visual-design-system)
4. [Component Architecture](#component-architecture)
5. [UX Implementations](#ux-implementations)
6. [Technical Requirements](#technical-requirements)
7. [Implementation Roadmap](#implementation-roadmap)

---

## Design Philosophy

### Core Principles

1. **Privacy-First Visual Language**
   - Every design decision reinforces privacy and security
   - Local-first iconography with shield and lock metaphors
   - Muted, professional colors conveying trustworthiness
   - Clear data flow visualization showing local-only operations

2. **Clarity Over Cleverness**
   - Transcription accuracy is paramount
   - High contrast text with generous line spacing
   - Clear typography hierarchy
   - Minimal visual noise

3. **Performance as Design**
   - Speed and responsiveness are features
   - Instant feedback on all interactions (<100ms)
   - Progressive disclosure of features
   - 60fps animations throughout

4. **Cultural Sensitivity**
   - Respect for multilingual users, especially Japanese
   - Font stacks supporting CJK characters
   - Appropriate information density
   - Cultural color considerations

5. **Professional Context**
   - Business-appropriate aesthetics
   - Keyboard-first navigation
   - Multi-window support
   - Export-friendly formats

### Design Process Framework

**Discovery → Definition → Design → Validation → Implementation**

Each phase includes:
- User research and competitive analysis
- Technical constraints assessment
- Iterative design with testing
- Performance validation
- Quality assurance cycles

---

## Brand Identity

### Brand Personality
- **Trustworthy**: Reliable, consistent privacy protection
- **Professional**: Serious about business needs
- **Minimal**: Clean, focused on essentials
- **Intelligent**: AI-powered but not overwhelming
- **Local**: Privacy-first, on-device processing

### Brand Positioning
*"KagiNote is the privacy-first transcription tool for professionals who need accurate, reliable meeting transcription without compromising sensitive information."*

### Voice & Tone
- Professional yet approachable
- Clear and direct communication
- Confident but humble
- Respectful of user intelligence

---

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
--color-secondary-500: #10B981;
--color-secondary-600: #059669;

/* Neutral Professional Grays */
--color-neutral-50: #F9FAFB;   /* Background light */
--color-neutral-500: #6B7280;  /* Text secondary */
--color-neutral-900: #111827;  /* Text darkest */

/* Feedback Colors */
--color-warning-500: #F59E0B;  /* Warning states */
--color-error-500: #EF4444;    /* Error states */
```

### Typography System

#### Font Stack
```css
--font-family-primary: 
  "SF Pro Text",                /* macOS native */
  "Segoe UI Variable Text",     /* Windows 11 native */
  system-ui, 
  -apple-system, 
  sans-serif;

--font-family-cjk: 
  "Noto Sans CJK JP",           /* Japanese support */
  "Hiragino Kaku Gothic ProN", 
  "Yu Gothic Medium";

--font-family-mono: 
  "SF Mono", 
  "Cascadia Code", 
  monospace;
```

#### Type Scale
```css
--text-xs: 0.75rem;     /* 12px - Timestamps */
--text-sm: 0.875rem;    /* 14px - Labels */
--text-base: 1rem;      /* 16px - Body text */
--text-lg: 1.125rem;    /* 18px - Large body */
--text-xl: 1.25rem;     /* 20px - Subheadings */
--text-2xl: 1.5rem;     /* 24px - Section headings */
--text-3xl: 1.875rem;   /* 30px - Page headings */

/* Line Heights */
--leading-normal: 1.5;   /* Body text */
--leading-relaxed: 1.75; /* CJK characters */
--leading-loose: 2;      /* Transcription text */
```

### Spacing System (4px base unit)
```css
--spacing-0: 0;
--spacing-1: 0.25rem;   /* 4px */
--spacing-2: 0.5rem;    /* 8px */
--spacing-3: 0.75rem;   /* 12px */
--spacing-4: 1rem;      /* 16px */
--spacing-6: 1.5rem;    /* 24px */
--spacing-8: 2rem;      /* 32px */

/* Layout-specific */
--sidebar-width: 280px;
--titlebar-height: 44px;
--toolbar-height: 56px;
--statusbar-height: 32px;
```

### Effects

#### Border Radius
```css
--radius-sm: 0.125rem;    /* 2px */
--radius-base: 0.25rem;   /* 4px */
--radius-md: 0.375rem;    /* 6px */
--radius-lg: 0.5rem;      /* 8px */
--radius-xl: 0.75rem;     /* 12px */
```

#### Shadows
```css
--shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
--shadow-base: 0 1px 3px 0 rgba(0, 0, 0, 0.1);
--shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
--shadow-lg: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
--shadow-window: 0 8px 32px rgba(0, 0, 0, 0.12);
```

#### Animations
```css
--ease-out: cubic-bezier(0, 0, 0.2, 1);
--duration-fast: 150ms;
--duration-normal: 200ms;
--transition-colors: color var(--duration-fast) var(--ease-out);
```

---

## Component Architecture

### Window Structure
```
┌─────────────────────────────────────────────────┐
│ Title Bar (Platform-specific controls)          │
├─────────┬───────────────────────────────────────┤
│ Sidebar │         Main Content Area             │
│ (280px) │                                       │
├─────────┴───────────────────────────────────────┤
│ Status Bar (Model info, Recording status)       │
└─────────────────────────────────────────────────┘
```

### Component Hierarchy
```
src/
├── components/
│   ├── ui/          # Base components
│   │   ├── Button.tsx
│   │   ├── Input.tsx
│   │   ├── Card.tsx
│   │   └── Icon.tsx
│   ├── layout/      # Structure components
│   │   ├── TitleBar.tsx
│   │   ├── Sidebar.tsx
│   │   └── StatusBar.tsx
│   └── features/    # Domain-specific
│       ├── AudioVisualizer.tsx
│       ├── TranscriptionController.tsx
│       └── TranscriptView.tsx
```

### Key Component Patterns

#### Audio Visualizer
- Real-time waveform with 60fps animations
- State-based coloring (green=recording, blue=playing)
- Responsive canvas rendering
- Connected to backend audio levels

#### Recording Controls
```tsx
interface RecordingControlsProps {
  isRecording: boolean;
  isPaused: boolean;
  duration: number;
  onStart: () => void;
  onPause: () => void;
  onStop: () => void;
}
```

#### Transcript Display
- Clickable timestamps for audio seeking
- Speaker identification with color coding
- Inline editing capabilities
- Confidence level indicators

#### Privacy Indicators
```
┌─────────────────────────┐
│ 🔒 Private & Local      │
│ 🏠 Processed on device  │
│ 🚫 No network required  │
└─────────────────────────┘
```

### Icon System
- **Size**: 24x24px base (16px, 32px variants)
- **Stroke**: 2px consistent
- **Style**: Outlined, minimal fills
- **Categories**: Navigation, Audio, Privacy, Files, Status

---

## UX Implementations

### Three Elite Design Variants

#### V4-Alpha: "Horizon" - Professional Hybrid
**Target**: Business Professionals
**Innovation**: Intelligent hybrid with context-aware UI

**Key Features**:
- Smart speaker bubbles with professional styling
- Confidence visualization (opacity indicators)
- Collapsible smart sidebar
- Keyboard-first navigation

```typescript
interface HorizonConfig {
  layout: {
    sidebarWidth: 280; // 60px collapsed
    bubbleMaxWidth: '70%';
  };
  speakers: {
    maxVisible: 8;
    colorPalette: 'professional';
    avatarStyle: 'initials';
  };
}
```

#### V4-Beta: "Timeline" - Temporal Power Tool
**Target**: Power Users & Analysts
**Innovation**: Revolutionary timeline interface

**Key Features**:
- Multi-resolution timeline (5s to full meeting)
- Speaker lanes with visual activity
- Advanced navigation (scrubbing, search)
- Inspector panel for segment details

```typescript
interface TimelineConfig {
  visualization: {
    laneHeight: 40; // px per speaker
    zoomLevels: [5, 30, 60, 300, 1800]; // seconds
  };
  analysis: {
    overlapDetection: boolean;
    topicBoundaries: boolean;
  };
}
```

#### V4-Gamma: "Pulse" - Adaptive AI Interface
**Target**: Modern Teams
**Innovation**: Intelligent interface that adapts to context

**Key Features**:
- Context-aware adaptation
- AI-powered enhancements
- Living interface elements
- Gesture-based interactions

**Adaptive States**:
1. Minimal (Start/Few speakers)
2. Active (Multiple speakers detected)
3. Review (Post-meeting with AI summary)

### Recommended Implementation: Professional Hybrid

Based on analysis, the optimal approach combines:
- Chat bubble system for speaker separation (V3 strength)
- Professional horizontal layout (V1 foundation)
- Timeline navigation for long meetings (new addition)
- Progressive disclosure of features

---

## Technical Requirements

### Core Technologies
- **Framework**: React 19 with TypeScript
- **Styling**: Tailwind CSS v3.4.17 + CSS Modules
- **State**: Zustand or Jotai
- **Animation**: Framer Motion
- **Build**: Vite with Tauri v2

### Performance Targets
- **First Paint**: <100ms
- **Interactive**: <300ms
- **60 FPS**: All animations
- **Memory**: <500MB for 1-hour session
- **Bundle Size**: <200KB gzipped

### Accessibility Requirements
- **WCAG 2.1 AA** compliance minimum
- **Keyboard Navigation**: 100% coverage
- **Screen Reader**: Full ARIA support
- **High Contrast**: Mode available
- **Font Scaling**: 50-200% support

### Platform-Specific Features

#### macOS Integration
- Traffic light controls positioning
- Translucent sidebars with backdrop blur
- SF Symbols integration
- Native context menus

#### Windows Integration
- Title bar customization for Windows 11
- Fluent Design materials (Mica/Acrylic)
- Snap layout support
- Segoe UI optimization

---

## Implementation Roadmap

### Phase 1: Foundation (Week 1-2)
✅ Tailwind configuration with design tokens
✅ Global styles and CSS custom properties
✅ Platform detection utilities
✅ Icon system implementation

### Phase 2: Core Components (Week 2-3)
✅ Base UI component library
✅ Layout components
✅ Component documentation

### Phase 3: Feature Components (Week 3-4)
✅ Audio visualization
✅ Recording controls with state management
✅ Transcript display with editing
✅ Settings panels

### Phase 4: Integration (Week 4-5)
✅ Tauri command integration
✅ Dark mode implementation
✅ Performance optimization
✅ Accessibility improvements

### Phase 5: Platform Polish (Week 5-6)
✅ macOS-specific enhancements
✅ Windows-specific adaptations
✅ Cross-platform testing
✅ Final audits

## Success Metrics

### User Experience
- Task completion rate: >95%
- Error rate: <2%
- Satisfaction score: 8.5/10
- Privacy confidence: 90%+

### Technical Performance
- Application launch: <2 seconds
- Model loading (cached): <1 second
- UI responsiveness: <100ms
- Animation performance: 60fps

### Accessibility
- Keyboard navigation: 100% coverage
- Screen reader compatibility: Full support
- Color contrast: WCAG AA compliant
- Focus management: Logical order

## Design Decisions Summary

### What We're Building
- **Privacy-first** visual language throughout
- **Professional** yet approachable interface
- **Chat-style** speaker separation for clarity
- **Timeline navigation** for power users
- **Adaptive features** based on context
- **Cross-platform** consistency with native feel

### What We're Avoiding
- Generic SaaS appearance
- Information overload
- Complex dashboards
- Cloud-first messaging
- Decorative elements
- Platform inconsistency

## Maintenance Strategy

### Design System Evolution
- Component versioning for compatibility
- Design token updates via CSS properties
- Regular accessibility audits
- Performance monitoring

### Quality Assurance
- Automated testing (>80% coverage)
- Cross-platform validation
- User feedback integration
- Continuous optimization

---

## Quick Reference

### Colors
- Primary: `#2563EB` (Trust Blue)
- Secondary: `#10B981` (Privacy Green)
- Text: `#111827` (Near Black)
- Background: `#FFFFFF` (White)

### Spacing
- Small: `8px`
- Medium: `16px`
- Large: `24px`
- XLarge: `32px`

### Typography
- Body: `16px/1.5`
- Heading: `24px/1.25`
- Caption: `14px/1.5`

### Breakpoints
- Compact: `800-1000px`
- Standard: `1000-1400px`
- Large: `1400px+`

---

*This unified design system provides KagiNote with a comprehensive foundation for creating a best-in-class desktop transcription application that prioritizes privacy, professionalism, and user experience.*

**Last Updated**: August 2025
**Version**: 1.0
**Status**: Production Ready