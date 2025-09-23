/// Real-time state management for the melody assistant
///
/// This module provides the MelodyAssistantState which coordinates chord generation,
/// timeline management, and user configuration for real-time chord progression drilling.

use super::{
    config::MelodyConfig,
    generator::{ChordGenerator, GenerationParameters, MarkovChordGenerator},
    theory::StandardMusicTheory,
    timeline::{MovingTimeline, TimelineDisplayData},
    types::*,
};
use crate::timing::{BeatEvent, TimeSignature};
use std::sync::{Arc, Mutex};

/// Real-time state for melody assistant
pub struct MelodyAssistantState {
    config: MelodyConfig,
    timeline: MovingTimeline,
    generator: MarkovChordGenerator,
    music_theory: StandardMusicTheory,
    key_selection: KeySelection,
    current_key: Note,
    is_running: bool,
    generation_ahead_beats: u32,
    last_generation_beat: u32,
}

impl MelodyAssistantState {
    /// Create new melody assistant with configuration
    pub fn new(config: MelodyConfig) -> Self {
        let timeline = MovingTimeline::new(
            config.default_time_signature,
            &config.timeline_config,
        );

        let music_theory = StandardMusicTheory::new();
        let generator = MarkovChordGenerator::new_default();

        Self {
            timeline,
            generator,
            music_theory,
            key_selection: config.default_key_selection.clone(),
            current_key: config.default_key_selection.primary_key.unwrap_or(Note::C),
            is_running: false,
            generation_ahead_beats: config.generation_ahead_beats,
            last_generation_beat: 0,
            config,
        }
    }

    /// Create melody assistant for specific key
    pub fn new_for_key(key: Note, is_major: bool) -> Self {
        let mut config = MelodyConfig::default();
        config.default_key_selection = KeySelection::for_key(key, is_major);
        Self::new(config)
    }

    /// Start generating chord progressions
    pub fn start(&mut self) {
        if !self.is_running {
            self.is_running = true;
            self.generate_initial_progression();
        }
    }

    /// Stop generating and clear timeline
    pub fn stop(&mut self) {
        self.is_running = false;
        self.timeline.clear();
        self.last_generation_beat = 0;
    }

    /// Update with beat event from metronome
    pub fn update_with_beat(&mut self, beat_event: &BeatEvent) {
        if !self.is_running {
            return;
        }

        // Update timeline with current beat
        self.timeline.update_with_beat(beat_event);

        // Generate more chords if needed
        let current_beat = beat_event.beat_number as u32;
        if self.should_generate_more_chords(current_beat) {
            self.generate_ahead(current_beat);
        }
    }

    /// Update key selection (user checkboxes for chromatic notes)
    pub fn update_key_selection(&mut self, key_selection: KeySelection) {
        // Update current key if primary key changed
        if let Some(primary_key) = key_selection.primary_key {
            if primary_key != self.current_key {
                self.current_key = primary_key;
                // Regenerate progression for new key
                if self.is_running {
                    self.generate_initial_progression();
                }
            }
        }

        self.key_selection = key_selection;
    }

    /// Update time signature
    pub fn update_time_signature(&mut self, time_signature: TimeSignature) {
        self.config.default_time_signature = time_signature;
        self.timeline.update_time_signature(time_signature);
    }

    /// Update timeline configuration
    pub fn update_timeline_config(&mut self, timeline_config: TimelineConfig) {
        self.config.timeline_config = timeline_config.clone();
        self.timeline.update_config(&timeline_config);
    }

    /// Get current timeline display data for UI
    pub fn get_timeline_display(&self) -> TimelineDisplayData {
        self.timeline.get_display_data()
    }

    /// Get current key selection
    pub fn get_key_selection(&self) -> &KeySelection {
        &self.key_selection
    }

    /// Get current primary key
    pub fn get_current_key(&self) -> Note {
        self.current_key
    }

    /// Check if melody assistant is running
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Get current configuration
    pub fn get_config(&self) -> &MelodyConfig {
        &self.config
    }

    /// Update generation parameters
    pub fn update_generation_params(&mut self, params: GenerationParameters) {
        self.generator.update_parameters(params);
    }

    /// Force regeneration of upcoming progression
    pub fn regenerate_progression(&mut self) {
        if self.is_running {
            // Clear future events and regenerate
            let current_beat = self.timeline.current_beat();
            self.timeline.timeline_mut().update(current_beat);
            self.generate_ahead(current_beat);
        }
    }

    /// Jump timeline to specific beat
    pub fn jump_to_beat(&mut self, beat: u32) {
        self.timeline.jump_to_beat(beat);
        self.last_generation_beat = beat;

        if self.is_running {
            self.generate_ahead(beat);
        }
    }

    /// Get music theory engine for analysis
    pub fn music_theory(&self) -> &StandardMusicTheory {
        &self.music_theory
    }

