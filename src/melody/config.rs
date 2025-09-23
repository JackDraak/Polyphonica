/// Configuration system for melody assistant module
///
/// This module provides the MelodyConfig struct for persisting user preferences
/// and generation parameters, with serde support for JSON serialization.

use super::types::*;
use crate::timing::TimeSignature;
use serde::{Deserialize, Serialize};

/// Complete configuration for melody assistant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MelodyConfig {
    /// Default key selection for chord generation
    pub default_key_selection: KeySelection,

    /// Default time signature for progression timing
    pub default_time_signature: TimeSignature,

    /// Timeline display and behavior configuration
    pub timeline_config: TimelineConfig,

    /// Chord generation parameters
    pub generation_config: GenerationConfig,

    /// How many beats ahead to generate chords
    pub generation_ahead_beats: u32,

    /// Music theory adherence (0.0 = random, 1.0 = strict theory)
    pub theory_adherence: f32,

    /// Weight for voice leading smoothness (0.0 = ignore, 1.0 = prioritize)
    pub voice_leading_weight: f32,

    /// Penalty for chord repetition (0.0 = allow, 1.0 = strong penalty)
    pub repetition_penalty: f32,

    /// UI and display preferences
    pub ui_config: UiConfig,
}

impl Default for MelodyConfig {
    fn default() -> Self {
        Self {
            default_key_selection: KeySelection::for_major_key(Note::C),
            default_time_signature: TimeSignature::new(4, 4),
            timeline_config: TimelineConfig::default(),
            generation_config: GenerationConfig::default(),
            generation_ahead_beats: 32, // 8 measures in 4/4
            theory_adherence: 0.9, // High theory adherence for musical results
            voice_leading_weight: 0.7, // Smooth voice leading preferred
            repetition_penalty: 0.5, // Moderate repetition penalty
            ui_config: UiConfig::default(),
        }
    }
}

impl MelodyConfig {
    /// Create configuration for specific key
    pub fn for_key(key: Note, is_major: bool) -> Self {
        let mut config = Self::default();
        config.default_key_selection = KeySelection::for_key(key, is_major);
        config
    }

    /// Create configuration for jazz-style progressions
    pub fn jazz_style() -> Self {
        let mut config = Self::default();
        config.generation_config.preferred_qualities = vec![
            ChordQuality::Major7,
            ChordQuality::Minor7,
            ChordQuality::Dominant7,
        ];
        config.generation_config.complexity_level = ComplexityLevel::Advanced;
        config.timeline_config.beats_per_chord = 2; // Faster chord changes
        config.theory_adherence = 0.95; // Strong theory adherence for jazz
        config.voice_leading_weight = 0.8; // Smooth voice leading important
        config
    }

    /// Create configuration for pop/rock progressions
    pub fn pop_style() -> Self {
        let mut config = Self::default();
        config.generation_config.preferred_qualities = vec![
            ChordQuality::Major,
            ChordQuality::Minor,
        ];
        config.generation_config.complexity_level = ComplexityLevel::Beginner;
        config.timeline_config.beats_per_chord = 4; // Slower chord changes
        config.theory_adherence = 0.8; // Allow some creative freedom
        config
    }

    /// Create configuration for practice/drilling
    pub fn practice_mode() -> Self {
        let mut config = Self::default();
        config.timeline_config.auto_advance = true;
        config.timeline_config.show_key_changes = true;
        config.ui_config.show_chord_notation = true;
        config.ui_config.show_roman_numerals = true;
        config.generation_ahead_beats = 48; // More lookahead for practice
        config
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.theory_adherence < 0.0 || self.theory_adherence > 1.0 {
            return Err(ConfigError::InvalidRange {
                field: "theory_adherence".to_string(),
                value: self.theory_adherence,
                min: 0.0,
                max: 1.0,
            });
        }

        if self.voice_leading_weight < 0.0 || self.voice_leading_weight > 1.0 {
            return Err(ConfigError::InvalidRange {
                field: "voice_leading_weight".to_string(),
                value: self.voice_leading_weight,
                min: 0.0,
                max: 1.0,
            });
        }

        if self.repetition_penalty < 0.0 || self.repetition_penalty > 1.0 {
            return Err(ConfigError::InvalidRange {
                field: "repetition_penalty".to_string(),
                value: self.repetition_penalty,
                min: 0.0,
                max: 1.0,
            });
        }

        if self.generation_ahead_beats == 0 {
            return Err(ConfigError::InvalidValue {
                field: "generation_ahead_beats".to_string(),
                reason: "Must be greater than 0".to_string(),
            });
        }

        if self.timeline_config.beats_per_chord == 0 {
            return Err(ConfigError::InvalidValue {
                field: "beats_per_chord".to_string(),
                reason: "Must be greater than 0".to_string(),
            });
        }

        Ok(())
    }

