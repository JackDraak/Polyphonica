/// Guitar Buddy - Musical Practice Companion
///
/// Phase 1: Advanced metronome with multiple click sounds and time signatures
/// Phase 2: Full accompaniment with drums, bass lines, and chord progressions
///
/// Uses Polyphonica real-time synthesis engine for precise, low-latency audio generation.

use polyphonica::{RealtimeEngine, Waveform, AdsrEnvelope, SampleData};
use eframe::egui;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Time signature representation
#[derive(Debug, Clone, Copy, PartialEq)]
struct TimeSignature {
    beats_per_measure: u8,
    note_value: u8, // 4 = quarter note, 8 = eighth note, etc.
}

impl TimeSignature {
    fn new(beats: u8, note_value: u8) -> Self {
        Self {
            beats_per_measure: beats,
            note_value,
        }
    }

    fn common_signatures() -> Vec<(&'static str, TimeSignature)> {
        vec![
            ("4/4", TimeSignature::new(4, 4)),
            ("3/4", TimeSignature::new(3, 4)),
            ("2/4", TimeSignature::new(2, 4)),
            ("6/8", TimeSignature::new(6, 8)),
            ("9/8", TimeSignature::new(9, 8)),
            ("12/8", TimeSignature::new(12, 8)),
            ("5/4", TimeSignature::new(5, 4)),
            ("7/8", TimeSignature::new(7, 8)),
        ]
    }

    fn display(&self) -> String {
        format!("{}/{}", self.beats_per_measure, self.note_value)
    }
}

/// Drum pattern system for Phase 2
#[derive(Debug, Clone)]
struct DrumPatternBeat {
    beat_position: f32,  // 1.0 = first beat, 1.5 = halfway to second beat, etc.
    samples: Vec<ClickType>,
    accent: bool,
}

#[derive(Debug, Clone)]
struct DrumPattern {
    name: String,
    time_signature: TimeSignature,
    tempo_range: (u32, u32),  // (min_bpm, max_bpm)
    beats: Vec<DrumPatternBeat>,
}

impl DrumPattern {
    fn basic_rock() -> Self {
        Self {
            name: "Basic Rock Beat".to_string(),
            time_signature: TimeSignature::new(4, 4),
            tempo_range: (80, 140),
            beats: vec![
                DrumPatternBeat { beat_position: 1.0, samples: vec![ClickType::AcousticKick, ClickType::HiHatClosed], accent: true },
                DrumPatternBeat { beat_position: 1.5, samples: vec![ClickType::HiHatClosed], accent: false },
                DrumPatternBeat { beat_position: 2.0, samples: vec![ClickType::AcousticSnare, ClickType::HiHatClosed], accent: false },
                DrumPatternBeat { beat_position: 2.5, samples: vec![ClickType::HiHatClosed], accent: false },
                DrumPatternBeat { beat_position: 3.0, samples: vec![ClickType::AcousticKick, ClickType::HiHatClosed], accent: false },
                DrumPatternBeat { beat_position: 3.5, samples: vec![ClickType::HiHatClosed], accent: false },
                DrumPatternBeat { beat_position: 4.0, samples: vec![ClickType::AcousticSnare, ClickType::HiHatClosed], accent: false },
                DrumPatternBeat { beat_position: 4.5, samples: vec![ClickType::HiHatClosed], accent: false },
            ],
        }
    }

    fn shuffle() -> Self {
        Self {
            name: "Shuffle Beat".to_string(),
            time_signature: TimeSignature::new(4, 4),
            tempo_range: (60, 120),
            beats: vec![
                DrumPatternBeat { beat_position: 1.0, samples: vec![ClickType::AcousticKick, ClickType::HiHatClosed], accent: true },
                DrumPatternBeat { beat_position: 1.67, samples: vec![ClickType::HiHatClosed], accent: false },
                DrumPatternBeat { beat_position: 2.0, samples: vec![ClickType::AcousticSnare, ClickType::HiHatClosed], accent: false },
                DrumPatternBeat { beat_position: 2.67, samples: vec![ClickType::HiHatClosed], accent: false },
                DrumPatternBeat { beat_position: 3.0, samples: vec![ClickType::AcousticKick, ClickType::HiHatClosed], accent: false },
                DrumPatternBeat { beat_position: 3.67, samples: vec![ClickType::HiHatClosed], accent: false },
                DrumPatternBeat { beat_position: 4.0, samples: vec![ClickType::AcousticSnare, ClickType::HiHatClosed], accent: false },
                DrumPatternBeat { beat_position: 4.67, samples: vec![ClickType::HiHatClosed], accent: false },
            ],
        }
    }

    fn ballad() -> Self {
        Self {
            name: "Ballad Beat".to_string(),
            time_signature: TimeSignature::new(4, 4),
            tempo_range: (60, 90),
            beats: vec![
                DrumPatternBeat { beat_position: 1.0, samples: vec![ClickType::AcousticKick, ClickType::HiHatClosed], accent: true },
                DrumPatternBeat { beat_position: 2.0, samples: vec![ClickType::AcousticSnare, ClickType::HiHatClosed], accent: false },
                DrumPatternBeat { beat_position: 3.0, samples: vec![ClickType::AcousticKick, ClickType::HiHatClosed], accent: false },
                DrumPatternBeat { beat_position: 4.0, samples: vec![ClickType::AcousticSnare, ClickType::HiHatClosed], accent: false },
            ],
        }
    }

