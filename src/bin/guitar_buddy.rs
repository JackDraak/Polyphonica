/// Guitar Buddy - Musical Practice Companion
///
/// Phase 1: Advanced metronome with multiple click sounds and time signatures
/// Phase 2: Full accompaniment with drums, bass lines, and chord progressions
///
/// Uses Polyphonica real-time synthesis engine for precise, low-latency audio generation.

use polyphonica::{RealtimeEngine, Waveform, AdsrEnvelope, SampleData};
use polyphonica::timing::{BeatClock, Metronome as NewMetronome, TimeSignature as TimingTimeSignature, ClickType as TimingClickType};
use polyphonica::patterns::{DrumPattern, PatternState, PatternLibrary};
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

    /// Convert to new timing module TimeSignature
    fn to_timing_signature(&self) -> TimingTimeSignature {
        TimingTimeSignature::new(self.beats_per_measure, self.note_value)
    }

    /// Convert from new timing module TimeSignature
    fn from_timing_signature(ts: TimingTimeSignature) -> Self {
        Self::new(ts.beats_per_measure, ts.note_value)
    }
}

/// Drum pattern system for Phase 2 - now using polyphonica::patterns module


/// Pattern playback state - now using polyphonica::patterns::PatternState
/// (local implementation removed)


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
            // Extended drum kit samples - map to available samples
            (ClickType::KickTight, "samples/drums/acoustic/kit_01/drumkit-kick.wav"),     // Reuse kick for tight variant
            (ClickType::HiHatLoose, "samples/drums/acoustic/kit_01/drumkit-hihat-open.wav"), // Use open hi-hat for loose
            (ClickType::HiHatVeryLoose, "samples/drums/acoustic/kit_01/drumkit-hihat-open.wav"), // Use open hi-hat for very loose
            (ClickType::CymbalSplash, "samples/drums/acoustic/kit_01/drumkit-hihat-open.wav"), // Use open hi-hat for cymbal splash
            (ClickType::CymbalRoll, "samples/drums/acoustic/kit_01/drumkit-hihat-open.wav"),   // Use open hi-hat for cymbal roll
            (ClickType::Ride, "samples/drums/acoustic/kit_01/drumkit-hihat.wav"),       // Use closed hi-hat for ride
            (ClickType::RideBell, "samples/drums/acoustic/kit_01/drumkit-stick.wav"),   // Use stick for ride bell
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

}

