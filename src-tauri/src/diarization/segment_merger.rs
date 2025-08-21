//! Segment Merger
//! 
//! Merges speaker diarization segments with transcription output to create
//! final segments with both speaker identification and transcribed text.

use super::types::*;
use anyhow::Result;
use std::collections::HashMap;
use tracing;

/// Transcription segment for merging
#[derive(Debug, Clone)]
pub struct TranscriptionSegment {
    pub start_time: f32,
    pub end_time: f32,
    pub text: String,
    pub confidence: f32,
    pub speaker_id: String, // May be placeholder initially
}

/// Segment merger service
pub struct SegmentMerger {
    config: DiarizationConfig,
}

impl SegmentMerger {
    /// Create a new segment merger
    pub fn new(config: DiarizationConfig) -> Self {
        tracing::info!("Initializing SegmentMerger");
        
        Self {
            config,
        }
    }
    
    /// Merge speaker segments with transcription segments
    pub async fn merge_segments(
        &self,
        speaker_segments: &[SpeakerSegment],
        transcription_segments: &[TranscriptionSegment],
    ) -> Result<Vec<FinalSegment>> {
        if speaker_segments.is_empty() && transcription_segments.is_empty() {
            return Ok(vec![]);
        }
        
        tracing::info!("Merging {} speaker segments with {} transcription segments", 
                      speaker_segments.len(), transcription_segments.len());
        
        let mut final_segments = Vec::new();
        
        // If we only have transcription segments, use them as-is
        if speaker_segments.is_empty() {
            for trans_seg in transcription_segments {
                final_segments.push(FinalSegment {
                    start_time: trans_seg.start_time,
                    end_time: trans_seg.end_time,
                    speaker_id: trans_seg.speaker_id.clone(),
                    text: trans_seg.text.clone(),
                    transcription_confidence: trans_seg.confidence,
                    speaker_confidence: 0.0,
                    overall_confidence: trans_seg.confidence,
                    was_merged: false,
                });
            }
            return Ok(final_segments);
        }
        
        // If we only have speaker segments, create segments without text
        if transcription_segments.is_empty() {
            for speaker_seg in speaker_segments {
                final_segments.push(FinalSegment {
                    start_time: speaker_seg.start_time,
                    end_time: speaker_seg.end_time,
                    speaker_id: speaker_seg.speaker_id.clone(),
                    text: String::new(),
                    transcription_confidence: 0.0,
                    speaker_confidence: speaker_seg.confidence,
                    overall_confidence: speaker_seg.confidence,
                    was_merged: false,
                });
            }
            return Ok(final_segments);
        }
        
        // Perform temporal alignment between speaker and transcription segments
        final_segments = self.align_segments(speaker_segments, transcription_segments).await?;
        
        // Post-process to handle overlaps and gaps
        final_segments = self.post_process_segments(final_segments).await?;
        
        // Sort by start time
        final_segments.sort_by(|a, b| a.start_time.partial_cmp(&b.start_time).unwrap());
        
        tracing::info!("Generated {} final segments", final_segments.len());
        Ok(final_segments)
    }
    
    /// Align speaker segments with transcription segments
    async fn align_segments(
        &self,
        speaker_segments: &[SpeakerSegment],
        transcription_segments: &[TranscriptionSegment],
    ) -> Result<Vec<FinalSegment>> {
        let mut final_segments = Vec::new();
        
        // For each transcription segment, find the best matching speaker segment(s)
        for trans_seg in transcription_segments {
            let overlapping_speakers = self.find_overlapping_speakers(trans_seg, speaker_segments).await?;
            
            if overlapping_speakers.is_empty() {
                // No speaker segments overlap - use original speaker ID or assign default
                final_segments.push(FinalSegment {
                    start_time: trans_seg.start_time,
                    end_time: trans_seg.end_time,
                    speaker_id: if trans_seg.speaker_id == "speaker_1" || trans_seg.speaker_id.is_empty() {
                        "unknown_speaker".to_string()
                    } else {
                        trans_seg.speaker_id.clone()
                    },
                    text: trans_seg.text.clone(),
                    transcription_confidence: trans_seg.confidence,
                    speaker_confidence: 0.0,
                    overall_confidence: trans_seg.confidence * 0.5, // Reduced due to no speaker match
                    was_merged: false,
                });
            } else if overlapping_speakers.len() == 1 {
                // Single speaker - straightforward alignment
                let speaker = &overlapping_speakers[0];
                final_segments.push(FinalSegment {
                    start_time: trans_seg.start_time,
                    end_time: trans_seg.end_time,
                    speaker_id: speaker.speaker_id.clone(),
                    text: trans_seg.text.clone(),
                    transcription_confidence: trans_seg.confidence,
                    speaker_confidence: speaker.confidence,
                    overall_confidence: (trans_seg.confidence + speaker.confidence) / 2.0,
                    was_merged: true,
                });
            } else {
                // Multiple speakers - need to split the transcription
                let splits = self.split_transcription_by_speakers(trans_seg, &overlapping_speakers).await?;
                final_segments.extend(splits);
            }
        }
        
        Ok(final_segments)
    }
    