    /// Apply preset configuration
    pub fn apply_preset(&mut self, preset: ConfigPreset) {
        match preset {
            ConfigPreset::Jazz => *self = Self::jazz_style(),
            ConfigPreset::Pop => *self = Self::pop_style(),
            ConfigPreset::Practice => *self = Self::practice_mode(),
            ConfigPreset::Default => *self = Self::default(),
        }
    }
}

/// Chord generation-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    /// Preferred chord qualities for generation
    pub preferred_qualities: Vec<ChordQuality>,

    /// Complexity level for chord selection
    pub complexity_level: ComplexityLevel,

    /// Allow chord inversions
    pub allow_inversions: bool,

    /// Maximum number of consecutive identical chords
    pub max_repetition: u8,

    /// Probability of key modulation
    pub modulation_probability: f32,

    /// Allowed modulation intervals (semitones)
    pub modulation_intervals: Vec<i8>,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            preferred_qualities: vec![
                ChordQuality::Major,
                ChordQuality::Minor,
                ChordQuality::Dominant7,
            ],
            complexity_level: ComplexityLevel::Intermediate,
            allow_inversions: true,
            max_repetition: 2,
            modulation_probability: 0.1, // 10% chance of modulation
            modulation_intervals: vec![5, 7, -5], // Perfect 4th, 5th, down 4th
        }
    }
}

/// Complexity levels for chord generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplexityLevel {
    /// Simple major/minor triads
    Beginner,
    /// Add some 7th chords and sus chords
    Intermediate,
    /// Full range of chord qualities and extensions
    Advanced,
    /// Professional level with complex harmonies
    Expert,
}

impl ComplexityLevel {
    /// Get allowed chord qualities for this complexity level
    pub fn allowed_qualities(&self) -> Vec<ChordQuality> {
        match self {
            ComplexityLevel::Beginner => vec![
                ChordQuality::Major,
                ChordQuality::Minor,
            ],
            ComplexityLevel::Intermediate => vec![
                ChordQuality::Major,
                ChordQuality::Minor,
                ChordQuality::Dominant7,
                ChordQuality::Sus2,
                ChordQuality::Sus4,
            ],
            ComplexityLevel::Advanced => vec![
                ChordQuality::Major,
                ChordQuality::Minor,
                ChordQuality::Major7,
                ChordQuality::Minor7,
                ChordQuality::Dominant7,
                ChordQuality::Sus2,
                ChordQuality::Sus4,
                ChordQuality::Diminished,
            ],
            ComplexityLevel::Expert => ChordQuality::all_qualities(),
        }
    }

    /// Get description of complexity level
    pub fn description(&self) -> &'static str {
        match self {
            ComplexityLevel::Beginner => "Simple major and minor chords",
            ComplexityLevel::Intermediate => "Basic chords plus 7ths and sus chords",
            ComplexityLevel::Advanced => "Wide variety of chord types",
            ComplexityLevel::Expert => "All chord types including complex harmonies",
        }
    }
}

/// UI and display configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Show chord symbols (C, Dm, G7, etc.)
    pub show_chord_symbols: bool,

    /// Show standard musical notation
    pub show_chord_notation: bool,

    /// Show Roman numeral analysis (I, ii, V, etc.)
    pub show_roman_numerals: bool,

    /// Show key center information
    pub show_key_center: bool,

    /// Show chord function labels (tonic, subdominant, etc.)
    pub show_chord_functions: bool,

    /// Timeline visualization style
    pub timeline_style: TimelineStyle,

    /// Color scheme for chord display
    pub color_scheme: ColorScheme,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            show_chord_symbols: true,
            show_chord_notation: false, // Musical notation requires more complex UI
            show_roman_numerals: false,
            show_key_center: true,
            show_chord_functions: false,
            timeline_style: TimelineStyle::Horizontal,
            color_scheme: ColorScheme::Default,
        }
    }
}

