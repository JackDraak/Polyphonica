/// Specialized drum sample collections and kits
///
/// This module provides pre-configured drum kits and collections for
/// easy setup of common drum configurations. It integrates with the
/// timing module's ClickType for seamless metronome and pattern usage.

use super::catalog::{SampleMetadata, DrumType};
use crate::timing::ClickType;
use crate::AdsrEnvelope;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete drum kit configuration
///
/// A DrumKit contains a collection of drum samples with their mappings
/// to ClickType values for integration with the timing system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrumKit {
    /// Kit name and identifier
    pub name: String,

    /// Display name for user interfaces
    pub display_name: String,

    /// Kit description
    pub description: String,

    /// Drum samples in this kit
    pub samples: HashMap<ClickType, DrumSample>,

    /// Default velocity for samples
    pub default_velocity: f32,

    /// Kit-wide volume adjustment
    pub volume: f32,
}

/// Individual drum sample with timing integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrumSample {
    /// Sample metadata
    pub metadata: SampleMetadata,

    /// Mapping to timing system
    pub click_type: ClickType,

    /// Velocity curve adjustment
    pub velocity_curve: VelocityCurve,

    /// Sample-specific volume
    pub volume: f32,
}

/// Velocity response curve for dynamic playing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VelocityCurve {
    /// Linear response (velocity = volume)
    Linear,

    /// Exponential response (more dynamic range)
    Exponential(f32),

    /// Logarithmic response (compressed dynamics)
    Logarithmic,

    /// Custom curve with control points
    Custom(Vec<(f32, f32)>),
}

impl DrumKit {
    /// Create a new empty drum kit
    pub fn new(name: &str, display_name: &str) -> Self {
        Self {
            name: name.to_string(),
            display_name: display_name.to_string(),
            description: String::new(),
            samples: HashMap::new(),
            default_velocity: 0.8,
            volume: 1.0,
        }
    }

    /// Add a drum sample to the kit
    pub fn add_sample(&mut self, click_type: ClickType, sample: DrumSample) {
        self.samples.insert(click_type, sample);
    }

    /// Get a drum sample by click type
    pub fn get_sample(&self, click_type: &ClickType) -> Option<&DrumSample> {
        self.samples.get(click_type)
    }

    /// Get all samples as metadata for loading
    pub fn all_sample_metadata(&self) -> Vec<SampleMetadata> {
        self.samples.values().map(|sample| sample.metadata.clone()).collect()
    }

    /// Get click types supported by this kit
    pub fn supported_click_types(&self) -> Vec<ClickType> {
        self.samples.keys().cloned().collect()
    }

    /// Set kit description
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// Set kit volume
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    /// Create acoustic drum kit 01 (the current default kit)
    pub fn acoustic_kit_01() -> Self {
        let mut kit = Self::new("acoustic_kit_01", "Acoustic Drum Kit 01")
            .with_description("Professional acoustic drum kit with natural room sound");

        // Kick drum
        let kick_sample = DrumSample {
            metadata: SampleMetadata::drum("kick", "samples/drums/acoustic/kit_01/drumkit-kick.wav", DrumType::Kick)
                .with_volume(1.0)
                .with_envelope(AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.15,
                    sustain_level: 0.2,
                    release_secs: 0.3,
                }),
            click_type: ClickType::AcousticKick,
            velocity_curve: VelocityCurve::Exponential(1.5),
            volume: 1.0,
        };
        kit.add_sample(ClickType::AcousticKick, kick_sample);

        // Snare drum
        let snare_sample = DrumSample {
            metadata: SampleMetadata::drum("snare", "samples/drums/acoustic/kit_01/drumkit-snare.wav", DrumType::Snare)
                .with_volume(0.9)
                .with_envelope(AdsrEnvelope {
                    attack_secs: 0.002,
                    decay_secs: 0.08,
                    sustain_level: 0.1,
                    release_secs: 0.15,
                }),
            click_type: ClickType::AcousticSnare,
            velocity_curve: VelocityCurve::Exponential(1.2),
            volume: 0.9,
        };
        kit.add_sample(ClickType::AcousticSnare, snare_sample);