    fn waltz() -> Self {
        Self {
            name: "Waltz Beat".to_string(),
            time_signature: TimeSignature::new(3, 4),
            tempo_range: (90, 180),
            beats: vec![
                DrumPatternBeat { beat_position: 1.0, samples: vec![ClickType::AcousticKick], accent: true },
                DrumPatternBeat { beat_position: 2.0, samples: vec![ClickType::AcousticSnare], accent: false },
                DrumPatternBeat { beat_position: 3.0, samples: vec![ClickType::AcousticSnare], accent: false },
            ],
        }
    }

    fn all_patterns() -> Vec<DrumPattern> {
        vec![
            Self::basic_rock(),
            Self::shuffle(),
            Self::ballad(),
            Self::waltz(),
        ]
    }
}

/// Pattern playback state - using discrete beat scheduling to avoid timing drift
#[derive(Debug, Clone)]
struct PatternState {
    current_pattern: Option<DrumPattern>,
    current_beat_index: usize,  // Index into current pattern's beats (0-based)
    next_beat_time: Option<Instant>,  // Absolute time when next beat should trigger
    pattern_enabled: bool,
}

impl PatternState {
    fn new() -> Self {
        Self {
            current_pattern: None,
            current_beat_index: 0,
            next_beat_time: None,
            pattern_enabled: false,
        }
    }

    fn set_pattern(&mut self, pattern: DrumPattern) {
        self.current_pattern = Some(pattern);
        self.current_beat_index = 0;
        self.next_beat_time = None;
    }

    fn clear_pattern(&mut self) {
        self.current_pattern = None;
        self.pattern_enabled = false;
        self.current_beat_index = 0;
        self.next_beat_time = None;
    }

    fn start(&mut self) {
        self.pattern_enabled = true;
        self.current_beat_index = 0;
        self.next_beat_time = None;
    }

    fn stop(&mut self) {
        self.pattern_enabled = false;
        self.current_beat_index = 0;
        self.next_beat_time = None;
    }

    /// Check if it's time to trigger pattern beats using discrete beat scheduling (eliminates timing drift)
    fn check_pattern_triggers(&mut self, tempo_bpm: f32) -> Vec<(ClickType, bool)> {
        if !self.pattern_enabled {
            return vec![];
        }

        let Some(ref pattern) = self.current_pattern else {
            return vec![];
        };

        let now = Instant::now();

        // Note: beat_interval_ms is calculated in helper methods as needed

        match self.next_beat_time {
            None => {
                // Start pattern - schedule first beat
                if pattern.beats.is_empty() {
                    return vec![];
                }

                // Find beats at position 1.0 (start of pattern) - collect triggers first
                let first_beat_triggers: Vec<(ClickType, bool)> = pattern.beats.iter()
                    .filter(|beat| (beat.beat_position - 1.0).abs() < 0.01)
                    .flat_map(|beat| beat.samples.iter().map(|&sample| (sample, beat.accent)))
                    .collect();

                if !first_beat_triggers.is_empty() {
                    // Schedule next beat after the first one
                    self.schedule_next_beat(tempo_bpm);
                    first_beat_triggers
                } else {
                    // No beat 1, schedule first available beat
                    self.current_beat_index = 0;
                    self.schedule_next_beat(tempo_bpm);
                    vec![]
                }
            }
            Some(next_time) => {
                // Check if it's time to trigger the next beat
                if now >= next_time {
                    // Trigger current beat
                    let current_beat = &pattern.beats[self.current_beat_index];
                    let triggers: Vec<_> = current_beat.samples.iter()
                        .map(|&sample| (sample, current_beat.accent))
                        .collect();

                    // Advance to next beat with timing reset (prevents drift)
                    self.advance_to_next_beat(tempo_bpm);

                    triggers
                } else {
                    // Not time yet
                    vec![]
                }
            }
        }
    }

    /// Schedule the next beat trigger time based on pattern position
    fn schedule_next_beat(&mut self, tempo_bpm: f32) {
        let Some(ref pattern) = self.current_pattern else {
            return;
        };

        if pattern.beats.is_empty() {
            return;
        }

        let beat_interval_ms = 60000.0 / tempo_bpm as f64;
        let current_beat = &pattern.beats[self.current_beat_index];

        // Calculate milliseconds from beat 1 to this beat position
        let ms_from_beat_1 = (current_beat.beat_position - 1.0) as f64 * beat_interval_ms;

        // Schedule trigger time
        self.next_beat_time = Some(Instant::now() + Duration::from_millis(ms_from_beat_1 as u64));
    }

    /// Advance to next beat in pattern and reset timing base (prevents drift accumulation)
    fn advance_to_next_beat(&mut self, tempo_bpm: f32) {
        let Some(ref pattern) = self.current_pattern else {
            return;
        };

        if pattern.beats.is_empty() {
            return;
        }

        // Advance beat index with looping
        self.current_beat_index = (self.current_beat_index + 1) % pattern.beats.len();

        let beat_interval_ms = 60000.0 / tempo_bpm as f64;
        let current_beat = &pattern.beats[self.current_beat_index];
        let next_beat_position = current_beat.beat_position;

        // Calculate interval to next beat (handles looping)
        let current_time = Instant::now();
        let interval_ms = if self.current_beat_index == 0 {
            // Looped back to start - calculate time to beat 1 of next measure
            let current_beat_in_pattern = &pattern.beats[pattern.beats.len() - 1];
            let loop_point = pattern.time_signature.beats_per_measure as f32 + 1.0;
            let remaining_time = (loop_point - current_beat_in_pattern.beat_position) as f64 * beat_interval_ms;
            let next_beat_time = (next_beat_position - 1.0) as f64 * beat_interval_ms;
            remaining_time + next_beat_time
        } else {
            // Normal advance within measure
            let prev_beat = &pattern.beats[self.current_beat_index - 1];
            (next_beat_position - prev_beat.beat_position) as f64 * beat_interval_ms
        };

        // Reset timing base - this prevents drift accumulation!
        self.next_beat_time = Some(current_time + Duration::from_millis(interval_ms as u64));
    }