/// Timeline visualization styles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimelineStyle {
    /// Horizontal timeline (left to right)
    Horizontal,
    /// Vertical timeline (top to bottom)
    Vertical,
    /// Circular timeline (like a clock)
    Circular,
}

/// Color schemes for chord display
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorScheme {
    /// Default color scheme
    Default,
    /// High contrast for accessibility
    HighContrast,
    /// Color-blind friendly palette
    ColorBlindFriendly,
    /// Monochrome for printing
    Monochrome,
}

/// Configuration presets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigPreset {
    /// Default balanced configuration
    Default,
    /// Jazz-style progressions
    Jazz,
    /// Pop/rock progressions
    Pop,
    /// Practice/drilling mode
    Practice,
}

impl ConfigPreset {
    /// Get all available presets
    pub fn all() -> Vec<ConfigPreset> {
        vec![
            ConfigPreset::Default,
            ConfigPreset::Jazz,
            ConfigPreset::Pop,
            ConfigPreset::Practice,
        ]
    }

    /// Get preset name for display
    pub fn name(&self) -> &'static str {
        match self {
            ConfigPreset::Default => "Default",
            ConfigPreset::Jazz => "Jazz",
            ConfigPreset::Pop => "Pop/Rock",
            ConfigPreset::Practice => "Practice",
        }
    }

    /// Get preset description
    pub fn description(&self) -> &'static str {
        match self {
            ConfigPreset::Default => "Balanced settings for general use",
            ConfigPreset::Jazz => "Complex harmonies with smooth voice leading",
            ConfigPreset::Pop => "Simple, popular chord progressions",
            ConfigPreset::Practice => "Enhanced features for chord drilling",
        }
    }
}

/// Configuration errors
#[derive(Debug, Clone)]
pub enum ConfigError {
    /// Value is outside valid range
    InvalidRange {
        field: String,
        value: f32,
        min: f32,
        max: f32,
    },
    /// Value is invalid for other reasons
    InvalidValue {
        field: String,
        reason: String,
    },
    /// Serialization/deserialization error
    SerializationError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::InvalidRange { field, value, min, max } => {
                write!(f, "Field '{}' value {} is outside valid range [{}, {}]",
                       field, value, min, max)
            }
            ConfigError::InvalidValue { field, reason } => {
                write!(f, "Field '{}' has invalid value: {}", field, reason)
            }
            ConfigError::SerializationError(msg) => {
                write!(f, "Configuration serialization error: {}", msg)
            }
        }
    }
}

impl std::error::Error for ConfigError {}

/// Configuration manager for persistence
pub struct ConfigManager;

impl ConfigManager {
    /// Save configuration to JSON file
    pub fn save_to_file(config: &MelodyConfig, path: &std::path::Path) -> Result<(), ConfigError> {
        config.validate()?;

        let json = serde_json::to_string_pretty(config)
            .map_err(|e| ConfigError::SerializationError(e.to_string()))?;

        std::fs::write(path, json)
            .map_err(|e| ConfigError::SerializationError(e.to_string()))?;

        Ok(())
    }

    /// Load configuration from JSON file
    pub fn load_from_file(path: &std::path::Path) -> Result<MelodyConfig, ConfigError> {
        let json = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::SerializationError(e.to_string()))?;

        let config: MelodyConfig = serde_json::from_str(&json)
            .map_err(|e| ConfigError::SerializationError(e.to_string()))?;

        config.validate()?;
        Ok(config)
    }

    /// Save configuration to default location
    pub fn save_default(config: &MelodyConfig) -> Result<(), ConfigError> {
        if let Some(config_dir) = dirs::config_dir() {
            let app_dir = config_dir.join("polyphonica");
            std::fs::create_dir_all(&app_dir)
                .map_err(|e| ConfigError::SerializationError(e.to_string()))?;

            let config_path = app_dir.join("melody_config.json");
            Self::save_to_file(config, &config_path)
        } else {
            Err(ConfigError::SerializationError(
                "Could not determine config directory".to_string()
            ))
        }
    }

    /// Load configuration from default location
    pub fn load_default() -> Result<MelodyConfig, ConfigError> {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("polyphonica").join("melody_config.json");
            if config_path.exists() {
                Self::load_from_file(&config_path)
            } else {
                Ok(MelodyConfig::default())
            }
        } else {
            Ok(MelodyConfig::default())
        }
    }
}