        // Hi-hat closed
        let hihat_closed_sample = DrumSample {
            metadata: SampleMetadata::drum("hihat", "samples/drums/acoustic/kit_01/drumkit-hihat.wav", DrumType::HiHat)
                .with_volume(0.7)
                .with_envelope(AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.05,
                    sustain_level: 0.0,
                    release_secs: 0.1,
                }),
            click_type: ClickType::HiHatClosed,
            velocity_curve: VelocityCurve::Linear,
            volume: 0.7,
        };
        kit.add_sample(ClickType::HiHatClosed, hihat_closed_sample);

        // Hi-hat open
        let hihat_open_sample = DrumSample {
            metadata: SampleMetadata::drum("hihat_open", "samples/drums/acoustic/kit_01/drumkit-hihat-open.wav", DrumType::HiHat)
                .with_volume(0.8)
                .with_envelope(AdsrEnvelope {
                    attack_secs: 0.002,
                    decay_secs: 0.2,
                    sustain_level: 0.3,
                    release_secs: 0.4,
                }),
            click_type: ClickType::HiHatOpen,
            velocity_curve: VelocityCurve::Linear,
            volume: 0.8,
        };
        kit.add_sample(ClickType::HiHatOpen, hihat_open_sample);

        // Rimshot
        let rimshot_sample = DrumSample {
            metadata: SampleMetadata::drum("rimshot", "samples/drums/acoustic/kit_01/drumkit-rimshot.wav", DrumType::Snare)
                .with_volume(0.8)
                .with_envelope(AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.06,
                    sustain_level: 0.0,
                    release_secs: 0.12,
                }),
            click_type: ClickType::RimShot,
            velocity_curve: VelocityCurve::Linear,
            volume: 0.8,
        };
        kit.add_sample(ClickType::RimShot, rimshot_sample);

        // Stick click
        let stick_sample = DrumSample {
            metadata: SampleMetadata::drum("stick", "samples/drums/acoustic/kit_01/drumkit-stick.wav", DrumType::Percussion)
                .with_volume(0.6)
                .with_envelope(AdsrEnvelope {
                    attack_secs: 0.0005,
                    decay_secs: 0.03,
                    sustain_level: 0.0,
                    release_secs: 0.06,
                }),
            click_type: ClickType::Stick,
            velocity_curve: VelocityCurve::Linear,
            volume: 0.6,
        };
        kit.add_sample(ClickType::Stick, stick_sample);

        kit
    }

    /// Create a synthetic drum kit using built-in waveforms
    pub fn synthetic_kit() -> Self {
        let mut kit = Self::new("synthetic_kit", "Synthetic Drum Kit")
            .with_description("Electronic drum sounds using synthesized waveforms");

        // These would use synthetic waveforms instead of samples
        // For now, we'll define the structure but note that sample paths are synthetic
        let kick_sample = DrumSample {
            metadata: SampleMetadata::new("synth_kick", "synthetic://kick", 60.0)
                .with_envelope(AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.2,
                    sustain_level: 0.1,
                    release_secs: 0.4,
                }),
            click_type: ClickType::AcousticKick,
            velocity_curve: VelocityCurve::Exponential(2.0),
            volume: 1.0,
        };
        kit.add_sample(ClickType::AcousticKick, kick_sample);

        kit
    }

    /// Get all available preset drum kits
    pub fn available_kits() -> Vec<DrumKit> {
        vec![
            Self::acoustic_kit_01(),
            Self::synthetic_kit(),
        ]
    }
}

impl DrumSample {
    /// Create a new drum sample
    pub fn new(metadata: SampleMetadata, click_type: ClickType) -> Self {
        Self {
            metadata,
            click_type,
            velocity_curve: VelocityCurve::Linear,
            volume: 1.0,
        }
    }

