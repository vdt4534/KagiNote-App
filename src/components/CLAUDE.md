# CLAUDE.md - Frontend Components

This file provides guidance for Claude Code when working with React components in KagiNote.

## Component Architecture

**shadcn/ui Migration Strategy:**
- All components use shadcn/ui (New York style) with Radix UI primitives
- Compatibility layer in `/ui/compat.ts` maintains backward compatibility
- Gradual migration approach - both old and new components coexist

## Component Structure

```
components/
├── ui/           # Reusable UI primitives (Button, Card, Badge, Input, etc.)
├── features/     # Feature-specific components (AudioVisualizer, SpeakerDisplay)
├── layout/       # Layout components (AppLayout, Sidebar, StatusBar)
└── compat.ts     # Compatibility layer for migration
```

## UI Component Migration Rules

**When working with UI components:**
1. **Use shadcn/ui first** - Check if shadcn/ui component exists before creating custom
2. **Maintain compatibility** - Export both new and legacy components from `/ui/compat.ts`
3. **Follow New York style** - Use shadcn/ui New York variant configuration
4. **Preserve accessibility** - All components must use Radix UI primitives

**Component Variants:**
- Use Class Variance Authority (CVA) for component variants
- Map KagiNote design tokens to shadcn/ui components
- Maintain consistent spacing with 4px base unit system

## Mobile Responsiveness

**Responsive Strategy:**
- Mobile-first approach (375px → 1440px+)
- Sheet component for mobile navigation sidebar
- Progressive enhancement from mobile to desktop
- Breakpoint management: mobile → tablet → desktop

**Critical Responsive Rules:**
- Icon sizing: Add min-width/min-height constraints to prevent collapse
- Navigation: Use Sheet for mobile, Sidebar for desktop
- Stats grids: Single column mobile → multi-column desktop
- Table layouts: Card view mobile → table desktop

## Icon System

**Complete Icon Coverage Required:**
- All 44 icons implemented in `/ui/Icon.tsx`
- Heroicons-style SVG implementations
- Uniform stroke width and visual style
- Zero external dependencies - inline SVG only

**Recently Added Icons:** `users`, `calendar`, `trash`, `grid`, `list`, `check`, `alert-triangle`, `brain`, `volume-x`, `chart-bar`

## Component Development Guidelines

**When creating new components:**
1. Check shadcn/ui component library first
2. Use TypeScript with strict mode
3. Implement responsive design patterns
4. Add proper accessibility attributes
5. Follow KagiNote design system colors and spacing

**Component Testing:**
- React component tests with Vitest
- Use Testing Library for component interaction testing
- Test responsive behavior across breakpoints
- Validate accessibility with screen readers

## Key Components

**Layout Components:**
- `AppLayout.tsx` - Main application wrapper with navigation
- `Sidebar.tsx` - Desktop navigation with active state indicators  
- `StatusBar.tsx` - Application status and system information

**Feature Components:**
- `AudioVisualizer.tsx` - Real-time audio visualization with WaveSurfer.js
- `TranscriptionController.tsx` - Main transcription logic and controls
- `SpeakerDisplay.tsx` - Speaker identification and management
- `DiarizationStatusIndicator.tsx` - Real-time diarization status

**UI Primitives:**
- Use shadcn/ui Button, Card, Badge, Input, Select, etc.
- Custom Icon component for complete icon coverage
- Toast system for user notifications

## Styling Guidelines

**Tailwind CSS Integration:**
- Utility-first approach with Tailwind CSS v3.4.17
- Custom design tokens in tailwind.config.js
- Seamless integration with shadcn/ui components
- Dark mode support via CSS variables

**Professional Design Language:**
- Privacy-first visual indicators (shield/lock icons)
- Business-appropriate aesthetics
- Trust Blue (#2563EB), Privacy Green (#10B981), Professional Grays
- System fonts first: SF Pro (macOS), Segoe UI (Windows)

This component system ensures consistent, accessible, and responsive UI throughout the application.