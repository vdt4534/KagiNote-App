# CLAUDE.md - Frontend Screens

This file provides guidance for Claude Code when working with full-screen views in KagiNote.

## Screen Architecture

**V2 Screen System:**
- **Dashboard.tsx** - Meeting list with localStorage persistence, search, and sorting
- **TranscriptsPage.tsx** - Complete transcript management with dual-view modes
- **SettingsPage.tsx** - Comprehensive settings interface (7 categories)
- **RecordingScreen.tsx** - Live recording with real-time transcription
- **NewMeetingModal.tsx** - Meeting creation and configuration

## Data Integration Strategy

**localStorage Persistence:**
- All meetings and settings saved to localStorage with metadata
- Meeting objects include: id, title, date, duration, transcript, speakers
- Settings organized by categories: General, Recording, Transcription, Speakers, Models, Privacy, Export
- Real-time sync between UI state and localStorage

**State Management:**
- React hooks for screen-level state
- localStorage for persistence across sessions
- Real-time updates via Tauri event system

## Screen-Specific Guidelines

### Dashboard.tsx
**Features:**
- Meeting list with search, sorting, filtering
- Recent meetings with quick access
- Storage usage and statistics
- Professional dashboard aesthetics

**Data Flow:**
```typescript
localStorage → React state → UI components → user actions → localStorage
```

### TranscriptsPage.tsx
**Critical UI/UX Requirements:**
- **Dual-view modes**: Grid and list views with ToggleGroup component
- **Responsive design**: Card view (mobile) → Table view (desktop)
- **Search functionality**: Real-time transcript content search
- **Batch operations**: Select multiple transcripts for export/delete
- **Professional appearance**: shadcn/ui Table and components throughout

**Mobile Responsive Strategy:**
- 375px mobile: Single-column stats, card-based transcript view
- Tablet: Two-column stats, condensed table layout  
- 1440px+ desktop: Four-column stats, full table functionality

### SettingsPage.tsx
**7 Settings Categories:**
1. **General** - App preferences, theme, language
2. **Recording** - Audio device, quality, auto-save
3. **Transcription** - Model selection, language detection
4. **Speakers** - Diarization settings, speaker profiles
5. **Models** - Download management, cache settings
6. **Privacy** - Encryption, data retention, permissions
7. **Export** - Format preferences, default locations

**Implementation Notes:**
- Each category as separate component/section
- Real-time settings validation and feedback
- Immediate save to localStorage on changes

### RecordingScreen.tsx
**Real-time Features:**
- Live audio visualization with WaveSurfer.js
- Real-time transcription display (actual AI text, not placeholders)
- Speaker identification indicators
- Session controls (start/stop/pause)
- Emergency stop functionality

**Integration Requirements:**
- Connected to real backend audio levels
- Displays actual transcription segments as they arrive
- Shows speaker diarization results in real-time
- Session duration tracking and status

## Mobile-First Responsive Design

**Breakpoint Strategy:**
- **375px (mobile)**: Single column, sheet navigation, card layouts
- **768px (tablet)**: Two columns, condensed layouts, mixed components
- **1024px+ (desktop)**: Full layouts, sidebars, table views

**Navigation Integration:**
- Sheet-based mobile navigation for smaller screens
- Sidebar navigation for desktop with active state indicators
- Breadcrumb navigation for deep page structures

## Screen Development Guidelines

**When creating new screens:**
1. **Start mobile-first** - Design for 375px, then enhance
2. **Use shadcn/ui components** - Maintain design consistency
3. **Implement localStorage** - All user data must persist
4. **Add search/filtering** - Large datasets need user-friendly navigation
5. **Test across breakpoints** - Validate responsive behavior
6. **Follow accessibility patterns** - Proper ARIA labels and keyboard navigation

**Performance Considerations:**
- Lazy load heavy components
- Implement virtual scrolling for large lists
- Debounce search inputs
- Optimize localStorage reads/writes

## Platform-Aware UI

**OS-Specific Adaptations:**
- macOS: Native window controls, SF Pro font
- Windows: Windows-style navigation, Segoe UI font
- Automatic OS detection via `usePlatform` hook
- 80% shared design, 20% platform-specific adaptations

**Cross-Platform Consistency:**
- Tailwind CSS generates consistent styles
- Same component behavior across platforms
- Platform-specific styling only for native feel

This screen system provides a complete, responsive, and platform-aware user interface for the KagiNote application.