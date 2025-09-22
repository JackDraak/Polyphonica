/// Pattern import/export functionality for JSON catalog format
///
/// This module provides bidirectional conversion between our internal pattern
/// representation and the JSON catalog format, enabling external pattern
/// management while preserving type safety.

use super::types::{DrumPattern, DrumPatternBeat, PatternGenre};
use crate::timing::{TimeSignature, ClickType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// JSON catalog format structure
#[derive(Debug, Serialize, Deserialize)]
pub struct PatternCatalog {
    pub catalog_version: String,
    pub description: String,
    pub created: String,
    pub drum_patterns: HashMap<String, JsonPattern>,
}

/// JSON pattern representation
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonPattern {
    pub name: String,
    pub time_signature: String,
    pub tempo_range: [u32; 2],
    pub pattern: Vec<JsonBeat>,
}

/// JSON beat representation
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonBeat {
    pub beat: f32,
    pub samples: Vec<String>,
    pub accent: bool,
}

/// Error types for pattern I/O operations
#[derive(Debug, thiserror::Error)]
pub enum PatternIoError {
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid time signature: {0}")]
    InvalidTimeSignature(String),

    #[error("Unknown sample type: {0}")]
    UnknownSampleType(String),

    #[error("Invalid pattern data: {0}")]
    InvalidPattern(String),
}

impl PatternCatalog {
    /// Load pattern catalog from JSON string
    pub fn from_json(json: &str) -> Result<Self, PatternIoError> {
        serde_json::from_str(json).map_err(PatternIoError::JsonError)
    }

    /// Save pattern catalog to JSON string
    pub fn to_json(&self) -> Result<String, PatternIoError> {
        serde_json::to_string_pretty(self).map_err(PatternIoError::JsonError)
    }

    /// Convert to internal DrumPattern collection
    pub fn to_patterns(&self) -> Result<Vec<DrumPattern>, PatternIoError> {
        let mut patterns = Vec::new();

        for (key, json_pattern) in &self.drum_patterns {
            let pattern = json_pattern.to_drum_pattern(key)?;
            patterns.push(pattern);
        }

        Ok(patterns)
    }

    /// Create catalog from internal DrumPattern collection
    pub fn from_patterns(patterns: &[DrumPattern]) -> Self {
        let mut drum_patterns = HashMap::new();

        for pattern in patterns {
            let json_pattern = JsonPattern::from_drum_pattern(pattern);
            drum_patterns.insert(pattern.name.clone(), json_pattern);
        }

        Self {
            catalog_version: "2.0".to_string(),
            description: "Guitar Buddy Pattern Catalog - Generated from Code".to_string(),
            created: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            drum_patterns,
        }
    }
}

impl JsonPattern {
    /// Convert to internal DrumPattern
    pub fn to_drum_pattern(&self, key: &str) -> Result<DrumPattern, PatternIoError> {
        // Parse time signature
        let time_sig = self.parse_time_signature()?;

        // Convert beats
        let mut beats = Vec::new();
        for json_beat in &self.pattern {
            let beat = json_beat.to_drum_pattern_beat()?;
            beats.push(beat);
        }

        // Create pattern
        let mut pattern = DrumPattern::new(key, time_sig);
        pattern.display_name = self.name.clone();
        pattern.tempo_range = (self.tempo_range[0], self.tempo_range[1]);
        pattern.beats = beats;

        // Try to infer genre from name/tags
        pattern.metadata.genre = Self::infer_genre(&self.name);
        pattern.metadata.description = format!("Imported from JSON catalog: {}", self.name);

        Ok(pattern)
    }

    /// Create from internal DrumPattern
    pub fn from_drum_pattern(pattern: &DrumPattern) -> Self {
        let time_sig_str = format!("{}/{}",
            pattern.time_signature.beats_per_measure,
            pattern.time_signature.note_value);

        let json_beats: Vec<JsonBeat> = pattern.beats.iter()
            .map(JsonBeat::from_drum_pattern_beat)
            .collect();

        Self {
            name: pattern.display_name.clone(),
            time_signature: time_sig_str,
            tempo_range: [pattern.tempo_range.0, pattern.tempo_range.1],
            pattern: json_beats,
        }
    }

    fn parse_time_signature(&self) -> Result<TimeSignature, PatternIoError> {
        let parts: Vec<&str> = self.time_signature.split('/').collect();
        if parts.len() != 2 {
            return Err(PatternIoError::InvalidTimeSignature(
                self.time_signature.clone()
            ));
        }

        let beats: u8 = parts[0].parse()
            .map_err(|_| PatternIoError::InvalidTimeSignature(self.time_signature.clone()))?;
        let note_value: u8 = parts[1].parse()
            .map_err(|_| PatternIoError::InvalidTimeSignature(self.time_signature.clone()))?;

        Ok(TimeSignature::new(beats, note_value))
    }

    fn infer_genre(name: &str) -> PatternGenre {
        let name_lower = name.to_lowercase();
        if name_lower.contains("rock") {
            PatternGenre::Rock
        } else if name_lower.contains("jazz") || name_lower.contains("swing") {
            PatternGenre::Jazz
        } else if name_lower.contains("latin") || name_lower.contains("samba") || name_lower.contains("bossa") {
            PatternGenre::Latin
        } else if name_lower.contains("funk") {
            PatternGenre::Funk
        } else if name_lower.contains("pop") || name_lower.contains("ballad") {
            PatternGenre::Pop
        } else if name_lower.contains("electronic") || name_lower.contains("house") {
            PatternGenre::Electronic
        } else if name_lower.contains("waltz") {
            PatternGenre::Classical
        } else {
            PatternGenre::Custom
        }
    }
}

impl JsonBeat {
    /// Convert to internal DrumPatternBeat
    pub fn to_drum_pattern_beat(&self) -> Result<DrumPatternBeat, PatternIoError> {
        let mut beat = DrumPatternBeat::new(self.beat);
        beat.accent = self.accent;

        // Convert sample names to ClickType
        for sample_name in &self.samples {
            let click_type = Self::parse_sample_name(sample_name)?;
            beat.samples.push(click_type);
        }

        Ok(beat)
    }

    /// Create from internal DrumPatternBeat
    pub fn from_drum_pattern_beat(beat: &DrumPatternBeat) -> Self {
        let sample_names: Vec<String> = beat.samples.iter()
            .map(|click_type| Self::click_type_to_sample_name(*click_type))
            .collect();

        Self {
            beat: beat.beat_position,
            samples: sample_names,
            accent: beat.accent,
        }
    }

    fn parse_sample_name(name: &str) -> Result<ClickType, PatternIoError> {
        match name {
            "kick" => Ok(ClickType::AcousticKick),
            "kick_tight" => Ok(ClickType::KickTight),
            "snare" => Ok(ClickType::AcousticSnare),
            "hihat_closed" => Ok(ClickType::HiHatClosed),
            "hihat_open" => Ok(ClickType::HiHatOpen),
            "hihat_loose" => Ok(ClickType::HiHatLoose),
            "hihat_very_loose" => Ok(ClickType::HiHatVeryLoose),
            "rimshot" => Ok(ClickType::RimShot),
            "stick" => Ok(ClickType::Stick),
            "cymbal_splash" => Ok(ClickType::CymbalSplash),
            "cymbal_roll" => Ok(ClickType::CymbalRoll),
            "ride" => Ok(ClickType::Ride),
            "ride_bell" => Ok(ClickType::RideBell),
            _ => Err(PatternIoError::UnknownSampleType(name.to_string())),
        }
    }

    fn click_type_to_sample_name(click_type: ClickType) -> String {
        match click_type {
            ClickType::AcousticKick => "kick".to_string(),
            ClickType::KickTight => "kick_tight".to_string(),
            ClickType::AcousticSnare => "snare".to_string(),
            ClickType::HiHatClosed => "hihat_closed".to_string(),
            ClickType::HiHatOpen => "hihat_open".to_string(),
            ClickType::HiHatLoose => "hihat_loose".to_string(),
            ClickType::HiHatVeryLoose => "hihat_very_loose".to_string(),
            ClickType::RimShot => "rimshot".to_string(),
            ClickType::Stick => "stick".to_string(),
            ClickType::CymbalSplash => "cymbal_splash".to_string(),
            ClickType::CymbalRoll => "cymbal_roll".to_string(),
            ClickType::Ride => "ride".to_string(),
            ClickType::RideBell => "ride_bell".to_string(),
            // Synthetic sounds - map to simple names
            ClickType::WoodBlock => "woodblock".to_string(),
            ClickType::DigitalBeep => "beep".to_string(),
            ClickType::Cowbell => "cowbell".to_string(),
            ClickType::ElectroClick => "click".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_pattern_conversion() {
        let json = r#"{
            "name": "Test Pattern",
            "time_signature": "4/4",
            "tempo_range": [80, 120],
            "pattern": [
                {"beat": 1.0, "samples": ["kick", "hihat_closed"], "accent": true},
                {"beat": 2.0, "samples": ["snare"], "accent": false}
            ]
        }"#;

        let json_pattern: JsonPattern = serde_json::from_str(json).unwrap();
        let drum_pattern = json_pattern.to_drum_pattern("test").unwrap();

        assert_eq!(drum_pattern.name, "test");
        assert_eq!(drum_pattern.display_name, "Test Pattern");
        assert_eq!(drum_pattern.beats.len(), 2);
        assert_eq!(drum_pattern.beats[0].samples.len(), 2);
        assert!(drum_pattern.beats[0].accent);
        assert!(!drum_pattern.beats[1].accent);
    }

    #[test]
    fn test_sample_name_conversion() {
        assert_eq!(JsonBeat::parse_sample_name("kick").unwrap(), ClickType::AcousticKick);
        assert_eq!(JsonBeat::parse_sample_name("hihat_closed").unwrap(), ClickType::HiHatClosed);
        assert!(JsonBeat::parse_sample_name("invalid").is_err());
    }

    #[test]
    fn test_time_signature_parsing() {
        let json_pattern = JsonPattern {
            name: "Test".to_string(),
            time_signature: "3/4".to_string(),
            tempo_range: [60, 120],
            pattern: vec![],
        };

        let time_sig = json_pattern.parse_time_signature().unwrap();
        assert_eq!(time_sig.beats_per_measure, 3);
        assert_eq!(time_sig.note_value, 4);
    }
}