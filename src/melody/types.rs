/// Core types for the melody assistant module
///
/// This module defines the fundamental data structures for representing
/// musical concepts: notes, chords, keys, and timing relationships.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Musical note representation (chromatic scale)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Note {
    /// The note C
    C,
    /// The note C# (C sharp)
    CSharp,
    /// The note D
    D,
    /// The note D# (D sharp)
    DSharp,
    /// The note E
    E,
    /// The note F
    F,
    /// The note F# (F sharp)
    FSharp,
    /// The note G
    G,
    /// The note G# (G sharp)
    GSharp,
    /// The note A
    A,
    /// The note A# (A sharp)
    ASharp,
    /// The note B
    B,
}

impl Note {
    /// Get all 12 chromatic notes in order
    pub fn all() -> [Note; 12] {
        [
            Note::C, Note::CSharp, Note::D, Note::DSharp,
            Note::E, Note::F, Note::FSharp, Note::G,
            Note::GSharp, Note::A, Note::ASharp, Note::B
        ]
    }

    /// Get note name as string
    pub fn name(&self) -> &'static str {
        match self {
            Note::C => "C",
            Note::CSharp => "C#",
            Note::D => "D",
            Note::DSharp => "D#",
            Note::E => "E",
            Note::F => "F",
            Note::FSharp => "F#",
            Note::G => "G",
            Note::GSharp => "G#",
            Note::A => "A",
            Note::ASharp => "A#",
            Note::B => "B",
        }
    }

    /// Get note as integer (0-11, C=0)
    pub fn as_semitone(&self) -> u8 {
        match self {
            Note::C => 0,
            Note::CSharp => 1,
            Note::D => 2,
            Note::DSharp => 3,
            Note::E => 4,
            Note::F => 5,
            Note::FSharp => 6,
            Note::G => 7,
            Note::GSharp => 8,
            Note::A => 9,
            Note::ASharp => 10,
            Note::B => 11,
        }
    }

    /// Create note from semitone (0-11)
    pub fn from_semitone(semitone: u8) -> Note {
        match semitone % 12 {
            0 => Note::C,
            1 => Note::CSharp,
            2 => Note::D,
            3 => Note::DSharp,
            4 => Note::E,
            5 => Note::F,
            6 => Note::FSharp,
            7 => Note::G,
            8 => Note::GSharp,
            9 => Note::A,
            10 => Note::ASharp,
            11 => Note::B,
            _ => unreachable!(),
        }
    }

    /// Transpose note by semitones
    pub fn transpose(&self, semitones: i8) -> Note {
        let current = self.as_semitone() as i8;
        let new_semitone = (current + semitones).rem_euclid(12) as u8;
        Note::from_semitone(new_semitone)
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Quality of a chord (major, minor, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChordQuality {
    /// Major triad (1-3-5)
    Major,
    /// Minor triad (1-♭3-5)
    Minor,
    /// Diminished triad (1-♭3-♭5)
    Diminished,
    /// Augmented triad (1-3-♯5)
    Augmented,
    /// Major seventh chord (1-3-5-7)
    Major7,
    /// Minor seventh chord (1-♭3-5-♭7)
    Minor7,
    /// Dominant seventh chord (1-3-5-♭7)
    Dominant7,
    /// Suspended second chord (1-2-5)
    Sus2,
    /// Suspended fourth chord (1-4-5)
    Sus4,
    /// Minor-major seventh chord (1-♭3-5-7)
    MinorMajor7,
}

impl ChordQuality {
    /// Get chord symbol suffix
    pub fn symbol(&self) -> &'static str {
        match self {
            ChordQuality::Major => "",
            ChordQuality::Minor => "m",
            ChordQuality::Diminished => "°",
            ChordQuality::Augmented => "+",
            ChordQuality::Major7 => "maj7",
            ChordQuality::Minor7 => "m7",
            ChordQuality::Dominant7 => "7",
            ChordQuality::Sus2 => "sus2",
            ChordQuality::Sus4 => "sus4",
            ChordQuality::MinorMajor7 => "m(maj7)",
        }
    }

    /// Get intervals from root (in semitones)
    pub fn intervals(&self) -> Vec<u8> {
        match self {
            ChordQuality::Major => vec![0, 4, 7],
            ChordQuality::Minor => vec![0, 3, 7],
            ChordQuality::Diminished => vec![0, 3, 6],
            ChordQuality::Augmented => vec![0, 4, 8],
            ChordQuality::Major7 => vec![0, 4, 7, 11],
            ChordQuality::Minor7 => vec![0, 3, 7, 10],
            ChordQuality::Dominant7 => vec![0, 4, 7, 10],
            ChordQuality::Sus2 => vec![0, 2, 7],
            ChordQuality::Sus4 => vec![0, 5, 7],
            ChordQuality::MinorMajor7 => vec![0, 3, 7, 11],
        }
    }

    /// Check if chord is dissonant (requires resolution)
    pub fn is_dissonant(&self) -> bool {
        matches!(self,
            ChordQuality::Diminished |
            ChordQuality::Augmented |
            ChordQuality::Dominant7 |
            ChordQuality::Sus2 |
            ChordQuality::Sus4
        )
    }
}

impl fmt::Display for ChordQuality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

/// Complete chord representation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Chord {
    /// Root note of the chord
    pub root: Note,
    /// Quality/type of the chord (major, minor, etc.)
    pub quality: ChordQuality,
    /// Inversion of the chord (0 = root position, 1 = first inversion, etc.)
    pub inversion: u8,
}