    /// Set generation ahead distance
    pub fn set_generation_ahead_beats(&mut self, beats: u32) {
        self.generation_ahead_beats = beats;
    }

    /// Generate initial chord progression
    fn generate_initial_progression(&mut self) {
        self.timeline.clear();
        self.last_generation_beat = 0;
        self.generate_ahead(0);
    }

    /// Generate chords ahead of current beat
    fn generate_ahead(&mut self, current_beat: u32) {
        use super::generator::GenerationContext;

        // Create generation context for this session
        let mut generation_context = GenerationContext::new(
            self.config.default_time_signature,
            120.0, // Default tempo - could be made configurable
        );
        generation_context.current_beat = current_beat;
        generation_context.current_key = Some(self.current_key);

        // Update generator parameters
        let generation_params = GenerationParameters {
            theory_adherence: self.config.theory_adherence,
            repetition_avoidance: self.config.repetition_penalty,
            voice_leading_weight: self.config.voice_leading_weight,
            cadence_strength: 0.8,
            modulation_tendency: 0.1,
            complexity_level: 0.3,
            rhythm_density: 0.3, // Default medium rhythm density
        };
        self.generator.update_parameters(generation_params);

        // Generate from last generation point to ahead target
        let target_beat = current_beat + self.generation_ahead_beats;
        let mut gen_beat = self.last_generation_beat.max(current_beat);
        let beats_per_chord = self.config.timeline_config.beats_per_chord as u32;

        while gen_beat < target_beat {
            // Update context for current beat
            generation_context.current_beat = gen_beat;

            // Get current chord for context
            let current_chord = self.timeline.timeline()
                .chord_at_beat(gen_beat.saturating_sub(1))
                .map(|event| &event.chord);

            // Generate next chord
            let generated = self.generator.generate_next_chord(
                current_chord,
                &self.key_selection,
                &generation_context,
            );

            if let Some(chord) = generated {
                let chord_event = ChordEvent::new(
                    chord.clone(),
                    gen_beat,
                    beats_per_chord,
                    self.current_key,
                );

                // Add to context history for better progression
                generation_context.add_chord(chord);

                self.timeline.timeline_mut().add_chord_event(chord_event);
            }

            gen_beat += beats_per_chord;
        }

        self.last_generation_beat = gen_beat;
    }

    /// Check if more chords need to be generated
    fn should_generate_more_chords(&self, current_beat: u32) -> bool {
        let target_beat = current_beat + self.generation_ahead_beats;
        self.last_generation_beat < target_beat
    }
}

/// Thread-safe wrapper for melody assistant state
pub struct SharedMelodyAssistantState {
    inner: Arc<Mutex<MelodyAssistantState>>,
}

impl SharedMelodyAssistantState {
    /// Create new shared melody assistant state
    pub fn new(config: MelodyConfig) -> Self {
        Self {
            inner: Arc::new(Mutex::new(MelodyAssistantState::new(config))),
        }
    }

    /// Create shared state for specific key
    pub fn new_for_key(key: Note, is_major: bool) -> Self {
        Self {
            inner: Arc::new(Mutex::new(MelodyAssistantState::new_for_key(key, is_major))),
        }
    }

    /// Execute operation with locked state
    pub fn with_state<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut MelodyAssistantState) -> R,
    {
        let mut state = self.inner.lock().unwrap();
        f(&mut *state)
    }

    /// Get timeline display data (read-only operation)
    pub fn get_timeline_display(&self) -> TimelineDisplayData {
        let state = self.inner.lock().unwrap();
        state.get_timeline_display()
    }

    /// Update with beat event
    pub fn update_with_beat(&self, beat_event: &BeatEvent) {
        let mut state = self.inner.lock().unwrap();
        state.update_with_beat(beat_event);
    }

    /// Start/stop operations
    pub fn start(&self) {
        let mut state = self.inner.lock().unwrap();
        state.start();
    }

    pub fn stop(&self) {
        let mut state = self.inner.lock().unwrap();
        state.stop();
    }

    /// Clone the Arc for sharing between threads
    pub fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// Builder for melody assistant state configuration
pub struct MelodyAssistantBuilder {
    config: MelodyConfig,
}

