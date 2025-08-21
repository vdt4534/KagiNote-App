# KagiNote Visual Wireframes & User Flow

## 🎯 Core User Journey

### User Flow Overview
```
┌─────────────┐     ┌──────────────┐     ┌────────────┐     ┌──────────────┐
│  Dashboard  │ --> │ New Meeting  │ --> │  Recording │ --> │    Review    │
│  (Home)     │     │   Setup      │     │    Live    │     │   Meeting    │
└─────────────┘     └──────────────┘     └────────────┘     └──────────────┘
       ↑                                         │                    │
       └─────────────────────────────────────────┴────────────────────┘
                            Save & Return
```

## 📱 Screen Designs

### 1. Dashboard - Meeting List View

```
┌────────────────────────────────────────────────────────────────────┐
│  🔒 KagiNote                                    ⚙️  👤  ?          │
│  100% Local Privacy • No Cloud Required                           │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│  ┌──────────────────────────────────────────────────────────┐     │
│  │  🔍 Search across all meetings...                        │     │
│  └──────────────────────────────────────────────────────────┘     │
│                                                                    │
│     ┌─────────────────┐  ┌──────────────┐  ┌──────────┐         │
│     │  + New Meeting  │  │ Import Audio │  │ Settings │         │
│     └─────────────────┘  └──────────────┘  └──────────┘         │
│                                                                    │
│  Your Meetings                                    Sort: Recent ▼  │
│  ─────────────────────────────────────────────────────────────   │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────┐     │
│  │  ┌──┐                                                    │     │
│  │  │📝│  Team Standup Meeting                             │     │
│  │  └──┘  Dec 20, 2024 • 15:32 • 3 speakers               │     │
│  │        ━━━━━━━━━━━━━━━━━━━━━━  95% accuracy            │     │
│  │                                                          │     │
│  │  "Discussed Q4 roadmap planning and sprint tasks..."    │     │
│  │                                              [Open →]   │     │
│  └──────────────────────────────────────────────────────────┘     │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────┐     │
│  │  ┌──┐                                                    │     │
│  │  │📝│  Client Presentation                              │     │
│  │  └──┘  Dec 19, 2024 • 45:12 • 5 speakers               │     │
│  │        ━━━━━━━━━━━━━━━━━━━━━━  92% accuracy            │     │
│  │                                                          │     │
│  │  "Product demo and feedback session with stakeholders..."│     │
│  │                                              [Open →]   │     │
│  └──────────────────────────────────────────────────────────┘     │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

### 2. New Meeting Setup

```
┌────────────────────────────────────────────────────────────────────┐
│  New Meeting Setup                                         [X]    │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│  Meeting Information                                              │
│  ┌──────────────────────────────────────────────────────────┐     │
│  │  Meeting Title                                           │     │
│  │  ┌──────────────────────────────────────────────────┐   │     │
│  │  │ Weekly Team Sync                                 │   │     │
│  │  └──────────────────────────────────────────────────┘   │     │
│  │                                                          │     │
│  │  Participants (optional)                                │     │
│  │  ┌──────────────────────────────────────────────────┐   │     │
│  │  │ John, Sarah, Mike                                │   │     │
│  │  └──────────────────────────────────────────────────┘   │     │
│  └──────────────────────────────────────────────────────────┘     │
│                                                                    │
│  Transcription Quality                                            │
│  ┌──────────────────────────────────────────────────────────┐     │
│  │  ⚡ Turbo         Fast processing, 4GB RAM              │     │
│  │  ✅ Standard      Balanced quality, 8GB RAM            │     │
│  │  🎯 High Accuracy Best quality, 16GB RAM               │     │
│  └──────────────────────────────────────────────────────────┘     │
│                                                                    │
│  Language Settings                                                │
│  ┌──────────────────────────────────────────────────────────┐     │
│  │  Primary: [English ▼]    ☑ Auto-detect others          │     │
│  └──────────────────────────────────────────────────────────┘     │
│                                                                    │
│  ✓ Microphone: MacBook Pro Microphone                            │
│  ✓ Model loaded: whisper-medium (1.5GB)                          │
│  ✓ Available RAM: 8.2GB                                          │
│                                                                    │
│         ┌─────────────────┐     ┌──────────┐                    │
│         │ Start Recording │     │  Cancel  │                    │
│         └─────────────────┘     └──────────┘                    │
└────────────────────────────────────────────────────────────────────┘
```

### 3. Active Recording - Live Transcription

```
┌────────────────────────────────────────────────────────────────────┐
│  🔴 Recording...                                    00:05:23  ⏹   │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│  ┌──────────────────────────────────────────────────────────┐     │
│  │     Audio Levels                                         │     │
│  │  ▁▃▅▇▅▃▁▂▄▆█▆▄▂▁▃▅▇█▇▅▃▁▂▄▆▇▅▃▁▂▄▆█▆▄▂▁▃▅        │     │
│  │  ████████░░░░░░░░  Voice Activity Detected              │     │
│  └──────────────────────────────────────────────────────────┘     │
│                                                                    │
│  Live Transcript                                                  │
│  ┌──────────────────────────────────────────────────────────┐     │
│  │                                                          │     │
│  │  Speaker 1 (00:04:58)                                   │     │
│  │  ─────────────────────                                  │     │
│  │  So the main focus for this sprint will be implementing │     │
│  │  the new authentication system. We need to ensure it's  │     │
│  │  compatible with our existing infrastructure.           │     │
│  │                                                          │     │
│  │  Speaker 2 (00:05:15)                                   │     │
│  │  ─────────────────────                                  │     │
│  │  I agree, but we should also consider the performance   │     │
│  │  implications of adding another layer of...             │     │
│  │                                                          │     │
│  │  [● Currently speaking...]                              │     │
│  │                                                          │     │
│  └──────────────────────────────────────────────────────────┘     │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────┐     │
│  │ 🔒 Processing locally • CPU: 15% • RAM: 2.1GB • RTF: 0.8x│     │
│  └──────────────────────────────────────────────────────────┘     │
│                                                                    │
│         ┌─────────┐  ┌─────────┐  ┌──────────┐                  │
│         │ ⏸ Pause │  │ ⏹ Stop  │  │ Settings │                  │
│         └─────────┘  └─────────┘  └──────────┘                  │
└────────────────────────────────────────────────────────────────────┘
```

### 4. Meeting Review & Playback

```
┌────────────────────────────────────────────────────────────────────┐
│  Team Standup Meeting                               📤 Export  ⚙️  │
│  Dec 20, 2024 • 15:32 duration • 3 speakers • 95% accuracy       │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│  ┌──────────────────────────────────────────────────────────┐     │
│  │  ▶ ━━━━━━━●━━━━━━━━━━━━━━━━━━━━━  05:23 / 15:32       │     │
│  │  [▶/⏸]  [⏮10s] [⏭10s]  🔊  Speed: 1.0x                │     │
│  └──────────────────────────────────────────────────────────┘     │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────┐     │
│  │  🔍 Search in transcript...                              │     │
│  └──────────────────────────────────────────────────────────┘     │
│                                                                    │
│  Transcript                                                       │
│  ┌──────────────────────────────────────────────────────────┐     │
│  │                                                          │     │
│  │  👤 John (00:00:12)                                     │     │
│  │  Good morning everyone, let's start with our daily      │     │
│  │  standup. Sarah, would you like to go first?            │     │
│  │                                                          │     │
│  │  👤 Sarah (00:00:25)                                    │     │
│  │  Sure! Yesterday I completed the API integration for    │     │
│  │  the payment system. Today I'll be working on the       │     │
│  │  testing suite for that integration.                    │     │
│  │                                                          │     │
│  │  👤 Mike (00:01:45)                                     │     │
│  │  I'm still blocked on the database migration issue.     │     │
│  │  I need help from DevOps to resolve the permissions...  │     │
│  │                                                          │     │
│  │  [Load more...]                                         │     │
│  └──────────────────────────────────────────────────────────┘     │
│                                                                    │
│  Speaker Distribution:  John 40% ████  Sarah 35% ███  Mike 25% ██ │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