    /// Get current beat number for visualizer (1-based)
    fn get_current_beat_number(&self) -> u8 {
        if let Some(ref pattern) = self.current_pattern {
            if !pattern.beats.is_empty() && self.current_beat_index < pattern.beats.len() {
                let current_beat = &pattern.beats[self.current_beat_index];
                (current_beat.beat_position.floor() as u8).max(1)
            } else {
                1
            }
        } else {
            1
        }
    }

    /// Get current beat position for display (1.0-based)
    fn get_current_beat_position(&self) -> f32 {
        if let Some(ref pattern) = self.current_pattern {
            if !pattern.beats.is_empty() && self.current_beat_index < pattern.beats.len() {
                pattern.beats[self.current_beat_index].beat_position
            } else {
                1.0
            }
        } else {
            1.0
        }
    }
}

/// Drum sample manager for loading and storing samples
#[derive(Debug, Clone)]
struct DrumSampleManager {
    samples: HashMap<ClickType, SampleData>,
}

impl DrumSampleManager {
    fn new() -> Self {
        Self {
            samples: HashMap::new(),
        }
    }

    fn load_drum_samples(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Load acoustic drum kit samples using relative paths from project root
        let sample_paths = vec![
            (ClickType::AcousticKick, "samples/drums/acoustic/kit_01/drumkit-kick.wav"),
            (ClickType::AcousticSnare, "samples/drums/acoustic/kit_01/drumkit-snare.wav"),
            (ClickType::HiHatClosed, "samples/drums/acoustic/kit_01/drumkit-hihat.wav"),
            (ClickType::HiHatOpen, "samples/drums/acoustic/kit_01/drumkit-hihat-open.wav"),
            (ClickType::RimShot, "samples/drums/acoustic/kit_01/drumkit-rimshot.wav"), // Dedicated rimshot sample
            (ClickType::Stick, "samples/drums/acoustic/kit_01/drumkit-stick.wav"),    // Dedicated stick sample
        ];

        for (click_type, path) in sample_paths {
            match SampleData::from_file(path, 440.0) {
                Ok(sample_data) => {
                    println!("✅ Loaded drum sample: {} from {}", click_type.name(), path);
                    self.samples.insert(click_type, sample_data);
                }
                Err(e) => {
                    println!("⚠️  Could not load {}: {} (falling back to synthetic)", path, e);
                    // Continue without the sample - will fall back to synthetic sound
                }
            }
        }

        Ok(())
    }

    fn get_sample(&self, click_type: &ClickType) -> Option<&SampleData> {
        self.samples.get(click_type)
    }

    fn has_sample(&self, click_type: &ClickType) -> bool {
        self.samples.contains_key(click_type)
    }
}

/// Beat event for coupling audio triggers with visualizer updates
#[derive(Debug, Clone)]
struct BeatEvent {
    beat_number: u8,         // 1-based beat number (1, 2, 3, 4)
    accent: bool,            // Whether this beat is accented
    samples: Vec<ClickType>, // Audio samples that were triggered
    timestamp: Instant,      // When this beat was triggered
}

/// Beat tracker - captures audio trigger events for visualizer coupling
#[derive(Debug, Clone)]
struct BeatTracker {
    current_beat: Option<BeatEvent>,  // Last triggered beat event
    beat_history: Vec<BeatEvent>,     // Recent beat events (for analysis)
    max_history: usize,               // Maximum events to keep in history
}

impl BeatTracker {
    fn new() -> Self {
        Self {
            current_beat: None,
            beat_history: Vec::new(),
            max_history: 32,  // Keep last 32 beat events
        }
    }

    /// Record a beat event from audio trigger
    fn record_beat(&mut self, event: BeatEvent) {
        self.current_beat = Some(event.clone());

        // Add to history with size limit
        self.beat_history.push(event);
        if self.beat_history.len() > self.max_history {
            self.beat_history.remove(0);
        }
    }

    /// Get current beat state for visualizer
    fn get_current_beat(&self) -> (u8, bool) {
        if let Some(ref event) = self.current_beat {
            (event.beat_number, event.accent)
        } else {
            (1, false)  // Default state
        }
    }

    /// Get last beat timestamp for timing analysis
    fn get_last_beat_time(&self) -> Option<Instant> {
        self.current_beat.as_ref().map(|event| event.timestamp)
    }
}

/// Different metronome click sound types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ClickType {
    // Synthetic sounds
    WoodBlock,      // Sharp percussive click
    DigitalBeep,    // Clean sine wave beep
    Cowbell,        // Metallic ring
    ElectroClick,   // Electronic click
    // Real drum samples
    AcousticKick,   // Acoustic kick drum
    AcousticSnare,  // Acoustic snare drum
    HiHatClosed,    // Closed hi-hat
    HiHatOpen,      // Open hi-hat
    RimShot,        // Snare rim (using snare sample with envelope)
    Stick,          // Drumstick click (using hi-hat)
}