impl MelodyAssistantBuilder {
    /// Create new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: MelodyConfig::default(),
        }
    }

    /// Set key selection
    pub fn with_key_selection(mut self, key_selection: KeySelection) -> Self {
        self.config.default_key_selection = key_selection;
        self
    }

    /// Set primary key (convenience for major/minor keys)
    pub fn with_key(mut self, key: Note, is_major: bool) -> Self {
        self.config.default_key_selection = KeySelection::for_key(key, is_major);
        self
    }

    /// Set time signature
    pub fn with_time_signature(mut self, time_signature: TimeSignature) -> Self {
        self.config.default_time_signature = time_signature;
        self
    }

    /// Set timeline configuration
    pub fn with_timeline_config(mut self, timeline_config: TimelineConfig) -> Self {
        self.config.timeline_config = timeline_config;
        self
    }

    /// Set theory adherence (0.0 to 1.0)
    pub fn with_theory_adherence(mut self, adherence: f32) -> Self {
        self.config.theory_adherence = adherence.clamp(0.0, 1.0);
        self
    }

    /// Set voice leading weight (0.0 to 1.0)
    pub fn with_voice_leading_weight(mut self, weight: f32) -> Self {
        self.config.voice_leading_weight = weight.clamp(0.0, 1.0);
        self
    }

    /// Set generation ahead distance in beats
    pub fn with_generation_ahead_beats(mut self, beats: u32) -> Self {
        self.config.generation_ahead_beats = beats;
        self
    }

    /// Build melody assistant state
    pub fn build(self) -> MelodyAssistantState {
        MelodyAssistantState::new(self.config)
    }

    /// Build shared melody assistant state
    pub fn build_shared(self) -> SharedMelodyAssistantState {
        SharedMelodyAssistantState::new(self.config)
    }
}

impl Default for MelodyAssistantBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timing::TimeSignature;

    #[test]
    fn test_melody_assistant_creation() {
        let config = MelodyConfig::default();
        let assistant = MelodyAssistantState::new(config);

        assert!(!assistant.is_running());
        assert_eq!(assistant.get_current_key(), Note::C);
    }

    #[test]
    fn test_melody_assistant_for_key() {
        let assistant = MelodyAssistantState::new_for_key(Note::G, true);
        assert_eq!(assistant.get_current_key(), Note::G);
        assert!(assistant.get_key_selection().is_note_enabled(Note::G));
    }

    #[test]
    fn test_start_stop_functionality() {
        let mut assistant = MelodyAssistantState::new_for_key(Note::C, true);

        assert!(!assistant.is_running());

        assistant.start();
        assert!(assistant.is_running());

        assistant.stop();
        assert!(!assistant.is_running());
    }

    #[test]
    fn test_key_selection_update() {
        let mut assistant = MelodyAssistantState::new_for_key(Note::C, true);
        assert_eq!(assistant.get_current_key(), Note::C);

        let new_key_selection = KeySelection::for_key(Note::G, true);
        assistant.update_key_selection(new_key_selection);

        assert_eq!(assistant.get_current_key(), Note::G);
    }

    #[test]
    fn test_beat_event_processing() {
        let mut assistant = MelodyAssistantState::new_for_key(Note::C, true);
        assistant.start();

        let beat_event = BeatEvent::new(
            1,
            false,
            vec![crate::timing::ClickType::WoodBlock],
            120.0,
            TimeSignature::new(4, 4),
        );

        // Should not panic
        assistant.update_with_beat(&beat_event);
    }

    #[test]
    fn test_timeline_display_data() {
        let assistant = MelodyAssistantState::new_for_key(Note::C, true);
        let display = assistant.get_timeline_display();

        // Should have valid data structure
        assert_eq!(display.current_key_center, Note::C);
        assert_eq!(display.current_beat, 0);
    }

    #[test]
    fn test_shared_melody_assistant() {
        let shared = SharedMelodyAssistantState::new_for_key(Note::C, true);

        shared.start();

        let is_running = shared.with_state(|state| state.is_running());
        assert!(is_running);

        shared.stop();

        let is_running = shared.with_state(|state| state.is_running());
        assert!(!is_running);
    }

    #[test]
    fn test_melody_assistant_builder() {
        let assistant = MelodyAssistantBuilder::new()
            .with_key(Note::F, false) // F minor
            .with_theory_adherence(0.8)
            .with_voice_leading_weight(0.7)
            .with_generation_ahead_beats(32)
            .build();

        assert_eq!(assistant.get_current_key(), Note::F);
        assert_eq!(assistant.config.theory_adherence, 0.8);
        assert_eq!(assistant.config.voice_leading_weight, 0.7);
        assert_eq!(assistant.generation_ahead_beats, 32);
    }

    #[test]
    fn test_time_signature_update() {
        let mut assistant = MelodyAssistantState::new_for_key(Note::C, true);
        let new_time_sig = TimeSignature::new(3, 4);

        assistant.update_time_signature(new_time_sig);
        assert_eq!(assistant.config.default_time_signature, new_time_sig);
    }

    #[test]
    fn test_jump_to_beat() {
        let mut assistant = MelodyAssistantState::new_for_key(Note::C, true);
        assistant.start();

        assistant.jump_to_beat(16);
        assert_eq!(assistant.timeline.current_beat(), 16);
    }

    #[test]
    fn test_regenerate_progression() {
        let mut assistant = MelodyAssistantState::new_for_key(Note::C, true);
        assistant.start();

        // Should not panic and should work when running
        assistant.regenerate_progression();
    }
}