    /// Set velocity curve
    pub fn with_velocity_curve(mut self, curve: VelocityCurve) -> Self {
        self.velocity_curve = curve;
        self
    }

    /// Set sample volume
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    /// Apply velocity curve to input velocity
    pub fn apply_velocity(&self, input_velocity: f32) -> f32 {
        let clamped = input_velocity.clamp(0.0, 1.0);

        match &self.velocity_curve {
            VelocityCurve::Linear => clamped,
            VelocityCurve::Exponential(power) => clamped.powf(*power),
            VelocityCurve::Logarithmic => {
                if clamped <= 0.0 {
                    0.0
                } else {
                    (clamped.ln() + 1.0).max(0.0)
                }
            }
            VelocityCurve::Custom(points) => {
                // Linear interpolation between control points
                if points.is_empty() {
                    return clamped;
                }

                // Find surrounding points
                let mut lower = (0.0, 0.0);
                let mut upper = (1.0, 1.0);

                for &(x, y) in points {
                    if x <= clamped && x > lower.0 {
                        lower = (x, y);
                    }
                    if x >= clamped && x < upper.0 {
                        upper = (x, y);
                    }
                }

                // Linear interpolation
                if (upper.0 - lower.0).abs() < f32::EPSILON {
                    lower.1
                } else {
                    let t = (clamped - lower.0) / (upper.0 - lower.0);
                    lower.1 + t * (upper.1 - lower.1)
                }
            }
        }
    }
}

impl VelocityCurve {
    /// Create an exponential curve with the given power
    pub fn exponential(power: f32) -> Self {
        Self::Exponential(power)
    }

    /// Create a custom curve from control points
    pub fn custom(points: Vec<(f32, f32)>) -> Self {
        Self::Custom(points)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drumkit_creation() {
        let kit = DrumKit::new("test_kit", "Test Kit");
        assert_eq!(kit.name, "test_kit");
        assert_eq!(kit.display_name, "Test Kit");
        assert_eq!(kit.samples.len(), 0);
    }

    #[test]
    fn test_acoustic_kit_01() {
        let kit = DrumKit::acoustic_kit_01();
        assert_eq!(kit.name, "acoustic_kit_01");

        // Should have basic drum samples
        assert!(kit.get_sample(&ClickType::AcousticKick).is_some());
        assert!(kit.get_sample(&ClickType::AcousticSnare).is_some());
        assert!(kit.get_sample(&ClickType::HiHatClosed).is_some());

        let supported = kit.supported_click_types();
        assert!(supported.contains(&ClickType::AcousticKick));
    }

    #[test]
    fn test_drum_sample_velocity() {
        let metadata = SampleMetadata::new("test", "test.wav", 440.0);
        let sample = DrumSample::new(metadata, ClickType::AcousticKick)
            .with_velocity_curve(VelocityCurve::Exponential(2.0));

        // Test exponential curve
        assert_eq!(sample.apply_velocity(0.5), 0.25); // 0.5^2
        assert_eq!(sample.apply_velocity(1.0), 1.0);  // 1.0^2
    }

    #[test]
    fn test_velocity_curve_linear() {
        let curve = VelocityCurve::Linear;
        let metadata = SampleMetadata::new("test", "test.wav", 440.0);
        let sample = DrumSample::new(metadata, ClickType::AcousticKick)
            .with_velocity_curve(curve);

        assert_eq!(sample.apply_velocity(0.5), 0.5);
        assert_eq!(sample.apply_velocity(0.8), 0.8);
    }

    #[test]
    fn test_available_kits() {
        let kits = DrumKit::available_kits();
        assert!(!kits.is_empty());

        let kit_names: Vec<String> = kits.iter().map(|k| k.name.clone()).collect();
        assert!(kit_names.contains(&"acoustic_kit_01".to_string()));
    }
}