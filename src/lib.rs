//! # Polyphonica - Real-time Audio Synthesis and Pattern Engine
//!
//! Polyphonica is a comprehensive Rust library for real-time audio synthesis, musical timing,
//! and pattern-based music generation. Originally developed as the audio engine for Guitar Buddy,
//! it provides modular, high-performance components for musical applications.
//!
//! ## Project Status: Prototype/Development
//!
//! This library is currently in active development. Core functionality is stable and tested,
//! but APIs may change between versions. Use in production environments at your own discretion.
//!
//! ## Core Features
//!
//! - **Real-time Audio Synthesis**: Polyphonic synthesis engine with multiple waveforms
//! - **Precision Musical Timing**: <1ms beat accuracy with discrete scheduling
//! - **Pattern Management**: Comprehensive drum pattern library with 6+ genres
//! - **Sample Management**: Efficient audio sample loading and caching
//! - **Modular Architecture**: 13 focused modules for maintainable code
//!
//! ## Quick Start
//!
//! ```rust
//! use polyphonica::{RealtimeEngine, Waveform, AdsrEnvelope};
//!
//! // Create a real-time synthesis engine
//! let mut engine = RealtimeEngine::new(44100.0);
//!
//! // Define an envelope
//! let envelope = AdsrEnvelope {
//!     attack_secs: 0.1,
//!     decay_secs: 0.1,
//!     sustain_level: 0.7,
//!     release_secs: 0.3,
//! };
//!
//! // Trigger a note
//! let voice_id = engine.trigger_note(Waveform::Sine, 440.0, envelope);
//!
//! // Process audio in your audio callback
//! let mut buffer = vec![0.0; 1024];
//! engine.process_buffer(&mut buffer);
//! ```
//!
//! ## Module Organization
//!
//! ### Core Audio Engine (`lib.rs`)
//! - Real-time polyphonic synthesis engine
//! - Waveform generation (sine, square, sawtooth, triangle, pulse, noise, samples)
//! - ADSR envelope processing
//! - Audio timeline rendering
//! - Voice management and allocation
//!
//! ### Timing System (`timing::`)
//! - High-precision beat timing with <1ms accuracy
//! - Discrete scheduling to prevent drift
//! - Metronome and pattern player implementations
//! - Beat event tracking and observation
//!
//! ### Pattern Management (`patterns::`)
//! - Comprehensive drum pattern library
//! - Pattern builders with fluent API
//! - Genre-specific collections (Rock, Jazz, Latin, Funk, Pop, Electronic)
//! - JSON import/export capabilities
//! - Real-time pattern state management
//!
//! ### Sample Management (`samples::`)
//! - Efficient WAV file loading and caching
//! - LRU cache with configurable memory limits
//! - Zero-allocation real-time sample playback
//! - DrumKit collections with velocity curves
//! - Sample metadata and catalog management
//!
//! ### Audio Processing (`audio::`)
//! - Audio streaming and device management
//! - Buffer format conversion utilities
//! - Audio effects and processing chains (planned)
//!
//! ### Visualization (`visualization::`)
//! - Beat visualization components
//! - Real-time audio spectrum analysis (planned)
//! - Waveform display utilities (planned)
//!
//! ### Configuration (`config::`)
//! - Application settings and preferences
//! - JSON-based configuration persistence
//! - Audio device configuration
//!
//! ## Performance Characteristics
//!
//! - **Voice Polyphony**: Up to 32 simultaneous voices
//! - **Timing Precision**: <1ms beat-to-beat consistency
//! - **Memory Management**: Zero-allocation audio processing
//! - **Sample Rates**: Supports 8kHz to 192kHz
//! - **Thread Safety**: Lock-free audio processing paths
//!
//! ## Audio Formats Supported
//!
//! - **Generation**: All waveforms generated at any frequency
//! - **Sample Loading**: WAV files (16/24/32-bit, mono/stereo)
//! - **Output**: f32 samples in [-1.0, 1.0] range
//! - **Real-time**: CPAL-compatible buffer processing
//!
//! ## Current Limitations
//!
//! - Sample loading limited to WAV format only
//! - No built-in audio effects or filtering
//! - Pattern system doesn't support real-time editing during playback
//! - Visualization module is minimal (mainly beat indicators)
//! - No MIDI input/output support
//! - Configuration system is basic
//!
//! ## Examples and Applications
//!
//! See the `src/bin/` directory for complete examples:
//! - `guitar-buddy`: Full GUI metronome and drum machine
//! - `pattern-export`: Pattern library export utility
//! - Various test applications demonstrating specific features
//!

use std::f32::consts::PI;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

fn validate_inputs(
    frequency: f32,
    duration_secs: f32,
    sample_rate: u32,
) -> Result<(), &'static str> {
    if frequency <= 0.0 || frequency > 20000.0 {
        return Err("Frequency must be between 0 and 20000 Hz");
    }
    if duration_secs < 0.0 {
        return Err("Duration must be non-negative");
    }
    if sample_rate == 0 || sample_rate > 192000 {
        return Err("Sample rate must be between 1 and 192000 Hz");
    }
    Ok(())
}

fn generate_sample(waveform: &Waveform, phase: f32, time_secs: f32, target_frequency: f32) -> f32 {
    match waveform {
        Waveform::Sine => phase.sin(),
        Waveform::Square => {
            if (phase % (2.0 * PI)).sin() >= 0.0 {
                1.0
            } else {
                -1.0
            }
        }
        Waveform::Sawtooth => {
            let normalized_phase = (phase / (2.0 * PI)) % 1.0;
            2.0 * normalized_phase - 1.0
        }
        Waveform::Triangle => {
            let normalized_phase = (phase / (2.0 * PI)) % 1.0;
            if normalized_phase < 0.5 {
                4.0 * normalized_phase - 1.0
            } else {
                3.0 - 4.0 * normalized_phase
            }
        }
        Waveform::Pulse { duty_cycle } => {
            let normalized_phase = (phase / (2.0 * PI)) % 1.0;
            if normalized_phase < *duty_cycle {
                1.0
            } else {
                -1.0
            }
        }
        Waveform::Noise => {
            // Linear congruential generator for deterministic noise
            let seed = ((time_secs * 1000.0) as u32)
                .wrapping_mul(1103515245)
                .wrapping_add(12345);
            
            (seed % 32768) as f32 / 16384.0 - 1.0
        }
        Waveform::Sample(sample_data) => {
            sample_data.get_sample_at_time(time_secs, target_frequency)
        }
        Waveform::DrumSample(sample_data) => sample_data.get_natural_sample_at_time(time_secs),
    }
}

/// Waveform types supported by the synthesis engine
///
/// This enum defines all waveform types that can be generated or played back
/// by the real-time synthesis engine. Each waveform has different characteristics
/// suitable for different musical applications.
///
/// # Examples
///
/// ```rust
/// use polyphonica::Waveform;
///
/// // Basic waveforms
/// let sine = Waveform::Sine;
/// let square = Waveform::Square;
///
/// // Pulse wave with 25% duty cycle
/// let pulse = Waveform::Pulse { duty_cycle: 0.25 };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Waveform {
    /// Pure sine wave - smooth, fundamental tone
    Sine,
    /// Square wave - rich in odd harmonics, classic electronic sound
    Square,
    /// Sawtooth wave - rich in all harmonics, classic synth lead sound
    Sawtooth,
    /// Triangle wave - similar to sine but with more harmonics
    Triangle,
    /// Pulse wave with configurable duty cycle
    ///
    /// `duty_cycle` should be between 0.0 and 1.0, where 0.5 creates a square wave
    Pulse {
        /// Duty cycle (0.0 to 1.0) - fraction of cycle that is high
        duty_cycle: f32,
    },
    /// White noise - random values for percussion and sound effects
    Noise,
    /// Audio sample with pitch shifting capability
    ///
    /// Used for melodic instruments where pitch shifting is desired
    Sample(SampleData),
    /// Drum sample played at natural speed without pitch shifting
    ///
    /// Optimized for percussive sounds where natural timbre is important
    DrumSample(SampleData),
}

/// Audio sample data container
///
/// Stores audio sample data along with metadata required for playback.
/// Supports both looped and one-shot playback modes, with configurable
/// pitch shifting capabilities.
///
/// # Examples
///
/// ```rust
/// use polyphonica::SampleData;
///
/// # fn example() -> Result<(), polyphonica::SampleError> {
/// // Load a sample from a WAV file
/// let sample_data = SampleData::from_file("kick.wav", 60.0)?;
///
/// // Add loop points for sustained playback
/// let looped_sample = sample_data.with_loop_points(1000, 5000)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct SampleData {
    /// Raw audio samples as f32 values in [-1.0, 1.0] range
    pub samples: Vec<f32>,
    /// Sample rate in Hz (e.g., 44100)
    pub sample_rate: u32,
    /// Base frequency for pitch shifting (Hz)
    pub base_frequency: f32,
    /// Loop start point in samples (None for one-shot playback)
    pub loop_start: Option<usize>,
    /// Loop end point in samples (None for one-shot playback)
    pub loop_end: Option<usize>,
    /// Sample metadata and file information
    pub metadata: SampleMetadata,
}

/// Metadata associated with audio samples
///
/// Contains file information and audio characteristics for loaded samples.
/// This metadata is preserved when samples are loaded and can be used for
/// display purposes or audio processing decisions.
#[derive(Debug, Clone, PartialEq)]
pub struct SampleMetadata {
    /// Original filename (without path)
    pub filename: String,
    /// Duration of the sample in seconds
    pub duration_secs: f32,
    /// Number of audio channels (1 for mono, 2 for stereo)
    pub channels: u16,
    /// Bit depth of original file (16, 24, or 32)
    pub bits_per_sample: u16,
}

/// Errors that can occur during sample loading and processing
///
/// This enum covers all error conditions that may arise when working with
/// audio samples, from file I/O issues to format compatibility problems.
#[derive(Debug)]
pub enum SampleError {
    /// File system or I/O related error
    IoError(std::io::Error),
    /// Invalid or corrupted audio file format
    FormatError(String),
    /// Audio format not supported by the engine
    UnsupportedFormat(String),
}

