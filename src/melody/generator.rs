/// Chord progression generator using Markov chains and music theory
///
/// This module implements intelligent chord generation that follows music theory
/// principles while maintaining some randomness for variety. The Markov chain
/// learns from common progressions and weights transitions based on harmonic function.

use super::types::*;
use super::theory::{ChordFunction, MusicTheory, StandardMusicTheory, VoiceLeading};
use crate::timing::TimeSignature;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Trait for chord progression generation
pub trait ChordGenerator {
    /// Generate next chord based on current state
    fn generate_next_chord(
        &mut self,
        current_chord: Option<&Chord>,
        key_selection: &KeySelection,
        context: &GenerationContext,
    ) -> Option<Chord>;

    /// Set generation parameters
    fn set_parameters(&mut self, params: GenerationParameters);

    /// Reset generator state
    fn reset(&mut self);

    /// Get current generation parameters
    fn get_parameters(&self) -> &GenerationParameters;
}

/// Generation context for informed chord decisions
#[derive(Debug, Clone)]
pub struct GenerationContext {
    pub current_beat: u32,
    pub measure_position: u8,
    pub recent_chords: Vec<Chord>, // Last 4-8 chords for context
    pub tempo_bpm: f32,
    pub time_signature: TimeSignature,
    pub current_key: Option<Note>,
}

impl GenerationContext {
    /// Create new generation context
    pub fn new(time_signature: TimeSignature, tempo_bpm: f32) -> Self {
        Self {
            current_beat: 0,
            measure_position: 1,
            recent_chords: Vec::new(),
            tempo_bpm,
            time_signature,
            current_key: None,
        }
    }

    /// Update context with new beat
    pub fn advance_beat(&mut self) {
        self.current_beat += 1;
        self.measure_position = (((self.current_beat - 1) % self.time_signature.beats_per_measure as u32) + 1) as u8;
    }

    /// Add chord to recent history
    pub fn add_chord(&mut self, chord: Chord) {
        self.recent_chords.push(chord);
        // Keep only last 8 chords for context
        if self.recent_chords.len() > 8 {
            self.recent_chords.remove(0);
        }
    }

    /// Check if we're on a strong beat (1 or 3 in 4/4)
    pub fn is_strong_beat(&self) -> bool {
        match self.time_signature.beats_per_measure {
            4 => self.measure_position == 1 || self.measure_position == 3,
            3 => self.measure_position == 1,
            2 => self.measure_position == 1,
            _ => self.measure_position == 1,
        }
    }

    /// Get last chord in history
    pub fn last_chord(&self) -> Option<&Chord> {
        self.recent_chords.last()
    }

    /// Check if chord was used recently
    pub fn was_used_recently(&self, chord: &Chord, lookback: usize) -> bool {
        let start = self.recent_chords.len().saturating_sub(lookback);
        self.recent_chords[start..].contains(chord)
    }
}

/// Parameters for chord generation behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationParameters {
    pub theory_adherence: f32,     // 0.0-1.0, how strictly to follow theory (default: 0.9)
    pub repetition_avoidance: f32, // 0.0-1.0, avoid repeating recent chords (default: 0.7)
    pub voice_leading_weight: f32, // 0.0-1.0, prioritize smooth voice leading (default: 0.6)
    pub cadence_strength: f32,     // 0.0-1.0, strength of authentic cadences (default: 0.8)
    pub modulation_tendency: f32,  // 0.0-1.0, likelihood of key changes (default: 0.1)
    pub complexity_level: f32,     // 0.0-1.0, use complex chords (7ths, extensions) (default: 0.3)
}

impl Default for GenerationParameters {
    fn default() -> Self {
        Self {
            theory_adherence: 0.9,      // High adherence to music theory
            repetition_avoidance: 0.7,  // Avoid repeating recent chords
            voice_leading_weight: 0.6,  // Prioritize smooth voice leading
            cadence_strength: 0.8,      // Strong tendency toward cadences
            modulation_tendency: 0.1,   // Low chance of modulation
            complexity_level: 0.3,      // Some 7th chords, mostly triads
        }
    }
}

