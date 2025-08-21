# Speaker Diarization Guide

**Complete user guide for KagiNote's speaker identification and management features**

## Overview

KagiNote's speaker diarization identifies and separates different speakers in your meetings in real-time. The system uses advanced voice recognition technology to create persistent speaker profiles, allowing you to customize names, colors, and track speaking statistics across multiple sessions.

## Quick Start

### Enabling Speaker Diarization

1. **Automatic Mode** (Default): Speaker diarization is enabled automatically for all new recordings
2. **Manual Configuration**: Access settings through the recording interface to adjust speaker detection sensitivity

### Basic Usage

1. **Start Recording**: Begin your meeting as usual - speakers will be detected automatically
2. **Speaker Detection**: New speakers appear with default names (Speaker 1, Speaker 2, etc.)
3. **Customize Speakers**: Click on any speaker bubble to rename and change colors
4. **Review Results**: View speaker statistics and time distribution after recording

## Speaker Management

### Creating and Editing Speaker Profiles

**Rename Speakers:**
```
1. Click on any speaker bubble during or after recording
2. Click the edit icon (pencil) next to the speaker name
3. Enter the new name and press Enter to save
4. The name will be remembered for future sessions
```

**Customize Speaker Colors:**
```
1. Click on a speaker bubble to open the profile panel
2. Click the color picker next to the speaker name
3. Choose from the predefined palette or enter a custom hex color
4. Colors are automatically saved and applied to all segments
```

**Speaker Profile Information:**
- **Display Name**: Custom name for easy identification
- **Color**: Visual indicator used throughout the interface
- **Voice Characteristics**: Automatically detected pitch, formants, and speaking rate
- **Statistics**: Total speaking time, segment count, average confidence
- **Notes**: Optional user notes for additional context

### Understanding Confidence Scores

Speaker identification includes confidence scores to help you assess accuracy:

**Confidence Levels:**
- **High (0.8-1.0)**: Very reliable speaker identification
- **Medium (0.5-0.8)**: Good identification with some uncertainty
- **Low (0.0-0.5)**: Uncertain identification that may need manual review

**Visual Indicators:**
- **Solid Colors**: High confidence segments
- **Faded Colors**: Medium confidence segments
- **Dotted Borders**: Low confidence segments requiring attention

### Managing Multiple Speakers

**Speaker Detection:**
- Supports up to 8 speakers simultaneously
- New speakers are automatically detected and assigned colors
- Similar voices are clustered to prevent over-segmentation

**Handling Similar Voices:**
- System may initially separate similar speakers
- Use the "Merge Speakers" function to combine incorrectly split profiles
- Adjust similarity threshold in settings for better initial clustering

**Overlapping Speech:**
- Segments where multiple speakers talk simultaneously are marked with special indicators
- Both speakers' colors are shown with a gradient or split visualization
- Overlapping segments can be reviewed and manually assigned if needed

## Advanced Features

### Voice Characteristics

KagiNote automatically analyzes voice characteristics for better speaker separation:

**Measured Characteristics:**
- **Pitch**: Average fundamental frequency (Hz)
- **Formants**: F1 and F2 frequencies for voice quality
- **Speaking Rate**: Words per minute when transcription is available
- **Energy Level**: Relative voice volume and intensity

**Using Voice Data:**
- Characteristics help distinguish speakers with similar-sounding voices
- Data is used to improve future speaker detection accuracy
- All voice analysis happens locally - no data is sent to external servers

### Speaker Search and Filtering

**Finding Speakers:**
- Search by speaker name in the speaker panel
- Filter by confidence level to review uncertain segments
- Sort by speaking time, segment count, or alphabetically

**Timeline Navigation:**
- Click on any speaker in the sidebar to jump to their segments
- Use keyboard shortcuts (1-8) to quickly select speakers
- Timeline view shows speaker distribution over time

### Session Statistics

**Per-Speaker Metrics:**
- Total speaking time and percentage of meeting
- Number of segments and average segment length
- Speaking rate (words per minute)
- Average confidence score

**Meeting Overview:**
- Total number of unique speakers detected
- Speaker change frequency
- Overall diarization confidence
- Processing time and performance metrics

## Export and Import

### Exporting Speaker Data

**Transcript Export with Speakers:**
```
1. Complete your recording session
2. Go to File → Export → Transcript with Speakers
3. Choose format: JSON, CSV, or formatted text
4. Select whether to include confidence scores and timestamps
5. Export includes speaker names, colors, and segment attribution
```

