/// Pattern library with predefined drum patterns
///
/// This module contains predefined drum patterns for various genres and styles.
/// It provides the core patterns that were originally defined in guitar_buddy.rs
/// but now in a more structured and extensible format.

use super::types::{DrumPattern, DrumPatternBeat, PatternGenre};
use crate::timing::{TimeSignature, ClickType};
use std::collections::HashMap;

/// Pattern library for managing collections of drum patterns
pub struct PatternLibrary {
    /// All patterns stored by name
    patterns: HashMap<String, DrumPattern>,

    /// Genre index for fast filtering
    genre_index: HashMap<PatternGenre, Vec<String>>,

    /// Difficulty index
    difficulty_index: HashMap<u8, Vec<String>>,
}

impl PatternLibrary {
    /// Create a new empty pattern library
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            genre_index: HashMap::new(),
            difficulty_index: HashMap::new(),
        }
    }

    /// Create a pattern library with default patterns
    pub fn with_defaults() -> Self {
        let mut library = Self::new();
        library.add_default_patterns();
        library
    }

    /// Add a pattern to the library
    pub fn add_pattern(&mut self, pattern: DrumPattern) {
        let name = pattern.name.clone();

        // Update genre index
        self.genre_index
            .entry(pattern.metadata.genre.clone())
            .or_insert_with(Vec::new)
            .push(name.clone());

        // Update difficulty index
        self.difficulty_index
            .entry(pattern.metadata.difficulty)
            .or_insert_with(Vec::new)
            .push(name.clone());

        // Store pattern
        self.patterns.insert(name, pattern);
    }

    /// Get a pattern by name
    pub fn get_pattern(&self, name: &str) -> Option<&DrumPattern> {
        self.patterns.get(name)
    }

    /// Get all patterns
    pub fn all_patterns(&self) -> Vec<&DrumPattern> {
        self.patterns.values().collect()
    }

    /// Get patterns by genre
    pub fn patterns_by_genre(&self, genre: &PatternGenre) -> Vec<&DrumPattern> {
        self.genre_index
            .get(genre)
            .map(|names| {
                names.iter()
                    .filter_map(|name| self.patterns.get(name))
                    .collect()
            })
            .unwrap_or_else(Vec::new)
    }

    /// Get patterns by difficulty
    pub fn patterns_by_difficulty(&self, difficulty: u8) -> Vec<&DrumPattern> {
        self.difficulty_index
            .get(&difficulty)
            .map(|names| {
                names.iter()
                    .filter_map(|name| self.patterns.get(name))
                    .collect()
            })
            .unwrap_or_else(Vec::new)
    }

    /// Search patterns by name or tag
    pub fn search_patterns(&self, query: &str) -> Vec<&DrumPattern> {
        let query_lower = query.to_lowercase();
        self.patterns.values()
            .filter(|pattern| {
                pattern.name.to_lowercase().contains(&query_lower)
                    || pattern.display_name.to_lowercase().contains(&query_lower)
                    || pattern.metadata.description.to_lowercase().contains(&query_lower)
                    || pattern.metadata.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    /// Get patterns suitable for a given tempo
    pub fn patterns_for_tempo(&self, tempo_bpm: u32) -> Vec<&DrumPattern> {
        self.patterns.values()
            .filter(|pattern| pattern.is_tempo_suitable(tempo_bpm))
            .collect()
    }

    /// Get pattern count
    pub fn count(&self) -> usize {
        self.patterns.len()
    }

    /// Clear all patterns
    pub fn clear(&mut self) {
        self.patterns.clear();
        self.genre_index.clear();
        self.difficulty_index.clear();
    }

    /// Add all default patterns to the library
    fn add_default_patterns(&mut self) {
        // Basic Rock Pattern
        let basic_rock = DrumPattern::new("basic_rock", TimeSignature::new(4, 4))
            .with_display_name("Basic Rock Beat")
            .with_tempo_range(80, 140)
            .with_genre(PatternGenre::Rock)
            .with_difficulty(2)
            .with_description("Classic rock beat with kick on 1 and 3, snare on 2 and 4")
            .with_tag("rock")
            .with_tag("basic")
            .with_tag("4/4")
            .with_beat(DrumPatternBeat::new(1.0)
                .with_samples(vec![ClickType::AcousticKick, ClickType::HiHatClosed])
                .with_accent(true))
            .with_beat(DrumPatternBeat::new(1.5)
                .with_sample(ClickType::HiHatClosed))
            .with_beat(DrumPatternBeat::new(2.0)
                .with_samples(vec![ClickType::AcousticSnare, ClickType::HiHatClosed]))
            .with_beat(DrumPatternBeat::new(2.5)
                .with_sample(ClickType::HiHatClosed))
            .with_beat(DrumPatternBeat::new(3.0)
                .with_samples(vec![ClickType::AcousticKick, ClickType::HiHatClosed]))
            .with_beat(DrumPatternBeat::new(3.5)
                .with_sample(ClickType::HiHatClosed))
            .with_beat(DrumPatternBeat::new(4.0)
                .with_samples(vec![ClickType::AcousticSnare, ClickType::HiHatClosed]))
            .with_beat(DrumPatternBeat::new(4.5)
                .with_sample(ClickType::HiHatClosed));

        self.add_pattern(basic_rock);

        // Shuffle Pattern
        let shuffle = DrumPattern::new("shuffle", TimeSignature::new(4, 4))
            .with_display_name("Shuffle Beat")
            .with_tempo_range(60, 120)
            .with_genre(PatternGenre::Blues)
            .with_difficulty(3)
            .with_description("Swung shuffle rhythm with triplet feel")
            .with_tag("shuffle")
            .with_tag("blues")
            .with_tag("swing")
            .with_beat(DrumPatternBeat::new(1.0)
                .with_samples(vec![ClickType::AcousticKick, ClickType::HiHatClosed])
                .with_accent(true))
            .with_beat(DrumPatternBeat::new(1.67)
                .with_sample(ClickType::HiHatClosed))
            .with_beat(DrumPatternBeat::new(2.0)
                .with_samples(vec![ClickType::AcousticSnare, ClickType::HiHatClosed]))
            .with_beat(DrumPatternBeat::new(2.67)
                .with_sample(ClickType::HiHatClosed))
            .with_beat(DrumPatternBeat::new(3.0)
                .with_samples(vec![ClickType::AcousticKick, ClickType::HiHatClosed]))
            .with_beat(DrumPatternBeat::new(3.67)
                .with_sample(ClickType::HiHatClosed))
            .with_beat(DrumPatternBeat::new(4.0)
                .with_samples(vec![ClickType::AcousticSnare, ClickType::HiHatClosed]))
            .with_beat(DrumPatternBeat::new(4.67)
                .with_sample(ClickType::HiHatClosed));

        self.add_pattern(shuffle);

        // Ballad Pattern
        let ballad = DrumPattern::new("ballad", TimeSignature::new(4, 4))
            .with_display_name("Ballad Beat")
            .with_tempo_range(60, 90)
            .with_genre(PatternGenre::Pop)
            .with_difficulty(1)
            .with_description("Simple ballad rhythm with emphasis on backbeats")
            .with_tag("ballad")
            .with_tag("pop")
            .with_tag("simple")
            .with_beat(DrumPatternBeat::new(1.0)
                .with_samples(vec![ClickType::AcousticKick, ClickType::HiHatClosed])
                .with_accent(true))
            .with_beat(DrumPatternBeat::new(2.0)
                .with_samples(vec![ClickType::AcousticSnare, ClickType::HiHatClosed]))
            .with_beat(DrumPatternBeat::new(3.0)
                .with_samples(vec![ClickType::AcousticKick, ClickType::HiHatClosed]))
            .with_beat(DrumPatternBeat::new(4.0)
                .with_samples(vec![ClickType::AcousticSnare, ClickType::HiHatClosed]));

        self.add_pattern(ballad);

        // Waltz Pattern
        let waltz = DrumPattern::new("waltz", TimeSignature::new(3, 4))
            .with_display_name("Waltz Beat")
            .with_tempo_range(90, 180)
            .with_genre(PatternGenre::Classical)
            .with_difficulty(2)
            .with_description("Traditional 3/4 waltz with strong downbeat")
            .with_tag("waltz")
            .with_tag("3/4")
            .with_tag("classical")
            .with_beat(DrumPatternBeat::new(1.0)
                .with_sample(ClickType::AcousticKick)
                .with_accent(true))
            .with_beat(DrumPatternBeat::new(2.0)
                .with_sample(ClickType::AcousticSnare))
            .with_beat(DrumPatternBeat::new(3.0)
                .with_sample(ClickType::AcousticSnare));

        self.add_pattern(waltz);
    }
}

impl Default for PatternLibrary {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Pattern factory for creating common patterns
pub struct PatternFactory;

impl PatternFactory {
    /// Create a basic rock pattern
    pub fn basic_rock() -> DrumPattern {
        DrumPattern::new("basic_rock", TimeSignature::new(4, 4))
            .with_display_name("Basic Rock Beat")
            .with_tempo_range(80, 140)
            .with_genre(PatternGenre::Rock)
            .with_difficulty(2)
            .with_beat(DrumPatternBeat::new(1.0)
                .with_samples(vec![ClickType::AcousticKick, ClickType::HiHatClosed])
                .with_accent(true))
            .with_beat(DrumPatternBeat::new(1.5)
                .with_sample(ClickType::HiHatClosed))
            .with_beat(DrumPatternBeat::new(2.0)
                .with_samples(vec![ClickType::AcousticSnare, ClickType::HiHatClosed]))
            .with_beat(DrumPatternBeat::new(2.5)
                .with_sample(ClickType::HiHatClosed))
            .with_beat(DrumPatternBeat::new(3.0)
                .with_samples(vec![ClickType::AcousticKick, ClickType::HiHatClosed]))
            .with_beat(DrumPatternBeat::new(3.5)
                .with_sample(ClickType::HiHatClosed))
            .with_beat(DrumPatternBeat::new(4.0)
                .with_samples(vec![ClickType::AcousticSnare, ClickType::HiHatClosed]))
            .with_beat(DrumPatternBeat::new(4.5)
                .with_sample(ClickType::HiHatClosed))
    }

    /// Create a shuffle pattern
    pub fn shuffle() -> DrumPattern {
        DrumPattern::new("shuffle", TimeSignature::new(4, 4))
            .with_display_name("Shuffle Beat")
            .with_tempo_range(60, 120)
            .with_genre(PatternGenre::Blues)
            .with_difficulty(3)
            .with_beat(DrumPatternBeat::new(1.0)
                .with_samples(vec![ClickType::AcousticKick, ClickType::HiHatClosed])
                .with_accent(true))
            .with_beat(DrumPatternBeat::new(1.67)
                .with_sample(ClickType::HiHatClosed))
            .with_beat(DrumPatternBeat::new(2.0)
                .with_samples(vec![ClickType::AcousticSnare, ClickType::HiHatClosed]))
            .with_beat(DrumPatternBeat::new(2.67)
                .with_sample(ClickType::HiHatClosed))
            .with_beat(DrumPatternBeat::new(3.0)
                .with_samples(vec![ClickType::AcousticKick, ClickType::HiHatClosed]))
            .with_beat(DrumPatternBeat::new(3.67)
                .with_sample(ClickType::HiHatClosed))
            .with_beat(DrumPatternBeat::new(4.0)
                .with_samples(vec![ClickType::AcousticSnare, ClickType::HiHatClosed]))
            .with_beat(DrumPatternBeat::new(4.67)
                .with_sample(ClickType::HiHatClosed))
    }

    /// Create a ballad pattern
    pub fn ballad() -> DrumPattern {
        DrumPattern::new("ballad", TimeSignature::new(4, 4))
            .with_display_name("Ballad Beat")
            .with_tempo_range(60, 90)
            .with_genre(PatternGenre::Pop)
            .with_difficulty(1)
            .with_beat(DrumPatternBeat::new(1.0)
                .with_samples(vec![ClickType::AcousticKick, ClickType::HiHatClosed])
                .with_accent(true))
            .with_beat(DrumPatternBeat::new(2.0)
                .with_samples(vec![ClickType::AcousticSnare, ClickType::HiHatClosed]))
            .with_beat(DrumPatternBeat::new(3.0)
                .with_samples(vec![ClickType::AcousticKick, ClickType::HiHatClosed]))
            .with_beat(DrumPatternBeat::new(4.0)
                .with_samples(vec![ClickType::AcousticSnare, ClickType::HiHatClosed]))
    }

    /// Create a waltz pattern
    pub fn waltz() -> DrumPattern {
        DrumPattern::new("waltz", TimeSignature::new(3, 4))
            .with_display_name("Waltz Beat")
            .with_tempo_range(90, 180)
            .with_genre(PatternGenre::Classical)
            .with_difficulty(2)
            .with_beat(DrumPatternBeat::new(1.0)
                .with_sample(ClickType::AcousticKick)
                .with_accent(true))
            .with_beat(DrumPatternBeat::new(2.0)
                .with_sample(ClickType::AcousticSnare))
            .with_beat(DrumPatternBeat::new(3.0)
                .with_sample(ClickType::AcousticSnare))
    }

    /// Get all factory patterns
    pub fn all_patterns() -> Vec<DrumPattern> {
        vec![
            Self::basic_rock(),
            Self::shuffle(),
            Self::ballad(),
            Self::waltz(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_library_creation() {
        let library = PatternLibrary::new();
        assert_eq!(library.count(), 0);

        let library_with_defaults = PatternLibrary::with_defaults();
        assert!(library_with_defaults.count() > 0);
    }

    #[test]
    fn test_pattern_library_operations() {
        let mut library = PatternLibrary::new();
        let pattern = PatternFactory::basic_rock();
        library.add_pattern(pattern.clone());

        assert_eq!(library.count(), 1);
        assert!(library.get_pattern("basic_rock").is_some());
    }

    #[test]
    fn test_genre_filtering() {
        let library = PatternLibrary::with_defaults();
        let rock_patterns = library.patterns_by_genre(&PatternGenre::Rock);
        assert!(!rock_patterns.is_empty());
    }

    #[test]
    fn test_tempo_filtering() {
        let library = PatternLibrary::with_defaults();
        let patterns_100bpm = library.patterns_for_tempo(100);
        assert!(!patterns_100bpm.is_empty());

        // All returned patterns should be suitable for 100 BPM
        for pattern in patterns_100bpm {
            assert!(pattern.is_tempo_suitable(100));
        }
    }

    #[test]
    fn test_pattern_search() {
        let library = PatternLibrary::with_defaults();
        let rock_results = library.search_patterns("rock");
        assert!(!rock_results.is_empty());

        let shuffle_results = library.search_patterns("shuffle");
        assert!(!shuffle_results.is_empty());
    }

    #[test]
    fn test_difficulty_filtering() {
        let library = PatternLibrary::with_defaults();
        let beginner_patterns = library.patterns_by_difficulty(1);
        assert!(!beginner_patterns.is_empty());
    }

    #[test]
    fn test_pattern_factory() {
        let patterns = PatternFactory::all_patterns();
        assert_eq!(patterns.len(), 4);

        let rock = PatternFactory::basic_rock();
        assert_eq!(rock.name, "basic_rock");
        assert_eq!(rock.metadata.genre, PatternGenre::Rock);
    }
}