/// Randomization trait for testability and dependency injection
pub trait Randomizer {
    fn weighted_choice<T: Clone>(&mut self, choices: &[(T, f32)]) -> Option<T>;
    fn random_float(&mut self) -> f32;
    fn random_usize(&mut self, max: usize) -> usize;
}

/// Default randomizer using system random
pub struct SystemRandomizer;

impl Randomizer for SystemRandomizer {
    fn weighted_choice<T: Clone>(&mut self, choices: &[(T, f32)]) -> Option<T> {
        if choices.is_empty() {
            return None;
        }

        let total_weight: f32 = choices.iter().map(|(_, weight)| weight).sum();
        if total_weight <= 0.0 {
            return None;
        }

        let mut random_value = self.random_float() * total_weight;

        for (item, weight) in choices {
            random_value -= weight;
            if random_value <= 0.0 {
                return Some(item.clone());
            }
        }

        // Fallback to last item if floating point errors occur
        choices.last().map(|(item, _)| item.clone())
    }

    fn random_float(&mut self) -> f32 {
        // Simple LCG for deterministic testing - in production could use proper random
        static mut SEED: u32 = 12345;
        unsafe {
            SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
            (SEED >> 16) as f32 / 65536.0
        }
    }

    fn random_usize(&mut self, max: usize) -> usize {
        if max == 0 {
            0
        } else {
            (self.random_float() * max as f32) as usize % max
        }
    }
}

/// Markov chain chord progression generator
pub struct MarkovChordGenerator {
    /// Transition weights: (from_chord, key) -> (to_chord, weight)
    transition_weights: HashMap<(Option<Chord>, Note), HashMap<Chord, f32>>,
    /// Music theory engine for harmonic analysis
    theory_engine: Arc<dyn MusicTheory>,
    /// Generation parameters
    parameters: GenerationParameters,
    /// Randomizer for chord selection
    randomizer: SystemRandomizer,
    /// Possible chord vocabulary
    chord_vocabulary: Vec<(ChordQuality, f32)>, // (quality, complexity_weight)
}

impl MarkovChordGenerator {
    /// Create with music theory engine
    pub fn new(theory_engine: StandardMusicTheory) -> Self {
        let mut generator = Self {
            transition_weights: HashMap::new(),
            theory_engine: Arc::new(theory_engine),
            parameters: GenerationParameters::default(),
            randomizer: SystemRandomizer,
            chord_vocabulary: Vec::new(),
        };

        generator.initialize_chord_vocabulary();
        generator.load_common_progressions();
        generator
    }

    /// Create with default theory
    pub fn new_default() -> Self {
        Self::new(StandardMusicTheory::new())
    }

    /// Create with custom theory engine
    pub fn with_theory_engine(
        theory_engine: Arc<dyn MusicTheory>,
    ) -> Self {
        let mut generator = Self {
            transition_weights: HashMap::new(),
            theory_engine,
            parameters: GenerationParameters::default(),
            randomizer: SystemRandomizer,
            chord_vocabulary: Vec::new(),
        };

        generator.initialize_chord_vocabulary();
        generator.load_common_progressions();
        generator
    }

    /// Initialize available chord qualities with complexity weights
    fn initialize_chord_vocabulary(&mut self) {
        self.chord_vocabulary = vec![
            // Simple triads (low complexity)
            (ChordQuality::Major, 0.0),
            (ChordQuality::Minor, 0.0),

            // Slightly more complex
            (ChordQuality::Diminished, 0.3),
            (ChordQuality::Sus2, 0.2),
            (ChordQuality::Sus4, 0.2),

            // 7th chords (medium complexity)
            (ChordQuality::Major7, 0.5),
            (ChordQuality::Minor7, 0.5),
            (ChordQuality::Dominant7, 0.4),

            // Complex chords (high complexity)
            (ChordQuality::Augmented, 0.8),
            (ChordQuality::MinorMajor7, 0.9),
        ];
    }

    /// Load common progressions to train the Markov chain
    fn load_common_progressions(&mut self) {
        for &key in &Note::all() {
            let progressions = self.theory_engine.get_common_progressions(key);

            for progression in progressions {
                self.train_on_progression(&progression, key);
            }
        }
    }