    /// Find speaker segments that overlap with a transcription segment
    async fn find_overlapping_speakers(
        &self,
        trans_seg: &TranscriptionSegment,
        speaker_segments: &[SpeakerSegment],
    ) -> Result<Vec<SpeakerSegment>> {
        let mut overlapping = Vec::new();
        
        for speaker_seg in speaker_segments {
            let overlap = self.calculate_temporal_overlap(
                trans_seg.start_time,
                trans_seg.end_time,
                speaker_seg.start_time,
                speaker_seg.end_time,
            );
            
            // Require significant overlap (>50% of transcription segment)
            let trans_duration = trans_seg.end_time - trans_seg.start_time;
            let overlap_ratio = if trans_duration > 0.0 { overlap / trans_duration } else { 0.0 };
            
            if overlap_ratio > 0.5 {
                overlapping.push(speaker_seg.clone());
            }
        }
        
        // Sort by overlap amount (descending)
        overlapping.sort_by(|a, b| {
            let overlap_a = self.calculate_temporal_overlap(
                trans_seg.start_time, trans_seg.end_time,
                a.start_time, a.end_time,
            );
            let overlap_b = self.calculate_temporal_overlap(
                trans_seg.start_time, trans_seg.end_time,
                b.start_time, b.end_time,
            );
            overlap_b.partial_cmp(&overlap_a).unwrap()
        });
        
        Ok(overlapping)
    }
    
    /// Calculate temporal overlap between two segments
    fn calculate_temporal_overlap(&self, start1: f32, end1: f32, start2: f32, end2: f32) -> f32 {
        let overlap_start = start1.max(start2);
        let overlap_end = end1.min(end2);
        (overlap_end - overlap_start).max(0.0)
    }
    
    /// Split a transcription segment among multiple speakers
    async fn split_transcription_by_speakers(
        &self,
        trans_seg: &TranscriptionSegment,
        speakers: &[SpeakerSegment],
    ) -> Result<Vec<FinalSegment>> {
        if speakers.is_empty() {
            return Ok(vec![]);
        }
        
        if speakers.len() == 1 {
            // Single speaker case
            let speaker = &speakers[0];
            return Ok(vec![FinalSegment {
                start_time: trans_seg.start_time,
                end_time: trans_seg.end_time,
                speaker_id: speaker.speaker_id.clone(),
                text: trans_seg.text.clone(),
                transcription_confidence: trans_seg.confidence,
                speaker_confidence: speaker.confidence,
                overall_confidence: (trans_seg.confidence + speaker.confidence) / 2.0,
                was_merged: true,
            }]);
        }
        
        // Multiple speakers - split text proportionally by overlap
        let mut segments = Vec::new();
        let total_duration = trans_seg.end_time - trans_seg.start_time;
        
        if total_duration <= 0.0 {
            return Ok(segments);
        }
        
        let mut accumulated_time = trans_seg.start_time;
        
        for (i, speaker) in speakers.iter().enumerate() {
            let overlap = self.calculate_temporal_overlap(
                trans_seg.start_time, trans_seg.end_time,
                speaker.start_time, speaker.end_time,
            );
            
            let proportion = overlap / total_duration;
            let segment_duration = total_duration * proportion;
            let segment_end = accumulated_time + segment_duration;
            
            // Split the text proportionally (rough approximation)
            let text_portion = if i == speakers.len() - 1 {
                // Last segment gets remaining text
                trans_seg.text.clone()
            } else {
                let words: Vec<&str> = trans_seg.text.split_whitespace().collect();
                let word_count = (words.len() as f32 * proportion).round() as usize;
                words.into_iter().take(word_count).collect::<Vec<_>>().join(" ")
            };
            
            segments.push(FinalSegment {
                start_time: accumulated_time,
                end_time: segment_end.min(trans_seg.end_time),
                speaker_id: speaker.speaker_id.clone(),
                text: text_portion,
                transcription_confidence: trans_seg.confidence,
                speaker_confidence: speaker.confidence,
                overall_confidence: (trans_seg.confidence + speaker.confidence) / 2.0,
                was_merged: true,
            });
            
            accumulated_time = segment_end;
            
            if accumulated_time >= trans_seg.end_time {
                break;
            }
        }
        
        Ok(segments)
    }
    
