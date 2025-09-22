/// Core pattern types and data structures
///
/// This module defines the fundamental data structures used for representing
/// drum patterns, rhythms, and musical arrangements. These types are designed
/// to integrate seamlessly with the timing system while providing flexibility
/// for complex rhythmic patterns.

use crate::timing::{TimeSignature, ClickType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single beat within a drum pattern
///
/// Represents a specific point in time within a pattern where one or more
/// drum samples should be triggered. The beat_position uses fractional
/// values to allow precise timing (1.0 = first beat, 1.5 = halfway to
/// second beat, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrumPatternBeat {
    /// Position within the measure (1.0 = first beat, 1.5 = halfway to second beat, etc.)
    pub beat_position: f32,

    /// Drum samples to trigger at this position
    pub samples: Vec<ClickType>,

    /// Whether this beat should be accented (emphasized)
    pub accent: bool,
}

/// Complete drum pattern definition
///
/// A DrumPattern contains all the information needed to play a rhythmic
/// pattern, including the time signature, tempo range, and individual
/// beat definitions with their timing and samples.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrumPattern {
    /// Pattern name for identification
    pub name: String,

    /// Display name for user interfaces
    pub display_name: String,

    /// Time signature for this pattern
    pub time_signature: TimeSignature,

    /// Recommended tempo range (min_bpm, max_bpm)
    pub tempo_range: (u32, u32),

    /// Individual beats that make up the pattern
    pub beats: Vec<DrumPatternBeat>,

    /// Pattern metadata
    pub metadata: PatternMetadata,
}

/// Additional metadata for patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMetadata {
    /// Genre classification
    pub genre: PatternGenre,

    /// Difficulty level (1-5)
    pub difficulty: u8,

    /// Pattern description
    pub description: String,

    /// Tags for search and categorization
    pub tags: Vec<String>,

    /// Pattern author or source
    pub author: Option<String>,

    /// Additional custom metadata
    pub custom_fields: HashMap<String, String>,
}

/// Pattern genre classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PatternGenre {
    Rock,
    Jazz,
    Blues,
    Latin,
    Funk,
    Pop,
    Electronic,
    World,
    Classical,
    Experimental,
    Custom,
}

impl PatternGenre {
    /// Get all available genres
    pub fn all() -> Vec<PatternGenre> {
        vec![
            PatternGenre::Rock,
            PatternGenre::Jazz,
            PatternGenre::Blues,
            PatternGenre::Latin,
            PatternGenre::Funk,
            PatternGenre::Pop,
            PatternGenre::Electronic,
            PatternGenre::World,
            PatternGenre::Classical,
            PatternGenre::Experimental,
            PatternGenre::Custom,
        ]
    }

    /// Get display name for genre
    pub fn display_name(&self) -> &'static str {
        match self {
            PatternGenre::Rock => "Rock",
            PatternGenre::Jazz => "Jazz",
            PatternGenre::Blues => "Blues",
            PatternGenre::Latin => "Latin",
            PatternGenre::Funk => "Funk",
            PatternGenre::Pop => "Pop",
            PatternGenre::Electronic => "Electronic",
            PatternGenre::World => "World",
            PatternGenre::Classical => "Classical",
            PatternGenre::Experimental => "Experimental",
            PatternGenre::Custom => "Custom",
        }
    }
}

impl DrumPattern {
    /// Create a new drum pattern
    pub fn new(name: &str, time_signature: TimeSignature) -> Self {
        Self {
            name: name.to_string(),
            display_name: name.to_string(),
            time_signature,
            tempo_range: (60, 120),
            beats: Vec::new(),
            metadata: PatternMetadata::default(),
        }
    }

    /// Set display name
    pub fn with_display_name(mut self, display_name: &str) -> Self {
        self.display_name = display_name.to_string();
        self
    }

    /// Set tempo range
    pub fn with_tempo_range(mut self, min_bpm: u32, max_bpm: u32) -> Self {
        self.tempo_range = (min_bpm, max_bpm);
        self
    }

    /// Add a beat to the pattern
    pub fn with_beat(mut self, beat: DrumPatternBeat) -> Self {
        self.beats.push(beat);
        self
    }

    /// Set genre
    pub fn with_genre(mut self, genre: PatternGenre) -> Self {
        self.metadata.genre = genre;
        self
    }

    /// Set difficulty
    pub fn with_difficulty(mut self, difficulty: u8) -> Self {
        self.metadata.difficulty = difficulty.clamp(1, 5);
        self
    }

    /// Set description
    pub fn with_description(mut self, description: &str) -> Self {
        self.metadata.description = description.to_string();
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.metadata.tags.push(tag.to_string());
        self
    }

    /// Check if tempo is within recommended range
    pub fn is_tempo_suitable(&self, tempo_bpm: u32) -> bool {
        tempo_bpm >= self.tempo_range.0 && tempo_bpm <= self.tempo_range.1
    }

