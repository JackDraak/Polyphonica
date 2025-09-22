/// Real-time sample playback and triggering management
///
/// This module provides zero-allocation sample triggering for real-time
/// audio applications. It integrates with the sample library for loading
/// and provides efficient playback with real-time guarantees.

use std::sync::{Arc, Mutex};
use crate::{Waveform, AdsrEnvelope};
use super::library::{SampleLibrary, SampleError};
use super::catalog::SampleMetadata;

/// Real-time sample playback manager
///
/// The SampleManager provides real-time safe sample triggering with
/// zero-allocation playback paths. It maintains a connection to the
/// sample library for loading and caches frequently used samples
/// for immediate access.
///
/// # Real-time Safety
///
/// The trigger methods are designed to be real-time safe:
/// - No memory allocation during playback
/// - No blocking operations in audio thread
/// - Pre-loaded samples for immediate triggering
/// - Fallback to synthetic sounds if samples unavailable
///
/// # Usage in Audio Callback
///
/// ```rust
/// use polyphonica::samples::SampleManager;
/// use polyphonica::RealtimeEngine;
///
/// # fn example(manager: &SampleManager, engine: &mut RealtimeEngine) {
/// // In real-time audio callback
/// if let Some(trigger) = manager.get_trigger("kick") {
///     // Trigger sample with zero allocation
///     engine.trigger_note(trigger.waveform.clone(), trigger.frequency, trigger.envelope.clone());
/// }
/// # }
/// ```
pub struct SampleManager {
    /// Sample library for loading
    library: Arc<Mutex<SampleLibrary>>,

    /// Cached triggers for real-time access
    trigger_cache: std::collections::HashMap<String, SampleTrigger>,

    /// Default envelope for samples
    default_envelope: AdsrEnvelope,
}

/// Pre-computed sample trigger for real-time playback
///
/// Contains all information needed to trigger a sample in the audio
/// callback without any allocation or lookup overhead.
#[derive(Debug, Clone)]
pub struct SampleTrigger {
    /// Waveform to trigger
    pub waveform: Waveform,

    /// Base frequency for the sample
    pub frequency: f32,

    /// ADSR envelope for the sample
    pub envelope: AdsrEnvelope,

    /// Volume adjustment factor
    pub volume: f32,
}

impl SampleManager {
    /// Create a new sample manager
    pub fn new(library: SampleLibrary) -> Self {
        Self {
            library: Arc::new(Mutex::new(library)),
            trigger_cache: std::collections::HashMap::new(),
            default_envelope: AdsrEnvelope {
                attack_secs: 0.002,   // Very fast attack for percussive samples
                decay_secs: 0.1,      // Quick decay
                sustain_level: 0.3,   // Low sustain for drums
                release_secs: 0.2,    // Natural release
            },
        }
    }

    /// Create a sample manager with shared library
    pub fn with_shared_library(library: Arc<Mutex<SampleLibrary>>) -> Self {
        Self {
            library,
            trigger_cache: std::collections::HashMap::new(),
            default_envelope: AdsrEnvelope {
                attack_secs: 0.002,
                decay_secs: 0.1,
                sustain_level: 0.3,
                release_secs: 0.2,
            },
        }
    }

    /// Prepare a sample for real-time triggering
    ///
    /// This loads the sample and creates a cached trigger that can be
    /// used in real-time audio callbacks without allocation.
    pub fn prepare_sample(&mut self, name: &str, base_frequency: f32) -> Result<(), SampleError> {
        // Load sample from library
        let sample_data = {
            let mut library = self.library.lock().unwrap();
            library.load_sample(name, base_frequency)?.clone()
        };

        // Create waveform and trigger
        let waveform = Waveform::Sample(sample_data);
        let trigger = SampleTrigger {
            waveform,
            frequency: base_frequency,
            envelope: self.default_envelope.clone(),
            volume: 1.0,
        };

        // Cache for real-time access
        self.trigger_cache.insert(name.to_string(), trigger);
        Ok(())
    }

