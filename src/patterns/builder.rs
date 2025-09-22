/// Pattern builder for creating custom drum patterns
///
/// This module provides a fluent API for creating custom drum patterns with
/// validation, error handling, and intelligent pattern analysis. The builder
/// ensures patterns are valid and provides helpful feedback for pattern creation.
use super::types::{DrumPattern, DrumPatternBeat, PatternGenre};
use crate::timing::{ClickType, TimeSignature};

/// Fluent builder for creating drum patterns
pub struct PatternBuilder {
    pattern: DrumPattern,
    errors: Vec<String>,
}

/// Pattern validation error
#[derive(Debug, Clone)]
pub enum PatternValidationError {
    /// Pattern has no beats
    EmptyPattern,

    /// Beat position is invalid (< 1.0 or > time signature beats)
    InvalidBeatPosition(f32),

    /// Beat has no samples
    EmptyBeat(f32),

    /// Duplicate beat positions
    DuplicateBeatPosition(f32),

    /// Pattern name is empty or invalid
    InvalidName(String),

    /// Tempo range is invalid
    InvalidTempoRange(u32, u32),
}

impl std::fmt::Display for PatternValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatternValidationError::EmptyPattern => {
                write!(f, "Pattern must have at least one beat")
            }
            PatternValidationError::InvalidBeatPosition(pos) => {
                write!(f, "Invalid beat position: {}", pos)
            }
            PatternValidationError::EmptyBeat(pos) => {
                write!(f, "Beat at position {} has no samples", pos)
            }
            PatternValidationError::DuplicateBeatPosition(pos) => {
                write!(f, "Duplicate beat at position {}", pos)
            }
            PatternValidationError::InvalidName(name) => {
                write!(f, "Invalid pattern name: {}", name)
            }
            PatternValidationError::InvalidTempoRange(min, max) => {
                write!(f, "Invalid tempo range: {}-{} BPM", min, max)
            }
        }
    }
}

impl std::error::Error for PatternValidationError {}

impl PatternBuilder {
    /// Create a new pattern builder
    pub fn new(name: &str, time_signature: TimeSignature) -> Self {
        Self {
            pattern: DrumPattern::new(name, time_signature),
            errors: Vec::new(),
        }
    }

    /// Set display name
    pub fn display_name(mut self, name: &str) -> Self {
        self.pattern.display_name = name.to_string();
        self
    }

    /// Set tempo range with validation
    pub fn tempo_range(mut self, min_bpm: u32, max_bpm: u32) -> Self {
        if min_bpm >= max_bpm {
            self.errors
                .push(format!("Invalid tempo range: {} >= {}", min_bpm, max_bpm));
        } else if min_bpm == 0 || max_bpm > 300 {
            self.errors.push(format!(
                "Tempo range out of bounds: {}-{} (should be 1-300)",
                min_bpm, max_bpm
            ));
        } else {
            self.pattern.tempo_range = (min_bpm, max_bpm);
        }
        self
    }

    /// Set genre
    pub fn genre(mut self, genre: PatternGenre) -> Self {
        self.pattern.metadata.genre = genre;
        self
    }

    /// Set difficulty (1-5)
    pub fn difficulty(mut self, difficulty: u8) -> Self {
        self.pattern.metadata.difficulty = difficulty.clamp(1, 5);
        self
    }

    /// Set description
    pub fn description(mut self, description: &str) -> Self {
        self.pattern.metadata.description = description.to_string();
        self
    }

    /// Add a tag
    pub fn tag(mut self, tag: &str) -> Self {
        if !tag.is_empty() {
            self.pattern.metadata.tags.push(tag.to_string());
        }
        self
    }

    /// Set author
    pub fn author(mut self, author: &str) -> Self {
        self.pattern.metadata.author = Some(author.to_string());
        self
    }

    /// Add custom metadata
    pub fn custom_field(mut self, key: &str, value: &str) -> Self {
        self.pattern
            .metadata
            .custom_fields
            .insert(key.to_string(), value.to_string());
        self
    }

    /// Add a beat at the specified position
    pub fn beat(self, position: f32) -> BeatBuilder {
        BeatBuilder::new(self, position)
    }

    /// Add a kick drum hit
    pub fn kick(self, position: f32) -> Self {
        self.beat(position).kick().build()
    }

    /// Add a snare hit
    pub fn snare(self, position: f32) -> Self {
        self.beat(position).snare().build()
    }

    /// Add a hi-hat hit
    pub fn hihat(self, position: f32) -> Self {
        self.beat(position).hihat_closed().build()
    }

    /// Add an accented kick hit
    pub fn kick_accent(self, position: f32) -> Self {
        self.beat(position).kick().accent().build()
    }