impl Chord {
    /// Create new chord
    pub fn new(root: Note, quality: ChordQuality) -> Self {
        Self {
            root,
            quality,
            inversion: 0,
        }
    }

    /// Create chord with specific inversion
    pub fn with_inversion(root: Note, quality: ChordQuality, inversion: u8) -> Self {
        Self {
            root,
            quality,
            inversion,
        }
    }

    /// Get chord symbol (e.g., "Cmaj7", "Dm", "G7")
    pub fn symbol(&self) -> String {
        let mut symbol = self.root.name().to_string();
        symbol.push_str(self.quality.symbol());

        if self.inversion > 0 {
            symbol.push_str(&format!("/{}", self.inversion_bass_note().name()));
        }

        symbol
    }

    /// Get chord tones as Notes
    pub fn chord_tones(&self) -> Vec<Note> {
        self.quality.intervals()
            .iter()
            .map(|&interval| self.root.transpose(interval as i8))
            .collect()
    }

    /// Get bass note (considering inversion)
    pub fn bass_note(&self) -> Note {
        if self.inversion == 0 {
            self.root
        } else {
            self.inversion_bass_note()
        }
    }

    /// Get bass note for specific inversion
    fn inversion_bass_note(&self) -> Note {
        let chord_tones = self.chord_tones();
        let inversion_index = (self.inversion as usize) % chord_tones.len();
        chord_tones[inversion_index]
    }

    /// Check if chord contains a specific note
    pub fn contains_note(&self, note: Note) -> bool {
        self.chord_tones().contains(&note)
    }

    /// Get root position version of this chord
    pub fn root_position(&self) -> Chord {
        Chord::new(self.root, self.quality)
    }
}

impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

/// Chord event with timing information
#[derive(Debug, Clone)]
pub struct ChordEvent {
    /// The chord to be played
    pub chord: Chord,
    /// Beat position when this chord starts
    pub beat_position: u32,
    /// Duration of this chord in beats
    pub duration_beats: u32,
    /// Key center for harmonic context
    pub key_center: Note,
    /// Whether this chord should be accented
    pub accent: bool,
}

impl ChordEvent {
    /// Create new chord event
    pub fn new(chord: Chord, beat_position: u32, duration_beats: u32, key_center: Note) -> Self {
        Self {
            chord,
            beat_position,
            duration_beats,
            key_center,
            accent: false,
        }
    }

    /// Create accented chord event
    pub fn with_accent(chord: Chord, beat_position: u32, duration_beats: u32, key_center: Note) -> Self {
        Self {
            chord,
            beat_position,
            duration_beats,
            key_center,
            accent: true,
        }
    }

    /// Get end beat position
    pub fn end_beat(&self) -> u32 {
        self.beat_position + self.duration_beats
    }

    /// Check if chord is active at given beat
    ///
    /// # Arguments
    /// * `beat` - Beat number to check
    ///
    /// # Returns
    /// `true` if the chord is playing during the specified beat
    pub fn is_active_at_beat(&self, beat: u32) -> bool {
        beat >= self.beat_position && beat < self.end_beat()
    }
}

/// Key signature with selected notes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeySelection {
    /// Array indicating which of the 12 chromatic notes are enabled
    pub enabled_notes: [bool; 12],
    /// Suggested primary key center for harmonic context
    pub primary_key: Option<Note>,
}

impl KeySelection {
    /// Create new key selection with all notes disabled
    pub fn new() -> Self {
        Self {
            enabled_notes: [false; 12],
            primary_key: None,
        }
    }

    /// Create key selection for major key
    pub fn for_major_key(key: Note) -> Self {
        let mut selection = Self::new();
        selection.primary_key = Some(key);

        // Enable major scale notes: W-W-H-W-W-W-H
        let major_intervals = [0, 2, 4, 5, 7, 9, 11];
        for &interval in &major_intervals {
            let note_index = key.transpose(interval).as_semitone() as usize;
            selection.enabled_notes[note_index] = true;
        }

        selection
    }

    /// Create key selection for minor key
    pub fn for_minor_key(key: Note) -> Self {
        let mut selection = Self::new();
        selection.primary_key = Some(key);

        // Enable natural minor scale notes: W-H-W-W-H-W-W
        let minor_intervals = [0, 2, 3, 5, 7, 8, 10];
        for &interval in &minor_intervals {
            let note_index = key.transpose(interval).as_semitone() as usize;
            selection.enabled_notes[note_index] = true;
        }

        selection
    }