### 5. Search Results View

```
┌────────────────────────────────────────────────────────────────────┐
│  Search: "authentication"                          23 results     │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│  Filter: [All Time ▼] [All Speakers ▼] [All Meetings ▼]         │
│                                                                    │
│  Search Results                                                   │
│  ─────────────────────────────────────────────────────────────   │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────┐     │
│  │  📝 Engineering Sync - Dec 18, 2024                     │     │
│  │                                                          │     │
│  │  "...implementing the new authentication system with     │     │
│  │  OAuth 2.0 support and multi-factor..."                 │     │
│  │                                                          │     │
│  │  Speaker: Mike • 12:34 mark            [Jump to →]     │     │
│  └──────────────────────────────────────────────────────────┘     │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────┐     │
│  │  📝 Security Review - Dec 15, 2024                      │     │
│  │                                                          │     │
│  │  "...the authentication flow needs to be revised to     │     │
│  │  meet the new compliance requirements..."               │     │
│  │                                                          │     │
│  │  Speaker: Sarah • 08:15 mark            [Jump to →]    │     │
│  └──────────────────────────────────────────────────────────┘     │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────┐     │
│  │  📝 Client Demo - Dec 10, 2024                          │     │
│  │                                                          │     │
│  │  "...customer asked about SSO authentication options    │     │
│  │  and whether we support SAML..."                        │     │
│  │                                                          │     │
│  │  Speaker: John • 23:42 mark             [Jump to →]    │     │
│  └──────────────────────────────────────────────────────────┘     │
│                                                                    │
│                    [← Previous] 1 of 3 [Next →]                  │
└────────────────────────────────────────────────────────────────────┘
```