impl ClickType {
    fn all() -> Vec<ClickType> {
        vec![
            // Synthetic sounds
            ClickType::WoodBlock,
            ClickType::DigitalBeep,
            ClickType::Cowbell,
            ClickType::ElectroClick,
            // Real drum samples
            ClickType::AcousticKick,
            ClickType::AcousticSnare,
            ClickType::HiHatClosed,
            ClickType::HiHatOpen,
            ClickType::RimShot,
            ClickType::Stick,
        ]
    }

    fn name(self) -> &'static str {
        match self {
            // Synthetic sounds
            ClickType::WoodBlock => "Wood Block",
            ClickType::DigitalBeep => "Digital Beep",
            ClickType::Cowbell => "Cowbell",
            ClickType::ElectroClick => "Electro Click",
            // Real drum samples
            ClickType::AcousticKick => "Acoustic Kick",
            ClickType::AcousticSnare => "Acoustic Snare",
            ClickType::HiHatClosed => "Hi-Hat Closed",
            ClickType::HiHatOpen => "Hi-Hat Open",
            ClickType::RimShot => "Rim Shot",
            ClickType::Stick => "Drum Stick",
        }
    }

    /// Generate the waveform and parameters for this click type
    fn get_sound_params(self, sample_manager: &DrumSampleManager) -> (Waveform, f32, AdsrEnvelope) {
        // Check if we have a sample for this click type
        if let Some(sample_data) = sample_manager.get_sample(&self) {
            return (
                Waveform::DrumSample(sample_data.clone()),
                440.0, // Frequency is ignored for drum samples
                self.get_sample_envelope()
            );
        }

        // Fall back to synthetic sound
        self.get_synthetic_params()
    }

    /// Get ADSR envelope for sample-based sounds
    /// For drums, we use minimal envelope shaping to preserve natural character
    fn get_sample_envelope(self) -> AdsrEnvelope {
        match self {
            ClickType::AcousticKick => AdsrEnvelope {
                attack_secs: 0.001,  // Instant attack
                decay_secs: 1.0,     // Let natural sample decay
                sustain_level: 0.0,  // No sustain - one-shot sample
                release_secs: 0.001, // Minimal release
            },
            ClickType::AcousticSnare => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 0.5,     // Let natural snare ring
                sustain_level: 0.0,
                release_secs: 0.001,
            },
            ClickType::HiHatClosed => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 0.2,     // Natural hi-hat decay
                sustain_level: 0.0,
                release_secs: 0.001,
            },
            ClickType::HiHatOpen => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 1.0,     // Let open hi-hat ring naturally
                sustain_level: 0.0,
                release_secs: 0.001,
            },
            ClickType::RimShot => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 0.3,     // Natural rim shot decay
                sustain_level: 0.0,
                release_secs: 0.001,
            },
            ClickType::Stick => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 0.1,     // Short stick click
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
    fn get_synthetic_params(self) -> (Waveform, f32, AdsrEnvelope) {
        match self {
            ClickType::WoodBlock => (
                Waveform::Noise,
                800.0, // High frequency for sharp click
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.05,
                    sustain_level: 0.0,
                    release_secs: 0.02,
                }
            ),
            ClickType::DigitalBeep => (
                Waveform::Sine,
                1000.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.08,
                    sustain_level: 0.0,
                    release_secs: 0.05,
                }
            ),
            ClickType::Cowbell => (
                Waveform::Square,
                800.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.15,
                    sustain_level: 0.0,
                    release_secs: 0.1,
                }
            ),
            ClickType::RimShot => (
                Waveform::Pulse { duty_cycle: 0.1 },
                400.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.03,
                    sustain_level: 0.0,
                    release_secs: 0.02,
                }
            ),
            ClickType::Stick => (
                Waveform::Triangle,
                2000.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.02,
                    sustain_level: 0.0,
                    release_secs: 0.01,
                }
            ),
            ClickType::ElectroClick => (
                Waveform::Pulse { duty_cycle: 0.25 },
                1200.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.04,
                    sustain_level: 0.0,
                    release_secs: 0.03,
                }
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
                }
            ),
            ClickType::AcousticSnare => (
                Waveform::Noise,
                800.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.15,
                    sustain_level: 0.0,
                    release_secs: 0.05,
                }
            ),
            ClickType::HiHatClosed => (
                Waveform::Pulse { duty_cycle: 0.1 },
                8000.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.08,
                    sustain_level: 0.0,
                    release_secs: 0.02,
                }
            ),
            ClickType::HiHatOpen => (
                Waveform::Pulse { duty_cycle: 0.1 },
                6000.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.25,
                    sustain_level: 0.0,
                    release_secs: 0.1,
                }
            ),
        }
    }
}

/// Metronome state and timing control
#[derive(Debug, Clone)]
struct MetronomeState {
    is_playing: bool,
    tempo_bpm: f32,
    time_signature: TimeSignature,
    click_type: ClickType,
    accent_first_beat: bool,
    volume: f32,
    current_beat: u8,
    last_beat_time: Option<Instant>,
    // Phase 2: Pattern support
    pattern_state: PatternState,
    pattern_mode: bool,  // true = pattern mode, false = metronome mode
    // Phase 2: Beat tracking for event-driven visualizer coupling
    beat_tracker: BeatTracker,
}

