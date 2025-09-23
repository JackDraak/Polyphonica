/// Timeline management for chord progressions synchronized to beat/metronome
///
/// This module provides the ChordTimeline for managing chord events over time,
/// and MovingTimeline for displaying current/next/following chord cues to users.

use super::types::*;
use crate::timing::{BeatEvent, TimeSignature};
use std::collections::VecDeque;

/// Timeline display data for UI components
#[derive(Debug, Clone)]
pub struct TimelineDisplayData {
    pub current_chord: Option<ChordEvent>,
    pub next_chord: Option<ChordEvent>,
    pub following_chord: Option<ChordEvent>,
    pub current_key_center: Note,
    pub next_key_center: Option<Note>,
    pub current_beat: u32,
    pub measures_ahead: u8,
}

impl TimelineDisplayData {
    /// Create empty timeline display
    pub fn empty() -> Self {
        Self {
            current_chord: None,
            next_chord: None,
            following_chord: None,
            current_key_center: Note::C,
            next_key_center: None,
            current_beat: 0,
            measures_ahead: 2,
        }
    }

    /// Check if timeline has active content
    pub fn has_content(&self) -> bool {
        self.current_chord.is_some() || self.next_chord.is_some()
    }

    /// Get all visible chord events
    pub fn visible_chords(&self) -> Vec<&ChordEvent> {
        let mut chords = Vec::new();
        if let Some(ref chord) = self.current_chord {
            chords.push(chord);
        }
        if let Some(ref chord) = self.next_chord {
            chords.push(chord);
        }
        if let Some(ref chord) = self.following_chord {
            chords.push(chord);
        }
        chords
    }
}

/// Manages chord events over time with beat synchronization
pub struct ChordTimeline {
    events: VecDeque<ChordEvent>,
    time_signature: TimeSignature,
    beats_per_chord: u32,
    auto_advance: bool,
}

impl ChordTimeline {
    /// Create new chord timeline
    pub fn new(time_signature: TimeSignature, config: &TimelineConfig) -> Self {
        Self {
            events: VecDeque::new(),
            time_signature,
            beats_per_chord: config.beats_per_chord as u32,
            auto_advance: config.auto_advance,
        }
    }

    /// Add chord event to timeline
    pub fn add_chord_event(&mut self, event: ChordEvent) {
        // Insert in correct position to maintain beat order
        let insert_pos = self.events
            .iter()
            .position(|e| e.beat_position > event.beat_position)
            .unwrap_or(self.events.len());

        self.events.insert(insert_pos, event);
    }

    /// Add chord at next available beat position
    pub fn add_chord_at_next_beat(&mut self, chord: Chord, key_center: Note) {
        let next_beat = self.get_next_available_beat();
        let event = ChordEvent::new(chord, next_beat, self.beats_per_chord, key_center);
        self.add_chord_event(event);
    }

    /// Get next available beat position
    fn get_next_available_beat(&self) -> u32 {
        self.events
            .back()
            .map(|e| e.end_beat())
            .unwrap_or(0)
    }

    /// Update timeline with current beat position
    pub fn update(&mut self, current_beat: u32) {
        if self.auto_advance {
            // Remove events that have finished playing
            while let Some(front) = self.events.front() {
                if front.end_beat() <= current_beat {
                    self.events.pop_front();
                } else {
                    break;
                }
            }
        }
    }

    /// Get chord active at specific beat
    pub fn chord_at_beat(&self, beat: u32) -> Option<&ChordEvent> {
        self.events
            .iter()
            .find(|event| event.is_active_at_beat(beat))
    }

    /// Get upcoming chord events within lookahead window
    pub fn upcoming_events(&self, from_beat: u32, measures_ahead: u8) -> Vec<&ChordEvent> {
        let beats_per_measure = self.time_signature.beats_per_measure as u32;
        let lookahead_beats = measures_ahead as u32 * beats_per_measure;
        let end_beat = from_beat + lookahead_beats;

        self.events
            .iter()
            .filter(|event| event.beat_position >= from_beat && event.beat_position < end_beat)
            .collect()
    }

    /// Clear all events from timeline
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Get total number of events
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Check if timeline is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Get all events (for debugging/inspection)
    pub fn all_events(&self) -> &VecDeque<ChordEvent> {
        &self.events
    }