## 🎨 Design System

### Color Palette
- **Primary Blue**: `#3B82F6` - Actions, links
- **Privacy Green**: `#10B981` - Success, secure status
- **Dark Gray**: `#1F2937` - Primary text
- **Light Gray**: `#F9FAFB` - Backgrounds
- **Red**: `#EF4444` - Stop, errors
- **Orange**: `#F59E0B` - Warnings

### Typography
- **Headers**: System font (SF Pro/Segoe UI) - Bold
- **Body**: System font - Regular
- **Monospace**: SF Mono/Consolas - Code, timestamps

### Key UI Components
- **Cards**: Rounded corners, subtle shadow
- **Buttons**: Primary (blue), Secondary (gray), Danger (red)
- **Status Indicators**: Green dot for active, gray for inactive
- **Progress Bars**: Show accuracy, processing status
- **Audio Waveform**: Real-time visualization

## 🔄 User Interaction Flows

### Starting a New Meeting
1. User clicks "New Meeting" on dashboard
2. Fills in meeting details (optional)
3. Selects quality tier based on hardware
4. System checks microphone and model status
5. User clicks "Start Recording"
6. Transitions to live recording view

### Searching for Information
1. User types query in search bar
2. Results appear instantly as they type
3. Can filter by date, speaker, or meeting
4. Click result to jump directly to that section
5. Transcript highlights the search term

### Reviewing a Meeting
1. Select meeting from dashboard
2. Audio player loads at top
3. Can scrub through timeline
4. Transcript follows audio position
5. Export options available in multiple formats

## 🔐 Privacy-First Design Elements

Throughout the interface:
- 🔒 Lock icon showing local processing
- "No cloud required" messaging
- Green status indicators for privacy
- Clear data location indicators
- Offline-first visual language