impl MetronomeState {
    fn new() -> Self {
        Self {
            is_playing: false,
            tempo_bpm: 120.0,
            time_signature: TimeSignature::new(4, 4),
            click_type: ClickType::WoodBlock,
            accent_first_beat: true,
            volume: 0.7,
            current_beat: 0,
            last_beat_time: None,
            pattern_state: PatternState::new(),
            pattern_mode: false,
            beat_tracker: BeatTracker::new(),
        }
    }

    /// Calculate time between beats in milliseconds
    fn beat_interval_ms(&self) -> f64 {
        60000.0 / self.tempo_bpm as f64
    }

    /// Check if it's time for the next beat
    fn should_trigger_beat(&mut self) -> bool {
        if !self.is_playing {
            return false;
        }

        let now = Instant::now();

        match self.last_beat_time {
            None => {
                // First beat
                self.last_beat_time = Some(now);
                self.current_beat = 1;
                true
            }
            Some(last_time) => {
                let elapsed_ms = now.duration_since(last_time).as_millis() as f64;
                if elapsed_ms >= self.beat_interval_ms() {
                    self.last_beat_time = Some(now);
                    self.current_beat = (self.current_beat % self.time_signature.beats_per_measure) + 1;
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Get volume for current beat (accent first beat if enabled)
    fn get_beat_volume(&self) -> f32 {
        if self.accent_first_beat && self.current_beat == 1 {
            (self.volume * 1.5).min(1.0) // 50% louder for first beat
        } else {
            self.volume
        }
    }

    /// Set drum pattern and switch to pattern mode
    fn set_pattern(&mut self, pattern: DrumPattern) {
        self.pattern_state.set_pattern(pattern);
        self.pattern_mode = true;
    }

    /// Clear pattern and return to metronome mode
    fn clear_pattern(&mut self) {
        self.pattern_state.clear_pattern();
        self.pattern_mode = false;
    }

    /// Check for pattern triggers and return samples to play
    fn check_pattern_triggers(&mut self) -> Vec<(ClickType, bool)> {
        if self.pattern_mode && self.is_playing {
            self.pattern_state.check_pattern_triggers(self.tempo_bpm)
        } else {
            vec![]
        }
    }

    fn start(&mut self) {
        self.is_playing = true;
        self.current_beat = 0;
        self.last_beat_time = None;
        if self.pattern_mode {
            self.pattern_state.start();
        }
    }

    fn stop(&mut self) {
        self.is_playing = false;
        self.current_beat = 0;
        self.last_beat_time = None;
        if self.pattern_mode {
            self.pattern_state.stop();
        }
    }

    fn pause(&mut self) {
        self.is_playing = false;
        if self.pattern_mode {
            self.pattern_state.stop();
        }
    }

    fn resume(&mut self) {
        self.is_playing = true;
        self.last_beat_time = Some(Instant::now());
        if self.pattern_mode {
            self.pattern_state.start();
        }
    }
}

/// Shared state between GUI and audio threads
#[derive(Clone)]
struct AppState {
    engine: Arc<Mutex<RealtimeEngine>>,
    metronome: Arc<Mutex<MetronomeState>>,
    drum_samples: Arc<Mutex<DrumSampleManager>>,
}

/// Main application for Guitar Buddy
struct GuitarBuddy {
    app_state: AppState,
    _audio_stream: Stream,
}

impl GuitarBuddy {
    fn new(_cc: &eframe::CreationContext<'_>) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize real-time engine
        let engine = Arc::new(Mutex::new(RealtimeEngine::new(44100.0)));

        // Initialize metronome state
        let metronome = Arc::new(Mutex::new(MetronomeState::new()));

        // Initialize and load drum samples
        let mut drum_samples = DrumSampleManager::new();
        drum_samples.load_drum_samples()?;
        let drum_samples = Arc::new(Mutex::new(drum_samples));

        // Create shared state
        let app_state = AppState {
            engine: engine.clone(),
            metronome: metronome.clone(),
            drum_samples: drum_samples.clone(),
        };

        // Setup audio stream
        let audio_stream = setup_audio_stream(app_state.clone())?;

        Ok(GuitarBuddy {
            app_state,
            _audio_stream: audio_stream,
        })
    }

    fn trigger_click(&self, is_accent: bool, beat_number: u8) {
        let mut metronome = self.app_state.metronome.lock().unwrap();
        let drum_samples = self.app_state.drum_samples.lock().unwrap();

        // Use different sound for accents to make them clearly distinct
        let (waveform, frequency, envelope) = if is_accent && metronome.accent_first_beat {
            // For accents, use a more prominent sound
            self.get_accent_sound(&metronome, &drum_samples)
        } else {
            // Regular click
            metronome.click_type.get_sound_params(&drum_samples)
        };

        let volume = metronome.volume;
        let click_type = metronome.click_type;

        // Record beat event for visualizer coupling
        let beat_event = BeatEvent {
            beat_number,
            accent: is_accent,
            samples: vec![click_type],
            timestamp: Instant::now(),
        };
        metronome.beat_tracker.record_beat(beat_event);

        drop(metronome);
        drop(drum_samples);

        let mut engine = self.app_state.engine.lock().unwrap();
        engine.trigger_note_with_volume(waveform, frequency, envelope, volume);
    }

    fn get_accent_sound(&self, metronome: &MetronomeState, drum_samples: &DrumSampleManager) -> (Waveform, f32, AdsrEnvelope) {
        // Choose accent sound based on the current click type
        match metronome.click_type {
            // For drum samples, use kick drum for accent
            ClickType::AcousticSnare | ClickType::HiHatClosed | ClickType::HiHatOpen |
            ClickType::RimShot | ClickType::Stick => {
                ClickType::AcousticKick.get_sound_params(drum_samples)
            }
            // For kick drum, use snare for accent
            ClickType::AcousticKick => {
                ClickType::AcousticSnare.get_sound_params(drum_samples)
            }
            // For synthetic sounds, use higher pitch and different waveform
            ClickType::WoodBlock => (
                Waveform::Square, // Different waveform
                1600.0,           // Higher pitch
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.1,
                    sustain_level: 0.0,
                    release_secs: 0.05,
                }
            ),
            ClickType::DigitalBeep => (
                Waveform::Square, // Different waveform
                2000.0,           // Much higher pitch
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.12,
                    sustain_level: 0.0,
                    release_secs: 0.06,
                }
            ),
            ClickType::Cowbell => (
                Waveform::Triangle, // Different waveform
                1600.0,             // Higher pitch
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.2,
                    sustain_level: 0.0,
                    release_secs: 0.15,
                }
            ),
            ClickType::ElectroClick => (
                Waveform::Sine, // Different waveform
                2400.0,         // Much higher pitch
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.06,
                    sustain_level: 0.0,
                    release_secs: 0.04,
                }
            ),
        }
    }