    /// Post-process segments to handle overlaps and gaps
    async fn post_process_segments(&self, mut segments: Vec<FinalSegment>) -> Result<Vec<FinalSegment>> {
        if segments.len() <= 1 {
            return Ok(segments);
        }
        
        // Sort by start time
        segments.sort_by(|a, b| a.start_time.partial_cmp(&b.start_time).unwrap());
        
        let mut processed = Vec::new();
        let mut current = segments[0].clone();
        
        for next in segments.into_iter().skip(1) {
            // Check for overlap
            if current.end_time > next.start_time {
                // Handle overlap
                if current.speaker_id == next.speaker_id {
                    // Same speaker - merge segments
                    current = self.merge_same_speaker_segments(&current, &next);
                } else {
                    // Different speakers - adjust boundaries
                    let (adjusted_current, adjusted_next) = self.adjust_segment_boundaries(&current, &next);
                    processed.push(adjusted_current);
                    current = adjusted_next;
                }
            } else {
                // No overlap - check for gap
                let gap = next.start_time - current.end_time;
                if gap > 0.1 && gap < self.config.min_segment_duration {
                    // Small gap - extend current segment to fill it
                    current.end_time = next.start_time;
                }
                
                processed.push(current);
                current = next;
            }
        }
        
        processed.push(current);
        Ok(processed)
    }
    
    /// Merge two segments from the same speaker
    fn merge_same_speaker_segments(&self, seg1: &FinalSegment, seg2: &FinalSegment) -> FinalSegment {
        FinalSegment {
            start_time: seg1.start_time,
            end_time: seg2.end_time,
            speaker_id: seg1.speaker_id.clone(),
            text: if seg1.text.is_empty() {
                seg2.text.clone()
            } else if seg2.text.is_empty() {
                seg1.text.clone()
            } else {
                format!("{} {}", seg1.text, seg2.text)
            },
            transcription_confidence: (seg1.transcription_confidence + seg2.transcription_confidence) / 2.0,
            speaker_confidence: (seg1.speaker_confidence + seg2.speaker_confidence) / 2.0,
            overall_confidence: (seg1.overall_confidence + seg2.overall_confidence) / 2.0,
            was_merged: true,
        }
    }
    
    /// Adjust boundaries of overlapping segments
    fn adjust_segment_boundaries(&self, seg1: &FinalSegment, seg2: &FinalSegment) -> (FinalSegment, FinalSegment) {
        let overlap_midpoint = (seg1.end_time + seg2.start_time) / 2.0;
        
        let mut adjusted_seg1 = seg1.clone();
        let mut adjusted_seg2 = seg2.clone();
        
        adjusted_seg1.end_time = overlap_midpoint;
        adjusted_seg2.start_time = overlap_midpoint;
        
        (adjusted_seg1, adjusted_seg2)
    }
    
    /// Get merger statistics
    pub fn get_stats(&self) -> HashMap<String, f32> {
        let mut stats = HashMap::new();
        stats.insert("min_segment_duration".to_string(), self.config.min_segment_duration);
        stats.insert("overlap_threshold".to_string(), 0.5); // 50% overlap threshold
        stats
    }
}