/// Beat event for coupling audio triggers with visualizer updates
/// Note: samples and timestamp fields are populated but not yet used -
/// they're intended for future analysis and debugging features
#[derive(Debug, Clone)]
struct BeatEvent {
    beat_number: u8,         // 1-based beat number (1, 2, 3, 4)
    accent: bool,            // Whether this beat is accented
    samples: Vec<ClickType>, // Audio samples that were triggered (for future use)
    timestamp: Instant,      // When this beat was triggered (for future use)
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
    // Extended drum kit samples (from JSON catalog)
    KickTight,      // Tight, punchy kick drum variant
    HiHatLoose,     // Loose hi-hat with medium decay
    HiHatVeryLoose, // Very loose hi-hat with long decay
    CymbalSplash,   // Splash cymbal for accents
    CymbalRoll,     // Cymbal roll/crash
    Ride,           // Ride cymbal for rhythm patterns
    RideBell,       // Ride bell for accents and highlights
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
            // Extended drum kit samples
            ClickType::KickTight,
            ClickType::HiHatLoose,
            ClickType::HiHatVeryLoose,
            ClickType::CymbalSplash,
            ClickType::CymbalRoll,
            ClickType::Ride,
            ClickType::RideBell,
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
            // Extended drum kit samples
            ClickType::KickTight => "Kick Tight",
            ClickType::HiHatLoose => "Hi-Hat Loose",
            ClickType::HiHatVeryLoose => "Hi-Hat Very Loose",
            ClickType::CymbalSplash => "Cymbal Splash",
            ClickType::CymbalRoll => "Cymbal Roll",
            ClickType::Ride => "Ride Cymbal",
            ClickType::RideBell => "Ride Bell",
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
            // Extended drum kit samples
            ClickType::KickTight => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 0.8,     // Slightly shorter than regular kick
                sustain_level: 0.0,
                release_secs: 0.001,
            },
            ClickType::HiHatLoose => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 0.5,     // Medium decay for loose hi-hat
                sustain_level: 0.0,
                release_secs: 0.001,
            },
            ClickType::HiHatVeryLoose => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 1.2,     // Longer decay for very loose
                sustain_level: 0.0,
                release_secs: 0.001,
            },
            ClickType::CymbalSplash => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 1.5,     // Long splash decay
                sustain_level: 0.0,
                release_secs: 0.001,
            },
            ClickType::CymbalRoll => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 2.0,     // Extended roll decay
                sustain_level: 0.0,
                release_secs: 0.001,
            },
            ClickType::Ride => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 0.8,     // Ride cymbal sustain
                sustain_level: 0.0,
                release_secs: 0.001,
            },
            ClickType::RideBell => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 0.3,     // Short bell ping
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
            // Extended drum kit samples - synthetic fallbacks
            ClickType::KickTight => (
                Waveform::Sine,
                80.0,  // Slightly higher than regular kick
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.2,   // Shorter decay for tight kick
                    sustain_level: 0.0,
                    release_secs: 0.05,
                }
            ),
            ClickType::HiHatLoose => (
                Waveform::Pulse { duty_cycle: 0.2 },
                5000.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.4,   // Medium decay
                    sustain_level: 0.0,
                    release_secs: 0.15,
                }
            ),
            ClickType::HiHatVeryLoose => (
                Waveform::Pulse { duty_cycle: 0.3 },
                4000.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.8,   // Long decay
                    sustain_level: 0.0,
                    release_secs: 0.3,
                }
            ),
            ClickType::CymbalSplash => (
                Waveform::Noise,
                4000.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 1.0,   // Splash decay
                    sustain_level: 0.0,
                    release_secs: 0.4,
                }
            ),
            ClickType::CymbalRoll => (
                Waveform::Noise,
                3000.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 1.5,   // Extended roll
                    sustain_level: 0.0,
                    release_secs: 0.6,
                }
            ),
            ClickType::Ride => (
                Waveform::Triangle,
                2000.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.5,   // Ride sustain
                    sustain_level: 0.0,
                    release_secs: 0.2,
                }
            ),
            ClickType::RideBell => (
                Waveform::Sine,
                3000.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.3,   // Bell ping
                    sustain_level: 0.0,
                    release_secs: 0.1,
                }
            ),
        }
    }
}

/// Metronome state and timing control
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
    pattern_library: PatternLibrary,
    // Phase 2: Beat tracking for event-driven visualizer coupling
    beat_tracker: BeatTracker,
    // New timing module (Phase 1 refactoring)
    new_metronome: NewMetronome,
}

impl MetronomeState {
    fn new() -> Self {
        let mut instance = Self {
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
            pattern_library: PatternLibrary::with_defaults(),
            beat_tracker: BeatTracker::new(),
            new_metronome: NewMetronome::new(TimingTimeSignature::new(4, 4)),
        };
        // Sync initial settings to new metronome
        instance.sync_to_new_metronome();
        instance
    }

