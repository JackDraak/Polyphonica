/// Core timing types shared across all timing components
///
/// This module defines the fundamental data structures used throughout the timing
/// subsystem. These types are designed to be lightweight, copyable, and suitable
/// for real-time audio processing.

use std::time::Instant;

/// Time signature representation for musical timing
///
/// Represents musical time signatures like 4/4, 3/4, 6/8, etc.
/// The beats_per_measure indicates how many beats are in each measure,
/// while note_value indicates what note value gets the beat.
///
/// # Examples
///
/// ```rust
/// use polyphonica::timing::TimeSignature;
///
/// let four_four = TimeSignature::new(4, 4);  // 4/4 time
/// let waltz = TimeSignature::new(3, 4);      // 3/4 waltz time
/// let compound = TimeSignature::new(6, 8);   // 6/8 compound time
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TimeSignature {
    pub beats_per_measure: u8,
    pub note_value: u8, // 4 = quarter note, 8 = eighth note, etc.
}

impl TimeSignature {
    /// Create a new time signature
    pub fn new(beats: u8, note_value: u8) -> Self {
        Self {
            beats_per_measure: beats,
            note_value,
        }
    }

    /// Get common time signatures with their display names
    pub fn common_signatures() -> Vec<(&'static str, TimeSignature)> {
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

    /// Get a display string for this time signature
    pub fn display(&self) -> String {
        format!("{}/{}", self.beats_per_measure, self.note_value)
    }

    /// Calculate the duration of one beat in milliseconds at given tempo
    pub fn beat_duration_ms(&self, tempo_bpm: f32) -> f64 {
        60000.0 / tempo_bpm as f64
    }

    /// Calculate the duration of one measure in milliseconds at given tempo
    pub fn measure_duration_ms(&self, tempo_bpm: f32) -> f64 {
        self.beat_duration_ms(tempo_bpm) * self.beats_per_measure as f64
    }
}

/// Different metronome click sound types
///
/// Represents the various sounds that can be used for metronome clicks
/// and drum pattern samples. Includes both synthetic waveforms and
/// real drum samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ClickType {
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
    RimShot,        // Snare rim shot
    Stick,          // Drumstick click
}

impl ClickType {
    /// Get all available click types
    pub fn all() -> Vec<ClickType> {
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

    /// Get the display name for this click type
    pub fn name(self) -> &'static str {
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

    /// Check if this is a synthetic sound (vs. a real sample)
    pub fn is_synthetic(self) -> bool {
        matches!(self,
            ClickType::WoodBlock |
            ClickType::DigitalBeep |
            ClickType::Cowbell |
            ClickType::ElectroClick
        )
    }

    /// Get synthetic sounds only
    pub fn synthetic_sounds() -> Vec<ClickType> {
        vec![
            ClickType::WoodBlock,
            ClickType::DigitalBeep,
            ClickType::Cowbell,
            ClickType::ElectroClick,
        ]
    }

    /// Get real drum samples only
    pub fn drum_samples() -> Vec<ClickType> {
        vec![
            ClickType::AcousticKick,
            ClickType::AcousticSnare,
            ClickType::HiHatClosed,
            ClickType::HiHatOpen,
            ClickType::RimShot,
            ClickType::Stick,
        ]
    }
}

/// Beat event for coupling audio triggers with visualization
///
/// Represents a single beat event that was triggered by the timing system.
/// This is used to couple audio playback with visual feedback through the
/// observer pattern.
///
/// # Design Notes
///
/// The BeatEvent captures the essential information about when a beat occurred,
/// what samples were triggered, and whether it was accented. This allows the
/// visualizer to show exactly what the user is hearing.
#[derive(Debug, Clone)]
pub struct BeatEvent {
    /// 1-based beat number within the measure (1, 2, 3, 4 for 4/4 time)
    pub beat_number: u8,

    /// Whether this beat is accented (emphasized)
    pub accent: bool,

    /// Audio samples that were triggered for this beat
    pub samples: Vec<ClickType>,

    /// When this beat was triggered (for timing analysis)
    pub timestamp: Instant,

    /// Current tempo when this beat was triggered (BPM)
    pub tempo_bpm: f32,

    /// Time signature when this beat was triggered
    pub time_signature: TimeSignature,
}

impl BeatEvent {
    /// Create a new beat event
    pub fn new(
        beat_number: u8,
        accent: bool,
        samples: Vec<ClickType>,
        tempo_bpm: f32,
        time_signature: TimeSignature,
    ) -> Self {
        Self {
            beat_number,
            accent,
            samples,
            timestamp: Instant::now(),
            tempo_bpm,
            time_signature,
        }
    }

    /// Check if this is the first beat of a measure (downbeat)
    pub fn is_downbeat(&self) -> bool {
        self.beat_number == 1
    }

    /// Get the expected interval to the next beat in milliseconds
    pub fn expected_next_beat_interval_ms(&self) -> f64 {
        self.time_signature.beat_duration_ms(self.tempo_bpm)
    }

    /// Check if any drum samples were triggered (vs. only synthetic sounds)
    pub fn has_drum_samples(&self) -> bool {
        self.samples.iter().any(|sample| !sample.is_synthetic())
    }
}

/// Result of checking for timing triggers
///
/// This represents what should happen when a timing system checks if it's
/// time to trigger any beats. It can contain multiple events for complex
/// patterns that trigger multiple samples simultaneously.
pub type TriggerResult = Vec<BeatEvent>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_signature_creation() {
        let four_four = TimeSignature::new(4, 4);
        assert_eq!(four_four.beats_per_measure, 4);
        assert_eq!(four_four.note_value, 4);
        assert_eq!(four_four.display(), "4/4");
    }

    #[test]
    fn test_time_signature_durations() {
        let four_four = TimeSignature::new(4, 4);
        let tempo_120 = 120.0;

        // At 120 BPM, each beat should be 500ms
        assert_eq!(four_four.beat_duration_ms(tempo_120), 500.0);

        // 4 beats per measure = 2000ms per measure
        assert_eq!(four_four.measure_duration_ms(tempo_120), 2000.0);
    }

    #[test]
    fn test_click_type_categorization() {
        assert!(ClickType::WoodBlock.is_synthetic());
        assert!(!ClickType::AcousticKick.is_synthetic());

        let synthetic = ClickType::synthetic_sounds();
        assert!(synthetic.contains(&ClickType::WoodBlock));
        assert!(!synthetic.contains(&ClickType::AcousticKick));

        let drums = ClickType::drum_samples();
        assert!(!drums.contains(&ClickType::WoodBlock));
        assert!(drums.contains(&ClickType::AcousticKick));
    }

    #[test]
    fn test_beat_event_creation() {
        let event = BeatEvent::new(
            1,
            true,
            vec![ClickType::AcousticKick],
            120.0,
            TimeSignature::new(4, 4),
        );

        assert_eq!(event.beat_number, 1);
        assert!(event.accent);
        assert!(event.is_downbeat());
        assert!(event.has_drum_samples());
        assert_eq!(event.expected_next_beat_interval_ms(), 500.0);
    }
}