//! Melody Assistant Module - Chord Progression Drilling for Musicians
//!
//! This module provides intelligent chord progression generation using music theory
//! and Markov chains, with real-time timeline display synchronized to metronome/patterns.
//!
//! # Core Features
//!
//! - **Music Theory-Based Generation**: ~90% adherence to established progressions
//! - **Markov Chain Intelligence**: Learns from common progressions
//! - **Key Selection Interface**: 12 chromatic note checkboxes
//! - **Moving Timeline Display**: Current/next/following chords with notation
//! - **Metronome Integration**: Beat-synchronized chord changes
//! - **Standalone Mode**: Practice chord progressions without backing tracks
//!
//! # Architecture
//!
//! The module follows polyphonica's established patterns:
//! - Trait-based design for extensibility
//! - Real-time state management for audio integration
//! - Configuration system for user preferences
//! - Clean UI component separation
//!
//! # Usage Example
//!
//! ```rust
//! use polyphonica::melody::{MelodyAssistantState, Note, KeySelection};
//!
//! // Create melody assistant for C major
//! let mut assistant = MelodyAssistantState::new_for_key(Note::C, true);
//!
//! // Enable only C major scale notes
//! let key_selection = KeySelection::for_key(Note::C, true);
//! assistant.update_key_selection(key_selection);
//!
//! // Start chord progression generation
//! assistant.start();
//!
//! // Get current timeline for UI display
//! let timeline = assistant.get_timeline_display();
//! println!("Current chord: {:?}", timeline.current_chord);
//! ```

pub mod config;
pub mod generator;
pub mod state;
pub mod theory;
pub mod timeline;
pub mod types;

// Re-export main types for convenient access
pub use config::{MelodyConfig, GenerationConfig, ComplexityLevel, ConfigPreset, UiConfig, ConfigManager};
pub use generator::{ChordGenerator, GenerationParameters, MarkovChordGenerator, GenerationContext};
pub use state::{MelodyAssistantState, SharedMelodyAssistantState, MelodyAssistantBuilder};
pub use theory::{CircleOfFifths, MusicTheory, StandardMusicTheory, VoiceLeading, ScaleType, ChordFunction};
pub use timeline::{ChordTimeline, MovingTimeline, TimelineDisplayData};
pub use types::{Chord, ChordEvent, ChordQuality, KeySelection, Note, TimelineConfig};

/// Create a melody assistant with default configuration
pub fn create_default_melody_assistant() -> MelodyAssistantState {
    let config = MelodyConfig::default();
    MelodyAssistantState::new(config)
}

/// Create a melody assistant configured for specific key
pub fn create_melody_assistant_for_key(key: Note, is_major: bool) -> MelodyAssistantState {
    let mut config = MelodyConfig::default();
    config.default_key_selection = KeySelection::for_key(key, is_major);
    MelodyAssistantState::new(config)
}