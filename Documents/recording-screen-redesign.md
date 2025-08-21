# KagiNote Recording Screen Redesign Proposal

## Executive Summary

This document proposes a comprehensive redesign of the KagiNote recording screen to address critical usability issues with layout proportions and transcript card overflow. The new design prioritizes the transcript content as the primary interface element while maintaining accessibility to essential controls.

## Current Issues Analysis

### 1. Layout Proportion Problems
- **Current State**: Two-column grid layout with audio controls taking 50% of horizontal space
- **Issue**: Transcript area is relegated to only half the screen width, severely limiting readability
- **Impact**: Users struggle to follow live transcriptions, especially in longer meetings

### 2. Transcript Card Overflow
- **Current State**: Individual transcript segments can grow indefinitely without internal scrolling
- **Issue**: Long segments push content off-screen, breaking the layout
- **Impact**: Users lose context and can't navigate efficiently through the transcript

### 3. Visual Hierarchy Imbalance
- **Current State**: Audio controls, meeting details, and system info compete for attention
- **Issue**: Secondary information takes prime real estate from primary content
- **Impact**: Cognitive overload and reduced focus on actual transcript content

## Proposed Design Solution

### Layout Architecture: Transcript-First Approach

```
┌─────────────────────────────────────────────────────────────┐
│  Compact Control Bar (8% - 64px)                           │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ [●REC] Meeting Title | 00:25:34 | [||][■] | Settings  │ │
│  └───────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Transcript Area (87% - Dynamic)                           │
│  ┌───────────────────────────────────────────────────────┐ │
│  │  ┌─────────────────────────────────────────────────┐  │ │
│  │  │ Speaker A • 00:12:45                            │  │ │
│  │  │ ┌───────────────────────────────────────────┐   │  │ │
│  │  │ │ Transcript text with internal scrolling    │   │  │ │
│  │  │ │ if content exceeds max-height (120px)...   │   │  │ │
│  │  │ └───────────────────────────────────────────┘   │  │ │
│  │  └─────────────────────────────────────────────────┘  │ │
│  │                                                        │ │
│  │  ┌─────────────────────────────────────────────────┐  │ │
│  │  │ Speaker B • 00:13:02                            │  │ │
│  │  │ ┌───────────────────────────────────────────┐   │  │ │
│  │  │ │ Next segment with its own scroll area...   │   │  │ │
│  │  │ └───────────────────────────────────────────┘   │  │ │
│  │  └─────────────────────────────────────────────────┘  │ │
│  └───────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  Status Bar (5% - 40px)                                    │
│  [Shield] Local Processing | [Model: Standard] | CPU: 15%  │
└─────────────────────────────────────────────────────────────┘
```

### Compact Control Bar Design (Collapsed State)

```
┌──────────────────────────────────────────────────────────────┐
│ [●] Live Recording • Team Standup          00:25:34          │
│ ├─ Recording indicator (pulsing)                             │
│ ├─ Meeting title (editable on click)                         │
│ └─ Duration counter                                          │
│                                                              │
│ Audio: ▁▃▅▇▅▃▁ [VAD]     [⏸][■]     [↓] More     [⚙]       │
│ ├─ Compact audio levels  ├─ Controls  ├─ Expand   └─ Settings│
└──────────────────────────────────────────────────────────────┘
```

### Expanded Control Panel (On-Demand)

When user clicks "More" (↓), a slide-down panel appears:

```
┌──────────────────────────────────────────────────────────────┐
│ Meeting Details                          [↑] Less            │
├──────────────────────────────────────────────────────────────┤
│ Model: Standard (Medium)        Language: English            │
│ Speakers: Auto-detecting         Quality: High Accuracy      │
│ Audio Device: MacBook Pro Mic   Sample Rate: 48kHz → 16kHz   │
│                                                              │
│ System Performance                                           │
│ CPU: 15% | RAM: 2.1GB | RTF: 0.8x | Processing: Local ✓     │
└──────────────────────────────────────────────────────────────┘
```

## Detailed Specifications

### 1. Control Bar (64px fixed height)
- **Position**: Fixed top
- **Height**: 64px (4rem)
- **Background**: Semi-transparent with backdrop blur
- **Z-index**: High to float above transcript during scroll
- **Contents**:
  - Recording status indicator (16px)
  - Meeting title (flex-grow)
  - Duration display (monospace font)
  - Compact audio visualizer (32px height)
  - Primary controls (40px square buttons)
  - Overflow menu for secondary actions

### 2. Transcript Area (calc(100vh - 104px))
- **Position**: Below control bar
- **Height**: Dynamic (viewport minus control bar and status bar)
- **Padding**: 24px horizontal, 16px vertical
- **Background**: Neutral-50 (light) / Neutral-900 (dark)
- **Scroll**: Vertical auto, hidden horizontal
- **Container**: Max-width 1200px, centered on large screens

### 3. Transcript Cards (Individual Segments)
- **Layout**: Vertical stack with 12px gap
- **Max Height**: 120px per card
- **Overflow**: Internal vertical scroll with subtle indicators
- **Border**: 1px solid with hover/active states
- **Padding**: 16px
- **Components**:
  ```
  ┌─────────────────────────────────────────┐
  │ [Avatar] Speaker Name    00:12:45  [⋮] │ <- Header (24px)
  ├─────────────────────────────────────────┤
  │ Scrollable content area                 │ <- Body (max 96px)
  │ with internal overflow-y: auto          │
  │ ▼ (scroll indicator if needed)          │
  └─────────────────────────────────────────┘
  ```

