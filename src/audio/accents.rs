use crate::audio::synthesis::{AudioSynthesis, AudioSampleAdapter};
use crate::timing::ClickType;
/// Accent sound generation for metronomes and rhythm emphasis
///
/// This module provides specialized accent sound generation for musical
/// applications, particularly metronomes where accented beats need to be
/// clearly distinguishable from regular beats.
use crate::{AdsrEnvelope, SampleData, Waveform};

/// Accent sound generator for rhythmic emphasis
///
/// Generates accent sounds that are clearly distinguishable from regular
/// beats while maintaining musical coherence. The accent strategy varies
/// based on the base click type to ensure optimal contrast.
pub struct AccentSoundGenerator;

impl AccentSoundGenerator {
    /// Generate accent sound parameters for a given click type
    ///
    /// Returns (waveform, frequency, envelope) for accent sound that contrasts
    /// well with the base click type while maintaining musical quality.
    ///
    /// # Strategy
    /// - For drum samples: Use contrasting drum sound (kick→snare, snare→kick)
    /// - For synthetic sounds: Use different waveform with higher pitch
    pub fn get_accent_sound(
        click_type: ClickType,
        sample_adapter: &AudioSampleAdapter,
    ) -> (Waveform, f32, AdsrEnvelope) {
        match click_type {
            // For drum samples, use contrasting drum for accent
            ClickType::AcousticSnare
            | ClickType::HiHatClosed
            | ClickType::HiHatOpen
            | ClickType::RimShot
            | ClickType::Stick
            | ClickType::HiHatLoose
            | ClickType::HiHatVeryLoose
            | ClickType::CymbalSplash
            | ClickType::CymbalRoll
            | ClickType::Ride
            | ClickType::RideBell => {
                // Use kick drum for accent
                ClickType::AcousticKick.get_audio_params(sample_adapter)
            }
            // For kick drum variants, use snare for accent
            ClickType::AcousticKick | ClickType::KickTight => {
                ClickType::AcousticSnare.get_audio_params(sample_adapter)
            }
            // For synthetic sounds, create contrasting synthetic accent
            ClickType::WoodBlock => Self::get_synthetic_accent_for_woodblock(),
            ClickType::DigitalBeep => Self::get_synthetic_accent_for_beep(),
            ClickType::Cowbell => Self::get_synthetic_accent_for_cowbell(),
            ClickType::ElectroClick => Self::get_synthetic_accent_for_electro(),
        }
    }

    /// Generate synthetic accent sound for wood block base
    fn get_synthetic_accent_for_woodblock() -> (Waveform, f32, AdsrEnvelope) {
        (
            Waveform::Square, // Different waveform from noise
            1600.0,           // Higher pitch than base 800Hz
            AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 0.1, // Longer decay than base
                sustain_level: 0.0,
                release_secs: 0.05,
            },
        )
    }

    /// Generate synthetic accent sound for digital beep base
    fn get_synthetic_accent_for_beep() -> (Waveform, f32, AdsrEnvelope) {
        (
            Waveform::Square, // Different waveform from sine
            2000.0,           // Much higher pitch than base 1000Hz
            AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 0.12, // Longer decay
                sustain_level: 0.0,
                release_secs: 0.06,
            },
        )
    }

    /// Generate synthetic accent sound for cowbell base
    fn get_synthetic_accent_for_cowbell() -> (Waveform, f32, AdsrEnvelope) {
        (
            Waveform::Triangle, // Different waveform from square
            1600.0,             // Higher pitch than base 800Hz
            AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 0.2, // Longer decay
                sustain_level: 0.0,
                release_secs: 0.15,
            },
        )
    }

    /// Generate synthetic accent sound for electro click base
    fn get_synthetic_accent_for_electro() -> (Waveform, f32, AdsrEnvelope) {
        (
            Waveform::Sine, // Different waveform from pulse
            2400.0,         // Much higher pitch than base 1200Hz
            AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 0.06, // Longer decay
                sustain_level: 0.0,
                release_secs: 0.04,
            },
        )
    }

    /// Calculate accent volume multiplier for pattern-based accents
    ///
    /// Pattern accents need volume boost since they use same samples,
    /// unlike metronome which uses different sounds for accent.
    pub fn get_accent_volume_multiplier(base_volume: f32, accent_intensity: f32) -> f32 {
        (base_volume * (1.0 + accent_intensity)).min(1.0)
    }

    /// Get default accent intensity for different contexts
    pub fn get_default_accent_intensity(context: AccentContext) -> f32 {
        match context {
            AccentContext::Metronome => 0.0, // Different sound, no volume boost needed
            AccentContext::DrumPattern => 0.5, // 50% louder for pattern accents
            AccentContext::Practice => 0.3,  // Moderate accent for practice
            AccentContext::Performance => 0.7, // Strong accent for performance
        }
    }
}

/// Sample provider trait for accent generation
///
/// This trait abstracts the sample source for accent generation,
/// allowing it to work with different sample management systems.
pub trait AccentSampleProvider {
    fn get_sample(&self, click_type: &ClickType) -> Option<&SampleData>;
}

/// Context for accent generation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccentContext {
    /// Regular metronome with click sound contrast
    Metronome,
    /// Drum pattern where same samples need volume differentiation
    DrumPattern,
    /// Practice session with moderate emphasis
    Practice,
    /// Performance with strong emphasis
    Performance,
}

/// Get accent sound parameters for a click type
///
/// Returns audio parameters for accent beats, typically with enhanced volume
/// and brightness to emphasize the beat.
pub fn get_accent_sound(
    click_type: ClickType,
    sample_adapter: &AudioSampleAdapter,
) -> (Waveform, f32, AdsrEnvelope) {
    AccentSoundGenerator::get_accent_sound(click_type, sample_adapter)
}

/// Implementation for the new modular SampleManager

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::synthesis::AudioSampleAdapter;
    use std::collections::HashMap;

    #[test]
    fn test_accent_generation_for_drum_samples() {
        let samples: HashMap<ClickType, SampleData> = HashMap::new();
        let adapter = AudioSampleAdapter::new();

        // Test that snare gets kick accent
        let (waveform, freq, envelope) =
            AccentSoundGenerator::get_accent_sound(ClickType::AcousticSnare, &adapter);

        // Should fall back to synthetic kick since no samples available
        assert_eq!(freq, 60.0); // Synthetic kick frequency
    }

    #[test]
    fn test_accent_generation_for_synthetic_sounds() {
        let samples: HashMap<ClickType, SampleData> = HashMap::new();
        let adapter = AudioSampleAdapter::new();

        // Test wood block accent
        let (waveform, freq, envelope) =
            AccentSoundGenerator::get_accent_sound(ClickType::WoodBlock, &adapter);

        assert_eq!(freq, 1600.0); // Accent frequency
        assert!(matches!(waveform, Waveform::Square)); // Different waveform
    }

    #[test]
    fn test_accent_volume_calculation() {
        let base_volume = 0.7;
        let accent_intensity = 0.5;

        let accent_volume =
            AccentSoundGenerator::get_accent_volume_multiplier(base_volume, accent_intensity);

        assert_eq!(accent_volume, 1.0); // Should cap at 1.0
    }

    #[test]
    fn test_accent_context_intensities() {
        assert_eq!(
            AccentSoundGenerator::get_default_accent_intensity(AccentContext::Metronome),
            0.0
        );
        assert_eq!(
            AccentSoundGenerator::get_default_accent_intensity(AccentContext::DrumPattern),
            0.5
        );
    }
}
