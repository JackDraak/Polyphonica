/// High-precision beat timing abstraction with discrete scheduling.
pub mod clock;
/// Simple metronome implementation using BeatClock for regular beats.
pub mod metronome;
/// Pattern player for complex rhythm sequences with BeatClock integration.
pub mod patterns;
/// Beat event tracking and observation for audio-visual coupling.
pub mod tracker;
/// Timing subsystem for Polyphonica - Precision timing for musical applications
///
/// This module provides the core timing infrastructure for Guitar Buddy and other
/// musical applications requiring precise beat timing. The architecture separates
/// concerns into:
///
/// - **Core Types**: Shared timing data structures (TimeSignature, BeatEvent, etc.)
/// - **BeatClock**: High-precision timing abstraction with discrete scheduling
/// - **Metronome**: Simple regular beat implementation using BeatClock
/// - **PatternPlayer**: Complex rhythm pattern implementation using BeatClock
/// - **BeatTracker**: Event observation and emission for visualizer coupling
///
/// # Design Principles
///
/// 1. **Precision over Accuracy**: Beat-to-beat consistency (<5ms) is more important
///    than long-term accuracy
/// 2. **Discrete Scheduling**: Timing base resets prevent cumulative drift
/// 3. **Event-Driven**: Audio triggers drive visualization through observer pattern
/// 4. **Trait-Based**: Clean interfaces enable testing and extensibility
/// 5. **Zero-Allocation**: Real-time performance without memory allocation
///
/// # Usage Example
///
/// ```rust
/// use polyphonica::timing::{BeatClock, Metronome, TimeSignature};
///
/// let mut metronome = Metronome::new(TimeSignature::new(4, 4));
/// metronome.start();
///
/// // In audio loop
/// for event in metronome.check_triggers(120.0) {
///     // Trigger audio samples based on beat events
/// }
/// ```
pub mod types;

// Re-export core types for convenient access
pub use clock::BeatClock;
pub use metronome::Metronome;
pub use patterns::PatternPlayer;
pub use tracker::{BeatObserver, BeatTracker};
pub use types::{BeatEvent, ClickType, TimeSignature};