### 4. Status Bar (40px fixed height)
- **Position**: Fixed bottom
- **Height**: 40px (2.5rem)
- **Background**: Slightly darker than main background
- **Contents**: Privacy indicators, model info, system metrics
- **Typography**: 12px (0.75rem) secondary text

## Responsive Breakpoints

### Desktop (≥1024px)
- Full layout as designed
- Transcript max-width: 1200px
- Optimal reading width: 80ch for transcript text

### Tablet (768px - 1023px)
- Control bar remains fixed
- Transcript takes full width with 16px padding
- Status bar auto-hides after 3 seconds of inactivity

### Mobile (< 768px)
- Simplified control bar with hamburger menu
- Transcript cards stack with 8px gaps
- Bottom sheet pattern for expanded controls

## Implementation Strategy

### CSS/Tailwind Changes Required

```css
/* Control Bar - Fixed positioning */
.control-bar {
  @apply fixed top-0 left-0 right-0 h-16 z-50;
  @apply bg-white/90 dark:bg-gray-900/90 backdrop-blur-lg;
  @apply border-b border-neutral-200 dark:border-neutral-700;
}

/* Transcript Container - Full height minus fixed elements */
.transcript-container {
  @apply pt-16 pb-10; /* Account for fixed elements */
  height: 100vh;
  @apply overflow-y-auto overflow-x-hidden;
}

/* Transcript Card - Contained scrolling */
.transcript-card {
  @apply relative max-h-[120px] overflow-hidden;
}

.transcript-card-content {
  @apply overflow-y-auto max-h-[96px];
  @apply scrollbar-thin scrollbar-thumb-neutral-300;
}

/* Scroll indicators */
.scroll-indicator {
  @apply absolute bottom-0 left-0 right-0 h-6;
  @apply bg-gradient-to-t from-white dark:from-gray-900;
  @apply pointer-events-none opacity-0 transition-opacity;
}

.has-overflow .scroll-indicator {
  @apply opacity-100;
}
```

### React Component Restructuring

1. **RecordingScreen.tsx**:
   - Remove two-column grid layout
   - Implement fixed control bar component
   - Full-width transcript view
   - Collapsible details panel

2. **TranscriptView.tsx**:
   - Add max-height constraint to individual cards
   - Implement internal scrolling per card
   - Add scroll indicators for overflowing content
   - Optimize virtualization for large transcripts

3. **New Component: CompactControlBar.tsx**:
   - Horizontal layout with all controls
   - Expandable panel for details
   - Responsive overflow handling

## Performance Optimizations

### Virtualization Strategy
- Use `react-window` for transcript segments
- Only render visible cards plus buffer
- Estimated improvement: 60% reduction in DOM nodes

### Scroll Performance
- Use CSS `contain: layout` on transcript cards
- Implement `will-change: scroll-position` on containers
- Debounce scroll event handlers (16ms)

### Memory Management
- Limit transcript buffer to last 500 segments
- Implement pagination for historical transcripts
- Lazy load older segments on demand

## Accessibility Improvements

1. **Keyboard Navigation**:
   - Tab through transcript cards
   - Arrow keys for scrolling within cards
   - Escape to collapse expanded panels

2. **Screen Reader Support**:
   - Proper ARIA labels for all controls
   - Live regions for new transcript segments
   - Descriptive headings hierarchy

3. **Visual Indicators**:
   - Clear focus states (2px outline)
   - Scroll indicators for overflow content
   - High contrast mode support

## Migration Path

### Phase 1: Layout Restructure (Week 1)
- Implement new layout proportions
- Create compact control bar
- Basic transcript area expansion

### Phase 2: Scroll Fixes (Week 2)  
- Add internal card scrolling
- Implement scroll indicators
- Fix overflow issues

### Phase 3: Polish & Optimization (Week 3)
- Add animations and transitions
- Implement virtualization
- Performance testing and refinement

## Success Metrics

- **Transcript Visibility**: 85% of viewport dedicated to content
- **Scroll Performance**: Maintain 60fps during active scrolling
- **Load Time**: < 100ms to display first transcript segment
- **Memory Usage**: < 50MB for 1-hour transcript
- **User Feedback**: Improved readability and navigation scores

## Technical Dependencies

- React 19 (existing)
- Tailwind CSS 3.4.17 (existing)
- react-window (new - for virtualization)
- Intersection Observer API (for lazy loading)

## Risk Mitigation

1. **Browser Compatibility**: Test on Safari, Chrome, Firefox, Edge
2. **Performance Regression**: Implement performance monitoring
3. **Accessibility**: Conduct screen reader testing
4. **Data Loss**: Maintain auto-save during UI transitions

## Conclusion

This redesign prioritizes the core user need - viewing and interacting with transcripts - while maintaining quick access to essential controls. The new layout provides:

- **85% more transcript viewing area**
- **Contained, predictable scrolling behavior**
- **Cleaner, more focused interface**
- **Better performance through virtualization**
- **Improved accessibility and keyboard navigation**

The implementation can be completed in phases, ensuring minimal disruption while delivering immediate improvements to the user experience.