    /// Sync settings to new metronome
    fn sync_to_new_metronome(&mut self) {
        self.new_metronome.set_time_signature(self.time_signature.to_timing_signature());
        // Convert ClickType
        let timing_click = match self.click_type {
            ClickType::WoodBlock => TimingClickType::WoodBlock,
            ClickType::DigitalBeep => TimingClickType::DigitalBeep,
            ClickType::Cowbell => TimingClickType::Cowbell,
            ClickType::ElectroClick => TimingClickType::ElectroClick,
            ClickType::AcousticKick => TimingClickType::AcousticKick,
            ClickType::AcousticSnare => TimingClickType::AcousticSnare,
            ClickType::HiHatClosed => TimingClickType::HiHatClosed,
            ClickType::HiHatOpen => TimingClickType::HiHatOpen,
            ClickType::RimShot => TimingClickType::RimShot,
            ClickType::Stick => TimingClickType::Stick,
            ClickType::KickTight => TimingClickType::KickTight,
            ClickType::HiHatLoose => TimingClickType::HiHatLoose,
            ClickType::HiHatVeryLoose => TimingClickType::HiHatVeryLoose,
            ClickType::CymbalSplash => TimingClickType::CymbalSplash,
            ClickType::CymbalRoll => TimingClickType::CymbalRoll,
            ClickType::Ride => TimingClickType::Ride,
            ClickType::RideBell => TimingClickType::RideBell,
        };
        self.new_metronome.set_click_type(timing_click);
        self.new_metronome.set_accent_first_beat(self.accent_first_beat);
    }

    /// Calculate time between beats in milliseconds
    fn beat_interval_ms(&self) -> f64 {
        60000.0 / self.tempo_bpm as f64
    }

    /// Check if it's time for the next beat (using new timing module)
    fn should_trigger_beat(&mut self) -> bool {
        if !self.is_playing {
            return false;
        }

        // Use new metronome timing with discrete scheduling
        let events = self.new_metronome.check_triggers(self.tempo_bpm);
        if !events.is_empty() {
            // Update current beat to match new metronome
            self.current_beat = events[0].beat_number;
            self.last_beat_time = Some(events[0].timestamp);
            true
        } else {
            false
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
            // Convert PatternTrigger to (ClickType, bool)
            self.pattern_state.check_pattern_triggers(self.tempo_bpm)
                .into_iter()
                .map(|trigger| (self.convert_timing_click_type(trigger.click_type), trigger.is_accent))
                .collect()
        } else {
            vec![]
        }
    }

    /// Convert timing module ClickType to local ClickType
    fn convert_timing_click_type(&self, timing_click: TimingClickType) -> ClickType {
        match timing_click {
            TimingClickType::WoodBlock => ClickType::WoodBlock,
            TimingClickType::DigitalBeep => ClickType::DigitalBeep,
            TimingClickType::Cowbell => ClickType::Cowbell,
            TimingClickType::ElectroClick => ClickType::ElectroClick,
            TimingClickType::AcousticKick => ClickType::AcousticKick,
            TimingClickType::AcousticSnare => ClickType::AcousticSnare,
            TimingClickType::HiHatClosed => ClickType::HiHatClosed,
            TimingClickType::HiHatOpen => ClickType::HiHatOpen,
            TimingClickType::RimShot => ClickType::RimShot,
            TimingClickType::Stick => ClickType::Stick,
            TimingClickType::KickTight => ClickType::KickTight,
            TimingClickType::HiHatLoose => ClickType::HiHatLoose,
            TimingClickType::HiHatVeryLoose => ClickType::HiHatVeryLoose,
            TimingClickType::CymbalSplash => ClickType::CymbalSplash,
            TimingClickType::CymbalRoll => ClickType::CymbalRoll,
            TimingClickType::Ride => ClickType::Ride,
            TimingClickType::RideBell => ClickType::RideBell,
        }
    }

    fn start(&mut self) {
        self.is_playing = true;
        self.current_beat = 0;
        self.last_beat_time = None;
        // Start new metronome with updated settings
        self.new_metronome.start();
        if self.pattern_mode {
            self.pattern_state.start();
        }
    }

    fn stop(&mut self) {
        self.is_playing = false;
        self.current_beat = 0;
        self.last_beat_time = None;
        // Stop new metronome
        self.new_metronome.stop();
        if self.pattern_mode {
            self.pattern_state.stop();
        }
    }

