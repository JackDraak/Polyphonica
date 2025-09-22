/// Sample metadata and configuration management
///
/// This module provides structured metadata and configuration for samples,
/// including categorization, envelope settings, and organizational tools
/// for managing large sample libraries.
use crate::AdsrEnvelope;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Comprehensive sample metadata
///
/// Contains all information needed to properly load, configure, and
/// categorize a sample within the library system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleMetadata {
    /// Unique identifier for the sample
    pub name: String,

    /// Display name for user interfaces
    pub display_name: String,

    /// File path relative to sample root
    pub file_path: String,

    /// Base frequency of the sample (Hz)
    pub base_frequency: f32,

    /// ADSR envelope configuration
    pub envelope: AdsrEnvelope,

    /// Sample category for organization
    pub category: SampleCategory,

    /// Tags for flexible filtering and search
    pub tags: Vec<String>,

    /// Volume adjustment factor (0.0 - 2.0)
    pub volume: f32,

    /// Whether sample supports pitch adjustment
    pub pitch_adjustable: bool,

    /// Preferred velocity range (0.0 - 1.0)
    pub velocity_range: (f32, f32),

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl SampleMetadata {
    /// Create new sample metadata with defaults
    pub fn new(name: &str, file_path: &str, base_frequency: f32) -> Self {
        Self {
            name: name.to_string(),
            display_name: name.to_string(),
            file_path: file_path.to_string(),
            base_frequency,
            envelope: AdsrEnvelope::default_drum(),
            category: SampleCategory::Percussion,
            tags: Vec::new(),
            volume: 1.0,
            pitch_adjustable: true,
            velocity_range: (0.0, 1.0),
            metadata: HashMap::new(),
        }
    }

    /// Create drum sample metadata with appropriate defaults
    pub fn drum(name: &str, file_path: &str, drum_type: DrumType) -> Self {
        let envelope = match drum_type {
            DrumType::Kick => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 0.15,
                sustain_level: 0.2,
                release_secs: 0.3,
            },
            DrumType::Snare => AdsrEnvelope {
                attack_secs: 0.002,
                decay_secs: 0.08,
                sustain_level: 0.1,
                release_secs: 0.15,
            },
            DrumType::HiHat => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 0.05,
                sustain_level: 0.0,
                release_secs: 0.1,
            },
            DrumType::Cymbal => AdsrEnvelope {
                attack_secs: 0.002,
                decay_secs: 0.2,
                sustain_level: 0.3,
                release_secs: 0.8,
            },
            DrumType::Tom => AdsrEnvelope {
                attack_secs: 0.002,
                decay_secs: 0.12,
                sustain_level: 0.2,
                release_secs: 0.25,
            },
            DrumType::Percussion => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 0.1,
                sustain_level: 0.1,
                release_secs: 0.2,
            },
        };

        Self {
            name: name.to_string(),
            display_name: name.to_string(),
            file_path: file_path.to_string(),
            base_frequency: 60.0, // Most drums around 60Hz
            envelope,
            category: SampleCategory::Drums,
            tags: vec![drum_type.to_string()],
            volume: 1.0,
            pitch_adjustable: false, // Most drums don't pitch well
            velocity_range: (0.0, 1.0),
            metadata: HashMap::new(),
        }
    }

    /// Add a tag to the sample
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    /// Set volume adjustment
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    /// Set envelope
    pub fn with_envelope(mut self, envelope: AdsrEnvelope) -> Self {
        self.envelope = envelope;
        self
    }

    /// Add metadata field
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Sample category for organization and filtering
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SampleCategory {
    /// Drum samples (kick, snare, hi-hat, etc.)
    Drums,

    /// General percussion (shakers, bells, etc.)
    Percussion,

    /// Melodic instruments
    Melodic,

    /// Bass instruments
    Bass,

    /// Lead instruments
    Lead,

    /// Pad/ambient sounds
    Pad,

    /// Sound effects
    SFX,

    /// Vocal samples
    Vocal,

    /// Synthetic sounds
    Synthetic,

    /// Uncategorized
    Other,
}

impl SampleCategory {
    /// Get all available categories
    pub fn all() -> Vec<SampleCategory> {
        vec![
            SampleCategory::Drums,
            SampleCategory::Percussion,
            SampleCategory::Melodic,
            SampleCategory::Bass,
            SampleCategory::Lead,
            SampleCategory::Pad,
            SampleCategory::SFX,
            SampleCategory::Vocal,
            SampleCategory::Synthetic,
            SampleCategory::Other,
        ]
    }

    /// Get display name for category
    pub fn display_name(&self) -> &'static str {
        match self {
            SampleCategory::Drums => "Drums",
            SampleCategory::Percussion => "Percussion",
            SampleCategory::Melodic => "Melodic",
            SampleCategory::Bass => "Bass",
            SampleCategory::Lead => "Lead",
            SampleCategory::Pad => "Pad",
            SampleCategory::SFX => "Sound Effects",
            SampleCategory::Vocal => "Vocal",
            SampleCategory::Synthetic => "Synthetic",
            SampleCategory::Other => "Other",
        }
    }
}

/// Drum type for specialized drum sample configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DrumType {
    Kick,
    Snare,
    HiHat,
    Cymbal,
    Tom,
    Percussion,
}