impl std::fmt::Display for SampleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SampleError::IoError(e) => write!(f, "IO error: {}", e),
            SampleError::FormatError(msg) => write!(f, "Format error: {}", msg),
            SampleError::UnsupportedFormat(msg) => write!(f, "Unsupported format: {}", msg),
        }
    }
}

impl std::error::Error for SampleError {}

impl From<std::io::Error> for SampleError {
    fn from(error: std::io::Error) -> Self {
        SampleError::IoError(error)
    }
}

impl From<hound::Error> for SampleError {
    fn from(error: hound::Error) -> Self {
        SampleError::FormatError(format!("WAV error: {}", error))
    }
}

impl SampleData {
    /// Load a WAV file as sample data
    pub fn from_file<P: AsRef<Path>>(path: P, base_frequency: f32) -> Result<Self, SampleError> {
        let path = path.as_ref();
        let mut reader = hound::WavReader::open(path)?;

        let spec = reader.spec();

        // Validate format
        if base_frequency <= 0.0 || base_frequency > 20000.0 {
            return Err(SampleError::FormatError(
                "Base frequency must be between 0 and 20000 Hz".to_string(),
            ));
        }

        // Read samples and convert to f32
        let samples: Result<Vec<f32>, _> = match spec.sample_format {
            hound::SampleFormat::Float => reader.samples::<f32>().collect(),
            hound::SampleFormat::Int => match spec.bits_per_sample {
                16 => reader
                    .samples::<i16>()
                    .map(|s| s.map(|sample| sample as f32 / i16::MAX as f32))
                    .collect(),
                24 => reader
                    .samples::<i32>()
                    .map(|s| s.map(|sample| (sample >> 8) as f32 / i32::MAX as f32))
                    .collect(),
                32 => reader
                    .samples::<i32>()
                    .map(|s| s.map(|sample| sample as f32 / i32::MAX as f32))
                    .collect(),
                _ => {
                    return Err(SampleError::UnsupportedFormat(format!(
                        "Unsupported bit depth: {}",
                        spec.bits_per_sample
                    )))
                }
            },
        };

        let mut samples = samples.map_err(|e| SampleError::FormatError(e.to_string()))?;

        // Convert stereo to mono by averaging channels
        if spec.channels == 2 {
            let mono_samples: Vec<f32> = samples
                .chunks_exact(2)
                .map(|stereo| (stereo[0] + stereo[1]) / 2.0)
                .collect();
            samples = mono_samples;
        } else if spec.channels > 2 {
            return Err(SampleError::UnsupportedFormat(format!(
                "Unsupported channel count: {}",
                spec.channels
            )));
        }

        let duration_secs = samples.len() as f32 / spec.sample_rate as f32;

        let metadata = SampleMetadata {
            filename: path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            duration_secs,
            channels: spec.channels,
            bits_per_sample: spec.bits_per_sample,
        };

        Ok(SampleData {
            samples,
            sample_rate: spec.sample_rate,
            base_frequency,
            loop_start: None,
            loop_end: None,
            metadata,
        })
    }

    /// Add loop points to the sample for sustained playback
    pub fn with_loop_points(mut self, start: usize, end: usize) -> Result<Self, SampleError> {
        if start >= self.samples.len() || end >= self.samples.len() || start >= end {
            return Err(SampleError::FormatError(
                "Invalid loop points: must be within sample bounds and start < end".to_string(),
            ));
        }

        self.loop_start = Some(start);
        self.loop_end = Some(end);
        Ok(self)
    }

    /// Get a sample at a specific time position with pitch shifting
    /// Get sample at natural playback speed (no pitch shifting) - ideal for drums
    pub fn get_natural_sample_at_time(&self, time_secs: f32) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }

        // Play at natural speed - no frequency-based pitch shifting
        let sample_pos = time_secs * self.sample_rate as f32;

        // Handle looping
        let (loop_start, loop_end) = match (self.loop_start, self.loop_end) {
            (Some(start), Some(end)) => (start as f32, end as f32),
            _ => (0.0, self.samples.len() as f32),
        };

        let effective_pos = if sample_pos >= loop_end {
            // For drums, often we want one-shot playback, so return 0 after sample ends
            if self.loop_start.is_none() && self.loop_end.is_none() {
                return 0.0; // Sample has ended naturally
            }
            // Loop back to start
            let loop_length = loop_end - loop_start;
            let overflow = sample_pos - loop_end;
            loop_start + (overflow % loop_length)
        } else {
            sample_pos
        };

        // Linear interpolation between samples
        let floor_pos = effective_pos.floor() as usize;
        let ceil_pos = (floor_pos + 1).min(self.samples.len() - 1);
        let fraction = effective_pos - floor_pos as f32;

        if floor_pos >= self.samples.len() {
            return 0.0;
        }

        let sample_low = self.samples[floor_pos];
        let sample_high = self.samples[ceil_pos];

        sample_low + fraction * (sample_high - sample_low)
    }

    pub fn get_sample_at_time(&self, time_secs: f32, target_frequency: f32) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }

        // Calculate playback speed based on frequency ratio
        let speed_ratio = target_frequency / self.base_frequency;
        let sample_pos = time_secs * self.sample_rate as f32 * speed_ratio;

        // Handle looping
        let (loop_start, loop_end) = match (self.loop_start, self.loop_end) {
            (Some(start), Some(end)) => (start as f32, end as f32),
            _ => (0.0, self.samples.len() as f32),
        };

        let effective_pos = if sample_pos >= loop_end {
            // Loop back to start
            let loop_length = loop_end - loop_start;
            let overflow = sample_pos - loop_end;
            loop_start + (overflow % loop_length)
        } else {
            sample_pos
        };

        // Bounds check
        if effective_pos < 0.0 || effective_pos >= self.samples.len() as f32 {
            return 0.0;
        }

        // Linear interpolation between samples
        let index = effective_pos as usize;
        let fraction = effective_pos - index as f32;

        if index + 1 >= self.samples.len() {
            self.samples[index]
        } else {
            let sample1 = self.samples[index];
            let sample2 = self.samples[index + 1];
            sample1 + (sample2 - sample1) * fraction
        }
    }
}

/// ADSR (Attack, Decay, Sustain, Release) envelope definition
///
/// Defines the amplitude envelope shape for audio synthesis. ADSR envelopes
/// are fundamental to creating natural-sounding musical instruments and effects.
///
/// # Envelope Phases
///
/// 1. **Attack**: Time to reach peak amplitude from zero
/// 2. **Decay**: Time to fall from peak to sustain level
/// 3. **Sustain**: Constant amplitude level while note is held
/// 4. **Release**: Time to fade to zero after note is released
///
/// # Examples
///
/// ```rust
/// use polyphonica::AdsrEnvelope;
///
/// // Piano-like envelope (quick attack, gradual decay)
/// let piano = AdsrEnvelope {
///     attack_secs: 0.01,
///     decay_secs: 0.3,
///     sustain_level: 0.4,
///     release_secs: 0.8,
/// };
///
/// // Organ-like envelope (no decay, full sustain)
/// let organ = AdsrEnvelope {
///     attack_secs: 0.1,
///     decay_secs: 0.0,
///     sustain_level: 1.0,
///     release_secs: 0.2,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AdsrEnvelope {
    /// Time in seconds to reach peak amplitude from zero
    pub attack_secs: f32,
    /// Time in seconds to decay from peak to sustain level
    pub decay_secs: f32,
    /// Sustain amplitude level (0.0 to 1.0)
    pub sustain_level: f32,
    /// Time in seconds to fade from sustain level to zero
    pub release_secs: f32,
}

/// A scheduled sound event with waveform, frequency sweep, and envelope
///
/// Represents a single audio event that can be rendered to a timeline.
/// Supports frequency sweeps (glissando) and complex envelope shaping.
/// Used primarily for non-real-time audio generation and composition.
///
/// # Examples
///
/// ```rust
/// use polyphonica::{SoundEvent, Waveform, AdsrEnvelope};
///
/// // Simple constant-frequency tone
/// let beep = SoundEvent {
///     waveform: Waveform::Sine,
///     start_frequency: 440.0,
///     end_frequency: 440.0,  // Same as start = no sweep
///     duration_secs: 0.5,
///     envelope: AdsrEnvelope {
///         attack_secs: 0.1,
///         decay_secs: 0.1,
///         sustain_level: 0.7,
///         release_secs: 0.3,
///     },
/// };
///
/// // Frequency sweep (glissando)
/// let sweep = SoundEvent {
///     waveform: Waveform::Sawtooth,
///     start_frequency: 220.0,
///     end_frequency: 880.0,  // Octave sweep
///     duration_secs: 2.0,
///     envelope: beep.envelope.clone(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct SoundEvent {
    /// Waveform type to generate
    pub waveform: Waveform,
    /// Starting frequency in Hz
    pub start_frequency: f32,
    /// Ending frequency in Hz (can be same as start for constant pitch)
    pub end_frequency: f32,
    /// Duration of the event in seconds
    pub duration_secs: f32,
    /// Amplitude envelope to apply
    pub envelope: AdsrEnvelope,
}