    /// Create key selection for any key (major or minor)
    pub fn for_key(key: Note, is_major: bool) -> Self {
        if is_major {
            Self::for_major_key(key)
        } else {
            Self::for_minor_key(key)
        }
    }

    /// Enable all chromatic notes
    pub fn all_notes() -> Self {
        Self {
            enabled_notes: [true; 12],
            primary_key: None,
        }
    }

    /// Get list of enabled notes
    pub fn enabled_note_list(&self) -> Vec<Note> {
        Note::all()
            .iter()
            .enumerate()
            .filter(|(i, _)| self.enabled_notes[*i])
            .map(|(_, &note)| note)
            .collect()
    }

    /// Check if note is enabled
    pub fn is_note_enabled(&self, note: Note) -> bool {
        self.enabled_notes[note.as_semitone() as usize]
    }

    /// Enable/disable a note
    pub fn set_note_enabled(&mut self, note: Note, enabled: bool) {
        self.enabled_notes[note.as_semitone() as usize] = enabled;
    }

    /// Get number of enabled notes
    pub fn enabled_count(&self) -> usize {
        self.enabled_notes.iter().filter(|&&enabled| enabled).count()
    }
}

impl Default for KeySelection {
    fn default() -> Self {
        // Default to C major
        Self::for_major_key(Note::C)
    }
}

/// Timeline window configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineConfig {
    /// How many measures to show ahead in the timeline
    pub measures_ahead: u8,
    /// Default chord duration in beats
    pub beats_per_chord: u8,
    /// Whether to auto-advance timeline with metronome
    pub auto_advance: bool,
    /// Whether to highlight key center changes in the display
    pub show_key_changes: bool,
}

impl Default for TimelineConfig {
    fn default() -> Self {
        Self {
            measures_ahead: 2,     // Show 2 measures ahead
            beats_per_chord: 4,    // 1 measure per chord in 4/4
            auto_advance: true,    // Auto-advance with beat
            show_key_changes: true, // Show key modulations
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_semitone_conversion() {
        assert_eq!(Note::C.as_semitone(), 0);
        assert_eq!(Note::CSharp.as_semitone(), 1);
        assert_eq!(Note::B.as_semitone(), 11);

        assert_eq!(Note::from_semitone(0), Note::C);
        assert_eq!(Note::from_semitone(11), Note::B);
        assert_eq!(Note::from_semitone(12), Note::C); // Wraps around
    }

    #[test]
    fn test_note_transposition() {
        assert_eq!(Note::C.transpose(7), Note::G); // Perfect fifth
        assert_eq!(Note::G.transpose(-7), Note::C); // Down a fifth
        assert_eq!(Note::B.transpose(1), Note::C);  // Wraps around
    }

    #[test]
    fn test_chord_symbol() {
        let c_major = Chord::new(Note::C, ChordQuality::Major);
        assert_eq!(c_major.symbol(), "C");

        let d_minor = Chord::new(Note::D, ChordQuality::Minor);
        assert_eq!(d_minor.symbol(), "Dm");

        let g7 = Chord::new(Note::G, ChordQuality::Dominant7);
        assert_eq!(g7.symbol(), "G7");
    }

    #[test]
    fn test_chord_tones() {
        let c_major = Chord::new(Note::C, ChordQuality::Major);
        let tones = c_major.chord_tones();
        assert_eq!(tones, vec![Note::C, Note::E, Note::G]);

        let f_minor7 = Chord::new(Note::F, ChordQuality::Minor7);
        let tones = f_minor7.chord_tones();
        assert_eq!(tones, vec![Note::F, Note::GSharp, Note::C, Note::DSharp]);
    }

    #[test]
    fn test_key_selection() {
        let c_major = KeySelection::for_major_key(Note::C);
        assert!(c_major.is_note_enabled(Note::C));
        assert!(c_major.is_note_enabled(Note::E));
        assert!(c_major.is_note_enabled(Note::G));
        assert!(!c_major.is_note_enabled(Note::CSharp));
        assert_eq!(c_major.enabled_count(), 7);

        let a_minor = KeySelection::for_minor_key(Note::A);
        assert!(a_minor.is_note_enabled(Note::A));
        assert!(a_minor.is_note_enabled(Note::C));
        assert!(a_minor.is_note_enabled(Note::E));
        assert!(!a_minor.is_note_enabled(Note::ASharp));
    }

    #[test]
    fn test_chord_event() {
        let chord = Chord::new(Note::C, ChordQuality::Major);
        let event = ChordEvent::new(chord, 0, 4, Note::C);

        assert!(event.is_active_at_beat(0));
        assert!(event.is_active_at_beat(3));
        assert!(!event.is_active_at_beat(4));
        assert_eq!(event.end_beat(), 4);
    }
}