    /// Update timeline configuration
    pub fn update_config(&mut self, config: &TimelineConfig) {
        self.beats_per_chord = config.beats_per_chord as u32;
        self.auto_advance = config.auto_advance;
    }

    /// Update time signature
    pub fn update_time_signature(&mut self, time_signature: TimeSignature) {
        self.time_signature = time_signature;
    }
}

/// Moving window timeline for displaying current/next/following chords
pub struct MovingTimeline {
    timeline: ChordTimeline,
    current_beat: u32,
    measures_ahead: u8,
}

impl MovingTimeline {
    /// Create new moving timeline
    pub fn new(time_signature: TimeSignature, config: &TimelineConfig) -> Self {
        Self {
            timeline: ChordTimeline::new(time_signature, config),
            current_beat: 0,
            measures_ahead: config.measures_ahead,
        }
    }

    /// Update with beat event from metronome
    pub fn update_with_beat(&mut self, beat_event: &BeatEvent) {
        self.current_beat = beat_event.beat_number as u32;
        self.timeline.update(self.current_beat);
    }

    /// Add chord progression to timeline
    pub fn add_chord_progression(&mut self, chords: &[Chord], key_centers: &[Note]) {
        for (chord, &key_center) in chords.iter().zip(key_centers.iter()) {
            self.timeline.add_chord_at_next_beat(chord.clone(), key_center);
        }
    }

    /// Get display data for UI
    pub fn get_display_data(&self) -> TimelineDisplayData {
        let current_chord = self.timeline.chord_at_beat(self.current_beat).cloned();

        // Find next and following chords
        let upcoming = self.timeline.upcoming_events(self.current_beat, self.measures_ahead);
        let mut future_chords = upcoming.iter()
            .filter(|event| event.beat_position > self.current_beat)
            .cloned()
            .collect::<Vec<_>>();
        future_chords.sort_by_key(|event| event.beat_position);

        let next_chord = future_chords.get(0).map(|&event| event.clone());
        let following_chord = future_chords.get(1).map(|&event| event.clone());

        // Determine key centers
        let current_key_center = current_chord
            .as_ref()
            .map(|e| e.key_center)
            .unwrap_or(Note::C);

        let next_key_center = next_chord
            .as_ref()
            .filter(|e| e.key_center != current_key_center)
            .map(|e| e.key_center);

        TimelineDisplayData {
            current_chord,
            next_chord,
            following_chord,
            current_key_center,
            next_key_center,
            current_beat: self.current_beat,
            measures_ahead: self.measures_ahead,
        }
    }

    /// Clear timeline and reset position
    pub fn clear(&mut self) {
        self.timeline.clear();
        self.current_beat = 0;
    }

    /// Jump to specific beat position
    pub fn jump_to_beat(&mut self, beat: u32) {
        self.current_beat = beat;
        self.timeline.update(beat);
    }

    /// Update configuration
    pub fn update_config(&mut self, config: &TimelineConfig) {
        self.measures_ahead = config.measures_ahead;
        self.timeline.update_config(config);
    }

    /// Update time signature
    pub fn update_time_signature(&mut self, time_signature: TimeSignature) {
        self.timeline.update_time_signature(time_signature);
    }

    /// Get underlying timeline for advanced operations
    pub fn timeline(&self) -> &ChordTimeline {
        &self.timeline
    }

    /// Get mutable access to timeline
    pub fn timeline_mut(&mut self) -> &mut ChordTimeline {
        &mut self.timeline
    }

    /// Get current beat position
    pub fn current_beat(&self) -> u32 {
        self.current_beat
    }

