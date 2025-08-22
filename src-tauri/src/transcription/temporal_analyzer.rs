use std::collections::VecDeque;

/// Temporal segment for tracking timeline overlaps and sequencing
#[derive(Debug, Clone)]
pub struct TemporalSegment {
    pub text: String,
    pub start_time: f32,
    pub end_time: f32,
    pub confidence: f32,
    pub speaker_id: String,
}

/// Temporal analyzer for detecting timeline overlaps and ensuring proper sequencing
pub struct TemporalAnalyzer {
    /// Recent segments for temporal analysis
    recent_segments: VecDeque<TemporalSegment>,
    /// Maximum segments to analyze
    max_segments: usize,
    /// Overlap threshold (seconds)
    overlap_threshold: f32,
    /// Minimum gap between segments (seconds)
    min_gap_threshold: f32,
}

impl TemporalAnalyzer {
    pub fn new(max_segments: usize, overlap_threshold: f32, min_gap_threshold: f32) -> Self {
        Self {
            recent_segments: VecDeque::with_capacity(max_segments),
            max_segments,
            overlap_threshold,
            min_gap_threshold,
        }
    }

    /// Check if a new segment has temporal conflicts with recent segments
    pub fn has_temporal_conflict(&self, segment: &TemporalSegment) -> bool {
        for existing in &self.recent_segments {
            if self.segments_overlap(existing, segment) {
                return true;
            }
        }
        false
    }

    /// Add a new segment to temporal tracking
    pub fn add_segment(&mut self, segment: TemporalSegment) {
        // Insert segment in chronological order
        let insert_pos = self.recent_segments
            .iter()
            .position(|s| s.start_time > segment.start_time)
            .unwrap_or(self.recent_segments.len());
            
        self.recent_segments.insert(insert_pos, segment);

        // Maintain size limit
        if self.recent_segments.len() > self.max_segments {
            self.recent_segments.pop_front();
        }
    }

    /// Detect overlapping segments in timeline
    fn segments_overlap(&self, seg1: &TemporalSegment, seg2: &TemporalSegment) -> bool {
        // Check for significant temporal overlap
        let overlap_start = seg1.start_time.max(seg2.start_time);
        let overlap_end = seg1.end_time.min(seg2.end_time);
        let overlap_duration = (overlap_end - overlap_start).max(0.0);
        
        overlap_duration > self.overlap_threshold
    }

    /// Get overlapping segments for a given time range
    pub fn find_overlapping_segments(&self, start_time: f32, end_time: f32) -> Vec<&TemporalSegment> {
        self.recent_segments
            .iter()
            .filter(|seg| {
                let overlap_start = seg.start_time.max(start_time);
                let overlap_end = seg.end_time.min(end_time);
                (overlap_end - overlap_start) > self.overlap_threshold
            })
            .collect()
    }

    /// Validate segment timing makes sense
    pub fn is_valid_timing(&self, segment: &TemporalSegment) -> bool {
        // Basic validity checks
        if segment.start_time < 0.0 || segment.end_time <= segment.start_time {
            return false;
        }

        // Duration should be reasonable (0.1s to 60s)
        let duration = segment.end_time - segment.start_time;
        if duration < 0.1 || duration > 60.0 {
            return false;
        }

        // Check if timing makes sense relative to recent segments
        if let Some(last_segment) = self.recent_segments.back() {
            // New segment shouldn't start too far in the past
            if segment.start_time < last_segment.start_time - 5.0 {
                return false;
            }
        }

        true
    }

    /// Suggest timing corrections for segments with conflicts
    pub fn suggest_timing_correction(&self, segment: &TemporalSegment) -> Option<(f32, f32)> {
        if !self.has_temporal_conflict(segment) {
            return None;
        }

        // Find the best gap to place this segment
        let mut best_start = segment.start_time;
        
        // Try to place after the last non-overlapping segment
        for existing in self.recent_segments.iter().rev() {
            if segment.start_time >= existing.end_time - self.overlap_threshold {
                break;
            }
            best_start = existing.end_time + self.min_gap_threshold;
        }

        let duration = segment.end_time - segment.start_time;
        Some((best_start, best_start + duration))
    }

    /// Merge segments with temporal overlaps
    pub fn merge_overlapping_segments(&mut self, new_segment: TemporalSegment) -> Vec<TemporalSegment> {
        let mut merged_segments = Vec::new();
        let mut current_segment = new_segment;
        
        // Find all segments that overlap with the new one
        let mut to_remove = Vec::new();
        
        for (idx, existing) in self.recent_segments.iter().enumerate() {
            if self.segments_overlap(existing, &current_segment) {
                // Merge the segments
                current_segment = self.merge_two_segments(&current_segment, existing);
                to_remove.push(idx);
            }
        }

        // Remove merged segments (in reverse order to maintain indices)
        for &idx in to_remove.iter().rev() {
            self.recent_segments.remove(idx);
        }

        merged_segments.push(current_segment);
        merged_segments
    }

