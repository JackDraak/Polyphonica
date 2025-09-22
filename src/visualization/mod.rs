/// Beat visualization module for Polyphonica
///
/// This module provides beat visualization capabilities for musical applications,
/// abstracting visual state management from specific GUI frameworks. The
/// architecture separates visual logic from GUI implementation:
///
/// - **BeatDisplay**: Core beat visualization state and logic
/// - **Visual State**: Current beat position, accent status, and timing info
/// - **Color Schemes**: Customizable color coding for different beat states
/// - **Layout**: Beat arrangement and spacing calculations
///
/// # Current Implementation Status
///
/// **Working Features:**
/// - Basic beat circle visualization with color coding
/// - Time signature display (4/4, 3/4, 2/4, 6/8, etc.)
/// - Beat accent highlighting
/// - Current beat position indication
///
/// **Limitations (Prototype Stage):**
/// - No waveform display or spectrum analysis
/// - Limited to simple circular beat indicators
/// - No advanced visualization effects or animations
/// - Color schemes are basic (not fully customizable)
/// - No accessibility features implemented yet
///
/// # Design Principles
///
/// 1. **GUI Framework Agnostic**: Core logic independent of egui, gtk, etc.
/// 2. **Event-Driven**: Updates triggered by beat events, not polling
/// 3. **Configurable**: Customizable colors, layouts, and display options (planned)
/// 4. **Real-time**: Efficient updates suitable for audio-rate beat events
/// 5. **Accessible**: Support for different visualization modes (planned)
///
/// # Usage Example
///
/// ```rust
/// use polyphonica::visualization::{BeatDisplay, BeatVisualState};
/// use polyphonica::timing::{TimeSignature, BeatTracker};
///
/// # fn example() {
/// let mut display = BeatDisplay::new(TimeSignature::new(4, 4));
/// let beat_tracker = BeatTracker::new();
///
/// // Generate visual state based on current beat tracker and settings
/// let visual_state = display.generate_visual_state(
///     &beat_tracker,
///     None,           // No pattern state
///     120.0,          // Tempo in BPM
///     true,           // Is playing
///     true,           // Accent first beat
/// );
///
/// // Render beats using visual state
/// for beat_visual in visual_state.beats {
///     // Render beat circle with beat_visual.color and beat_visual.symbol
///     println!("Beat {}: {:?} {}",
///         beat_visual.beat_number,
///         beat_visual.color,
///         display.get_symbol_char(beat_visual.symbol)
///     );
/// }
/// # }
/// ```
pub mod beat_display;

// Re-export core types for convenient access
pub use beat_display::{
    BeatColorScheme, BeatDisplay, BeatVisual, BeatVisualState, VisualizationMode,
};