impl DrumType {
    pub fn to_string(&self) -> String {
        match self {
            DrumType::Kick => "kick".to_string(),
            DrumType::Snare => "snare".to_string(),
            DrumType::HiHat => "hihat".to_string(),
            DrumType::Cymbal => "cymbal".to_string(),
            DrumType::Tom => "tom".to_string(),
            DrumType::Percussion => "percussion".to_string(),
        }
    }
}

/// Sample catalog for managing collections of samples
///
/// Provides organization, filtering, and batch operations on sample
/// metadata collections.
pub struct SampleCatalog {
    /// All samples in the catalog
    samples: HashMap<String, SampleMetadata>,

    /// Category index for fast filtering
    category_index: HashMap<SampleCategory, Vec<String>>,

    /// Tag index for fast search
    tag_index: HashMap<String, Vec<String>>,
}

impl SampleCatalog {
    /// Create a new empty catalog
    pub fn new() -> Self {
        Self {
            samples: HashMap::new(),
            category_index: HashMap::new(),
            tag_index: HashMap::new(),
        }
    }

    /// Add a sample to the catalog
    pub fn add_sample(&mut self, sample: SampleMetadata) {
        let name = sample.name.clone();

        // Update category index
        self.category_index
            .entry(sample.category.clone())
            .or_insert_with(Vec::new)
            .push(name.clone());

        // Update tag index
        for tag in &sample.tags {
            self.tag_index
                .entry(tag.clone())
                .or_insert_with(Vec::new)
                .push(name.clone());
        }

        // Store sample
        self.samples.insert(name, sample);
    }

    /// Get a sample by name
    pub fn get_sample(&self, name: &str) -> Option<&SampleMetadata> {
        self.samples.get(name)
    }

    /// Get all samples in a category
    pub fn get_by_category(&self, category: &SampleCategory) -> Vec<&SampleMetadata> {
        self.category_index
            .get(category)
            .map(|names| {
                names
                    .iter()
                    .filter_map(|name| self.samples.get(name))
                    .collect()
            })
            .unwrap_or_else(Vec::new)
    }

    /// Search samples by tag
    pub fn search_by_tag(&self, tag: &str) -> Vec<&SampleMetadata> {
        self.tag_index
            .get(tag)
            .map(|names| {
                names
                    .iter()
                    .filter_map(|name| self.samples.get(name))
                    .collect()
            })
            .unwrap_or_else(Vec::new)
    }

    /// Get all samples
    pub fn all_samples(&self) -> Vec<&SampleMetadata> {
        self.samples.values().collect()
    }

    /// Get sample count
    pub fn count(&self) -> usize {
        self.samples.len()
    }

    /// Get available categories with counts
    pub fn categories(&self) -> Vec<(SampleCategory, usize)> {
        self.category_index
            .iter()
            .map(|(cat, samples)| (cat.clone(), samples.len()))
            .collect()
    }

    /// Get available tags with counts
    pub fn tags(&self) -> Vec<(String, usize)> {
        self.tag_index
            .iter()
            .map(|(tag, samples)| (tag.clone(), samples.len()))
            .collect()
    }

    /// Clear all samples
    pub fn clear(&mut self) {
        self.samples.clear();
        self.category_index.clear();
        self.tag_index.clear();
    }
}

impl Default for SampleCatalog {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for AdsrEnvelope with drum presets
trait DrumEnvelopeExt {
    fn default_drum() -> Self;
}

impl DrumEnvelopeExt for AdsrEnvelope {
    fn default_drum() -> Self {
        Self {
            attack_secs: 0.002,
            decay_secs: 0.1,
            sustain_level: 0.3,
            release_secs: 0.2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_metadata_creation() {
        let sample = SampleMetadata::new("kick", "drums/kick.wav", 60.0);
        assert_eq!(sample.name, "kick");
        assert_eq!(sample.file_path, "drums/kick.wav");
        assert_eq!(sample.base_frequency, 60.0);
    }

    #[test]
    fn test_drum_sample_creation() {
        let sample = SampleMetadata::drum("kick", "drums/kick.wav", DrumType::Kick);
        assert_eq!(sample.category, SampleCategory::Drums);
        assert!(sample.tags.contains(&"kick".to_string()));
    }

    #[test]
    fn test_sample_catalog() {
        let mut catalog = SampleCatalog::new();

        let kick = SampleMetadata::drum("kick", "drums/kick.wav", DrumType::Kick);
        catalog.add_sample(kick);

        assert_eq!(catalog.count(), 1);
        assert!(catalog.get_sample("kick").is_some());

        let drums = catalog.get_by_category(&SampleCategory::Drums);
        assert_eq!(drums.len(), 1);
    }

    #[test]
    fn test_tag_search() {
        let mut catalog = SampleCatalog::new();

        let kick =
            SampleMetadata::drum("kick", "drums/kick.wav", DrumType::Kick).with_tag("acoustic");
        catalog.add_sample(kick);

        let acoustic_samples = catalog.search_by_tag("acoustic");
        assert_eq!(acoustic_samples.len(), 1);
    }

    #[test]
    fn test_sample_category_display() {
        assert_eq!(SampleCategory::Drums.display_name(), "Drums");
        assert_eq!(SampleCategory::SFX.display_name(), "Sound Effects");
    }
}