    /// Merge two temporal segments
    fn merge_two_segments(&self, seg1: &TemporalSegment, seg2: &TemporalSegment) -> TemporalSegment {
        // Determine which segment should contribute primary text
        let primary_segment = if seg1.confidence > seg2.confidence {
            seg1
        } else {
            seg2
        };

        // Merge text intelligently
        let merged_text = if seg1.text.contains(&seg2.text) {
            seg1.text.clone()
        } else if seg2.text.contains(&seg1.text) {
            seg2.text.clone()
        } else {
            // Concatenate with proper spacing
            format!("{} {}", seg1.text, seg2.text)
        };

        TemporalSegment {
            text: merged_text,
            start_time: seg1.start_time.min(seg2.start_time),
            end_time: seg1.end_time.max(seg2.end_time),
            confidence: (seg1.confidence + seg2.confidence) / 2.0,
            speaker_id: primary_segment.speaker_id.clone(),
        }
    }

    /// Get segments within a specific time window
    pub fn get_segments_in_window(&self, start: f32, end: f32) -> Vec<&TemporalSegment> {
        self.recent_segments
            .iter()
            .filter(|seg| seg.start_time < end && seg.end_time > start)
            .collect()
    }

    /// Calculate total speaking time for a speaker
    pub fn get_speaker_duration(&self, speaker_id: &str) -> f32 {
        self.recent_segments
            .iter()
            .filter(|seg| seg.speaker_id == speaker_id)
            .map(|seg| seg.end_time - seg.start_time)
            .sum()
    }

    /// Find gaps in the timeline where no one was speaking
    pub fn find_silence_gaps(&self, min_gap_duration: f32) -> Vec<(f32, f32)> {
        let mut gaps = Vec::new();
        
        let segments: Vec<&TemporalSegment> = self.recent_segments.iter().collect();
        for window in segments.windows(2) {
            let gap_start = window[0].end_time;
            let gap_end = window[1].start_time;
            let gap_duration = gap_end - gap_start;
            
            if gap_duration >= min_gap_duration {
                gaps.push((gap_start, gap_end));
            }
        }
        
        gaps
    }

    /// Clear all temporal data (useful for new sessions)
    pub fn clear(&mut self) {
        self.recent_segments.clear();
    }

    /// Get analysis statistics
    pub fn get_stats(&self) -> (usize, f32, f32) {
        if self.recent_segments.is_empty() {
            return (0, 0.0, 0.0);
        }

        let total_duration = self.recent_segments
            .iter()
            .map(|seg| seg.end_time - seg.start_time)
            .sum::<f32>();

        let avg_confidence = self.recent_segments
            .iter()
            .map(|seg| seg.confidence)
            .sum::<f32>() / self.recent_segments.len() as f32;

        (self.recent_segments.len(), total_duration, avg_confidence)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_segment(text: &str, start: f32, end: f32, confidence: f32, speaker: &str) -> TemporalSegment {
        TemporalSegment {
            text: text.to_string(),
            start_time: start,
            end_time: end,
            confidence,
            speaker_id: speaker.to_string(),
        }
    }

    #[test]
    fn test_no_overlap_detection() {
        let analyzer = TemporalAnalyzer::new(10, 0.5, 0.1);
        
        let seg1 = create_test_segment("Hello", 1.0, 2.0, 0.9, "speaker_1");
        let seg2 = create_test_segment("World", 3.0, 4.0, 0.9, "speaker_1");
        
        assert!(!analyzer.segments_overlap(&seg1, &seg2));
    }

    #[test]
    fn test_overlap_detection() {
        let analyzer = TemporalAnalyzer::new(10, 0.3, 0.1);
        
        let seg1 = create_test_segment("Hello", 1.0, 3.0, 0.9, "speaker_1");
        let seg2 = create_test_segment("World", 2.0, 4.0, 0.9, "speaker_2");
        
        assert!(analyzer.segments_overlap(&seg1, &seg2));
    }

    #[test]
    fn test_temporal_conflict_detection() {
        let mut analyzer = TemporalAnalyzer::new(10, 0.3, 0.1);
        
        let seg1 = create_test_segment("Hello", 1.0, 3.0, 0.9, "speaker_1");
        analyzer.add_segment(seg1);
        
        let seg2 = create_test_segment("World", 2.0, 4.0, 0.9, "speaker_2");
        assert!(analyzer.has_temporal_conflict(&seg2));
    }

    #[test]
    fn test_timing_validation() {
        let analyzer = TemporalAnalyzer::new(10, 0.3, 0.1);
        
        // Valid segment
        let valid_seg = create_test_segment("Hello", 1.0, 2.0, 0.9, "speaker_1");
        assert!(analyzer.is_valid_timing(&valid_seg));
        
        // Invalid segment (end before start)
        let invalid_seg = create_test_segment("Hello", 2.0, 1.0, 0.9, "speaker_1");
        assert!(!analyzer.is_valid_timing(&invalid_seg));
        
        // Invalid segment (too long)
        let too_long_seg = create_test_segment("Hello", 1.0, 70.0, 0.9, "speaker_1");
        assert!(!analyzer.is_valid_timing(&too_long_seg));
    }

    #[test]
    fn test_segment_merging() {
        let analyzer = TemporalAnalyzer::new(10, 0.3, 0.1);
        
        let seg1 = create_test_segment("Hello", 1.0, 3.0, 0.9, "speaker_1");
        let seg2 = create_test_segment("World", 2.0, 4.0, 0.8, "speaker_1");
        
        let merged = analyzer.merge_two_segments(&seg1, &seg2);
        
        assert_eq!(merged.start_time, 1.0);
        assert_eq!(merged.end_time, 4.0);
        assert!(merged.text.contains("Hello") && merged.text.contains("World"));
    }
}