    /// Add multiple beats from a simple notation
    /// Format: "K.S." where K=kick, S=snare, .=hihat, space=rest
    pub fn from_notation(mut self, notation: &str) -> Self {
        let mut position = 1.0;
        let step = 0.5; // Eighth note steps

        for ch in notation.chars() {
            match ch {
                'K' | 'k' => {
                    self = self.kick(position);
                }
                'S' | 's' => {
                    self = self.snare(position);
                }
                'H' | 'h' | '.' => {
                    self = self.hihat(position);
                }
                ' ' | '-' => {
                    // Rest - just advance position
                }
                _ => {
                    self.errors
                        .push(format!("Unknown notation character: '{}'", ch));
                }
            }
            position += step;
        }

        self
    }

    /// Validate the pattern and return any errors
    pub fn validate(&self) -> Result<(), Vec<PatternValidationError>> {
        let mut errors = Vec::new();

        // Check pattern name
        if self.pattern.name.is_empty() {
            errors.push(PatternValidationError::InvalidName(
                "Pattern name cannot be empty".to_string(),
            ));
        }

        // Check if pattern has beats
        if self.pattern.beats.is_empty() {
            errors.push(PatternValidationError::EmptyPattern);
            return Err(errors);
        }

        // Check tempo range
        let (min, max) = self.pattern.tempo_range;
        if min >= max || min == 0 || max > 300 {
            errors.push(PatternValidationError::InvalidTempoRange(min, max));
        }

        // Check beat positions and duplicates
        let max_position = self.pattern.time_signature.beats_per_measure as f32 + 0.999; // Beat N goes from N.0 to N.999
        let mut positions = std::collections::HashSet::new();

        for beat in &self.pattern.beats {
            // Check position validity
            if beat.beat_position < 1.0 || beat.beat_position > max_position {
                errors.push(PatternValidationError::InvalidBeatPosition(
                    beat.beat_position,
                ));
            }

            // Check for empty beats
            if beat.samples.is_empty() {
                errors.push(PatternValidationError::EmptyBeat(beat.beat_position));
            }

            // Check for duplicates (with small tolerance for floating point)
            let pos_key = (beat.beat_position * 100.0) as i32;
            if positions.contains(&pos_key) {
                errors.push(PatternValidationError::DuplicateBeatPosition(
                    beat.beat_position,
                ));
            } else {
                positions.insert(pos_key);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Build the pattern, returning an error if validation fails
    pub fn build(mut self) -> Result<DrumPattern, Vec<PatternValidationError>> {
        // Add any builder-level errors
        if !self.errors.is_empty() {
            return Err(self
                .errors
                .into_iter()
                .map(PatternValidationError::InvalidName)
                .collect());
        }

        // Validate the pattern
        self.validate()?;

        // Sort beats by position
        self.pattern
            .beats
            .sort_by(|a, b| a.beat_position.partial_cmp(&b.beat_position).unwrap());

        Ok(self.pattern)
    }

    /// Build the pattern, returning a default pattern if validation fails
    pub fn build_or_default(self) -> DrumPattern {
        match self.build() {
            Ok(pattern) => pattern,
            Err(_) => {
                // Return a simple default pattern
                DrumPattern::new("default", TimeSignature::new(4, 4))
                    .with_beat(
                        DrumPatternBeat::new(1.0)
                            .with_sample(ClickType::AcousticKick)
                            .with_accent(true),
                    )
                    .with_beat(DrumPatternBeat::new(2.0).with_sample(ClickType::AcousticSnare))
                    .with_beat(DrumPatternBeat::new(3.0).with_sample(ClickType::AcousticKick))
                    .with_beat(DrumPatternBeat::new(4.0).with_sample(ClickType::AcousticSnare))
            }
        }
    }
}

/// Builder for individual beats within a pattern
pub struct BeatBuilder {
    pattern_builder: PatternBuilder,
    beat: DrumPatternBeat,
}

impl BeatBuilder {
    fn new(pattern_builder: PatternBuilder, position: f32) -> Self {
        Self {
            pattern_builder,
            beat: DrumPatternBeat::new(position),
        }
    }

    /// Add kick drum sample
    pub fn kick(mut self) -> Self {
        self.beat.samples.push(ClickType::AcousticKick);
        self
    }

    /// Add snare drum sample
    pub fn snare(mut self) -> Self {
        self.beat.samples.push(ClickType::AcousticSnare);
        self
    }

    /// Add closed hi-hat sample
    pub fn hihat_closed(mut self) -> Self {
        self.beat.samples.push(ClickType::HiHatClosed);
        self
    }

    /// Add open hi-hat sample
    pub fn hihat_open(mut self) -> Self {
        self.beat.samples.push(ClickType::HiHatOpen);
        self
    }

    /// Add rim shot sample
    pub fn rimshot(mut self) -> Self {
        self.beat.samples.push(ClickType::RimShot);
        self
    }

    /// Add stick click sample
    pub fn stick(mut self) -> Self {
        self.beat.samples.push(ClickType::Stick);
        self
    }

    /// Add custom sample
    pub fn sample(mut self, sample: ClickType) -> Self {
        self.beat.samples.push(sample);
        self
    }

    /// Add multiple samples
    pub fn samples(mut self, samples: Vec<ClickType>) -> Self {
        self.beat.samples.extend(samples);
        self
    }

    /// Mark this beat as accented
    pub fn accent(mut self) -> Self {
        self.beat.accent = true;
        self
    }

    /// Build this beat and return to pattern builder
    pub fn build(mut self) -> PatternBuilder {
        self.pattern_builder.pattern.beats.push(self.beat);
        self.pattern_builder
    }
}

/// Pattern template for quick pattern creation
pub struct PatternTemplate;

impl PatternTemplate {
    /// Create a basic rock pattern template
    pub fn rock_4_4() -> PatternBuilder {
        PatternBuilder::new("rock_template", TimeSignature::new(4, 4))
            .display_name("Rock Template")
            .genre(PatternGenre::Rock)
            .difficulty(2)
            .tempo_range(80, 140)
            .description("Basic rock pattern template")
    }

    /// Create a jazz pattern template
    pub fn jazz_4_4() -> PatternBuilder {
        PatternBuilder::new("jazz_template", TimeSignature::new(4, 4))
            .display_name("Jazz Template")
            .genre(PatternGenre::Jazz)
            .difficulty(3)
            .tempo_range(120, 200)
            .description("Jazz pattern template")
    }

    /// Create a waltz pattern template
    pub fn waltz_3_4() -> PatternBuilder {
        PatternBuilder::new("waltz_template", TimeSignature::new(3, 4))
            .display_name("Waltz Template")
            .genre(PatternGenre::Classical)
            .difficulty(2)
            .tempo_range(90, 180)
            .description("Waltz pattern template")
    }

    /// Create a latin pattern template
    pub fn latin_4_4() -> PatternBuilder {
        PatternBuilder::new("latin_template", TimeSignature::new(4, 4))
            .display_name("Latin Template")
            .genre(PatternGenre::Latin)
            .difficulty(3)
            .tempo_range(100, 160)
            .description("Latin pattern template")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_builder_basic() {
        let pattern = PatternBuilder::new("test", TimeSignature::new(4, 4))
            .display_name("Test Pattern")
            .tempo_range(80, 120)
            .genre(PatternGenre::Rock)
            .difficulty(2)
            .kick(1.0)
            .snare(2.0)
            .build()
            .unwrap();

        assert_eq!(pattern.name, "test");
        assert_eq!(pattern.display_name, "Test Pattern");
        assert_eq!(pattern.beats.len(), 2);
    }

    #[test]
    fn test_beat_builder() {
        let pattern = PatternBuilder::new("test", TimeSignature::new(4, 4))
            .beat(1.0)
            .kick()
            .hihat_closed()
            .accent()
            .build()
            .beat(2.0)
            .snare()
            .build()
            .build()
            .unwrap();

        assert_eq!(pattern.beats.len(), 2);
        assert_eq!(pattern.beats[0].samples.len(), 2);
        assert!(pattern.beats[0].accent);
        assert!(!pattern.beats[1].accent);
    }

    #[test]
    fn test_notation_builder() {
        let pattern = PatternBuilder::new("notation_test", TimeSignature::new(4, 4))
            .from_notation("K.S.")
            .build()
            .unwrap();

        assert_eq!(pattern.beats.len(), 4); // K.S. creates 4 beats: K(1.0), .(1.5), S(2.0), .(2.5)
    }

    #[test]
    fn test_pattern_validation_empty() {
        let result = PatternBuilder::new("empty", TimeSignature::new(4, 4)).build();

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, PatternValidationError::EmptyPattern)));
    }

    #[test]
    fn test_pattern_validation_invalid_position() {
        let result = PatternBuilder::new("invalid_pos", TimeSignature::new(4, 4))
            .kick(5.0) // Invalid for 4/4 time
            .build();

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, PatternValidationError::InvalidBeatPosition(_))));
    }

    #[test]
    fn test_pattern_validation_invalid_tempo() {
        let result = PatternBuilder::new("invalid_tempo", TimeSignature::new(4, 4))
            .tempo_range(120, 80) // Min > max
            .kick(1.0)
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_build_or_default() {
        let pattern = PatternBuilder::new("", TimeSignature::new(4, 4)).build_or_default();

        assert_eq!(pattern.name, "default");
        assert!(!pattern.beats.is_empty());
    }

    #[test]
    fn test_pattern_templates() {
        let rock = PatternTemplate::rock_4_4()
            .kick(1.0)
            .snare(2.0)
            .build()
            .unwrap();

        assert_eq!(rock.metadata.genre, PatternGenre::Rock);
        assert_eq!(rock.time_signature.beats_per_measure, 4);
    }
}
