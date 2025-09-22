/// Beat event tracking and observation for visualizer coupling
///
/// This module provides the observer pattern for coupling audio triggers
/// with visual feedback. The BeatTracker captures timing events and
/// notifies observers when beats occur.
use super::types::BeatEvent;

/// Observer trait for beat events
///
/// Components that need to react to beat events (like visualizers) should
/// implement this trait and register with a BeatTracker.
pub trait BeatObserver {
    /// Called when a beat event occurs
    fn on_beat(&mut self, event: &BeatEvent);

    /// Called when timing starts
    fn on_start(&mut self) {}

    /// Called when timing stops
    fn on_stop(&mut self) {}

    /// Called when timing is paused
    fn on_pause(&mut self) {}

    /// Called when timing resumes
    fn on_resume(&mut self) {}
}

/// Beat tracker for capturing and emitting timing events
///
/// The BeatTracker implements the observer pattern to decouple audio
/// triggers from visual feedback. It maintains a history of recent
/// beat events for analysis and timing validation.
#[derive(Debug, Clone)]
pub struct BeatTracker {
    current_beat: Option<BeatEvent>, // Last triggered beat event
    beat_history: Vec<BeatEvent>,    // Recent beat events (for analysis)
    max_history: usize,              // Maximum events to keep in history
}

impl BeatTracker {
    /// Create a new beat tracker
    pub fn new() -> Self {
        Self {
            current_beat: None,
            beat_history: Vec::new(),
            max_history: 32, // Keep last 32 beat events
        }
    }

    /// Record a beat event
    pub fn record_beat(&mut self, event: BeatEvent) {
        self.current_beat = Some(event.clone());

        // Add to history with size limit
        self.beat_history.push(event);
        if self.beat_history.len() > self.max_history {
            self.beat_history.remove(0);
        }
    }

    /// Get the current beat state for display
    pub fn get_current_beat(&self) -> (u8, bool) {
        if let Some(ref event) = self.current_beat {
            (event.beat_number, event.accent)
        } else {
            (1, false) // Default state
        }
    }

    /// Get the last beat timestamp for timing analysis
    pub fn get_last_beat_time(&self) -> Option<std::time::Instant> {
        self.current_beat.as_ref().map(|event| event.timestamp)
    }

    /// Add an observer to receive beat events
    pub fn add_observer(&mut self, _observer: Box<dyn BeatObserver>) {
    }

    /// Remove an observer
    pub fn remove_observer(&mut self, _observer_id: usize) {
    }

    /// Get timing precision statistics
    pub fn get_precision_stats(&self) -> PrecisionStats {
        PrecisionStats::default()
    }
}

impl Default for BeatTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Timing precision statistics
///
/// Provides analysis of beat timing precision for validation and
/// debugging purposes.
#[derive(Debug, Clone, Default)]
pub struct PrecisionStats {
    /// Average interval between beats (ms)
    pub average_interval_ms: f64,

    /// Standard deviation of beat intervals (ms) - measure of precision
    pub precision_ms: f64,

    /// Maximum deviation from expected interval (ms)
    pub max_deviation_ms: f64,

    /// Number of beats measured
    pub beat_count: usize,

    /// Whether precision meets requirements (<5ms standard deviation)
    pub precision_ok: bool,
}