/// Generate audio samples for a single waveform
///
/// Creates a buffer of audio samples for the specified waveform, frequency, and duration.
/// This is a utility function for non-real-time audio generation. For real-time synthesis,
/// use the `RealtimeEngine` instead.
///
/// # Parameters
///
/// - `waveform`: The type of waveform to generate
/// - `frequency`: Frequency in Hz (valid range: 0.1 to 20000 Hz)
/// - `duration_secs`: Duration in seconds (must be non-negative)
/// - `sample_rate`: Sample rate in Hz (valid range: 1 to 192000 Hz)
///
/// # Returns
///
/// Vector of f32 samples in the range [-1.0, 1.0]. Returns empty vector if parameters are invalid.
///
/// # Examples
///
/// ```rust
/// use polyphonica::{generate_wave, Waveform};
///
/// // Generate 1 second of 440Hz sine wave
/// let samples = generate_wave(Waveform::Sine, 440.0, 1.0, 44100);
/// assert_eq!(samples.len(), 44100);
///
/// // Generate pulse wave with 25% duty cycle
/// let pulse = generate_wave(
///     Waveform::Pulse { duty_cycle: 0.25 },
///     220.0,
///     0.5,
///     48000
/// );
/// ```
pub fn generate_wave(
    waveform: Waveform,
    frequency: f32,
    duration_secs: f32,
    sample_rate: u32,
) -> Vec<f32> {
    if validate_inputs(frequency, duration_secs, sample_rate).is_err() {
        return Vec::new();
    }
    let total_samples = (duration_secs * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(total_samples);

    for i in 0..total_samples {
        let t = i as f32 / sample_rate as f32;
        let phase = 2.0 * PI * frequency * t;
        let sample = generate_sample(&waveform, phase, t, frequency);
        samples.push(sample);
    }

    samples
}

pub fn apply_envelope(samples: &mut [f32], envelope: &AdsrEnvelope, sample_rate: u32) {
    let total_samples = samples.len();
    if total_samples == 0 {
        return;
    }

    let attack_samples = (envelope.attack_secs * sample_rate as f32) as usize;
    let decay_samples = (envelope.decay_secs * sample_rate as f32) as usize;
    let release_samples = (envelope.release_secs * sample_rate as f32) as usize;

    // Ensure we don't exceed the total sample count
    let attack_end = attack_samples.min(total_samples);
    let decay_end = (attack_samples + decay_samples).min(total_samples);
    let sustain_end = total_samples.saturating_sub(release_samples);
    let release_start = sustain_end;

    for (i, sample) in samples.iter_mut().enumerate() {
        let envelope_value = if i < attack_end {
            // Attack phase: linear ramp from 0 to 1
            if attack_samples > 0 {
                i as f32 / attack_samples as f32
            } else {
                1.0
            }
        } else if i < decay_end {
            // Decay phase: linear ramp from 1 to sustain_level
            if decay_samples > 0 {
                let decay_progress = (i - attack_samples) as f32 / decay_samples as f32;
                1.0 - decay_progress * (1.0 - envelope.sustain_level)
            } else {
                envelope.sustain_level
            }
        } else if i < sustain_end {
            // Sustain phase: constant at sustain_level
            envelope.sustain_level
        } else {
            // Release phase: linear ramp from sustain_level to 0
            if release_samples > 0 {
                let release_progress = (i - release_start) as f32 / release_samples as f32;
                envelope.sustain_level * (1.0 - release_progress)
            } else {
                0.0
            }
        };

        *sample *= envelope_value;
    }
}

pub fn render_event(event: &SoundEvent, sample_rate: u32) -> Vec<f32> {
    if validate_inputs(event.start_frequency, event.duration_secs, sample_rate).is_err() {
        return Vec::new();
    }
    if validate_inputs(event.end_frequency, event.duration_secs, sample_rate).is_err() {
        return Vec::new();
    }
    let total_samples = (event.duration_secs * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(total_samples);

    for i in 0..total_samples {
        let t = i as f32 / sample_rate as f32;
        let progress = t / event.duration_secs;

        // Linear interpolation between start and end frequency
        let current_frequency =
            event.start_frequency + (event.end_frequency - event.start_frequency) * progress;

        let phase = 2.0 * PI * current_frequency * t;
        let sample = generate_sample(&event.waveform, phase, t, current_frequency);
        samples.push(sample);
    }

    // Apply the ADSR envelope
    apply_envelope(&mut samples, &event.envelope, sample_rate);

    samples
}

pub fn render_timeline(
    events: &[(f32, SoundEvent)],
    total_duration_secs: f32,
    sample_rate: u32,
) -> Vec<f32> {
    if total_duration_secs < 0.0 || sample_rate == 0 || sample_rate > 192000 {
        return Vec::new();
    }
    let total_samples = (total_duration_secs * sample_rate as f32) as usize;
    let mut master_buffer = vec![0.0; total_samples];

    for (start_time, event) in events {
        // Calculate the start sample index
        let start_sample_index = (*start_time * sample_rate as f32) as usize;

        // Skip events that start after the total duration
        if start_sample_index >= total_samples {
            continue;
        }

        // Render the event's audio samples
        let event_samples = render_event(event, sample_rate);

        // Mix the event samples into the master buffer
        for (i, sample) in event_samples.iter().enumerate() {
            let buffer_index = start_sample_index + i;

            // Stop if we exceed the master buffer length
            if buffer_index >= total_samples {
                break;
            }

            // Add the sample to the master buffer
            master_buffer[buffer_index] += sample;
        }
    }

    // Clamp all samples to prevent clipping
    for sample in master_buffer.iter_mut() {
        *sample = sample.clamp(-1.0, 1.0);
    }

    master_buffer
}

// ============================================================================
// REAL-TIME ENGINE MODULE
// ============================================================================

/// Maximum number of simultaneous voices for polyphonic playback
pub const MAX_VOICES: usize = 32;

/// Atomic f32 wrapper for lock-free parameter updates
#[derive(Debug)]
pub struct AtomicF32 {
    bits: AtomicU32,
}

impl AtomicF32 {
    pub fn new(value: f32) -> Self {
        AtomicF32 {
            bits: AtomicU32::new(value.to_bits()),
        }
    }

    pub fn load(&self, ordering: Ordering) -> f32 {
        f32::from_bits(self.bits.load(ordering))
    }

    pub fn store(&self, value: f32, ordering: Ordering) {
        self.bits.store(value.to_bits(), ordering);
    }
}

/// Real-time voice state for polyphonic synthesis
#[derive(Debug)]
pub struct Voice {
    /// Current waveform for this voice
    pub waveform: Waveform,
    /// Current oscillator phase (0.0 to 2π)
    pub phase: f32,
    /// Current frequency in Hz
    pub frequency: f32,
    /// Target frequency for sweeps
    pub target_frequency: f32,
    /// Voice amplitude (0.0 to 1.0)
    pub amplitude: f32,
    /// Current envelope state
    pub envelope_state: EnvelopeState,
    /// ADSR envelope parameters
    pub envelope: AdsrEnvelope,
    /// Voice is active and should generate audio
    pub active: AtomicBool,
    /// Voice ID for tracking
    pub voice_id: u32,
    /// Sample time offset for samples
    pub sample_time: f32,
    /// Volume scaling (0.0 to 1.0)
    pub volume: f32,
}

/// Current state within ADSR envelope
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvelopePhase {
    Attack,
    Decay,
    Sustain,
    Release,
    Finished,
}

/// Running envelope state for real-time processing
#[derive(Debug, Clone)]
pub struct EnvelopeState {
    pub phase: EnvelopePhase,
    pub phase_time: f32,
    pub current_level: f32,
    pub release_level: f32, // Level when release was triggered
}

impl Default for EnvelopeState {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvelopeState {
    pub fn new() -> Self {
        EnvelopeState {
            phase: EnvelopePhase::Attack,
            phase_time: 0.0,
            current_level: 0.0,
            release_level: 0.0,
        }
    }

    /// Update envelope state and return current amplitude
    pub fn update(&mut self, envelope: &AdsrEnvelope, dt: f32, note_released: bool) -> f32 {
        // Handle release trigger
        if note_released
            && self.phase != EnvelopePhase::Release
            && self.phase != EnvelopePhase::Finished
        {
            self.phase = EnvelopePhase::Release;
            self.phase_time = 0.0;
            self.release_level = self.current_level;
        }

        self.phase_time += dt;

        match self.phase {
            EnvelopePhase::Attack => {
                if envelope.attack_secs <= 0.0 {
                    self.current_level = 1.0;
                    self.phase = EnvelopePhase::Decay;
                    self.phase_time = 0.0;
                } else if self.phase_time >= envelope.attack_secs {
                    self.current_level = 1.0;
                    self.phase = EnvelopePhase::Decay;
                    self.phase_time = 0.0;
                } else {
                    self.current_level = self.phase_time / envelope.attack_secs;
                }
            }
            EnvelopePhase::Decay => {
                if envelope.decay_secs <= 0.0 {
                    self.current_level = envelope.sustain_level;
                    self.phase = EnvelopePhase::Sustain;
                    self.phase_time = 0.0;
                } else if self.phase_time >= envelope.decay_secs {
                    self.current_level = envelope.sustain_level;
                    self.phase = EnvelopePhase::Sustain;
                    self.phase_time = 0.0;
                } else {
                    let progress = self.phase_time / envelope.decay_secs;
                    self.current_level = 1.0 - progress * (1.0 - envelope.sustain_level);
                }
            }
            EnvelopePhase::Sustain => {
                self.current_level = envelope.sustain_level;
                // Stay in sustain until note is released
            }
            EnvelopePhase::Release => {
                if envelope.release_secs <= 0.0 {
                    self.current_level = 0.0;
                    self.phase = EnvelopePhase::Finished;
                } else if self.phase_time >= envelope.release_secs {
                    self.current_level = 0.0;
                    self.phase = EnvelopePhase::Finished;
                } else {
                    let progress = self.phase_time / envelope.release_secs;
                    self.current_level = self.release_level * (1.0 - progress);
                }
            }
            EnvelopePhase::Finished => {
                self.current_level = 0.0;
            }
        }

        self.current_level.clamp(0.0, 1.0)
    }

    pub fn is_finished(&self) -> bool {
        self.phase == EnvelopePhase::Finished
    }

    /// Trigger note release
    pub fn release(&mut self) {
        if self.phase != EnvelopePhase::Release && self.phase != EnvelopePhase::Finished {
            self.phase = EnvelopePhase::Release;
            self.phase_time = 0.0;
            self.release_level = self.current_level;
        }
    }
}

impl Voice {
    pub fn new(voice_id: u32) -> Self {
        Voice {
            waveform: Waveform::Sine,
            phase: 0.0,
            frequency: 440.0,
            target_frequency: 440.0,
            amplitude: 1.0,
            envelope_state: EnvelopeState::new(),
            envelope: AdsrEnvelope {
                attack_secs: 0.1,
                decay_secs: 0.1,
                sustain_level: 0.7,
                release_secs: 0.3,
            },
            active: AtomicBool::new(false),
            voice_id,
            sample_time: 0.0,
            volume: 1.0,
        }
    }

    /// Reset voice to inactive state
    pub fn reset(&mut self) {
        self.active.store(false, Ordering::Relaxed);
        self.phase = 0.0;
        self.sample_time = 0.0;
        self.envelope_state = EnvelopeState::new();
        self.volume = 1.0;
    }

    /// Trigger a note with the given parameters
    pub fn trigger_note(&mut self, waveform: Waveform, frequency: f32, envelope: AdsrEnvelope) {
        self.waveform = waveform;
        self.frequency = frequency;
        self.target_frequency = frequency;
        self.envelope = envelope;
        self.envelope_state = EnvelopeState::new();
        self.phase = 0.0;
        self.sample_time = 0.0;
        self.volume = 1.0;
        self.active.store(true, Ordering::Relaxed);
    }

    /// Trigger a note with volume scaling
    pub fn trigger_note_with_volume(
        &mut self,
        waveform: Waveform,
        frequency: f32,
        envelope: AdsrEnvelope,
        volume: f32,
    ) {
        self.waveform = waveform;
        self.frequency = frequency;
        self.target_frequency = frequency;
        self.envelope = envelope;
        self.envelope_state = EnvelopeState::new();
        self.phase = 0.0;
        self.sample_time = 0.0;
        self.volume = volume; // Store volume for use during sample generation
        self.active.store(true, Ordering::Relaxed);
    }

    /// Release the current note
    pub fn release_note(&mut self) {
        self.envelope_state.release();
    }

    /// Generate the next audio sample
    pub fn process_sample(&mut self, sample_rate: f32) -> f32 {
        if !self.active.load(Ordering::Relaxed) {
            return 0.0;
        }

        let dt = 1.0 / sample_rate;

        // Update envelope
        let envelope_amplitude = self.envelope_state.update(&self.envelope, dt, false);

        // If envelope is finished, deactivate voice
        if self.envelope_state.is_finished() {
            self.active.store(false, Ordering::Relaxed);
            return 0.0;
        }

        // Generate waveform sample
        let waveform_sample =
            generate_sample(&self.waveform, self.phase, self.sample_time, self.frequency);

        // Update phase for next sample
        self.phase += 2.0 * PI * self.frequency / sample_rate;
        self.phase %= 2.0 * PI;

        // Update sample time for sample-based waveforms
        self.sample_time += dt;

        // Apply envelope, amplitude, and volume
        waveform_sample * envelope_amplitude * self.amplitude * self.volume
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }
}

impl Clone for Voice {
    fn clone(&self) -> Self {
        Voice {
            waveform: self.waveform.clone(),
            phase: self.phase,
            frequency: self.frequency,
            target_frequency: self.target_frequency,
            amplitude: self.amplitude,
            envelope_state: self.envelope_state.clone(),
            envelope: self.envelope.clone(),
            active: AtomicBool::new(self.active.load(Ordering::Relaxed)),
            voice_id: self.voice_id,
            sample_time: self.sample_time,
            volume: self.volume,
        }
    }
}

/// Real-time polyphonic synthesis engine
///
/// The core engine for real-time audio synthesis supporting up to 32 simultaneous voices.
/// Designed for use in audio applications requiring low-latency, high-quality synthesis
/// with automatic voice management and allocation.
///
/// # Features
///
/// - **Polyphonic synthesis**: Up to 32 simultaneous voices
/// - **Voice allocation**: Automatic voice stealing when all voices are in use
/// - **Multiple waveforms**: Sine, square, sawtooth, triangle, pulse, noise, and samples
/// - **ADSR envelopes**: Per-voice envelope processing
/// - **Real-time safe**: Zero-allocation audio processing
/// - **Master volume**: Global volume control with atomic updates
/// - **Stereo output**: Supports both mono and stereo buffer processing
///
/// # Usage Pattern
///
/// 1. Create engine with desired sample rate
/// 2. Configure master volume and parameters
/// 3. Trigger notes as needed (returns voice IDs for control)
/// 4. Process audio buffers in your audio callback
/// 5. Release notes or use panic stop as needed
///
/// # Examples
///
/// ```rust
/// use polyphonica::{RealtimeEngine, Waveform, AdsrEnvelope};
///
/// // Create engine for 44.1kHz audio
/// let mut engine = RealtimeEngine::new(44100.0);
///
/// // Set master volume to 50%
/// engine.set_master_volume(0.5);
///
/// // Define a piano-like envelope
/// let envelope = AdsrEnvelope {
///     attack_secs: 0.01,
///     decay_secs: 0.3,
///     sustain_level: 0.4,
///     release_secs: 0.8,
/// };
///
/// // Trigger a note (returns voice ID for later control)
/// let voice_id = engine.trigger_note(Waveform::Sine, 440.0, envelope);
///
/// // Process audio in your callback
/// let mut buffer = vec![0.0; 1024];
/// engine.process_buffer(&mut buffer);
///
/// // Release the note when done
/// if let Some(id) = voice_id {
///     engine.release_note(id);
/// }
/// ```
///
/// # Thread Safety
///
/// The engine is designed for single-threaded use within an audio callback.
/// Master volume can be updated from other threads using atomic operations.
/// Voice triggering and buffer processing should happen on the same thread.
///
/// # Performance Notes
///
/// - Audio processing is allocation-free once voices are allocated
/// - Voice stealing uses a simple oldest-voice strategy
/// - All samples are clamped to [-1.0, 1.0] to prevent clipping
/// - Inactive voices are automatically detected and recycled
pub struct RealtimeEngine {
    /// Voice pool for polyphonic synthesis
    voices: [Voice; MAX_VOICES],
    /// Master volume (0.0 to 1.0)
    master_volume: AtomicF32,
    /// Current sample rate
    sample_rate: f32,
    /// Next voice ID for allocation
    next_voice_id: u32,
}

impl RealtimeEngine {
    /// Create a new real-time synthesis engine
    pub fn new(sample_rate: f32) -> Self {
        // Initialize voice array
        let mut voices = Vec::with_capacity(MAX_VOICES);
        for i in 0..MAX_VOICES {
            voices.push(Voice::new(i as u32));
        }

        // Convert to fixed-size array
        let voices: [Voice; MAX_VOICES] = voices.try_into().unwrap();

        RealtimeEngine {
            voices,
            master_volume: AtomicF32::new(1.0),
            sample_rate,
            next_voice_id: 0,
        }
    }

    /// Set the sample rate (call this when audio device sample rate changes)
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    /// Set master volume (0.0 to 1.0)
    pub fn set_master_volume(&self, volume: f32) {
        self.master_volume
            .store(volume.clamp(0.0, 1.0), Ordering::Relaxed);
    }

    /// Get current master volume
    pub fn get_master_volume(&self) -> f32 {
        self.master_volume.load(Ordering::Relaxed)
    }

    /// Trigger a new note (finds an available voice)
    pub fn trigger_note(
        &mut self,
        waveform: Waveform,
        frequency: f32,
        envelope: AdsrEnvelope,
    ) -> Option<u32> {
        // First, try to find an inactive voice
        for voice in &mut self.voices {
            if !voice.is_active() {
                voice.trigger_note(waveform, frequency, envelope);
                self.next_voice_id += 1;
                voice.voice_id = self.next_voice_id;
                return Some(voice.voice_id);
            }
        }

        // If no inactive voice found, steal the oldest voice (voice stealing)
        if let Some(oldest_voice) = self.voices.iter_mut().min_by_key(|v| v.voice_id) {
            oldest_voice.trigger_note(waveform, frequency, envelope);
            self.next_voice_id += 1;
            oldest_voice.voice_id = self.next_voice_id;
            Some(oldest_voice.voice_id)
        } else {
            None
        }
    }

    /// Trigger a new note with volume control (finds an available voice)
    pub fn trigger_note_with_volume(
        &mut self,
        waveform: Waveform,
        frequency: f32,
        envelope: AdsrEnvelope,
        volume: f32,
    ) -> Option<u32> {
        // First, try to find an inactive voice
        for voice in &mut self.voices {
            if !voice.is_active() {
                voice.trigger_note_with_volume(waveform, frequency, envelope, volume);
                self.next_voice_id += 1;
                voice.voice_id = self.next_voice_id;
                return Some(voice.voice_id);
            }
        }

        // If no inactive voice found, steal the oldest voice (voice stealing)
        if let Some(oldest_voice) = self.voices.iter_mut().min_by_key(|v| v.voice_id) {
            oldest_voice.trigger_note_with_volume(waveform, frequency, envelope, volume);
            self.next_voice_id += 1;
            oldest_voice.voice_id = self.next_voice_id;
            Some(oldest_voice.voice_id)
        } else {
            None
        }
    }

    /// Release a specific note by voice ID
    pub fn release_note(&mut self, voice_id: u32) {
        for voice in &mut self.voices {
            if voice.voice_id == voice_id && voice.is_active() {
                voice.release_note();
                break;
            }
        }
    }

    /// Release all currently active notes
    pub fn release_all_notes(&mut self) {
        for voice in &mut self.voices {
            if voice.is_active() {
                voice.release_note();
            }
        }
    }

    /// Stop all notes immediately (for panic button)
    pub fn stop_all_notes(&mut self) {
        for voice in &mut self.voices {
            voice.reset();
        }
    }

    /// Get number of currently active voices
    pub fn get_active_voice_count(&self) -> usize {
        self.voices.iter().filter(|v| v.is_active()).count()
    }

    /// Process a buffer of audio samples (CPAL-compatible interface)
    pub fn process_buffer(&mut self, output: &mut [f32]) {
        let master_vol = self.master_volume.load(Ordering::Relaxed);

        for sample in output.iter_mut() {
            let mut mixed_sample = 0.0;

            // Mix all active voices
            for voice in &mut self.voices {
                if voice.is_active() {
                    mixed_sample += voice.process_sample(self.sample_rate);
                }
            }

            // Apply master volume and clipping prevention
            *sample = (mixed_sample * master_vol).clamp(-1.0, 1.0);
        }
    }

    /// Process interleaved stereo buffer (common CPAL format)
    pub fn process_stereo_buffer(&mut self, output: &mut [f32]) {
        assert!(output.len() % 2 == 0, "Stereo buffer must have even length");

        let master_vol = self.master_volume.load(Ordering::Relaxed);

        for chunk in output.chunks_exact_mut(2) {
            let mut mixed_sample = 0.0;

            // Mix all active voices
            for voice in &mut self.voices {
                if voice.is_active() {
                    mixed_sample += voice.process_sample(self.sample_rate);
                }
            }

            // Apply master volume and clipping prevention
            let final_sample = (mixed_sample * master_vol).clamp(-1.0, 1.0);

            // Copy mono signal to both stereo channels
            chunk[0] = final_sample; // Left
            chunk[1] = final_sample; // Right
        }
    }

    /// Convenience method for triggering multiple notes at once (chords)
    pub fn trigger_chord(&mut self, notes: &[(Waveform, f32)], envelope: AdsrEnvelope) -> Vec<u32> {
        let mut voice_ids = Vec::new();
        for (waveform, frequency) in notes {
            if let Some(voice_id) =
                self.trigger_note(waveform.clone(), *frequency, envelope.clone())
            {
                voice_ids.push(voice_id);
            }
        }
        voice_ids
    }

    /// Update voice parameters for real-time modulation
    pub fn set_voice_frequency(&mut self, voice_id: u32, frequency: f32) {
        for voice in &mut self.voices {
            if voice.voice_id == voice_id && voice.is_active() {
                voice.frequency = frequency;
                break;
            }
        }
    }

    /// Set voice amplitude for real-time volume control
    pub fn set_voice_amplitude(&mut self, voice_id: u32, amplitude: f32) {
        for voice in &mut self.voices {
            if voice.voice_id == voice_id && voice.is_active() {
                voice.amplitude = amplitude.clamp(0.0, 1.0);
                break;
            }
        }
    }
}

// Thread-safe wrapper for shared access
pub type SharedRealtimeEngine = Arc<std::sync::Mutex<RealtimeEngine>>;

impl Default for RealtimeEngine {
    fn default() -> Self {
        Self::new(44100.0)
    }
}

// Timing subsystem for precise musical timing
pub mod patterns;
pub mod samples;
pub mod timing;

// Audio processing subsystem for synthesis and streaming
pub mod audio;

// Beat visualization subsystem for musical displays
pub mod visualization;

// Configuration management subsystem for application settings
pub mod config;

#[cfg(test)]
mod tests {
    use super::*;

    const TOLERANCE: f32 = 1e-6;

    #[test]
    fn test_sine_wave() {
        let samples = generate_wave(Waveform::Sine, 1.0, 1.0, 4);
        assert_eq!(samples.len(), 4);

        // At t=0: sin(0) = 0
        assert!((samples[0] - 0.0).abs() < TOLERANCE);
        // At t=0.25: sin(π/2) = 1
        assert!((samples[1] - 1.0).abs() < TOLERANCE);
        // At t=0.5: sin(π) = 0
        assert!((samples[2] - 0.0).abs() < TOLERANCE);
        // At t=0.75: sin(3π/2) = -1
        assert!((samples[3] - (-1.0)).abs() < TOLERANCE);
    }

    #[test]
    fn test_square_wave() {
        let samples = generate_wave(Waveform::Square, 1.0, 1.0, 8);
        assert_eq!(samples.len(), 8);

        // First half should be positive
        for i in 0..4 {
            assert_eq!(samples[i], 1.0);
        }
        // Second half should be negative
        for i in 4..8 {
            assert_eq!(samples[i], -1.0);
        }
    }

    #[test]
    fn test_sawtooth_wave() {
        let samples = generate_wave(Waveform::Sawtooth, 1.0, 1.0, 4);
        assert_eq!(samples.len(), 4);

        // Sawtooth should go from -1 to 1 linearly over one period
        // At 1Hz with 4 samples/sec: t=0,0.25,0.5,0.75
        // normalized_phase: 0, 0.25, 0.5, 0.75
        // sawtooth: 2*phase-1 = -1, -0.5, 0, 0.5
        assert!((samples[0] - (-1.0)).abs() < TOLERANCE);
        assert!((samples[1] - (-0.5)).abs() < TOLERANCE);
        assert!((samples[2] - 0.0).abs() < TOLERANCE);
        assert!((samples[3] - 0.5).abs() < TOLERANCE);
    }

    #[test]
    fn test_triangle_wave() {
        let samples = generate_wave(Waveform::Triangle, 1.0, 1.0, 8);
        assert_eq!(samples.len(), 8);

        // Triangle wave: starts at -1, goes to 1, back to -1
        // Values: [-1.0, -0.5, 0.0, 0.5, 1.0, 0.5, 0.0, -0.5]
        assert!((samples[0] - (-1.0)).abs() < TOLERANCE);
        assert!((samples[4] - 1.0).abs() < TOLERANCE); // Peak at sample 4
        assert!((samples[2] - 0.0).abs() < TOLERANCE); // Zero crossing at sample 2
    }

    #[test]
    fn test_sample_range() {
        let waveforms = [
            Waveform::Sine,
            Waveform::Square,
            Waveform::Sawtooth,
            Waveform::Triangle,
        ];

        for waveform in &waveforms {
            let samples = generate_wave(waveform.clone(), 440.0, 0.1, 44100);

            for sample in &samples {
                assert!(
                    *sample >= -1.0 && *sample <= 1.0,
                    "Sample {} out of range [-1.0, 1.0] for waveform {:?}",
                    sample,
                    waveform
                );
            }
        }
    }

    #[test]
    fn test_correct_sample_count() {
        let samples = generate_wave(Waveform::Sine, 440.0, 2.5, 44100);
        let expected_samples = (2.5 * 44100.0) as usize;
        assert_eq!(samples.len(), expected_samples);
    }

    #[test]
    fn test_frequency_accuracy() {
        // Generate one second of 1Hz sine wave at 100 samples/sec
        let samples = generate_wave(Waveform::Sine, 1.0, 1.0, 100);

        // Check that we complete exactly one cycle
        assert!((samples[0] - samples[100 - 1]).abs() < 0.1); // Should be close to same value

        // Check peak occurs at quarter period
        let quarter_period_idx = 25; // 0.25 seconds * 100 samples/sec
        assert!((samples[quarter_period_idx] - 1.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_adsr_envelope_basic() {
        let envelope = AdsrEnvelope {
            attack_secs: 0.1,
            decay_secs: 0.1,
            sustain_level: 0.5,
            release_secs: 0.1,
        };

        let mut samples = vec![1.0; 40]; // 0.4 seconds at 100 samples/sec
        apply_envelope(&mut samples, &envelope, 100);

        // Attack phase (0-10 samples): should ramp from 0 to 1
        assert!((samples[0] - 0.0).abs() < TOLERANCE);
        assert!((samples[5] - 0.5).abs() < 0.1);
        assert!((samples[9] - 0.9).abs() < 0.1);

        // Decay phase (10-20 samples): should ramp from 1 to 0.5
        assert!((samples[10] - 1.0).abs() < 0.1);
        assert!((samples[15] - 0.75).abs() < 0.1);
        assert!((samples[19] - 0.5).abs() < 0.1);

        // Sustain phase (20-30 samples): should stay at 0.5
        for i in 20..30 {
            assert!((samples[i] - 0.5).abs() < TOLERANCE);
        }

        // Release phase (30-40 samples): should ramp from 0.5 to 0
        assert!((samples[30] - 0.5).abs() < 0.1);
        assert!((samples[35] - 0.25).abs() < 0.1);
        assert!((samples[39] - 0.0).abs() < 0.1);
    }

    #[test]
    fn test_adsr_envelope_empty_samples() {
        let envelope = AdsrEnvelope {
            attack_secs: 0.1,
            decay_secs: 0.1,
            sustain_level: 0.5,
            release_secs: 0.1,
        };

        let mut samples: Vec<f32> = vec![];
        apply_envelope(&mut samples, &envelope, 44100);
        assert_eq!(samples.len(), 0);
    }

    #[test]
    fn test_adsr_envelope_attack_only() {
        let envelope = AdsrEnvelope {
            attack_secs: 1.0,
            decay_secs: 0.0,
            sustain_level: 1.0,
            release_secs: 0.0,
        };

        let mut samples = vec![1.0; 10]; // 0.1 seconds at 100 samples/sec
        apply_envelope(&mut samples, &envelope, 100);

        // Should be in attack phase for all samples
        // Attack spans 100 samples (1.0s * 100 samples/sec), but we only have 10 samples
        // So sample[5] should be 5/100 = 0.05, not 0.5
        assert!((samples[0] - 0.0).abs() < TOLERANCE);
        assert!((samples[5] - 0.05).abs() < 0.1);
        assert!((samples[9] - 0.09).abs() < 0.1);
    }

    #[test]
    fn test_adsr_envelope_sustain_only() {
        let envelope = AdsrEnvelope {
            attack_secs: 0.0,
            decay_secs: 0.0,
            sustain_level: 0.7,
            release_secs: 0.0,
        };

        let mut samples = vec![1.0; 10];
        apply_envelope(&mut samples, &envelope, 100);

        // All samples should be at sustain level
        for sample in &samples {
            assert!((sample - 0.7).abs() < TOLERANCE);
        }
    }

    #[test]
    fn test_adsr_envelope_with_waveform() {
        // Generate a sine wave and apply envelope
        let mut samples = generate_wave(Waveform::Sine, 440.0, 0.4, 100);
        let original_peak = samples.iter().fold(0.0, |max, &x| x.abs().max(max));

        let envelope = AdsrEnvelope {
            attack_secs: 0.1,
            decay_secs: 0.1,
            sustain_level: 0.5,
            release_secs: 0.1,
        };

        apply_envelope(&mut samples, &envelope, 100);

        // Check that envelope was applied correctly
        let enveloped_peak = samples.iter().fold(0.0, |max, &x| x.abs().max(max));
        assert!(enveloped_peak < original_peak);

        // Start should be near silence
        assert!(samples[0].abs() < 0.1);
        // End should be near silence
        assert!(samples[samples.len() - 1].abs() < 0.1);
    }

    #[test]
    fn test_adsr_envelope_zero_sustain() {
        let envelope = AdsrEnvelope {
            attack_secs: 0.1,
            decay_secs: 0.1,
            sustain_level: 0.0,
            release_secs: 0.1,
        };

        let mut samples = vec![1.0; 40]; // 0.4 seconds at 100 samples/sec
        apply_envelope(&mut samples, &envelope, 100);

        // Sustain phase should be silent
        for i in 20..30 {
            assert!((samples[i] - 0.0).abs() < TOLERANCE);
        }
    }

    #[test]
    fn test_adsr_envelope_bounds() {
        let envelope = AdsrEnvelope {
            attack_secs: 0.1,
            decay_secs: 0.1,
            sustain_level: 0.5,
            release_secs: 0.1,
        };

        let mut samples = vec![2.0; 40]; // Start with values > 1.0
        apply_envelope(&mut samples, &envelope, 100);

        // All samples should be within reasonable bounds after envelope
        for sample in &samples {
            assert!(sample.abs() <= 2.0); // Original amplitude was 2.0
        }
    }

    #[test]
    fn test_sound_event_basic() {
        let event = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 440.0,
            end_frequency: 440.0,
            duration_secs: 1.0,
            envelope: AdsrEnvelope {
                attack_secs: 0.1,
                decay_secs: 0.1,
                sustain_level: 0.5,
                release_secs: 0.1,
            },
        };

        let samples = render_event(&event, 100);
        assert_eq!(samples.len(), 100);

        // Check that envelope is applied (start should be near zero)
        assert!(samples[0].abs() < 0.1);
        // End should be near zero (release phase)
        assert!(samples[samples.len() - 1].abs() < 0.1);
    }

    #[test]
    fn test_sound_event_frequency_sweep() {
        let event = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 100.0,
            end_frequency: 200.0,
            duration_secs: 1.0,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 1.0,
                release_secs: 0.0,
            },
        };

        let samples = render_event(&event, 1000);

        // For a frequency sweep, the effective frequency should increase over time
        // We can't easily test the exact frequencies, but we can check that
        // the result is different from a constant frequency
        let constant_freq_event = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 150.0, // Middle frequency
            end_frequency: 150.0,
            duration_secs: 1.0,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 1.0,
                release_secs: 0.0,
            },
        };

        let constant_samples = render_event(&constant_freq_event, 1000);

        // The sweep should produce different results than constant frequency
        let mut differences = 0;
        for (sweep_sample, constant_sample) in samples.iter().zip(constant_samples.iter()) {
            if (sweep_sample - constant_sample).abs() > 0.01 {
                differences += 1;
            }
        }

        // Should have significant differences due to frequency sweep
        assert!(differences > 100);
    }

    #[test]
    fn test_sound_event_constant_frequency() {
        let event = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 440.0,
            end_frequency: 440.0,
            duration_secs: 0.1,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 1.0,
                release_secs: 0.0,
            },
        };

        let samples = render_event(&event, 100);
        let direct_samples = generate_wave(Waveform::Sine, 440.0, 0.1, 100);

        // When start_frequency == end_frequency, should match direct generation
        for (event_sample, direct_sample) in samples.iter().zip(direct_samples.iter()) {
            assert!((event_sample - direct_sample).abs() < TOLERANCE);
        }
    }

    #[test]
    fn test_sound_event_different_waveforms() {
        let base_event = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 440.0,
            end_frequency: 880.0,
            duration_secs: 0.1,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 1.0,
                release_secs: 0.0,
            },
        };

        let waveforms = [
            Waveform::Sine,
            Waveform::Square,
            Waveform::Sawtooth,
            Waveform::Triangle,
        ];

        for waveform in &waveforms {
            let mut event = base_event.clone();
            event.waveform = waveform.clone();

            let samples = render_event(&event, 100);
            assert_eq!(samples.len(), 10);

            // All samples should be within range
            for sample in &samples {
                assert!(
                    *sample >= -1.0 && *sample <= 1.0,
                    "Sample {} out of range for waveform {:?}",
                    sample,
                    waveform
                );
            }
        }
    }

    #[test]
    fn test_sound_event_zero_duration() {
        let event = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 440.0,
            end_frequency: 440.0,
            duration_secs: 0.0,
            envelope: AdsrEnvelope {
                attack_secs: 0.1,
                decay_secs: 0.1,
                sustain_level: 0.5,
                release_secs: 0.1,
            },
        };

        let samples = render_event(&event, 44100);
        assert_eq!(samples.len(), 0);
    }

    #[test]
    fn test_sound_event_complex_envelope() {
        let event = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 440.0,
            end_frequency: 440.0,
            duration_secs: 0.8,
            envelope: AdsrEnvelope {
                attack_secs: 0.2,
                decay_secs: 0.2,
                sustain_level: 0.3,
                release_secs: 0.2,
            },
        };

        let samples = render_event(&event, 100);
        assert_eq!(samples.len(), 80);

        // Attack phase: should start near 0 and increase
        assert!(samples[0].abs() < 0.1);
        assert!(samples[10].abs() > samples[0].abs());

        // Release phase: should end near 0
        assert!(samples[samples.len() - 1].abs() < 0.1);
    }

    #[test]
    fn test_sound_event_extreme_frequency_sweep() {
        let event = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 20.0,
            end_frequency: 20000.0,
            duration_secs: 0.1,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 1.0,
                release_secs: 0.0,
            },
        };

        let samples = render_event(&event, 44100);
        assert_eq!(samples.len(), 4410);

        // All samples should still be within valid range
        for sample in &samples {
            assert!(
                *sample >= -1.0 && *sample <= 1.0,
                "Sample {} out of range during extreme frequency sweep",
                sample
            );
        }
    }

    #[test]
    fn test_render_timeline_empty() {
        let events: &[(f32, SoundEvent)] = &[];
        let timeline = render_timeline(events, 1.0, 100);

        assert_eq!(timeline.len(), 100);
        for sample in &timeline {
            assert_eq!(*sample, 0.0);
        }
    }

    #[test]
    fn test_render_timeline_single_event() {
        let event = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 440.0,
            end_frequency: 440.0,
            duration_secs: 0.5,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 1.0,
                release_secs: 0.0,
            },
        };

        let events = &[(0.0, event.clone())];
        let timeline = render_timeline(events, 1.0, 100);
        let direct_samples = render_event(&event, 100);

        assert_eq!(timeline.len(), 100);

        // First half should match the direct rendering
        for i in 0..50 {
            assert!((timeline[i] - direct_samples[i]).abs() < TOLERANCE);
        }

        // Second half should be silent
        for i in 50..100 {
            assert_eq!(timeline[i], 0.0);
        }
    }

    #[test]
    fn test_render_timeline_multiple_sequential_events() {
        let event1 = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 440.0,
            end_frequency: 440.0,
            duration_secs: 0.3,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 1.0,
                release_secs: 0.0,
            },
        };

        let event2 = SoundEvent {
            waveform: Waveform::Square,
            start_frequency: 880.0,
            end_frequency: 880.0,
            duration_secs: 0.3,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 1.0,
                release_secs: 0.0,
            },
        };

        let events = &[(0.0, event1), (0.5, event2)];
        let timeline = render_timeline(events, 1.0, 100);

        assert_eq!(timeline.len(), 100);

        // Check that there's activity in both time periods
        let first_period_active = timeline[0..30].iter().any(|&s| s.abs() > 0.1);
        let gap_period_silent = timeline[30..50].iter().all(|&s| s.abs() < 0.01);
        let second_period_active = timeline[50..80].iter().any(|&s| s.abs() > 0.1);

        assert!(first_period_active, "First event should be audible");
        assert!(gap_period_silent, "Gap between events should be silent");
        assert!(second_period_active, "Second event should be audible");
    }

    #[test]
    fn test_render_timeline_overlapping_events() {
        let event1 = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 440.0,
            end_frequency: 440.0,
            duration_secs: 0.6,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 0.5,
                release_secs: 0.0,
            },
        };

        let event2 = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 440.0,
            end_frequency: 440.0,
            duration_secs: 0.6,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 0.5,
                release_secs: 0.0,
            },
        };

        let events = &[(0.0, event1), (0.3, event2)];
        let timeline = render_timeline(events, 1.0, 100);

        // In the overlap region (0.3-0.6 seconds, samples 30-60),
        // the amplitude should be roughly double due to mixing
        let single_event_samples = render_event(&events[0].1, 100);

        // Check overlap region has higher amplitude than single event
        let overlap_start = 30;
        let overlap_end = 60;

        // Check that overlapping signals are being mixed (not necessarily louder due to phase differences)
        // The key test is that the timeline is different from the single event in the overlap region
        let mut significant_differences = 0;
        for i in overlap_start..overlap_end {
            if (timeline[i] - single_event_samples[i]).abs() > 0.01 {
                significant_differences += 1;
            }
        }

        // There should be significant differences in the overlap region due to mixing
        assert!(
            significant_differences > 5,
            "Overlap region should show mixing effects (found {} significant differences)",
            significant_differences
        );
    }

    #[test]
    fn test_render_timeline_clipping_prevention() {
        // Create multiple loud events that would clip if not clamped
        let loud_event = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 440.0,
            end_frequency: 440.0,
            duration_secs: 0.5,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 1.0,
                release_secs: 0.0,
            },
        };

        // Schedule 5 overlapping events to force clipping
        let events = &[
            (0.0, loud_event.clone()),
            (0.1, loud_event.clone()),
            (0.2, loud_event.clone()),
            (0.3, loud_event.clone()),
            (0.4, loud_event.clone()),
        ];

        let timeline = render_timeline(events, 1.0, 100);

        // All samples should be clamped to [-1.0, 1.0]
        for sample in &timeline {
            assert!(
                *sample >= -1.0 && *sample <= 1.0,
                "Sample {} should be clamped to [-1.0, 1.0]",
                sample
            );
        }
    }

    #[test]
    fn test_render_timeline_events_beyond_duration() {
        let event = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 440.0,
            end_frequency: 440.0,
            duration_secs: 0.5,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 1.0,
                release_secs: 0.0,
            },
        };

        // Event starts after timeline ends
        let events = &[(2.0, event.clone())];
        let timeline = render_timeline(events, 1.0, 100);

        // Should be completely silent
        for sample in &timeline {
            assert_eq!(*sample, 0.0);
        }
    }

    #[test]
    fn test_render_timeline_partial_event_cutoff() {
        let event = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 440.0,
            end_frequency: 440.0,
            duration_secs: 0.6,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 1.0,
                release_secs: 0.0,
            },
        };

        // Event starts at 0.7s but timeline ends at 1.0s
        let events = &[(0.7, event)];
        let timeline = render_timeline(events, 1.0, 100);

        assert_eq!(timeline.len(), 100);

        // First 70 samples should be silent
        for i in 0..70 {
            assert_eq!(timeline[i], 0.0);
        }

        // Last 30 samples should have some audio (partial event)
        let has_audio = timeline[70..].iter().any(|&s| s.abs() > 0.1);
        assert!(has_audio, "Should have partial audio from cutoff event");
    }

    #[test]
    fn test_render_timeline_different_waveforms() {
        let sine_event = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 440.0,
            end_frequency: 440.0,
            duration_secs: 0.3,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 1.0,
                release_secs: 0.0,
            },
        };

        let square_event = SoundEvent {
            waveform: Waveform::Square,
            start_frequency: 880.0,
            end_frequency: 880.0,
            duration_secs: 0.3,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 1.0,
                release_secs: 0.0,
            },
        };

        let events = &[(0.0, sine_event), (0.0, square_event)];
        let timeline = render_timeline(events, 1.0, 100);

        // Mixed waveforms should produce different results than either alone
        let sine_only = render_timeline(&[(0.0, events[0].1.clone())], 1.0, 100);
        let square_only = render_timeline(&[(0.0, events[1].1.clone())], 1.0, 100);

        let mut timeline_differs_from_sine = false;
        let mut timeline_differs_from_square = false;

        for i in 0..30 {
            if (timeline[i] - sine_only[i]).abs() > 0.01 {
                timeline_differs_from_sine = true;
            }
            if (timeline[i] - square_only[i]).abs() > 0.01 {
                timeline_differs_from_square = true;
            }
        }

        assert!(
            timeline_differs_from_sine,
            "Mixed timeline should differ from sine alone"
        );
        assert!(
            timeline_differs_from_square,
            "Mixed timeline should differ from square alone"
        );
    }

    #[test]
    fn test_render_timeline_zero_duration() {
        let event = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 440.0,
            end_frequency: 440.0,
            duration_secs: 0.5,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 1.0,
                release_secs: 0.0,
            },
        };

        let events = &[(0.0, event)];
        let timeline = render_timeline(events, 0.0, 100);

        assert_eq!(timeline.len(), 0);
    }

    #[test]
    fn test_sample_data_creation() {
        // Create a simple test sample: 1 second of 440Hz sine wave
        let sample_rate = 44100;
        let duration = 1.0;
        let frequency = 440.0;

        let mut test_samples = Vec::new();
        for i in 0..(sample_rate as usize) {
            let t = i as f32 / sample_rate as f32;
            let sample = (2.0 * PI * frequency * t).sin();
            test_samples.push(sample);
        }

        let metadata = SampleMetadata {
            filename: "test_sine.wav".to_string(),
            duration_secs: duration,
            channels: 1,
            bits_per_sample: 16,
        };

        let sample_data = SampleData {
            samples: test_samples,
            sample_rate,
            base_frequency: frequency,
            loop_start: None,
            loop_end: None,
            metadata,
        };

        // Test basic functionality
        assert_eq!(sample_data.sample_rate, 44100);
        assert_eq!(sample_data.base_frequency, 440.0);
        assert_eq!(sample_data.metadata.filename, "test_sine.wav");

        // Test sample retrieval at base frequency
        let sample_at_start = sample_data.get_sample_at_time(0.0, 440.0);
        assert!((sample_at_start - 0.0).abs() < TOLERANCE); // Sine starts at 0

        // For 440Hz, quarter period is 1/(4*440) = 0.000568 seconds
        let quarter_period = 1.0 / (4.0 * 440.0);
        let sample_at_quarter = sample_data.get_sample_at_time(quarter_period, 440.0);
        assert!((sample_at_quarter - 1.0).abs() < 0.1); // Should be near peak
    }

    #[test]
    fn test_sample_pitch_shifting() {
        // Create a test sample
        let sample_rate = 1000; // Low sample rate for testing
        let base_freq = 100.0;

        let mut test_samples = Vec::new();
        for i in 0..sample_rate {
            let t = i as f32 / sample_rate as f32;
            let sample = (2.0 * PI * base_freq * t).sin();
            test_samples.push(sample);
        }

        let sample_data = SampleData {
            samples: test_samples,
            sample_rate,
            base_frequency: base_freq,
            loop_start: None,
            loop_end: None,
            metadata: SampleMetadata {
                filename: "test.wav".to_string(),
                duration_secs: 1.0,
                channels: 1,
                bits_per_sample: 16,
            },
        };

        // Test octave up (2x frequency)
        let sample_at_base = sample_data.get_sample_at_time(0.5, base_freq);
        let sample_at_octave = sample_data.get_sample_at_time(0.25, base_freq * 2.0);

        // When pitched up 2x, position 0.25 should sound like position 0.5 at base freq
        assert!((sample_at_base - sample_at_octave).abs() < 0.1);
    }

    #[test]
    fn test_sample_waveform_integration() {
        // Create a simple sample
        let test_samples = vec![0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5];

        let sample_data = SampleData {
            samples: test_samples,
            sample_rate: 8,
            base_frequency: 1.0, // 1Hz
            loop_start: None,
            loop_end: None,
            metadata: SampleMetadata {
                filename: "test.wav".to_string(),
                duration_secs: 1.0,
                channels: 1,
                bits_per_sample: 16,
            },
        };

        // Test with generate_wave function
        let waveform = Waveform::Sample(sample_data);
        let generated = generate_wave(waveform, 1.0, 0.5, 8);

        assert_eq!(generated.len(), 4); // 0.5 seconds at 8 samples/sec

        // Verify we get reasonable values
        for sample in &generated {
            assert!(*sample >= -1.1 && *sample <= 1.1); // Allow some interpolation error
        }
    }

    // ======================================================================
    // REAL-TIME ENGINE TESTS
    // ======================================================================

    #[test]
    fn test_realtime_engine_creation() {
        let engine = RealtimeEngine::new(44100.0);
        assert_eq!(engine.sample_rate, 44100.0);
        assert_eq!(engine.get_active_voice_count(), 0);
        assert!((engine.get_master_volume() - 1.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_atomic_f32() {
        let atomic = AtomicF32::new(42.5);
        assert!((atomic.load(Ordering::Relaxed) - 42.5).abs() < TOLERANCE);

        atomic.store(17.25, Ordering::Relaxed);
        assert!((atomic.load(Ordering::Relaxed) - 17.25).abs() < TOLERANCE);
    }

    #[test]
    fn test_envelope_state_attack_phase() {
        let mut envelope_state = EnvelopeState::new();
        let envelope = AdsrEnvelope {
            attack_secs: 0.1,
            decay_secs: 0.1,
            sustain_level: 0.7,
            release_secs: 0.2,
        };

        // At start, should be in attack phase
        assert_eq!(envelope_state.phase, EnvelopePhase::Attack);
        assert!((envelope_state.current_level - 0.0).abs() < TOLERANCE);

        // Halfway through attack
        let level = envelope_state.update(&envelope, 0.05, false);
        assert_eq!(envelope_state.phase, EnvelopePhase::Attack);
        assert!((level - 0.5).abs() < 0.1);

        // End of attack
        envelope_state.update(&envelope, 0.05, false);
        assert_eq!(envelope_state.phase, EnvelopePhase::Decay);
        assert!((envelope_state.current_level - 1.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_envelope_state_full_cycle() {
        let mut envelope_state = EnvelopeState::new();
        let envelope = AdsrEnvelope {
            attack_secs: 0.1,
            decay_secs: 0.1,
            sustain_level: 0.6,
            release_secs: 0.1,
        };

        let dt = 0.01; // 10ms steps

        // Attack phase
        for _ in 0..11 {
            // Need one extra step to fully complete attack
            envelope_state.update(&envelope, dt, false);
        }
        assert_eq!(envelope_state.phase, EnvelopePhase::Decay);

        // Decay phase
        for _ in 0..11 {
            // Need one extra step to fully complete decay
            envelope_state.update(&envelope, dt, false);
        }
        assert_eq!(envelope_state.phase, EnvelopePhase::Sustain);
        assert!((envelope_state.current_level - 0.6).abs() < 0.1);

        // Sustain phase (should stay at sustain level)
        for _ in 0..10 {
            let level = envelope_state.update(&envelope, dt, false);
            assert!((level - 0.6).abs() < 0.1);
        }
        assert_eq!(envelope_state.phase, EnvelopePhase::Sustain);

        // Trigger release
        envelope_state.release();
        assert_eq!(envelope_state.phase, EnvelopePhase::Release);

        // Release phase
        for _ in 0..11 {
            // Need one extra step to fully complete release
            envelope_state.update(&envelope, dt, false);
        }
        assert_eq!(envelope_state.phase, EnvelopePhase::Finished);
        assert!(envelope_state.is_finished());
        assert!((envelope_state.current_level - 0.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_voice_lifecycle() {
        let mut voice = Voice::new(0);
        assert!(!voice.is_active());

        let envelope = AdsrEnvelope {
            attack_secs: 0.01,
            decay_secs: 0.01,
            sustain_level: 0.8,
            release_secs: 0.01,
        };

        // Trigger a note
        voice.trigger_note(Waveform::Sine, 440.0, envelope);
        assert!(voice.is_active());
        assert_eq!(voice.frequency, 440.0);
        assert_eq!(voice.envelope_state.phase, EnvelopePhase::Attack);

        // Process some samples
        let sample_rate = 44100.0;
        for _ in 0..100 {
            let sample = voice.process_sample(sample_rate);
            // Should be producing audio
            if voice.is_active() {
                // Sample might be near zero due to envelope attack, but should be valid
                assert!(!sample.is_nan());
                assert!(sample >= -1.0 && sample <= 1.0);
            }
        }

        // Release the note
        voice.release_note();
        assert_eq!(voice.envelope_state.phase, EnvelopePhase::Release);

        // Process until voice becomes inactive
        let mut iterations = 0;
        while voice.is_active() && iterations < 1000 {
            voice.process_sample(sample_rate);
            iterations += 1;
        }
        assert!(!voice.is_active());
    }

    #[test]
    fn test_realtime_engine_single_note() {
        let mut engine = RealtimeEngine::new(44100.0);

        let envelope = AdsrEnvelope {
            attack_secs: 0.01,
            decay_secs: 0.01,
            sustain_level: 0.8,
            release_secs: 0.01,
        };

        // Trigger a note
        let voice_id = engine.trigger_note(Waveform::Sine, 440.0, envelope);
        assert!(voice_id.is_some());
        assert_eq!(engine.get_active_voice_count(), 1);

        // Process a buffer
        let mut buffer = vec![0.0; 1024];
        engine.process_buffer(&mut buffer);

        // Should have generated audio
        let has_audio = buffer.iter().any(|&s| s.abs() > 0.01);
        assert!(has_audio, "Engine should generate audio for active voice");

        // All samples should be in valid range
        for sample in &buffer {
            assert!(*sample >= -1.0 && *sample <= 1.0);
            assert!(!sample.is_nan());
        }

        // Release the note
        engine.release_note(voice_id.unwrap());

        // Process until voice becomes inactive
        for _ in 0..100 {
            engine.process_buffer(&mut buffer);
            if engine.get_active_voice_count() == 0 {
                break;
            }
        }
        assert_eq!(engine.get_active_voice_count(), 0);
    }

    #[test]
    fn test_realtime_engine_polyphonic() {
        let mut engine = RealtimeEngine::new(44100.0);

        let envelope = AdsrEnvelope {
            attack_secs: 0.01,
            decay_secs: 0.1,
            sustain_level: 0.6,
            release_secs: 0.1,
        };

        // Trigger multiple notes
        let voice1 = engine.trigger_note(Waveform::Sine, 440.0, envelope.clone());
        let voice2 = engine.trigger_note(Waveform::Square, 554.37, envelope.clone()); // C# above A
        let voice3 = engine.trigger_note(Waveform::Sawtooth, 659.25, envelope.clone()); // E above A

        assert!(voice1.is_some());
        assert!(voice2.is_some());
        assert!(voice3.is_some());
        assert_eq!(engine.get_active_voice_count(), 3);

        // Process a buffer
        let mut buffer = vec![0.0; 1024];
        engine.process_buffer(&mut buffer);

        // Should have generated mixed audio
        let has_audio = buffer.iter().any(|&s| s.abs() > 0.1);
        assert!(
            has_audio,
            "Engine should generate mixed audio for multiple voices"
        );

        // Release all notes
        engine.release_all_notes();

        // Process until all voices become inactive
        for _ in 0..200 {
            engine.process_buffer(&mut buffer);
            if engine.get_active_voice_count() == 0 {
                break;
            }
        }
        assert_eq!(engine.get_active_voice_count(), 0);
    }

    #[test]
    fn test_realtime_engine_voice_stealing() {
        let mut engine = RealtimeEngine::new(44100.0);

        let envelope = AdsrEnvelope {
            attack_secs: 0.01,
            decay_secs: 0.5,
            sustain_level: 0.8,
            release_secs: 0.5,
        };

        // Fill all voices
        let mut voice_ids = Vec::new();
        for i in 0..MAX_VOICES {
            let voice_id = engine.trigger_note(Waveform::Sine, 440.0 + i as f32, envelope.clone());
            assert!(voice_id.is_some());
            voice_ids.push(voice_id.unwrap());
        }
        assert_eq!(engine.get_active_voice_count(), MAX_VOICES);

        // Trigger one more note (should steal oldest voice)
        let extra_voice = engine.trigger_note(Waveform::Square, 880.0, envelope);
        assert!(extra_voice.is_some());
        assert_eq!(engine.get_active_voice_count(), MAX_VOICES); // Still at max

        // Process some audio to ensure engine handles voice stealing correctly
        let mut buffer = vec![0.0; 512];
        engine.process_buffer(&mut buffer);

        let has_audio = buffer.iter().any(|&s| s.abs() > 0.01);
        assert!(
            has_audio,
            "Engine should still generate audio after voice stealing"
        );
    }

    #[test]
    fn test_realtime_engine_stereo_buffer() {
        let mut engine = RealtimeEngine::new(44100.0);

        let envelope = AdsrEnvelope {
            attack_secs: 0.01,
            decay_secs: 0.1,
            sustain_level: 0.8,
            release_secs: 0.1,
        };

        // Trigger a note
        engine.trigger_note(Waveform::Sine, 440.0, envelope);

        // Process stereo buffer
        let mut stereo_buffer = vec![0.0; 1024]; // 512 stereo samples
        engine.process_stereo_buffer(&mut stereo_buffer);

        // Check that left and right channels are identical (mono signal)
        for chunk in stereo_buffer.chunks_exact(2) {
            assert!(
                (chunk[0] - chunk[1]).abs() < TOLERANCE,
                "Left and right channels should be identical"
            );
        }

        // Should have generated audio
        let has_audio = stereo_buffer.iter().any(|&s| s.abs() > 0.01);
        assert!(has_audio, "Engine should generate stereo audio");
    }

    #[test]
    fn test_realtime_engine_master_volume() {
        let mut engine = RealtimeEngine::new(44100.0);

        let envelope = AdsrEnvelope {
            attack_secs: 0.0,
            decay_secs: 0.0,
            sustain_level: 1.0,
            release_secs: 0.0,
        };

        // Trigger a note
        engine.trigger_note(Waveform::Sine, 440.0, envelope);

        // Test with different master volumes
        let mut buffer = vec![0.0; 512];

        // Full volume
        engine.set_master_volume(1.0);
        engine.process_buffer(&mut buffer);
        let max_amplitude_full = buffer.iter().map(|s| s.abs()).fold(0.0, f32::max);

        // Half volume
        engine.set_master_volume(0.5);
        buffer.fill(0.0);
        engine.process_buffer(&mut buffer);
        let max_amplitude_half = buffer.iter().map(|s| s.abs()).fold(0.0, f32::max);

        // Half volume should be roughly half the amplitude
        assert!(max_amplitude_half < max_amplitude_full);
        assert!((max_amplitude_half * 2.0 - max_amplitude_full).abs() < 0.1);

        // Zero volume
        engine.set_master_volume(0.0);
        buffer.fill(0.0);
        engine.process_buffer(&mut buffer);
        let max_amplitude_zero = buffer.iter().map(|s| s.abs()).fold(0.0, f32::max);
        assert!(
            max_amplitude_zero < 0.001,
            "Zero volume should produce silent output"
        );
    }

    #[test]
    fn test_realtime_engine_chord_trigger() {
        let mut engine = RealtimeEngine::new(44100.0);

        let envelope = AdsrEnvelope {
            attack_secs: 0.01,
            decay_secs: 0.1,
            sustain_level: 0.7,
            release_secs: 0.1,
        };

        // Trigger a C Major chord
        let chord_notes = &[
            (Waveform::Sine, 261.63), // C
            (Waveform::Sine, 329.63), // E
            (Waveform::Sine, 392.00), // G
        ];

        let voice_ids = engine.trigger_chord(chord_notes, envelope);
        assert_eq!(voice_ids.len(), 3);
        assert_eq!(engine.get_active_voice_count(), 3);

        // Process audio
        let mut buffer = vec![0.0; 1024];
        engine.process_buffer(&mut buffer);

        // Should generate harmonic content
        let has_audio = buffer.iter().any(|&s| s.abs() > 0.1);
        assert!(has_audio, "Chord should generate audible audio");

        // Release all notes
        for voice_id in voice_ids {
            engine.release_note(voice_id);
        }
    }

    #[test]
    fn test_realtime_engine_parameter_updates() {
        let mut engine = RealtimeEngine::new(44100.0);

        let envelope = AdsrEnvelope {
            attack_secs: 0.01,
            decay_secs: 0.5,
            sustain_level: 0.8,
            release_secs: 0.5,
        };

        // Trigger a note
        let voice_id = engine
            .trigger_note(Waveform::Sine, 440.0, envelope)
            .unwrap();

        // Update voice frequency
        engine.set_voice_frequency(voice_id, 880.0);

        // Update voice amplitude
        engine.set_voice_amplitude(voice_id, 0.3);

        // Process some audio to ensure parameters are applied
        let mut buffer = vec![0.0; 512];
        engine.process_buffer(&mut buffer);

        let has_audio = buffer.iter().any(|&s| s.abs() > 0.01);
        assert!(
            has_audio,
            "Engine should generate audio with updated parameters"
        );
    }

    #[test]
    fn test_realtime_engine_panic_stop() {
        let mut engine = RealtimeEngine::new(44100.0);

        let envelope = AdsrEnvelope {
            attack_secs: 0.01,
            decay_secs: 1.0,
            sustain_level: 0.8,
            release_secs: 1.0,
        };

        // Trigger multiple notes
        for i in 0..8 {
            engine.trigger_note(Waveform::Sine, 440.0 + i as f32 * 100.0, envelope.clone());
        }
        assert_eq!(engine.get_active_voice_count(), 8);

        // Panic stop
        engine.stop_all_notes();
        assert_eq!(engine.get_active_voice_count(), 0);

        // Process buffer - should be silent
        let mut buffer = vec![0.0; 1024];
        engine.process_buffer(&mut buffer);

        let max_amplitude = buffer.iter().map(|s| s.abs()).fold(0.0, f32::max);
        assert!(max_amplitude < 0.001, "Panic stop should produce silence");
    }

    #[test]
    fn test_realtime_engine_different_waveforms() {
        let mut engine = RealtimeEngine::new(44100.0);

        let envelope = AdsrEnvelope {
            attack_secs: 0.0,
            decay_secs: 0.0,
            sustain_level: 1.0,
            release_secs: 0.0,
        };

        // Test all waveform types
        let waveforms = vec![
            Waveform::Sine,
            Waveform::Square,
            Waveform::Sawtooth,
            Waveform::Triangle,
            Waveform::Pulse { duty_cycle: 0.5 },
            Waveform::Noise,
        ];

        for waveform in waveforms {
            engine.stop_all_notes(); // Clear previous

            let voice_id = engine.trigger_note(waveform, 440.0, envelope.clone());
            assert!(voice_id.is_some());

            let mut buffer = vec![0.0; 256];
            engine.process_buffer(&mut buffer);

            // Each waveform should generate some audio
            let has_audio = buffer.iter().any(|&s| s.abs() > 0.01);
            assert!(has_audio, "Waveform should generate audio");

            // All samples should be in valid range
            for sample in &buffer {
                assert!(*sample >= -1.0 && *sample <= 1.0);
                assert!(!sample.is_nan());
            }
        }
    }
}