    /// Trigger a specific drum sample for pattern playback
    fn trigger_pattern_sample(&self, click_type: ClickType, is_accent: bool, beat_number: u8, samples: Vec<ClickType>) {
        let mut metronome = self.app_state.metronome.lock().unwrap();
        let drum_samples = self.app_state.drum_samples.lock().unwrap();

        let (waveform, frequency, envelope) = click_type.get_sound_params(&drum_samples);

        // Pattern accents need volume boost since they use same samples, unlike metronome which uses different sounds
        let volume = if is_accent {
            (metronome.volume * 1.5).min(1.0)  // 50% louder for pattern accents
        } else {
            metronome.volume
        };

        // Record beat event for visualizer coupling (only once per beat, not per sample)
        if click_type == samples[0] {  // Only record event for the first sample in the group
            let beat_event = BeatEvent {
                beat_number,
                accent: is_accent,
                samples: samples.clone(),
                timestamp: Instant::now(),
            };
            metronome.beat_tracker.record_beat(beat_event);
        }

        drop(metronome);
        drop(drum_samples);

        let mut engine = self.app_state.engine.lock().unwrap();
        engine.trigger_note_with_volume(waveform, frequency, envelope, volume);
    }
}

impl eframe::App for GuitarBuddy {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for metronome beats and drum patterns
        {
            let mut metronome = self.app_state.metronome.lock().unwrap();

