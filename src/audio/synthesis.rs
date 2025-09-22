use crate::timing::ClickType;
/// Audio synthesis and parameter generation for click types
///
/// This module provides the core audio synthesis capabilities extracted from
/// the monolithic guitar_buddy.rs implementation. It handles waveform generation,
/// envelope configuration, and audio parameter mapping for different click types.
use crate::{AdsrEnvelope, SampleData, Waveform};
use std::collections::HashMap;

/// Audio synthesis capabilities for click types
///
/// This trait provides a clean interface for generating audio parameters
/// for different click types, supporting both sample-based and synthetic sounds.
pub trait AudioSynthesis {
    /// Get complete audio parameters for this click type with legacy sample manager
    ///
    /// Returns (waveform, frequency, envelope) tuple suitable for real-time synthesis.
    /// Automatically chooses between sample-based and synthetic sounds based on availability.
    fn get_legacy_audio_params(
        &self,
        sample_adapter: &LegacySampleAdapter,
    ) -> (Waveform, f32, AdsrEnvelope);

    /// Get ADSR envelope parameters for sample-based sounds
    ///
    /// Returns envelope parameters optimized for natural sample playback,
    /// typically with minimal shaping to preserve sample character.
    fn get_sample_envelope(&self) -> AdsrEnvelope;

    /// Get synthetic sound parameters as fallback
    ///
    /// Returns (waveform, frequency, envelope) for procedural sound generation
    /// when sample-based sounds are not available.
    fn get_synthetic_params(&self) -> (Waveform, f32, AdsrEnvelope);
}

impl AudioSynthesis for ClickType {
    /// Generate the waveform and parameters for this click type (legacy compatibility)
    fn get_legacy_audio_params(
        &self,
        sample_adapter: &LegacySampleAdapter,
    ) -> (Waveform, f32, AdsrEnvelope) {
        // Check if we have a sample for this click type
        if let Some(sample_data) = sample_adapter.get_sample(self) {
            return (
                Waveform::DrumSample(sample_data.clone()),
                440.0, // Frequency is ignored for drum samples
                self.get_sample_envelope(),
            );
        }

        // Fall back to synthetic sound
        self.get_synthetic_params()
    }