// Add to ChordQuality implementation
impl ChordQuality {
    /// Get all available chord qualities
    pub fn all_qualities() -> Vec<ChordQuality> {
        vec![
            ChordQuality::Major,
            ChordQuality::Minor,
            ChordQuality::Diminished,
            ChordQuality::Augmented,
            ChordQuality::Major7,
            ChordQuality::Minor7,
            ChordQuality::Dominant7,
            ChordQuality::Sus2,
            ChordQuality::Sus4,
            ChordQuality::MinorMajor7,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = MelodyConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.default_key_selection.primary_key, Some(Note::C));
        assert_eq!(config.theory_adherence, 0.9);
    }

    #[test]
    fn test_config_presets() {
        let jazz_config = MelodyConfig::jazz_style();
        assert!(jazz_config.validate().is_ok());
        assert_eq!(jazz_config.timeline_config.beats_per_chord, 2);

        let pop_config = MelodyConfig::pop_style();
        assert!(pop_config.validate().is_ok());
        assert_eq!(pop_config.timeline_config.beats_per_chord, 4);
    }

    #[test]
    fn test_config_validation() {
        let mut config = MelodyConfig::default();

        // Valid configuration
        assert!(config.validate().is_ok());

        // Invalid theory adherence
        config.theory_adherence = 1.5;
        assert!(config.validate().is_err());

        config.theory_adherence = -0.1;
        assert!(config.validate().is_err());

        // Reset and test invalid generation ahead beats
        config = MelodyConfig::default();
        config.generation_ahead_beats = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_complexity_levels() {
        let beginner = ComplexityLevel::Beginner;
        let beginner_qualities = beginner.allowed_qualities();
        assert_eq!(beginner_qualities.len(), 2); // Only Major and Minor

        let expert = ComplexityLevel::Expert;
        let expert_qualities = expert.allowed_qualities();
        assert!(expert_qualities.len() > 5); // Many more qualities
    }

    #[test]
    fn test_config_serialization() {
        let config = MelodyConfig::default();

        // Test serialization
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("default_key_selection"));

        // Test deserialization
        let deserialized: MelodyConfig = serde_json::from_str(&json).unwrap();
        assert!(deserialized.validate().is_ok());
        assert_eq!(deserialized.theory_adherence, config.theory_adherence);
    }

    #[test]
    fn test_config_for_key() {
        let config = MelodyConfig::for_key(Note::G, false); // G minor
        assert_eq!(config.default_key_selection.primary_key, Some(Note::G));
        assert!(config.default_key_selection.is_note_enabled(Note::G));
        assert!(config.default_key_selection.is_note_enabled(Note::ASharp)); // Bb in G minor
    }

    #[test]
    fn test_apply_preset() {
        let mut config = MelodyConfig::default();
        let original_adherence = config.theory_adherence;

        config.apply_preset(ConfigPreset::Jazz);
        assert_ne!(config.theory_adherence, original_adherence); // Should have changed
        assert_eq!(config.timeline_config.beats_per_chord, 2);
    }

    #[test]
    fn test_chord_quality_all() {
        let all_qualities = ChordQuality::all_qualities();
        assert!(all_qualities.contains(&ChordQuality::Major));
        assert!(all_qualities.contains(&ChordQuality::Dominant7));
        assert!(all_qualities.contains(&ChordQuality::Diminished));
        assert!(all_qualities.len() >= 10);
    }

    #[test]
    fn test_config_error_display() {
        let error = ConfigError::InvalidRange {
            field: "test_field".to_string(),
            value: 2.0,
            min: 0.0,
            max: 1.0,
        };

        let error_str = error.to_string();
        assert!(error_str.contains("test_field"));
        assert!(error_str.contains("2"));
    }
}