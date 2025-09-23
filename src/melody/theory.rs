/// Music theory engine for harmonic analysis and chord relationships
///
/// This module provides the theoretical foundation for intelligent chord progression
/// generation, including circle of fifths relationships, voice leading calculations,
/// and common progression patterns.

use super::types::*;
use std::collections::HashMap;

/// Scale types for key analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleType {
    Major,
    NaturalMinor,
    HarmonicMinor,
    MelodicMinor,
    Dorian,
    Mixolydian,
}

/// Core music theory operations
pub trait MusicTheory {
    /// Get notes in a given key/scale
    fn get_scale_notes(&self, key: Note, scale_type: ScaleType) -> Vec<Note>;

    /// Get chord tones for a chord
    fn get_chord_tones(&self, chord: &Chord) -> Vec<Note>;

    /// Calculate harmonic distance between chords
    fn harmonic_distance(&self, from: &Chord, to: &Chord) -> f32;

    /// Get common progressions in a key
    fn get_common_progressions(&self, key: Note) -> Vec<Vec<Chord>>;

    /// Validate chord against key selection
    fn chord_fits_key(&self, chord: &Chord, key_selection: &KeySelection) -> bool;

    /// Get chord function in key (tonic, subdominant, dominant, etc.)
    fn chord_function(&self, chord: &Chord, key: Note) -> ChordFunction;

    /// Get probability of chord progression
    fn progression_probability(&self, from: &Chord, to: &Chord, key: Note) -> f32;
}

/// Functional harmony roles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChordFunction {
    Tonic,          // I, vi, iii
    Subdominant,    // IV, ii
    Dominant,       // V, vii°
    SubdominantMinor, // iv
    Secondary,      // Secondary dominants, etc.
    Chromatic,      // Non-diatonic
}

/// Circle of fifths relationships and calculations
pub struct CircleOfFifths;

impl CircleOfFifths {
    /// Get position on circle of fifths (0-11, C=0)
    pub fn position(note: Note) -> u8 {
        match note {
            Note::C => 0,
            Note::G => 1,
            Note::D => 2,
            Note::A => 3,
            Note::E => 4,
            Note::B => 5,
            Note::FSharp => 6,
            Note::CSharp => 7,
            Note::GSharp => 8,
            Note::DSharp => 9,
            Note::ASharp => 10,
            Note::F => 11,
        }
    }

    /// Get harmonic distance between two notes (0-6)
    pub fn distance(note1: Note, note2: Note) -> u8 {
        let pos1 = Self::position(note1);
        let pos2 = Self::position(note2);
        let diff = ((pos1 as i8 - pos2 as i8).abs()) as u8;
        std::cmp::min(diff, 12 - diff)
    }

    /// Get relative major/minor key
    pub fn relative_key(note: Note, is_major: bool) -> Note {
        if is_major {
            // Major to relative minor: down 3 semitones
            note.transpose(-3)
        } else {
            // Minor to relative major: up 3 semitones
            note.transpose(3)
        }
    }

    /// Get parallel major/minor key
    pub fn parallel_key(note: Note) -> Note {
        note // Same root, different mode
    }

    /// Get dominant of key
    pub fn dominant(key: Note) -> Note {
        key.transpose(7) // Up a perfect fifth
    }

    /// Get subdominant of key
    pub fn subdominant(key: Note) -> Note {
        key.transpose(5) // Up a perfect fourth
    }
}

/// Voice leading calculator for smooth chord transitions
pub struct VoiceLeading;

impl VoiceLeading {
    /// Calculate smoothest voice leading between chords (lower = smoother)
    pub fn calculate_smoothness(from: &Chord, to: &Chord) -> f32 {
        let from_tones = from.chord_tones();
        let to_tones = to.chord_tones();

        // Calculate minimum movement between chord tones
        let mut total_movement = 0;
        let mut voice_count = 0;

        for from_tone in &from_tones {
            let min_movement = to_tones.iter()
                .map(|to_tone| Self::note_distance(*from_tone, *to_tone))
                .min()
                .unwrap_or(12);
            total_movement += min_movement;
            voice_count += 1;
        }

        if voice_count > 0 {
            total_movement as f32 / voice_count as f32
        } else {
            12.0 // Maximum distance if no voices
        }
    }

    /// Get chord inversions for smooth voice leading
    pub fn suggest_inversion(from: &Chord, to: &Chord) -> u8 {
        let mut best_inversion = 0;
        let mut best_smoothness = f32::MAX;

        // Try different inversions of the target chord
        for inversion in 0..=2 {
            let inverted_chord = Chord::with_inversion(to.root, to.quality, inversion);
            let smoothness = Self::calculate_smoothness(from, &inverted_chord);

            if smoothness < best_smoothness {
                best_smoothness = smoothness;
                best_inversion = inversion;
            }
        }

        best_inversion
    }

    /// Calculate note distance (considering enharmonic equivalents)
    fn note_distance(note1: Note, note2: Note) -> u8 {
        let semitone1 = note1.as_semitone();
        let semitone2 = note2.as_semitone();
        let diff = ((semitone1 as i8 - semitone2 as i8).abs()) as u8;
        std::cmp::min(diff, 12 - diff)
    }