    fn pause(&mut self) {
        self.is_playing = false;
        // Pause new metronome
        self.new_metronome.pause();
        if self.pattern_mode {
            self.pattern_state.stop();
        }
    }

    fn resume(&mut self) {
        self.is_playing = true;
        self.last_beat_time = Some(Instant::now());
        // Resume new metronome
        self.new_metronome.resume();
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

impl AppState {
    fn new(
        engine: Arc<Mutex<RealtimeEngine>>,
        metronome: Arc<Mutex<MetronomeState>>,
        drum_samples: Arc<Mutex<DrumSampleManager>>,
    ) -> Self {
        Self {
            engine,
            metronome,
            drum_samples,
        }
    }
}

/// GUI Components Module
mod gui_components {
    use super::*;
    use egui::{Ui, Color32, Slider};

    /// Status display panel component
    pub struct StatusDisplayPanel;

    impl StatusDisplayPanel {
        pub fn show(ui: &mut Ui, app_state: &AppState) {
            let metronome = app_state.metronome.lock().unwrap();
            let is_playing = metronome.is_playing;
            let current_beat = metronome.current_beat;
            let time_sig = metronome.time_signature;
            let tempo = metronome.tempo_bpm;
            drop(metronome);

            ui.horizontal(|ui| {
                if is_playing {
                    ui.colored_label(Color32::GREEN, "♪ PLAYING");
                    ui.separator();
                    ui.label(format!("Beat: {}/{}", current_beat, time_sig.beats_per_measure));
                } else {
                    ui.colored_label(Color32::GRAY, "⏸ STOPPED");
                }
                ui.separator();
                ui.label(format!("Tempo: {:.0} BPM", tempo));
                ui.separator();
                ui.label(format!("Time: {}", time_sig.display()));
            });
        }
    }

    /// Transport controls panel component
    pub struct TransportControlsPanel;

    impl TransportControlsPanel {
        pub fn show(ui: &mut Ui, app_state: &AppState) {
            ui.horizontal(|ui| {
                let mut metronome = app_state.metronome.lock().unwrap();

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
        }
    }

    /// Tempo control panel component
    pub struct TempoControlPanel;

    impl TempoControlPanel {
        pub fn show(ui: &mut Ui, app_state: &AppState) {
            ui.horizontal(|ui| {
                ui.label("Tempo (BPM):");
                let mut metronome = app_state.metronome.lock().unwrap();
                ui.add(Slider::new(&mut metronome.tempo_bpm, 40.0..=200.0)
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
        }
    }

    /// Time signature selection panel component
    pub struct TimeSignaturePanel;

    impl TimeSignaturePanel {
        pub fn show(ui: &mut Ui, app_state: &AppState) {
            ui.horizontal(|ui| {
                ui.label("Time Signature:");
                let mut metronome = app_state.metronome.lock().unwrap();
                let mut current_sig = metronome.time_signature;

                for &(name, sig) in &TimeSignature::common_signatures() {
                    if ui.radio_value(&mut current_sig, sig, name).clicked() {
                        metronome.time_signature = current_sig;
                        metronome.current_beat = 0; // Reset beat counter
                        // Sync to new metronome
                        metronome.sync_to_new_metronome();
                    }
                }
            });
        }
    }

    /// Click sound selection panel component
    pub struct ClickSoundPanel;

    impl ClickSoundPanel {
        pub fn show(ui: &mut Ui, app_state: &AppState) {
            ui.horizontal(|ui| {
                ui.label("Click Sound:");
                let mut metronome = app_state.metronome.lock().unwrap();
                let mut current_click = metronome.click_type;

                for &click_type in &ClickType::all() {
                    if ui.radio_value(&mut current_click, click_type, click_type.name()).clicked() {
                        metronome.click_type = current_click;
                        // Sync to new metronome
                        metronome.sync_to_new_metronome();
                    }
                }
            });
        }
    }

    /// Volume controls panel component
    pub struct VolumeControlsPanel;

    impl VolumeControlsPanel {
        pub fn show(ui: &mut Ui, app_state: &AppState) {
            ui.horizontal(|ui| {
                ui.label("Volume:");
                let mut metronome = app_state.metronome.lock().unwrap();
                ui.add(Slider::new(&mut metronome.volume, 0.0..=1.0)
                    .step_by(0.01)
                    .suffix("%"));

                ui.separator();
                ui.checkbox(&mut metronome.accent_first_beat, "Accent first beat");
            });
        }
    }

    /// Test controls panel component
    pub struct TestControlsPanel;

    impl TestControlsPanel {
        pub fn show<F>(ui: &mut Ui, app_state: &AppState, trigger_click_fn: F)
        where
            F: Fn(bool, u8) + Clone
        {
            ui.horizontal(|ui| {
                if ui.button("🔊 Test Click").clicked() {
                    let metronome = app_state.metronome.lock().unwrap();
                    let drum_samples = app_state.drum_samples.lock().unwrap();
                    let (waveform, frequency, envelope) = metronome.click_type.get_sound_params(&drum_samples);
                    drop(metronome);
                    drop(drum_samples);

                    let mut engine = app_state.engine.lock().unwrap();
                    engine.trigger_note(waveform, frequency, envelope);
                }

                if ui.button("🔊 Test Accent").clicked() {
                    let trigger_click = trigger_click_fn.clone();
                    trigger_click(true, 1);  // Test accent as beat 1
                }
            });
        }
    }

    /// Pattern selection panel component
    pub struct PatternSelectionPanel;

    impl PatternSelectionPanel {
        pub fn show(ui: &mut Ui, app_state: &AppState) {
            ui.collapsing("🥁 Drum Patterns", |ui| {
                let mut metronome = app_state.metronome.lock().unwrap();

                ui.horizontal(|ui| {
                    ui.label("Mode:");
                    let mut pattern_mode = metronome.pattern_mode;
                    if ui.radio_value(&mut pattern_mode, false, "Metronome").clicked() {
                        metronome.clear_pattern();
                    }
                    if ui.radio_value(&mut pattern_mode, true, "Drum Pattern").clicked() && !metronome.pattern_mode {
                        // Set default pattern when switching to pattern mode
                        let was_playing = metronome.is_playing;
                        let basic_rock = metronome.pattern_library.get_pattern("basic_rock").cloned();
                        if let Some(pattern) = basic_rock {
                            metronome.set_pattern(pattern);
                        }
                        // If metronome was playing, continue playing in pattern mode
                        if was_playing {
                            metronome.pattern_state.start();
                        }
                    }
                });

                if metronome.pattern_mode {
                    ui.separator();
                    ui.label("Available Patterns:");

                    let mut current_pattern_name = metronome.pattern_state.current_pattern()
                        .map(|p| p.name.clone())
                        .unwrap_or_else(|| "None".to_string());

                    let available_patterns: Vec<_> = metronome.pattern_library.all_patterns().into_iter().cloned().collect();
                    for pattern in available_patterns {
                        if ui.radio_value(&mut current_pattern_name, pattern.name.clone(), &pattern.name).clicked() {
                            metronome.set_pattern(pattern);
                        }
                    }

                    ui.separator();
                    if let Some(pattern) = metronome.pattern_state.current_pattern() {
                        ui.label(format!("Time: {}", Self::format_time_signature(&pattern.time_signature)));
                        ui.label(format!("Tempo Range: {}-{} BPM", pattern.tempo_range.0, pattern.tempo_range.1));

                        // Show current pattern position if playing
                        if metronome.pattern_state.is_playing() {
                            ui.label(format!("Position: {:.1}", metronome.pattern_state.current_beat_position()));
                        }
                    }
                }
            });
        }

        fn format_time_signature(time_sig: &TimingTimeSignature) -> String {
            format!("{}/{}", time_sig.beats_per_measure, time_sig.note_value)
        }
    }

    /// Beat visualization panel component
    pub struct BeatVisualizationPanel;

    impl BeatVisualizationPanel {
        pub fn show(ui: &mut Ui, app_state: &AppState) {
            ui.collapsing("Beat Visualization", |ui| {
                ui.horizontal(|ui| {
                    let metronome = app_state.metronome.lock().unwrap();
                    let is_playing = metronome.is_playing;
                    let current_beat = metronome.current_beat;

                    // Use last triggered beat for both metronome and pattern modes (coupled with audio)
                    let (current_beat_display, time_sig) = if metronome.pattern_mode {
                        // For pattern mode, use BeatTracker for last triggered beat
                        let (tracked_beat, _) = metronome.beat_tracker.get_current_beat();
                        let pattern_beat = if metronome.is_playing && tracked_beat > 0 {
                            tracked_beat
                        } else {
                            1 // Default to beat 1 when not playing or no triggers yet
                        };
                        let time_sig = metronome.pattern_state.current_pattern()
                            .map(|p| TimeSignature::from_timing_signature(p.time_signature))
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
                            if let Some(pattern) = metronome.pattern_state.current_pattern() {
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
                            Color32::YELLOW
                        } else if is_current {
                            Color32::GREEN
                        } else if is_accent {
                            Color32::LIGHT_GRAY
                        } else {
                            Color32::GRAY
                        };

                        let symbol = if is_accent { "●" } else { "○" };
                        ui.colored_label(color, symbol);
                    }
                });

                let metronome = app_state.metronome.lock().unwrap();
                ui.label(format!("Interval: {:.0}ms between beats", metronome.beat_interval_ms()));

                // Show additional pattern info if in pattern mode
                if metronome.pattern_mode && metronome.pattern_state.is_playing() {
                    ui.label(format!("Pattern Position: {:.2}", metronome.pattern_state.current_beat_position()));
                }
            });
        }
    }
}

use gui_components::*;

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
        let app_state = AppState::new(
            engine.clone(),
            metronome.clone(),
            drum_samples.clone(),
        );

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
            ClickType::RimShot | ClickType::Stick | ClickType::HiHatLoose |
            ClickType::HiHatVeryLoose | ClickType::CymbalSplash | ClickType::CymbalRoll |
            ClickType::Ride | ClickType::RideBell => {
                ClickType::AcousticKick.get_sound_params(drum_samples)
            }
            // For kick drum variants, use snare for accent
            ClickType::AcousticKick | ClickType::KickTight => {
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
                // Get beat number before mutable borrow
                let beat_number = metronome.pattern_state.current_beat_number();
                let pattern_triggers = metronome.check_pattern_triggers();
                if !pattern_triggers.is_empty() {
                    // Collect beat information for BeatTracker event
                    let _has_accent = pattern_triggers.iter().any(|(_, is_accent)| *is_accent);
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

            // Status display panel
            StatusDisplayPanel::show(ui, &self.app_state);

            ui.separator();

            // Transport controls panel
            TransportControlsPanel::show(ui, &self.app_state);

            ui.separator();

            // Tempo control panel
            TempoControlPanel::show(ui, &self.app_state);

            ui.separator();

            // Time signature panel
            TimeSignaturePanel::show(ui, &self.app_state);

            ui.separator();

            // Click sound panel
            ClickSoundPanel::show(ui, &self.app_state);

            ui.separator();

            // Volume controls panel
            VolumeControlsPanel::show(ui, &self.app_state);

            ui.separator();

            // Test controls panel
            let trigger_click_fn = |is_accent: bool, beat_number: u8| {
                self.trigger_click(is_accent, beat_number);
            };
            TestControlsPanel::show(ui, &self.app_state, trigger_click_fn);

            ui.separator();

            // Pattern selection panel
            PatternSelectionPanel::show(ui, &self.app_state);

            ui.separator();

            // Beat visualization panel
            BeatVisualizationPanel::show(ui, &self.app_state);

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