    /// Train on a single chord progression
    fn train_on_progression(&mut self, progression: &[Chord], key: Note) {
        if progression.is_empty() {
            return;
        }

        // Add transition from None to first chord
        self.add_transition(None, &progression[0], key, 1.0);

        // Add transitions between consecutive chords
        for window in progression.windows(2) {
            if let [from, to] = window {
                self.add_transition(Some(from.clone()), to, key, 1.0);
            }
        }
    }

    /// Add transition weight to the Markov chain
    fn add_transition(&mut self, from: Option<Chord>, to: &Chord, key: Note, weight: f32) {
        let transitions = self.transition_weights
            .entry((from, key))
            .or_insert_with(HashMap::new);

        *transitions.entry(to.clone()).or_insert(0.0) += weight;
    }

    /// Generate candidate chords for current context
    fn generate_candidates(
        &mut self,
        current_chord: Option<&Chord>,
        key_selection: &KeySelection,
        context: &GenerationContext,
    ) -> Vec<(Chord, f32)> {
        let mut candidates = Vec::new();

        // Get enabled notes from key selection
        let enabled_notes = key_selection.enabled_note_list();

        if enabled_notes.is_empty() {
            return candidates;
        }

        // Determine current key center
        let current_key = key_selection.primary_key.unwrap_or(enabled_notes[0]);

        // Generate chords for each enabled root note
        for &root in &enabled_notes {
            for &(quality, complexity) in &self.chord_vocabulary {
                // Filter by complexity level
                if complexity > self.parameters.complexity_level {
                    continue;
                }

                let chord = Chord::new(root, quality);

                // Check if chord fits the key selection
                if !self.theory_engine.chord_fits_key(&chord, key_selection) {
                    continue;
                }

                // Calculate weight for this chord
                let weight = self.calculate_chord_weight(
                    &chord,
                    current_chord,
                    current_key,
                    context,
                );

                if weight > 0.0 {
                    candidates.push((chord, weight));
                }
            }
        }

        candidates
    }

    /// Calculate weight for a candidate chord
    fn calculate_chord_weight(
        &self,
        candidate: &Chord,
        current_chord: Option<&Chord>,
        key: Note,
        context: &GenerationContext,
    ) -> f32 {
        let mut weight = 1.0;

        // Music theory weight
        if let Some(current) = current_chord {
            let theory_weight = self.theory_engine.progression_probability(current, candidate, key);
            weight *= 1.0 + (theory_weight - 0.5) * self.parameters.theory_adherence;
        }

        // Markov chain weight
        let markov_key = (current_chord.cloned(), key);
        if let Some(transitions) = self.transition_weights.get(&markov_key) {
            if let Some(&markov_weight) = transitions.get(candidate) {
                weight *= 1.0 + markov_weight * 0.5;
            }
        }

        // Voice leading weight
        if let Some(current) = current_chord {
            let voice_leading_smoothness = VoiceLeading::calculate_smoothness(current, candidate);
            let smoothness_bonus = (6.0 - voice_leading_smoothness) / 6.0;
            weight *= 1.0 + smoothness_bonus * self.parameters.voice_leading_weight;
        }

        // Repetition avoidance
        if context.was_used_recently(candidate, 3) {
            weight *= 1.0 - self.parameters.repetition_avoidance;
        }

        // Strong beat emphasis (prefer stable chords on strong beats)
        if context.is_strong_beat() {
            let function = self.theory_engine.chord_function(candidate, key);
            if matches!(function, ChordFunction::Tonic) {
                weight *= 1.0 + self.parameters.cadence_strength * 0.3;
            }
        }

        // Ensure weight is positive
        weight.max(0.01)
    }

    /// Update generation parameters
    pub fn update_parameters(&mut self, params: GenerationParameters) {
        self.parameters = params;
    }
}

impl Default for MarkovChordGenerator {
    fn default() -> Self {
        Self::new_default()
    }
}