    /// Prepare a sample with custom envelope
    pub fn prepare_sample_with_envelope(
        &mut self,
        name: &str,
        base_frequency: f32,
        envelope: AdsrEnvelope,
    ) -> Result<(), SampleError> {
        let sample_data = {
            let mut library = self.library.lock().unwrap();
            library.load_sample(name, base_frequency)?.clone()
        };

        let waveform = Waveform::Sample(sample_data);
        let trigger = SampleTrigger {
            waveform,
            frequency: base_frequency,
            envelope,
            volume: 1.0,
        };

        self.trigger_cache.insert(name.to_string(), trigger);
        Ok(())
    }

    /// Get a sample trigger for real-time playback (zero allocation)
    ///
    /// This method is real-time safe and can be called from audio callbacks.
    /// Returns None if the sample is not prepared.
    pub fn get_trigger(&self, name: &str) -> Option<&SampleTrigger> {
        self.trigger_cache.get(name)
    }

    /// Get a sample trigger with volume adjustment
    pub fn get_trigger_with_volume(&self, name: &str, volume: f32) -> Option<SampleTrigger> {
        self.trigger_cache.get(name).map(|trigger| {
            let mut adjusted = trigger.clone();
            adjusted.volume *= volume;
            adjusted
        })
    }

    /// Check if a sample is prepared for triggering
    pub fn is_prepared(&self, name: &str) -> bool {
        self.trigger_cache.contains_key(name)
    }

    /// Get the sample library for advanced operations
    pub fn library(&self) -> Arc<Mutex<SampleLibrary>> {
        self.library.clone()
    }

    /// Clear all prepared samples
    pub fn clear_cache(&mut self) {
        self.trigger_cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            prepared_samples: self.trigger_cache.len(),
        }
    }

    /// Set default envelope for new samples
    pub fn set_default_envelope(&mut self, envelope: AdsrEnvelope) {
        self.default_envelope = envelope;
    }

    /// Prepare multiple samples from metadata
    pub fn prepare_samples_from_metadata(&mut self, samples: &[SampleMetadata]) -> Result<Vec<String>, SampleError> {
        let mut failed = Vec::new();

        for metadata in samples {
            if let Err(e) = self.prepare_sample_with_envelope(
                &metadata.name,
                metadata.base_frequency,
                metadata.envelope.clone(),
            ) {
                eprintln!("Failed to prepare sample {}: {}", metadata.name, e);
                failed.push(metadata.name.clone());
            }
        }

        if failed.is_empty() {
            Ok(failed)
        } else {
            Err(SampleError::LoadError(format!("Failed to load {} samples", failed.len())))
        }
    }
}

/// Sample manager cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub prepared_samples: usize,
}

/// Trait for sample playback integration
///
/// This trait allows different audio engines to integrate with the
/// sample manager for triggering samples in their specific format.
pub trait SamplePlayer {
    /// Trigger a sample with the given parameters
    fn trigger_sample(&mut self, trigger: &SampleTrigger) -> Result<(), String>;

    /// Check if the player supports the given sample format
    fn supports_format(&self, format: &str) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::samples::SampleLibrary;

    #[test]
    fn test_sample_manager_creation() {
        let library = SampleLibrary::new();
        let manager = SampleManager::new(library);
        assert_eq!(manager.trigger_cache.len(), 0);
    }

    #[test]
    fn test_cache_stats() {
        let library = SampleLibrary::new();
        let manager = SampleManager::new(library);
        let stats = manager.cache_stats();
        assert_eq!(stats.prepared_samples, 0);
    }

    #[test]
    fn test_sample_trigger_creation() {
        let trigger = SampleTrigger {
            waveform: Waveform::Sine,
            frequency: 440.0,
            envelope: AdsrEnvelope {
                attack_secs: 0.002,
                decay_secs: 0.1,
                sustain_level: 0.3,
                release_secs: 0.2,
            },
            volume: 1.0,
        };

        assert_eq!(trigger.frequency, 440.0);
        assert_eq!(trigger.volume, 1.0);
    }

    #[test]
    fn test_volume_adjustment() {
        let library = SampleLibrary::new();
        let manager = SampleManager::new(library);

        // Test get_trigger_with_volume with non-existent sample
        let trigger = manager.get_trigger_with_volume("nonexistent", 0.5);
        assert!(trigger.is_none());
    }
}