**Speaker Profile Export:**
```
1. Navigate to Settings → Speaker Management
2. Click "Export All Profiles"
3. Includes speaker names, colors, voice characteristics, and statistics
4. Does NOT include raw audio or voice samples (privacy-first design)
5. Exported data can be imported on other devices or installations
```

### Import Options

**Profile Import:**
- Import previously exported speaker profiles
- Merge with existing profiles or replace entirely
- Voice characteristics help match speakers across sessions

**Transcript Import with Speakers:**
- Import existing transcripts with speaker attributions
- System will attempt to match imported speakers with existing profiles
- Manual review recommended for best accuracy

## Privacy and Security

### Data Protection

**Local Processing:**
- All speaker identification happens on your device
- No audio or voice data is ever sent to external servers
- Speaker embeddings are mathematical representations, not voice samples

**Encrypted Storage:**
- Speaker profiles stored with AES-256 encryption
- Voice characteristics and embeddings are anonymized
- Complete data control with export/delete options

**Memory Protection:**
- Audio buffers are securely wiped after processing
- No temporary audio files stored on disk
- Real-time processing minimizes data retention

### User Control

**Data Management:**
- View, edit, or delete any speaker profile
- Export all data at any time
- Clear all speaker data for fresh start
- No automatic data sharing or cloud sync

## Keyboard Shortcuts

**Speaker Management:**
- `1-8`: Select speakers 1-8 for quick navigation
- `Cmd/Ctrl + E`: Edit currently selected speaker
- `Cmd/Ctrl + C`: Copy speaker profile
- `Cmd/Ctrl + M`: Merge selected speakers
- `Space`: Play/pause at current speaker segment

**Timeline Navigation:**
- `→/←`: Move between speaker segments
- `Shift + →/←`: Jump to next/previous speaker change
- `Cmd/Ctrl + →/←`: Go to start/end of current speaker's segment

## Best Practices

### Optimal Speaker Detection

**Audio Setup:**
- Use good quality microphones when possible
- Minimize background noise and echo
- Ensure speakers are reasonably separated in space
- Avoid overlapping speech for critical content

**Session Management:**
- Let each speaker talk for at least 2-3 seconds initially
- Rename speakers early in the session for consistency
- Review and correct low-confidence segments promptly
- Use consistent names across multiple sessions

### Performance Optimization

**Hardware Considerations:**
- 8GB+ RAM recommended for 6+ speakers
- Apple Silicon or modern CPU for best performance
- Close other audio-intensive applications during recording

**Configuration Tuning:**
- Reduce max speakers if not needed (saves memory)
- Increase minimum segment duration for cleaner separation
- Adjust similarity threshold based on voice similarity in your group

## Integration with Transcription

### Combined Workflow

**Real-time Operation:**
- Transcription and diarization work simultaneously
- Speaker identification appears within 1-2 seconds
- Text is automatically attributed to the correct speaker

**Post-Processing:**
- Review speaker assignments after recording
- Correct any misattributions manually
- Merge or split segments as needed

**Export Integration:**
- Combined transcripts include speaker names and timing
- Professional formatting options available
- Compatible with popular meeting software export formats

## Troubleshooting

### Common Issues

**Speakers Not Detected:**
- Check that minimum speaking duration is met (>1 second)
- Verify audio quality and microphone levels
- Increase VAD sensitivity in settings
- Ensure speakers have distinct voices

**Too Many Speakers Detected:**
- Increase similarity threshold to merge similar voices
- Enable adaptive clustering for dynamic adjustment
- Review and merge incorrectly split speakers manually

**Poor Performance:**
- Reduce maximum speaker count in settings
- Close other applications to free memory
- Check available disk space for caching
- Consider using lower quality audio input if needed

For detailed troubleshooting steps, see [DIARIZATION-TROUBLESHOOTING.md](DIARIZATION-TROUBLESHOOTING.md).

## Support and Updates

Speaker diarization is actively developed with regular improvements to accuracy and performance. Check the [GitHub repository](https://github.com/example/kaginote) for updates and feature requests.

For technical support:
- Review the troubleshooting guide first
- Check debug logs with `RUST_LOG=debug`
- Report issues with sample audio (no personal content) if possible
- Include system specifications and configuration details