impl ChordGenerator for MarkovChordGenerator {
    fn generate_next_chord(
        &mut self,
        current_chord: Option<&Chord>,
        key_selection: &KeySelection,
        context: &GenerationContext,
    ) -> Option<Chord> {
        let candidates = self.generate_candidates(current_chord, key_selection, context);

        if candidates.is_empty() {
            // Fallback: generate a simple chord from enabled notes
            let enabled_notes = key_selection.enabled_note_list();
            if !enabled_notes.is_empty() {
                let root_index = self.randomizer.random_usize(enabled_notes.len());
                let root = enabled_notes[root_index];
                return Some(Chord::new(root, ChordQuality::Major));
            }
            return None;
        }

        // Use weighted random selection
        self.randomizer.weighted_choice(&candidates)
    }

    fn set_parameters(&mut self, params: GenerationParameters) {
        self.parameters = params;
    }

    fn reset(&mut self) {
        // Clear any state that needs resetting
        // Markov weights are kept as they represent learned patterns
    }

    fn get_parameters(&self) -> &GenerationParameters {
        &self.parameters
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock randomizer for deterministic testing
    struct MockRandomizer {
        values: Vec<f32>,
        index: usize,
    }

    impl MockRandomizer {
        fn new(values: Vec<f32>) -> Self {
            Self { values, index: 0 }
        }
    }

    impl Randomizer for MockRandomizer {
        fn weighted_choice<T: Clone>(&mut self, choices: &[(T, f32)]) -> Option<T> {
            if choices.is_empty() {
                return None;
            }
            // For testing, just return first choice
            Some(choices[0].0.clone())
        }

        fn random_float(&mut self) -> f32 {
            let value = self.values.get(self.index).copied().unwrap_or(0.5);
            self.index = (self.index + 1) % self.values.len();
            value
        }

        fn random_usize(&mut self, max: usize) -> usize {
            if max == 0 {
                0
            } else {
                (self.random_float() * max as f32) as usize % max
            }
        }
    }

    #[test]
    fn test_generation_context() {
        let mut context = GenerationContext::new(TimeSignature::new(4, 4), 120.0);

        assert_eq!(context.measure_position, 1);
        assert!(context.is_strong_beat());

        context.advance_beat();
        assert_eq!(context.measure_position, 2);
        assert!(!context.is_strong_beat());

        context.advance_beat();
        assert_eq!(context.measure_position, 3);
        assert!(context.is_strong_beat());
    }

    #[test]
    fn test_chord_generation() {
        let mut generator = MarkovChordGenerator::new();
        let key_selection = KeySelection::for_major_key(Note::C);
        let context = GenerationContext::new(TimeSignature::new(4, 4), 120.0);

        // Should generate a chord in C major
        let chord = generator.generate_next_chord(None, &key_selection, &context);
        assert!(chord.is_some());

        let chord = chord.unwrap();
        // Chord should fit the key
        assert!(generator.theory_engine.chord_fits_key(&chord, &key_selection));
    }

    #[test]
    fn test_progression_training() {
        let mut generator = MarkovChordGenerator::new();

        // Train on I-V-vi-IV progression in C
        let progression = vec![
            Chord::new(Note::C, ChordQuality::Major),  // I
            Chord::new(Note::G, ChordQuality::Major),  // V
            Chord::new(Note::A, ChordQuality::Minor),  // vi
            Chord::new(Note::F, ChordQuality::Major),  // IV
        ];

        generator.train_on_progression(&progression, Note::C);

        // Check that transitions were added
        let key = (Some(Chord::new(Note::C, ChordQuality::Major)), Note::C);
        assert!(generator.transition_weights.contains_key(&key));
    }

    #[test]
    fn test_parameters() {
        let mut generator = MarkovChordGenerator::new();

        let mut params = GenerationParameters::default();
        params.theory_adherence = 0.5;
        params.complexity_level = 0.8;

        generator.set_parameters(params.clone());
        assert_eq!(generator.get_parameters().theory_adherence, 0.5);
        assert_eq!(generator.get_parameters().complexity_level, 0.8);
    }

    #[test]
    fn test_repetition_avoidance() {
        let mut context = GenerationContext::new(TimeSignature::new(4, 4), 120.0);
        let c_major = Chord::new(Note::C, ChordQuality::Major);

        // Add chord to recent history
        context.add_chord(c_major.clone());

        // Should detect recent usage
        assert!(context.was_used_recently(&c_major, 2));
        assert!(!context.was_used_recently(&Chord::new(Note::G, ChordQuality::Major), 2));
    }
}