    /// Get pattern duration in beats
    pub fn duration_beats(&self) -> f32 {
        self.beats.iter()
            .map(|beat| beat.beat_position)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(self.time_signature.beats_per_measure as f32)
    }

    /// Get all beats at a specific position
    pub fn beats_at_position(&self, position: f32) -> Vec<&DrumPatternBeat> {
        self.beats.iter()
            .filter(|beat| (beat.beat_position - position).abs() < 0.01)
            .collect()
    }

    /// Get pattern complexity score (0-100)
    pub fn complexity_score(&self) -> u8 {
        let beat_count = self.beats.len();
        let sample_diversity = self.beats.iter()
            .flat_map(|beat| &beat.samples)
            .collect::<std::collections::HashSet<_>>()
            .len();
        let accent_count = self.beats.iter().filter(|beat| beat.accent).count();

        // Simple scoring algorithm
        let score = (beat_count * 2 + sample_diversity * 5 + accent_count).min(100);
        score as u8
    }
}

impl DrumPatternBeat {
    /// Create a new drum pattern beat
    pub fn new(position: f32) -> Self {
        Self {
            beat_position: position,
            samples: Vec::new(),
            accent: false,
        }
    }

    /// Add a sample to this beat
    pub fn with_sample(mut self, sample: ClickType) -> Self {
        self.samples.push(sample);
        self
    }

    /// Add multiple samples to this beat
    pub fn with_samples(mut self, samples: Vec<ClickType>) -> Self {
        self.samples.extend(samples);
        self
    }

    /// Set accent
    pub fn with_accent(mut self, accent: bool) -> Self {
        self.accent = accent;
        self
    }

    /// Check if this beat has any samples
    pub fn has_samples(&self) -> bool {
        !self.samples.is_empty()
    }
}

impl Default for PatternMetadata {
    fn default() -> Self {
        Self {
            genre: PatternGenre::Custom,
            difficulty: 1,
            description: String::new(),
            tags: Vec::new(),
            author: None,
            custom_fields: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drum_pattern_creation() {
        let pattern = DrumPattern::new("test", TimeSignature::new(4, 4))
            .with_display_name("Test Pattern")
            .with_tempo_range(80, 120)
            .with_genre(PatternGenre::Rock)
            .with_difficulty(2);

        assert_eq!(pattern.name, "test");
        assert_eq!(pattern.display_name, "Test Pattern");
        assert_eq!(pattern.tempo_range, (80, 120));
        assert_eq!(pattern.metadata.genre, PatternGenre::Rock);
        assert_eq!(pattern.metadata.difficulty, 2);
    }

    #[test]
    fn test_drum_pattern_beat_creation() {
        let beat = DrumPatternBeat::new(1.0)
            .with_sample(ClickType::AcousticKick)
            .with_sample(ClickType::HiHatClosed)
            .with_accent(true);

        assert_eq!(beat.beat_position, 1.0);
        assert_eq!(beat.samples.len(), 2);
        assert!(beat.accent);
        assert!(beat.has_samples());
    }

    #[test]
    fn test_tempo_suitability() {
        let pattern = DrumPattern::new("test", TimeSignature::new(4, 4))
            .with_tempo_range(80, 120);

        assert!(pattern.is_tempo_suitable(100));
        assert!(!pattern.is_tempo_suitable(60));
        assert!(!pattern.is_tempo_suitable(150));
    }

    #[test]
    fn test_pattern_complexity() {
        let mut pattern = DrumPattern::new("complex", TimeSignature::new(4, 4));

        // Add some beats with different samples
        pattern = pattern
            .with_beat(DrumPatternBeat::new(1.0).with_sample(ClickType::AcousticKick).with_accent(true))
            .with_beat(DrumPatternBeat::new(2.0).with_sample(ClickType::AcousticSnare))
            .with_beat(DrumPatternBeat::new(3.0).with_sample(ClickType::HiHatClosed));

        let score = pattern.complexity_score();
        assert!(score > 0);
    }

    #[test]
    fn test_beats_at_position() {
        let pattern = DrumPattern::new("test", TimeSignature::new(4, 4))
            .with_beat(DrumPatternBeat::new(1.0).with_sample(ClickType::AcousticKick))
            .with_beat(DrumPatternBeat::new(1.0).with_sample(ClickType::HiHatClosed))
            .with_beat(DrumPatternBeat::new(2.0).with_sample(ClickType::AcousticSnare));

        let beats_at_1 = pattern.beats_at_position(1.0);
        assert_eq!(beats_at_1.len(), 2);

        let beats_at_2 = pattern.beats_at_position(2.0);
        assert_eq!(beats_at_2.len(), 1);
    }

    #[test]
    fn test_pattern_genre_display() {
        assert_eq!(PatternGenre::Rock.display_name(), "Rock");
        assert_eq!(PatternGenre::Jazz.display_name(), "Jazz");
        assert_eq!(PatternGenre::Electronic.display_name(), "Electronic");
    }
}