            if metronome.pattern_mode {
                // Pattern mode: only play drum patterns
                let pattern_triggers = metronome.check_pattern_triggers();
                if !pattern_triggers.is_empty() {
                    // Collect beat information for BeatTracker event
                    let beat_number = metronome.pattern_state.get_current_beat_number();
                    let has_accent = pattern_triggers.iter().any(|(_, is_accent)| *is_accent);
                    let all_samples: Vec<ClickType> = pattern_triggers.iter().map(|(click_type, _)| *click_type).collect();

                    // Visualizer state is now handled by BeatTracker (no manual updates needed)

                    drop(metronome);
                    for (click_type, is_accent) in pattern_triggers {
                        self.trigger_pattern_sample(click_type, is_accent, beat_number, all_samples.clone());
                    }
                }
            } else {
                // Metronome mode: only play metronome clicks
                if metronome.should_trigger_beat() {
                    let is_accent = metronome.current_beat == 1;
                    let beat_number = metronome.current_beat;

                    // Visualizer state is now handled by BeatTracker (no manual updates needed)

                    drop(metronome);
                    self.trigger_click(is_accent, beat_number);
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎸 Guitar Buddy - Practice Companion");
            ui.separator();

            // Metronome status display
            let metronome = self.app_state.metronome.lock().unwrap();
            let is_playing = metronome.is_playing;
            let current_beat = metronome.current_beat;
            let time_sig = metronome.time_signature;
            let tempo = metronome.tempo_bpm;
            drop(metronome);

            ui.horizontal(|ui| {
                if is_playing {
                    ui.colored_label(egui::Color32::GREEN, "♪ PLAYING");
                    ui.separator();
                    ui.label(format!("Beat: {}/{}", current_beat, time_sig.beats_per_measure));
                } else {
                    ui.colored_label(egui::Color32::GRAY, "⏸ STOPPED");
                }
                ui.separator();
                ui.label(format!("Tempo: {:.0} BPM", tempo));
                ui.separator();
                ui.label(format!("Time: {}", time_sig.display()));
            });

            ui.separator();

            // Transport controls
            ui.horizontal(|ui| {
                let mut metronome = self.app_state.metronome.lock().unwrap();

                if metronome.is_playing {
                    if ui.button("⏸ Pause").clicked() {
                        metronome.pause();
                    }
                    if ui.button("⏹ Stop").clicked() {
                        metronome.stop();
                    }
                } else {
                    if ui.button("▶ Start").clicked() {
                        metronome.start();
                    }
                    if metronome.last_beat_time.is_some() {
                        if ui.button("▶ Resume").clicked() {
                            metronome.resume();
                        }
                    }
                }
            });

            ui.separator();

            // Tempo control
            ui.horizontal(|ui| {
                ui.label("Tempo (BPM):");
                let mut metronome = self.app_state.metronome.lock().unwrap();
                ui.add(egui::Slider::new(&mut metronome.tempo_bpm, 40.0..=200.0)
                    .step_by(1.0)
                    .suffix(" BPM"));

                // Preset tempo buttons
                ui.separator();
                for &(name, bpm) in &[("Slow", 60.0), ("Med", 120.0), ("Fast", 160.0)] {
                    if ui.button(name).clicked() {
                        metronome.tempo_bpm = bpm;
                    }
                }
            });

            ui.separator();

            // Time signature selection
            ui.horizontal(|ui| {
                ui.label("Time Signature:");
                let mut metronome = self.app_state.metronome.lock().unwrap();
                let mut current_sig = metronome.time_signature;

                for &(name, sig) in &TimeSignature::common_signatures() {
                    if ui.radio_value(&mut current_sig, sig, name).clicked() {
                        metronome.time_signature = current_sig;
                        metronome.current_beat = 0; // Reset beat counter
                    }
                }
            });

            ui.separator();

            // Click sound selection
            ui.horizontal(|ui| {
                ui.label("Click Sound:");
                let mut metronome = self.app_state.metronome.lock().unwrap();
                let mut current_click = metronome.click_type;

                for &click_type in &ClickType::all() {
                    if ui.radio_value(&mut current_click, click_type, click_type.name()).clicked() {
                        metronome.click_type = current_click;
                    }
                }
            });

            ui.separator();

            // Volume and accent controls
            ui.horizontal(|ui| {
                ui.label("Volume:");
                let mut metronome = self.app_state.metronome.lock().unwrap();
                ui.add(egui::Slider::new(&mut metronome.volume, 0.0..=1.0)
                    .step_by(0.01)
                    .suffix("%"));

                ui.separator();
                ui.checkbox(&mut metronome.accent_first_beat, "Accent first beat");
            });

            ui.separator();

            // Test click button
            ui.horizontal(|ui| {
                if ui.button("🔊 Test Click").clicked() {
                    let metronome = self.app_state.metronome.lock().unwrap();
                    let drum_samples = self.app_state.drum_samples.lock().unwrap();
                    let (waveform, frequency, envelope) = metronome.click_type.get_sound_params(&drum_samples);
                    drop(metronome);
                    drop(drum_samples);

                    let mut engine = self.app_state.engine.lock().unwrap();
                    engine.trigger_note(waveform, frequency, envelope);
                }

                if ui.button("🔊 Test Accent").clicked() {
                    self.trigger_click(true, 1);  // Test accent as beat 1
                }
            });

            ui.separator();

            // Phase 2: Drum Pattern Library
            ui.collapsing("🥁 Drum Patterns (Phase 2)", |ui| {
                let mut metronome = self.app_state.metronome.lock().unwrap();

                ui.horizontal(|ui| {
                    ui.label("Mode:");
                    let mut pattern_mode = metronome.pattern_mode;
                    if ui.radio_value(&mut pattern_mode, false, "Metronome").clicked() {
                        metronome.clear_pattern();
                    }
                    if ui.radio_value(&mut pattern_mode, true, "Drum Pattern").clicked() && !metronome.pattern_mode {
                        // Set default pattern when switching to pattern mode
                        let was_playing = metronome.is_playing;
                        metronome.set_pattern(DrumPattern::basic_rock());
                        // If metronome was playing, continue playing in pattern mode
                        if was_playing {
                            metronome.pattern_state.start();
                        }
                    }
                });

                if metronome.pattern_mode {
                    ui.separator();
                    ui.label("Available Patterns:");

                    let mut current_pattern_name = metronome.pattern_state.current_pattern
                        .as_ref()
                        .map(|p| p.name.clone())
                        .unwrap_or_else(|| "None".to_string());

                    for pattern in DrumPattern::all_patterns() {
                        if ui.radio_value(&mut current_pattern_name, pattern.name.clone(), &pattern.name).clicked() {
                            metronome.set_pattern(pattern.clone());
                        }
                    }

                    ui.separator();
                    if let Some(ref pattern) = metronome.pattern_state.current_pattern {
                        ui.label(format!("Time: {}", pattern.time_signature.display()));
                        ui.label(format!("Tempo Range: {}-{} BPM", pattern.tempo_range.0, pattern.tempo_range.1));

                        // Show current pattern position if playing
                        if metronome.pattern_state.pattern_enabled {
                            ui.label(format!("Position: {:.1}", metronome.pattern_state.get_current_beat_position()));
                        }
                    }
                }

                drop(metronome);
            });

            ui.separator();

            // Beat visualization
            ui.collapsing("Beat Visualization", |ui| {
                ui.horizontal(|ui| {
                    let metronome = self.app_state.metronome.lock().unwrap();

                    // Use last triggered beat for both metronome and pattern modes (coupled with audio)
                    let (current_beat_display, time_sig) = if metronome.pattern_mode {
                        // For pattern mode, use BeatTracker for last triggered beat
                        let (tracked_beat, _) = metronome.beat_tracker.get_current_beat();
                        let pattern_beat = if metronome.is_playing && tracked_beat > 0 {
                            tracked_beat
                        } else {
                            1 // Default to beat 1 when not playing or no triggers yet
                        };
                        let time_sig = metronome.pattern_state.current_pattern
                            .as_ref()
                            .map(|p| p.time_signature)
                            .unwrap_or(metronome.time_signature);
                        (pattern_beat, time_sig)
                    } else {
                        // For metronome mode, use BeatTracker for last triggered beat
                        let (tracked_beat, _) = metronome.beat_tracker.get_current_beat();
                        let metronome_beat = if metronome.is_playing && tracked_beat > 0 {
                            tracked_beat
                        } else {
                            current_beat // Fallback to current_beat if no triggers yet
                        };
                        (metronome_beat, metronome.time_signature)
                    };

                    for beat in 1..=time_sig.beats_per_measure {
                        let is_current = beat == current_beat_display && is_playing;

                        // Check accents: use actual triggered accent when showing current beat, pattern definition otherwise
                        let is_accent = if is_current && metronome.is_playing {
                            // For current beat, use BeatTracker accent status (coupled with audio)
                            let (_, tracked_accent) = metronome.beat_tracker.get_current_beat();
                            tracked_accent
                        } else if metronome.pattern_mode {
                            // For non-current beats in pattern mode, show pattern definition
                            if let Some(ref pattern) = metronome.pattern_state.current_pattern {
                                pattern.beats.iter().any(|pattern_beat| {
                                    let beat_num = pattern_beat.beat_position.floor() as u8;
                                    beat_num == beat && pattern_beat.accent
                                })
                            } else {
                                false
                            }
                        } else {
                            // For metronome mode, use metronome setting
                            beat == 1 && metronome.accent_first_beat
                        };

                        let color = if is_current && is_accent {
                            egui::Color32::YELLOW
                        } else if is_current {
                            egui::Color32::GREEN
                        } else if is_accent {
                            egui::Color32::LIGHT_GRAY
                        } else {
                            egui::Color32::GRAY
                        };

                        let symbol = if is_accent { "●" } else { "○" };
                        ui.colored_label(color, symbol);
                    }
                });

                let metronome = self.app_state.metronome.lock().unwrap();
                ui.label(format!("Interval: {:.0}ms between beats", metronome.beat_interval_ms()));

                // Show additional pattern info if in pattern mode
                if metronome.pattern_mode && metronome.pattern_state.pattern_enabled {
                    ui.label(format!("Pattern Position: {:.2}", metronome.pattern_state.get_current_beat_position()));
                }
            });

            ui.separator();

            // Phase 2 preview
            ui.collapsing("Coming in Phase 2", |ui| {
                ui.label("🥁 Drum patterns and backing tracks");
                ui.label("🎹 Piano chord progressions");
                ui.label("🎸 Bass line accompaniment");
                ui.label("🎵 Key and chord change management");
                ui.label("📚 Practice session recording");
                ui.label("🎯 Tempo trainer with gradual speed changes");
            });
        });