    /// Get ADSR envelope for sample-based sounds
    /// For drums, we use minimal envelope shaping to preserve natural character
    fn get_sample_envelope(&self) -> AdsrEnvelope {
        match self {
            ClickType::AcousticKick => AdsrEnvelope {
                attack_secs: 0.001,  // Instant attack
                decay_secs: 1.0,     // Let natural sample decay
                sustain_level: 0.0,  // No sustain - one-shot sample
                release_secs: 0.001, // Minimal release
            },
            ClickType::AcousticSnare => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 0.5, // Let natural snare ring
                sustain_level: 0.0,
                release_secs: 0.001,
            },
            ClickType::HiHatClosed => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 0.2, // Natural hi-hat decay
                sustain_level: 0.0,
                release_secs: 0.001,
            },
            ClickType::HiHatOpen => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 1.0, // Let open hi-hat ring naturally
                sustain_level: 0.0,
                release_secs: 0.001,
            },
            ClickType::RimShot => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 0.3, // Natural rim shot decay
                sustain_level: 0.0,
                release_secs: 0.001,
            },
            ClickType::Stick => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 0.1, // Short stick click
                sustain_level: 0.0,
                release_secs: 0.001,
            },
            // Extended drum kit samples
            ClickType::KickTight => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 0.8, // Slightly shorter than regular kick
                sustain_level: 0.0,
                release_secs: 0.001,
            },
            ClickType::HiHatLoose => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 0.5, // Medium decay for loose hi-hat
                sustain_level: 0.0,
                release_secs: 0.001,
            },
            ClickType::HiHatVeryLoose => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 1.2, // Longer decay for very loose
                sustain_level: 0.0,
                release_secs: 0.001,
            },
            ClickType::CymbalSplash => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 1.5, // Long splash decay
                sustain_level: 0.0,
                release_secs: 0.001,
            },
            ClickType::CymbalRoll => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 2.0, // Extended roll decay
                sustain_level: 0.0,
                release_secs: 0.001,
            },
            ClickType::Ride => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 0.8, // Ride cymbal sustain
                sustain_level: 0.0,
                release_secs: 0.001,
            },
            ClickType::RideBell => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 0.3, // Short bell ping
                sustain_level: 0.0,
                release_secs: 0.001,
            },
            // For synthetic sounds, use default
            _ => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 0.1,
                sustain_level: 0.0,
                release_secs: 0.05,
            },
        }
    }

    /// Get synthetic sound parameters (fallback when no sample available)
    fn get_synthetic_params(&self) -> (Waveform, f32, AdsrEnvelope) {
        match self {
            ClickType::WoodBlock => (
                Waveform::Noise,
                800.0, // High frequency for sharp click
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.05,
                    sustain_level: 0.0,
                    release_secs: 0.02,
                },
            ),
            ClickType::DigitalBeep => (
                Waveform::Sine,
                1000.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.08,
                    sustain_level: 0.0,
                    release_secs: 0.05,
                },
            ),
            ClickType::Cowbell => (
                Waveform::Square,
                800.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.15,
                    sustain_level: 0.0,
                    release_secs: 0.1,
                },
            ),
            ClickType::RimShot => (
                Waveform::Pulse { duty_cycle: 0.1 },
                400.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.03,
                    sustain_level: 0.0,
                    release_secs: 0.02,
                },
            ),
            ClickType::Stick => (
                Waveform::Triangle,
                2000.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.02,
                    sustain_level: 0.0,
                    release_secs: 0.01,
                },
            ),
            ClickType::ElectroClick => (
                Waveform::Pulse { duty_cycle: 0.25 },
                1200.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.04,
                    sustain_level: 0.0,
                    release_secs: 0.03,
                },
            ),
            // For drum samples without sample data, provide synthetic alternatives
            ClickType::AcousticKick => (
                Waveform::Sine,
                60.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.3,
                    sustain_level: 0.0,
                    release_secs: 0.1,
                },
            ),
            ClickType::AcousticSnare => (
                Waveform::Noise,
                800.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.15,
                    sustain_level: 0.0,
                    release_secs: 0.05,
                },
            ),
            ClickType::HiHatClosed => (
                Waveform::Pulse { duty_cycle: 0.1 },
                8000.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.08,
                    sustain_level: 0.0,
                    release_secs: 0.02,
                },
            ),
            ClickType::HiHatOpen => (
                Waveform::Pulse { duty_cycle: 0.1 },
                6000.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.25,
                    sustain_level: 0.0,
                    release_secs: 0.1,
                },
            ),
            // Extended drum kit samples - synthetic fallbacks
            ClickType::KickTight => (
                Waveform::Sine,
                80.0, // Slightly higher than regular kick
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.2, // Shorter decay for tight kick
                    sustain_level: 0.0,
                    release_secs: 0.05,
                },
            ),
            ClickType::HiHatLoose => (
                Waveform::Pulse { duty_cycle: 0.2 },
                5000.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.4, // Medium decay
                    sustain_level: 0.0,
                    release_secs: 0.15,
                },
            ),
            ClickType::HiHatVeryLoose => (
                Waveform::Pulse { duty_cycle: 0.3 },
                4000.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.8, // Long decay
                    sustain_level: 0.0,
                    release_secs: 0.3,
                },
            ),
            ClickType::CymbalSplash => (
                Waveform::Noise,
                4000.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 1.0, // Splash decay
                    sustain_level: 0.0,
                    release_secs: 0.4,
                },
            ),
            ClickType::CymbalRoll => (
                Waveform::Noise,
                3000.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 1.5, // Extended roll
                    sustain_level: 0.0,
                    release_secs: 0.6,
                },
            ),
            ClickType::Ride => (
                Waveform::Triangle,
                2000.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.5, // Ride sustain
                    sustain_level: 0.0,
                    release_secs: 0.2,
                },
            ),
            ClickType::RideBell => (
                Waveform::Sine,
                3000.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.3, // Bell ping
                    sustain_level: 0.0,
                    release_secs: 0.1,
                },
            ),
        }
    }
}

