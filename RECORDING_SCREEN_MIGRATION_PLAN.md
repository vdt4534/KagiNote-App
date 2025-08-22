# Recording Screen shadcn/ui Migration Plan

## Executive Summary
The Recording Screen (New Recording page) in KagiNote currently uses custom UI components. This migration plan outlines the strategy to refactor these components to use shadcn/ui while maintaining the privacy-first design system and ensuring backward compatibility.

## Component Analysis

### Current Component Usage in Recording Screen

| Component | Current Implementation | shadcn/ui Equivalent | Migration Priority |
|-----------|----------------------|---------------------|-------------------|
| Card | Custom Card with CardHeader/CardBody | shadcn/ui Card | High |
| Button | Custom Button with variants | button-new.tsx (partial) | High |
| Badge | Custom Badge with status colors | badge-new.tsx (partial) | High |
| Icon | Custom Icon wrapper | Keep custom (Lucide integration) | Low |
| Modal | MeetingDetailsPanel (custom) | shadcn/ui Dialog | Medium |

### Feature Components Dependencies

1. **ControlBar**
   - Uses: Button, Icon, Badge, AudioVisualizer
   - Migration Impact: High (core recording controls)

2. **TranscriptView**
   - Uses: Custom scrollable container
   - Migration: shadcn/ui ScrollArea

3. **DiarizationStatusIndicator**
   - Uses: Badge, Icon
   - Migration Impact: Medium

4. **SpeakerActivityDisplay**
   - Uses: Custom display components
   - Migration: Keep custom with shadcn primitives

5. **MeetingDetailsPanel**
   - Uses: Custom panel/modal
   - Migration: shadcn/ui Sheet or Dialog

## Design System Compatibility Assessment

### Color System Alignment
✅ **Compatible**: shadcn/ui uses CSS variables that align with KagiNote's design tokens
- Trust Blue (#2563EB) → --primary
- Privacy Green (#10B981) → --success/secondary
- Professional Grays → --neutral scale

### Component Patterns
✅ **Button**: button-new.tsx already implements compatibility layer
✅ **Badge**: badge-new.tsx partially implemented
⚠️ **Card**: Needs migration to maintain CardHeader/CardBody API
⚠️ **Modal**: MeetingDetailsPanel needs Dialog/Sheet migration

### Privacy-First Indicators
✅ Shield/lock icons maintained through Icon component
✅ Local processing badges preserved
✅ Status indicators compatible with shadcn variants

## Migration Strategy

### Phase 1: Component Installation & Setup
```bash
# Install required shadcn components
npx shadcn@latest add card
npx shadcn@latest add dialog
npx shadcn@latest add sheet
npx shadcn@latest add scroll-area
npx shadcn@latest add separator
```

### Phase 2: Compatibility Wrappers

#### Card Compatibility Wrapper
```typescript
// src/components/ui/card-compat.tsx
import { Card as ShadcnCard, CardContent, CardHeader as ShadcnCardHeader } from "@/components/ui/card-new"
import { cn } from "@/lib/utils"

// Maintain existing CardHeader/CardBody API
export const Card = ShadcnCard

export const CardHeader = ({ className, ...props }: any) => (
  <ShadcnCardHeader className={cn("pb-2 sm:pb-3", className)} {...props} />
)

export const CardBody = ({ className, ...props }: any) => (
  <CardContent className={cn("p-0", className)} {...props} />
)
```

### Phase 3: Progressive Component Migration

#### Level 1: Atomic Components (Week 1)
- [x] Button → Use existing button-new.tsx
- [x] Badge → Use existing badge-new.tsx
- [ ] Card → Migrate to card-new.tsx with compatibility wrapper

#### Level 2: Composite Components (Week 2)
- [ ] MeetingDetailsPanel → Dialog/Sheet
- [ ] TranscriptView scrolling → ScrollArea
- [ ] Status indicators → Badge variants

#### Level 3: Feature Components (Week 3)
- [ ] ControlBar → Update imports only
- [ ] RecordingScreen → Update imports
- [ ] Keep AudioVisualizer custom (performance critical)

## Implementation Checklist

### Immediate Actions (Safe to implement now)
1. ✅ shadcn/ui already initialized (components.json exists)
2. ✅ Partial migration started (button-new.tsx, badge-new.tsx)
3. [ ] Install Card component from shadcn/ui
4. [ ] Create compatibility wrapper for Card
5. [ ] Update RecordingScreen imports

### Testing Requirements
- [ ] Visual regression testing before/after
- [ ] Performance validation (maintain <100ms interactions)
- [ ] Platform testing (Windows/macOS/Linux)
- [ ] Dark mode compatibility
- [ ] Accessibility audit (keyboard navigation, ARIA)

### Rollback Plan
```bash
# Create backup branch
git checkout -b recording-screen-pre-migration
git checkout -b shadcn-migration-recording-screen

# Keep compatibility imports
// src/components/ui/index.ts
export * from './card-compat' // Use compat during migration
// export * from './Card' // Original implementation preserved
```

## Code Migration Examples

### Before (Current Implementation)
```typescript
import { Card, CardHeader, CardBody } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Badge } from '@/components/ui/Badge';
```

### After (shadcn/ui Migration)
```typescript
import { Card, CardHeader, CardBody } from '@/components/ui/card-compat';
import { Button } from '@/components/ui/button-new';
import { Badge } from '@/components/ui/badge-new';
```

## Performance Considerations

### Critical Components to Keep Custom
1. **AudioVisualizer** - Real-time waveform rendering
2. **TranscriptionController** - Tauri IPC communication
3. **SpeakerActivityDisplay** - Real-time speaker updates

### shadcn/ui Performance Benefits
- Smaller bundle size with tree-shaking
- Radix UI primitives for accessibility
- CSS-in-JS eliminated (uses Tailwind)
- Better React 19 compatibility

## Risk Assessment

### Low Risk
- Button/Badge migration (compatibility layer exists)
- Card migration (simple wrapper needed)
- Import path updates

### Medium Risk
- MeetingDetailsPanel → Dialog migration
- Dark mode styling consistency
- Platform-specific adaptations

### Mitigation Strategies
1. Use compatibility wrappers for gradual migration
2. Maintain parallel implementations during transition
3. Feature flag for A/B testing
4. Comprehensive test coverage before deployment

## Success Metrics

- [ ] All components render identically post-migration
- [ ] Performance metrics maintained or improved
- [ ] Zero accessibility regressions
- [ ] Type safety preserved
- [ ] Bundle size reduced by >10%

## Next Steps

1. **Install Card component from shadcn/ui**
2. **Create card-compat.tsx wrapper**
3. **Update RecordingScreen.tsx imports**
4. **Run visual regression tests**
5. **Deploy to staging for validation**

## Notes

- shadcn/ui components are already partially integrated (button-new, badge-new)
- Design system is compatible with shadcn/ui's approach
- Privacy-first visual language will be preserved
- Performance-critical components will remain custom