        // Request repaint for smooth beat timing
        ctx.request_repaint_after(Duration::from_millis(10));
    }
}

/// Setup CPAL audio stream for real-time metronome output
fn setup_audio_stream(app_state: AppState) -> Result<Stream, Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let device = host.default_output_device()
        .ok_or("No audio output device available")?;

    let config = device.default_output_config()?;

    println!("🎸 Guitar Buddy Audio System");
    println!("Audio device: {}", device.name()?);
    println!("Sample rate: {} Hz", config.sample_rate().0);
    println!("Channels: {}", config.channels());

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => create_stream::<f32>(&device, &config.into(), app_state),
        cpal::SampleFormat::I16 => create_stream::<i16>(&device, &config.into(), app_state),
        cpal::SampleFormat::U16 => create_stream::<u16>(&device, &config.into(), app_state),
        _ => return Err("Unsupported audio format".into()),
    }?;

    stream.play()?;
    Ok(stream)
}

/// Create audio stream for specific sample format
fn create_stream<T>(
    device: &Device,
    config: &StreamConfig,
    app_state: AppState,
) -> Result<Stream, Box<dyn std::error::Error>>
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    let channels = config.channels as usize;

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            let mut f32_buffer = vec![0.0f32; data.len()];

            // Process audio with the engine
            {
                let mut engine = app_state.engine.lock().unwrap();
                if channels == 1 {
                    engine.process_buffer(&mut f32_buffer);
                } else {
                    engine.process_stereo_buffer(&mut f32_buffer);
                }
            }

            // Convert back to target format
            for (dst, &src) in data.iter_mut().zip(f32_buffer.iter()) {
                *dst = T::from_sample(src);
            }
        },
        |err| eprintln!("Audio stream error: {}", err),
        None,
    )?;

    Ok(stream)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎸 Guitar Buddy - Musical Practice Companion");
    println!("============================================");
    println!("Phase 1: Advanced Metronome");
    println!("Initializing audio system...");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([700.0, 500.0])
            .with_title("Guitar Buddy - Practice Companion")
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Guitar Buddy",
        options,
        Box::new(|cc| {
            match GuitarBuddy::new(cc) {
                Ok(app) => {
                    println!("✅ Guitar Buddy initialized successfully!");
                    println!("🎵 Audio output active - ready to rock!");
                    Ok(Box::new(app))
                }
                Err(e) => {
                    eprintln!("❌ Failed to initialize Guitar Buddy: {}", e);
                    std::process::exit(1);
                }
            }
        }),
    )
    .map_err(|e| format!("GUI error: {}", e).into())
}