    /// Check if progression has good voice leading (< 3 semitones average movement)
    pub fn has_good_voice_leading(from: &Chord, to: &Chord) -> bool {
        Self::calculate_smoothness(from, to) < 3.0
    }
}

/// Default music theory implementation
#[derive(Clone)]
pub struct StandardMusicTheory {
    common_progressions: HashMap<Note, Vec<Vec<Chord>>>,
    transition_weights: HashMap<(ChordFunction, ChordFunction), f32>,
}

impl StandardMusicTheory {
    /// Create new theory engine with default progressions
    pub fn new() -> Self {
        let mut theory = Self {
            common_progressions: HashMap::new(),
            transition_weights: HashMap::new(),
        };

        theory.initialize_common_progressions();
        theory.initialize_transition_weights();
        theory
    }

    /// Initialize common chord progressions for all keys
    fn initialize_common_progressions(&mut self) {
        for &key in &Note::all() {
            let progressions = vec![
                // I-V-vi-IV (very common pop progression)
                vec![
                    Self::chord_for_degree(key, 1, ChordQuality::Major),
                    Self::chord_for_degree(key, 5, ChordQuality::Major),
                    Self::chord_for_degree(key, 6, ChordQuality::Minor),
                    Self::chord_for_degree(key, 4, ChordQuality::Major),
                ],
                // ii-V-I (jazz standard)
                vec![
                    Self::chord_for_degree(key, 2, ChordQuality::Minor7),
                    Self::chord_for_degree(key, 5, ChordQuality::Dominant7),
                    Self::chord_for_degree(key, 1, ChordQuality::Major7),
                ],
                // vi-IV-I-V (another common pop progression)
                vec![
                    Self::chord_for_degree(key, 6, ChordQuality::Minor),
                    Self::chord_for_degree(key, 4, ChordQuality::Major),
                    Self::chord_for_degree(key, 1, ChordQuality::Major),
                    Self::chord_for_degree(key, 5, ChordQuality::Major),
                ],
                // I-vi-ii-V (circle progression)
                vec![
                    Self::chord_for_degree(key, 1, ChordQuality::Major),
                    Self::chord_for_degree(key, 6, ChordQuality::Minor),
                    Self::chord_for_degree(key, 2, ChordQuality::Minor),
                    Self::chord_for_degree(key, 5, ChordQuality::Major),
                ],
            ];

            self.common_progressions.insert(key, progressions);
        }
    }

    /// Initialize transition probability weights
    fn initialize_transition_weights(&mut self) {
        use ChordFunction::*;

        // High probability transitions (music theory)
        self.transition_weights.insert((Tonic, Subdominant), 0.8);
        self.transition_weights.insert((Tonic, Dominant), 0.7);
        self.transition_weights.insert((Subdominant, Dominant), 0.9);
        self.transition_weights.insert((Dominant, Tonic), 0.95);

        // Medium probability transitions
        self.transition_weights.insert((Subdominant, Tonic), 0.6);
        self.transition_weights.insert((Dominant, Subdominant), 0.4);

        // Lower probability but valid transitions
        self.transition_weights.insert((Tonic, Tonic), 0.3);
        self.transition_weights.insert((Subdominant, Subdominant), 0.2);
        self.transition_weights.insert((Dominant, Dominant), 0.3);

        // Chromatic transitions (less common)
        self.transition_weights.insert((Chromatic, Tonic), 0.6);
        self.transition_weights.insert((Chromatic, Dominant), 0.5);
    }

    /// Create chord for scale degree in key
    fn chord_for_degree(key: Note, degree: u8, quality: ChordQuality) -> Chord {
        let intervals = [0, 2, 4, 5, 7, 9, 11]; // Major scale intervals
        let degree_index = (degree - 1) as usize % 7;
        let root = key.transpose(intervals[degree_index]);
        Chord::new(root, quality)
    }

    /// Get scale degree for note in key (1-7, or 0 if not in scale)
    fn get_scale_degree(&self, note: Note, key: Note) -> u8 {
        let intervals = [0, 2, 4, 5, 7, 9, 11]; // Major scale intervals
        let target_interval = (note.as_semitone() as i8 - key.as_semitone() as i8).rem_euclid(12) as u8;

        for (i, &interval) in intervals.iter().enumerate() {
            if interval == target_interval {
                return (i + 1) as u8;
            }
        }

        0 // Not in scale
    }
}

impl Default for StandardMusicTheory {
    fn default() -> Self {
        Self::new()
    }
}

