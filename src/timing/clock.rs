use super::types::{TimeSignature, TriggerResult};
/// BeatClock trait and implementations for high-precision timing
///
/// This module provides the core timing abstraction used by metronomes and
/// pattern players. The BeatClock trait provides discrete beat scheduling
/// to prevent timing drift.
use std::time::{Duration, Instant};

/// High-precision timing abstraction for musical beats
///
/// BeatClock provides the foundation for all timing in the system. It uses
/// discrete beat scheduling to prevent the cumulative timing drift that
/// occurs with floating-point position tracking.
///
/// # Design Philosophy
///
/// Traditional timing systems accumulate timing errors by tracking the current
/// position as a floating-point value and incrementing it on each beat. This
/// leads to cumulative drift over time.
///
/// BeatClock implementations use discrete scheduling: each beat is scheduled
/// as an absolute future time, and the timing base is reset on each beat.
/// This prevents drift accumulation and maintains precision over long periods.
///
/// # Precision Requirements
///
/// Implementations should achieve <5ms beat-to-beat precision, with <1ms
/// preferred for professional musical practice. Precision (consistency)
/// is more important than accuracy (correctness over long periods).
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
///     println!("Beat {} at {:?}", event.beat_number, event.timestamp);
/// }
/// ```
pub trait BeatClock {
    /// Start the clock from the beginning
    ///
    /// Resets the clock to beat 1 and begins timing. Any previous timing
    /// state is cleared.
    fn start(&mut self);

    /// Stop the clock and reset position
    ///
    /// Stops timing and resets to the initial state. The next call to
    /// start() will begin from beat 1.
    fn stop(&mut self);

    /// Pause the clock without resetting position
    ///
    /// Suspends timing but maintains current beat position. The next call
    /// to resume() will continue from the current position.
    fn pause(&mut self);

    /// Resume the clock from current position
    ///
    /// Resumes timing from the current beat position after a pause.
    /// Has no effect if the clock is not paused.
    fn resume(&mut self);

    /// Check if any beats should be triggered at the given tempo
    ///
    /// This is the core method called in the audio loop. It returns a vector
    /// of BeatEvent objects representing beats that should be triggered now.
    ///
    /// Implementations should use discrete scheduling to maintain precision:
    /// - Calculate absolute trigger times, not relative offsets
    /// - Reset timing base on each beat to prevent drift
    /// - Return events only when the scheduled time has arrived
    ///
    /// # Arguments
    ///
    /// * `tempo_bpm` - Current tempo in beats per minute
    ///
    /// # Returns
    ///
    /// Vector of BeatEvent objects to trigger. Empty if no beats are due.
    fn check_triggers(&mut self, tempo_bpm: f32) -> TriggerResult;

    /// Check if the clock is currently running
    fn is_running(&self) -> bool;

    /// Get the current beat position (for display purposes)
    ///
    /// Returns the beat number within the current measure (1-based).
    /// This is primarily for visual feedback and should not be used
    /// for timing decisions.
    fn current_beat(&self) -> u8;

    /// Get the time signature for this clock
    fn time_signature(&self) -> TimeSignature;

    /// Set a new time signature
    ///
    /// Changes the time signature for future timing. May reset the current
    /// beat position depending on the implementation.
    fn set_time_signature(&mut self, time_signature: TimeSignature);
}

/// Discrete beat scheduler implementation
///
/// This provides the core discrete scheduling algorithm used by BeatClock
/// implementations. It prevents timing drift by scheduling absolute future
/// times and resetting the timing base on each beat.
#[derive(Debug, Clone)]
pub struct DiscreteScheduler {
    /// Current state of the scheduler
    state: SchedulerState,

    /// Time signature for beat calculations
    time_signature: TimeSignature,

    /// Current beat within the measure (1-based)
    current_beat: u8,

    /// Absolute time when the next beat should trigger
    next_beat_time: Option<Instant>,

    /// Whether the scheduler is currently running
    is_running: bool,
}

#[derive(Debug, Clone)]
enum SchedulerState {
    /// Scheduler is stopped
    Stopped,

    /// Scheduler is running normally
    Running,

    /// Scheduler is paused (maintains position)
    Paused { paused_at: Instant },
}

impl DiscreteScheduler {
    /// Create a new discrete scheduler
    pub fn new(time_signature: TimeSignature) -> Self {
        Self {
            state: SchedulerState::Stopped,
            time_signature,
            current_beat: 1,
            next_beat_time: None,
            is_running: false,
        }
    }

    /// Start the scheduler from beat 1
    pub fn start(&mut self) {
        self.state = SchedulerState::Running;
        self.current_beat = 1;
        self.next_beat_time = None;
        self.is_running = true;
    }

    /// Stop the scheduler and reset
    pub fn stop(&mut self) {
        self.state = SchedulerState::Stopped;
        self.current_beat = 1;
        self.next_beat_time = None;
        self.is_running = false;
    }

    /// Pause the scheduler
    pub fn pause(&mut self) {
        if matches!(self.state, SchedulerState::Running) {
            self.state = SchedulerState::Paused {
                paused_at: Instant::now(),
            };
            self.is_running = false;
        }
    }

    /// Resume the scheduler
    pub fn resume(&mut self) {
        if let SchedulerState::Paused { paused_at } = self.state {
            // Calculate how long we were paused and adjust next beat time
            let pause_duration = Instant::now().duration_since(paused_at);
            if let Some(ref mut next_time) = self.next_beat_time {
                *next_time += pause_duration;
            }
            self.state = SchedulerState::Running;
            self.is_running = true;
        }
    }