    /// Check if timeline has upcoming content
    pub fn has_upcoming_content(&self) -> bool {
        !self.timeline.upcoming_events(self.current_beat, self.measures_ahead).is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timing::TimeSignature;

    fn create_test_chord(root: Note, quality: ChordQuality) -> Chord {
        Chord::new(root, quality)
    }

    #[test]
    fn test_chord_timeline_creation() {
        let time_sig = TimeSignature::new(4, 4);
        let config = TimelineConfig::default();
        let timeline = ChordTimeline::new(time_sig, &config);

        assert!(timeline.is_empty());
        assert_eq!(timeline.event_count(), 0);
    }

    #[test]
    fn test_add_chord_events() {
        let time_sig = TimeSignature::new(4, 4);
        let config = TimelineConfig::default();
        let mut timeline = ChordTimeline::new(time_sig, &config);

        let chord = create_test_chord(Note::C, ChordQuality::Major);
        timeline.add_chord_at_next_beat(chord, Note::C);

        assert_eq!(timeline.event_count(), 1);
        assert!(!timeline.is_empty());
    }

    #[test]
    fn test_chord_beat_positioning() {
        let time_sig = TimeSignature::new(4, 4);
        let config = TimelineConfig::default();
        let mut timeline = ChordTimeline::new(time_sig, &config);

        let c_major = create_test_chord(Note::C, ChordQuality::Major);
        let f_major = create_test_chord(Note::F, ChordQuality::Major);

        timeline.add_chord_at_next_beat(c_major.clone(), Note::C);
        timeline.add_chord_at_next_beat(f_major.clone(), Note::C);

        // First chord should start at beat 0
        let first_event = timeline.chord_at_beat(0);
        assert!(first_event.is_some());
        assert_eq!(first_event.unwrap().chord.root, Note::C);

        // Second chord should start at beat 4 (after first chord duration)
        let second_event = timeline.chord_at_beat(4);
        assert!(second_event.is_some());
        assert_eq!(second_event.unwrap().chord.root, Note::F);
    }

    #[test]
    fn test_timeline_update_and_cleanup() {
        let time_sig = TimeSignature::new(4, 4);
        let mut config = TimelineConfig::default();
        config.auto_advance = true;
        let mut timeline = ChordTimeline::new(time_sig, &config);

        let chord = create_test_chord(Note::C, ChordQuality::Major);
        timeline.add_chord_at_next_beat(chord, Note::C);

        assert_eq!(timeline.event_count(), 1);

        // Update past the chord duration
        timeline.update(5);

        // Event should be removed due to auto_advance
        assert_eq!(timeline.event_count(), 0);
    }

    #[test]
    fn test_upcoming_events() {
        let time_sig = TimeSignature::new(4, 4);
        let config = TimelineConfig::default();
        let mut timeline = ChordTimeline::new(time_sig, &config);

        // Add several chords
        let chords = [
            (Note::C, ChordQuality::Major),
            (Note::F, ChordQuality::Major),
            (Note::G, ChordQuality::Major),
            (Note::C, ChordQuality::Major),
        ];

        for (root, quality) in &chords {
            let chord = create_test_chord(*root, *quality);
            timeline.add_chord_at_next_beat(chord, Note::C);
        }

        // Get upcoming events from beat 0, looking 2 measures ahead (8 beats)
        let upcoming = timeline.upcoming_events(0, 2);
        assert_eq!(upcoming.len(), 2); // Should see first 2 chords
    }

    #[test]
    fn test_moving_timeline() {
        let time_sig = TimeSignature::new(4, 4);
        let config = TimelineConfig::default();
        let mut moving_timeline = MovingTimeline::new(time_sig, &config);

        // Add chord progression
        let chords = vec![
            create_test_chord(Note::C, ChordQuality::Major),
            create_test_chord(Note::F, ChordQuality::Major),
            create_test_chord(Note::G, ChordQuality::Major),
        ];
        let key_centers = vec![Note::C, Note::C, Note::C];

        moving_timeline.add_chord_progression(&chords, &key_centers);

        // Test display data at beat 0
        let display = moving_timeline.get_display_data();
        assert!(display.current_chord.is_some());
        assert!(display.next_chord.is_some());
        assert_eq!(display.current_key_center, Note::C);
    }

    #[test]
    fn test_timeline_display_data() {
        let display = TimelineDisplayData::empty();
        assert!(!display.has_content());
        assert_eq!(display.current_key_center, Note::C);
        assert_eq!(display.visible_chords().len(), 0);
    }

    #[test]
    fn test_key_center_changes() {
        let time_sig = TimeSignature::new(4, 4);
        let config = TimelineConfig::default();
        let mut moving_timeline = MovingTimeline::new(time_sig, &config);

        // Add progression with key change
        let chords = vec![
            create_test_chord(Note::C, ChordQuality::Major),
            create_test_chord(Note::G, ChordQuality::Major), // Modulate to G
        ];
        let key_centers = vec![Note::C, Note::G];

        moving_timeline.add_chord_progression(&chords, &key_centers);

        let display = moving_timeline.get_display_data();
        assert_eq!(display.current_key_center, Note::C);
        assert_eq!(display.next_key_center, Some(Note::G));
    }
}