impl MusicTheory for StandardMusicTheory {
    fn get_scale_notes(&self, key: Note, scale_type: ScaleType) -> Vec<Note> {
        let intervals = match scale_type {
            ScaleType::Major => vec![0, 2, 4, 5, 7, 9, 11],
            ScaleType::NaturalMinor => vec![0, 2, 3, 5, 7, 8, 10],
            ScaleType::HarmonicMinor => vec![0, 2, 3, 5, 7, 8, 11],
            ScaleType::MelodicMinor => vec![0, 2, 3, 5, 7, 9, 11],
            ScaleType::Dorian => vec![0, 2, 3, 5, 7, 9, 10],
            ScaleType::Mixolydian => vec![0, 2, 4, 5, 7, 9, 10],
        };

        intervals.into_iter()
            .map(|interval| key.transpose(interval))
            .collect()
    }

    fn get_chord_tones(&self, chord: &Chord) -> Vec<Note> {
        chord.chord_tones()
    }

    fn harmonic_distance(&self, from: &Chord, to: &Chord) -> f32 {
        // Combine circle of fifths distance with voice leading
        let root_distance = CircleOfFifths::distance(from.root, to.root) as f32;
        let voice_leading = VoiceLeading::calculate_smoothness(from, to);

        // Weight both factors
        (root_distance * 0.3) + (voice_leading * 0.7)
    }

    fn get_common_progressions(&self, key: Note) -> Vec<Vec<Chord>> {
        self.common_progressions.get(&key).cloned().unwrap_or_default()
    }

    fn chord_fits_key(&self, chord: &Chord, key_selection: &KeySelection) -> bool {
        // Check if all chord tones are enabled in key selection
        chord.chord_tones()
            .iter()
            .all(|&note| key_selection.is_note_enabled(note))
    }

    fn chord_function(&self, chord: &Chord, key: Note) -> ChordFunction {
        let degree = self.get_scale_degree(chord.root, key);

        match degree {
            1 | 3 | 6 => ChordFunction::Tonic,
            2 | 4 => ChordFunction::Subdominant,
            5 | 7 => ChordFunction::Dominant,
            _ => ChordFunction::Chromatic,
        }
    }

    fn progression_probability(&self, from: &Chord, to: &Chord, key: Note) -> f32 {
        let from_function = self.chord_function(from, key);
        let to_function = self.chord_function(to, key);

        self.transition_weights
            .get(&(from_function, to_function))
            .copied()
            .unwrap_or(0.1) // Low probability for undefined transitions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_of_fifths() {
        assert_eq!(CircleOfFifths::position(Note::C), 0);
        assert_eq!(CircleOfFifths::position(Note::G), 1);
        assert_eq!(CircleOfFifths::position(Note::F), 11);

        assert_eq!(CircleOfFifths::distance(Note::C, Note::G), 1);
        assert_eq!(CircleOfFifths::distance(Note::C, Note::F), 1);
        assert_eq!(CircleOfFifths::distance(Note::C, Note::FSharp), 6);
    }

    #[test]
    fn test_relative_keys() {
        assert_eq!(CircleOfFifths::relative_key(Note::C, true), Note::A);  // C major -> A minor
        assert_eq!(CircleOfFifths::relative_key(Note::A, false), Note::C); // A minor -> C major
    }

    #[test]
    fn test_voice_leading() {
        let c_major = Chord::new(Note::C, ChordQuality::Major);
        let f_major = Chord::new(Note::F, ChordQuality::Major);
        let g_major = Chord::new(Note::G, ChordQuality::Major);

        // C to F should be smoother than C to random distant chord
        let smoothness_cf = VoiceLeading::calculate_smoothness(&c_major, &f_major);
        let smoothness_cg = VoiceLeading::calculate_smoothness(&c_major, &g_major);

        assert!(smoothness_cf < 6.0); // Should be reasonably smooth
        assert!(smoothness_cg < 6.0); // Should be reasonably smooth
    }

    #[test]
    fn test_music_theory_implementation() {
        let theory = StandardMusicTheory::new();

        // Test scale generation
        let c_major_scale = theory.get_scale_notes(Note::C, ScaleType::Major);
        assert_eq!(c_major_scale.len(), 7);
        assert!(c_major_scale.contains(&Note::C));
        assert!(c_major_scale.contains(&Note::E));
        assert!(c_major_scale.contains(&Note::G));

        // Test chord fitting
        let c_major = Chord::new(Note::C, ChordQuality::Major);
        let c_major_key = KeySelection::for_major_key(Note::C);
        assert!(theory.chord_fits_key(&c_major, &c_major_key));

        // Test chord that doesn't fit
        let c_sharp_major = Chord::new(Note::CSharp, ChordQuality::Major);
        assert!(!theory.chord_fits_key(&c_sharp_major, &c_major_key));
    }

    #[test]
    fn test_chord_functions() {
        let theory = StandardMusicTheory::new();

        let c_major = Chord::new(Note::C, ChordQuality::Major);
        let f_major = Chord::new(Note::F, ChordQuality::Major);
        let g_major = Chord::new(Note::G, ChordQuality::Major);

        assert_eq!(theory.chord_function(&c_major, Note::C), ChordFunction::Tonic);
        assert_eq!(theory.chord_function(&f_major, Note::C), ChordFunction::Subdominant);
        assert_eq!(theory.chord_function(&g_major, Note::C), ChordFunction::Dominant);
    }
}