    /// Check if a beat should be triggered
    ///
    /// Returns true if it's time for the next beat. When true is returned,
    /// the caller should create a BeatEvent and call advance_beat().
    pub fn should_trigger(&mut self, tempo_bpm: f32) -> bool {
        if !self.is_running || !matches!(self.state, SchedulerState::Running) {
            return false;
        }

        let now = Instant::now();

        match self.next_beat_time {
            None => {
                // First beat - trigger immediately and schedule next
                self.schedule_next_beat(tempo_bpm);
                true
            }
            Some(scheduled_time) => {
                // Check if scheduled time has arrived
                if now >= scheduled_time {
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Advance to the next beat and schedule its timing
    ///
    /// This should be called after should_trigger() returns true and the
    /// BeatEvent has been created. It advances the beat counter and schedules
    /// the next beat timing.
    pub fn advance_beat(&mut self, tempo_bpm: f32) {
        // Advance beat counter with wrapping
        self.current_beat += 1;
        if self.current_beat > self.time_signature.beats_per_measure {
            self.current_beat = 1;
        }

        // Schedule next beat with timing base reset (prevents drift)
        self.schedule_next_beat(tempo_bpm);
    }

    /// Get current beat number
    pub fn current_beat(&self) -> u8 {
        self.current_beat
    }

    /// Get time signature
    pub fn time_signature(&self) -> TimeSignature {
        self.time_signature
    }

    /// Set time signature
    pub fn set_time_signature(&mut self, time_signature: TimeSignature) {
        self.time_signature = time_signature;
        // Reset beat if it exceeds new time signature
        if self.current_beat > time_signature.beats_per_measure {
            self.current_beat = 1;
        }
    }

    /// Check if scheduler is running
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Schedule the next beat trigger time
    ///
    /// This is the core of the discrete scheduling algorithm. Instead of
    /// accumulating timing offsets, it calculates the absolute time when
    /// the next beat should occur and resets the timing base.
    fn schedule_next_beat(&mut self, tempo_bpm: f32) {
        let beat_interval_ms = self.time_signature.beat_duration_ms(tempo_bpm);
        let next_beat_delay = Duration::from_millis(beat_interval_ms as u64);

        // Reset timing base - this is crucial for preventing drift
        self.next_beat_time = Some(Instant::now() + next_beat_delay);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_discrete_scheduler_basic() {
        let mut scheduler = DiscreteScheduler::new(TimeSignature::new(4, 4));

        // Should not trigger when stopped
        assert!(!scheduler.should_trigger(120.0));
        assert_eq!(scheduler.current_beat(), 1);

        // Start and should trigger first beat immediately
        scheduler.start();
        assert!(scheduler.is_running());
        assert!(scheduler.should_trigger(120.0));

        // Advance beat
        scheduler.advance_beat(120.0);
        assert_eq!(scheduler.current_beat(), 2);

        // Should not trigger again immediately
        assert!(!scheduler.should_trigger(120.0));
    }

    #[test]
    fn test_discrete_scheduler_timing() {
        let mut scheduler = DiscreteScheduler::new(TimeSignature::new(4, 4));
        let tempo = 240.0; // 250ms per beat for faster testing

        scheduler.start();

        // First beat should trigger immediately
        assert!(scheduler.should_trigger(tempo));
        scheduler.advance_beat(tempo);

        // Should not trigger immediately after advance
        assert!(!scheduler.should_trigger(tempo));

        // Wait for next beat (250ms + small buffer)
        thread::sleep(Duration::from_millis(260));

        // Should trigger next beat
        assert!(scheduler.should_trigger(tempo));
        scheduler.advance_beat(tempo);
        assert_eq!(scheduler.current_beat(), 3);
    }

    #[test]
    fn test_discrete_scheduler_pause_resume() {
        let mut scheduler = DiscreteScheduler::new(TimeSignature::new(4, 4));

        scheduler.start();
        assert!(scheduler.is_running());

        scheduler.pause();
        assert!(!scheduler.is_running());

        scheduler.resume();
        assert!(scheduler.is_running());
    }

    #[test]
    fn test_discrete_scheduler_beat_wrapping() {
        let mut scheduler = DiscreteScheduler::new(TimeSignature::new(3, 4));

        scheduler.start();

        // Test that beats wrap correctly from 3 back to 1
        assert_eq!(scheduler.current_beat(), 1);

        // Manually advance through several beats to test wrapping
        scheduler.advance_beat(120.0);
        assert_eq!(scheduler.current_beat(), 2);

        scheduler.advance_beat(120.0);
        assert_eq!(scheduler.current_beat(), 3);

        scheduler.advance_beat(120.0);
        assert_eq!(scheduler.current_beat(), 1); // Should wrap back to 1

        scheduler.advance_beat(120.0);
        assert_eq!(scheduler.current_beat(), 2);
    }

    #[test]
    fn test_time_signature_change() {
        let mut scheduler = DiscreteScheduler::new(TimeSignature::new(4, 4));

        scheduler.start();
        scheduler.advance_beat(120.0);
        scheduler.advance_beat(120.0);
        scheduler.advance_beat(120.0);
        assert_eq!(scheduler.current_beat(), 4);

        // Change to 3/4 - should reset to beat 1
        scheduler.set_time_signature(TimeSignature::new(3, 4));
        assert_eq!(scheduler.current_beat(), 1);
        assert_eq!(scheduler.time_signature().beats_per_measure, 3);
    }
}