/// Legacy compatibility wrapper for transitioning from DrumSampleManager
///
/// This provides a temporary bridge while we migrate from the legacy DrumSampleManager
/// to the modular SampleManager system.
pub struct LegacySampleAdapter {
    samples: HashMap<ClickType, SampleData>,
}

impl Default for LegacySampleAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl LegacySampleAdapter {
    pub fn new() -> Self {
        Self {
            samples: HashMap::new(),
        }
    }

    pub fn load_drum_samples(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Load acoustic drum kit samples using relative paths from project root
        let sample_paths = vec![
            (
                ClickType::AcousticKick,
                "samples/drums/acoustic/kit_01/drumkit-kick.wav",
            ),
            (
                ClickType::AcousticSnare,
                "samples/drums/acoustic/kit_01/drumkit-snare.wav",
            ),
            (
                ClickType::HiHatClosed,
                "samples/drums/acoustic/kit_01/drumkit-hihat.wav",
            ),
            (
                ClickType::HiHatOpen,
                "samples/drums/acoustic/kit_01/drumkit-hihat-open.wav",
            ),
            (
                ClickType::RimShot,
                "samples/drums/acoustic/kit_01/drumkit-rimshot.wav",
            ),
            (
                ClickType::Stick,
                "samples/drums/acoustic/kit_01/drumkit-stick.wav",
            ),
            // Extended drum kit samples - map to available samples
            (
                ClickType::KickTight,
                "samples/drums/acoustic/kit_01/drumkit-kick.wav",
            ),
            (
                ClickType::HiHatLoose,
                "samples/drums/acoustic/kit_01/drumkit-hihat-open.wav",
            ),
            (
                ClickType::HiHatVeryLoose,
                "samples/drums/acoustic/kit_01/drumkit-hihat-open.wav",
            ),
            (
                ClickType::CymbalSplash,
                "samples/drums/acoustic/kit_01/drumkit-hihat-open.wav",
            ),
            (
                ClickType::CymbalRoll,
                "samples/drums/acoustic/kit_01/drumkit-hihat-open.wav",
            ),
            (
                ClickType::Ride,
                "samples/drums/acoustic/kit_01/drumkit-hihat.wav",
            ),
            (
                ClickType::RideBell,
                "samples/drums/acoustic/kit_01/drumkit-stick.wav",
            ),
        ];

        for (click_type, path) in sample_paths {
            match SampleData::from_file(path, 440.0) {
                Ok(sample_data) => {
                    println!("✅ Loaded drum sample: {} from {}", click_type.name(), path);
                    self.samples.insert(click_type, sample_data);
                }
                Err(e) => {
                    println!(
                        "⚠️  Could not load {}: {} (falling back to synthetic)",
                        path, e
                    );
                    // Continue without the sample - will fall back to synthetic sound
                }
            }
        }

        Ok(())
    }

    pub fn get_sample(&self, click_type: &ClickType) -> Option<&SampleData> {
        self.samples.get(click_type)
    }
}

/// Legacy audio parameters function for transitioning guitar_buddy.rs
///
/// This provides the same interface as the old ClickTypeAudioExt::get_sound_params
/// method to ease the transition to the modular audio system.
pub fn get_legacy_sound_params(
    click_type: ClickType,
    sample_adapter: &LegacySampleAdapter,
) -> (Waveform, f32, AdsrEnvelope) {
    click_type.get_legacy_audio_params(sample_adapter)
}
