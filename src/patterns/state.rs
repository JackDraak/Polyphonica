/// Pattern playback state management
///
/// This module handles the real-time state of pattern playback, including
/// beat scheduling, pattern progression, and timing precision. It uses
/// discrete beat scheduling to prevent timing drift during playback.
use super::types::DrumPattern;
use crate::timing::ClickType;
use std::time::{Duration, Instant};

/// Pattern playback state manager
///
/// Manages the real-time playback state of drum patterns using discrete
/// beat scheduling to maintain timing precision. This prevents the timing
/// drift that can accumulate with relative timing approaches.
#[derive(Debug, Clone)]
pub struct PatternState {
    /// Currently loaded pattern
    current_pattern: Option<DrumPattern>,

    /// Current beat index in the pattern (0-based)
    current_beat_index: usize,

    /// Absolute time when next beat should trigger
    next_beat_time: Option<Instant>,

    /// Whether pattern playback is enabled
    pattern_enabled: bool,

    /// Pattern playback statistics
    stats: PatternStats,
}

/// Pattern playback statistics
#[derive(Debug, Clone)]
pub struct PatternStats {
    /// Total beats played
    pub beats_played: u64,

    /// Pattern loops completed
    pub loops_completed: u32,

    /// Playback start time
    pub start_time: Option<Instant>,

    /// Last beat trigger time
    pub last_beat_time: Option<Instant>,
}

/// Pattern trigger result
#[derive(Debug, Clone)]
pub struct PatternTrigger {
    /// Sample to trigger
    pub click_type: ClickType,

    /// Whether this trigger is accented
    pub is_accent: bool,

    /// Beat position within measure
    pub beat_position: f32,

    /// Beat number for display (1-based)
    pub beat_number: u8,
}

impl PatternState {
    /// Create a new pattern state
    pub fn new() -> Self {
        Self {
            current_pattern: None,
            current_beat_index: 0,
            next_beat_time: None,
            pattern_enabled: false,
            stats: PatternStats::new(),
        }
    }

    /// Load a pattern for playback
    pub fn set_pattern(&mut self, pattern: DrumPattern) {
        self.current_pattern = Some(pattern);
        self.reset_playback_state();
    }

    /// Clear the current pattern
    pub fn clear_pattern(&mut self) {
        self.current_pattern = None;
        self.reset_playback_state();
    }

    /// Start pattern playback
    pub fn start(&mut self) {
        self.pattern_enabled = true;
        self.reset_playback_state();
        self.stats.start_time = Some(Instant::now());
    }

    /// Stop pattern playback
    pub fn stop(&mut self) {
        self.pattern_enabled = false;
        self.reset_playback_state();
        self.stats.start_time = None;
    }

    /// Pause pattern playback (preserves state)
    pub fn pause(&mut self) {
        self.pattern_enabled = false;
    }

    /// Resume pattern playback
    pub fn resume(&mut self) {
        self.pattern_enabled = true;
    }

    /// Check if pattern is currently playing
    pub fn is_playing(&self) -> bool {
        self.pattern_enabled && self.current_pattern.is_some()
    }

    /// Get current pattern reference
    pub fn current_pattern(&self) -> Option<&DrumPattern> {
        self.current_pattern.as_ref()
    }

    /// Get current beat position for display
    pub fn current_beat_position(&self) -> f32 {
        if let Some(ref pattern) = self.current_pattern {
            if !pattern.beats.is_empty() && self.current_beat_index < pattern.beats.len() {
                pattern.beats[self.current_beat_index].beat_position
            } else {
                1.0
            }
        } else {
            1.0
        }
    }

    /// Get current beat number (1-based)
    pub fn current_beat_number(&self) -> u8 {
        if let Some(ref pattern) = self.current_pattern {
            if !pattern.beats.is_empty() && self.current_beat_index < pattern.beats.len() {
                let beat_position = pattern.beats[self.current_beat_index].beat_position;
                (beat_position.floor() as u8).max(1)
            } else {
                1
            }
        } else {
            1
        }
    }

    /// Check for pattern triggers using discrete beat scheduling
    ///
    /// This method uses absolute time scheduling to prevent timing drift
    /// that can accumulate with relative timing approaches.
    pub fn check_pattern_triggers(&mut self, tempo_bpm: f32) -> Vec<PatternTrigger> {
        if !self.pattern_enabled {
            return vec![];
        }

        let Some(ref pattern) = self.current_pattern else {
            return vec![];
        };

        if pattern.beats.is_empty() {
            return vec![];
        }

        let now = Instant::now();

        match self.next_beat_time {
            None => {
                // Start pattern playback - find first beat at position 1.0
                let first_beat_triggers = self.collect_triggers_at_position(1.0);
                if !first_beat_triggers.is_empty() {
                    self.schedule_next_beat(tempo_bpm);
                    self.stats.beats_played += 1;
                    self.stats.last_beat_time = Some(now);
                    first_beat_triggers
                } else {
                    // No beat at position 1.0, schedule first available beat
                    self.current_beat_index = 0;
                    self.schedule_next_beat(tempo_bpm);
                    vec![]
                }
            }
            Some(next_time) => {
                // Check if it's time for the next beat
                if now >= next_time {
                    let current_beat = &pattern.beats[self.current_beat_index];
                    let _triggers = vec![PatternTrigger {
                        click_type: current_beat.samples[0], // Take first sample for now
                        is_accent: current_beat.accent,
                        beat_position: current_beat.beat_position,
                        beat_number: self.current_beat_number(),
                    }];

                    // Collect all sample triggers for this beat
                    let all_triggers: Vec<PatternTrigger> = current_beat
                        .samples
                        .iter()
                        .map(|&sample| PatternTrigger {
                            click_type: sample,
                            is_accent: current_beat.accent,
                            beat_position: current_beat.beat_position,
                            beat_number: self.current_beat_number(),
                        })
                        .collect();

                    // Advance to next beat with timing reset
                    self.advance_to_next_beat(tempo_bpm);
                    self.stats.beats_played += 1;
                    self.stats.last_beat_time = Some(now);

                    all_triggers
                } else {
                    // Not time for next beat yet
                    vec![]
                }
            }
        }
    }

    /// Get playback statistics
    pub fn stats(&self) -> &PatternStats {
        &self.stats
    }

    /// Reset internal playback state
    fn reset_playback_state(&mut self) {
        self.current_beat_index = 0;
        self.next_beat_time = None;
    }

    /// Collect all triggers at a specific beat position
    fn collect_triggers_at_position(&self, position: f32) -> Vec<PatternTrigger> {
        let Some(ref pattern) = self.current_pattern else {
            return vec![];
        };

        pattern
            .beats
            .iter()
            .filter(|beat| (beat.beat_position - position).abs() < 0.01)
            .flat_map(|beat| {
                beat.samples.iter().map(|&sample| PatternTrigger {
                    click_type: sample,
                    is_accent: beat.accent,
                    beat_position: beat.beat_position,
                    beat_number: (beat.beat_position.floor() as u8).max(1),
                })
            })
            .collect()
    }

    /// Schedule the next beat trigger time
    fn schedule_next_beat(&mut self, tempo_bpm: f32) {
        let Some(ref pattern) = self.current_pattern else {
            return;
        };

        if pattern.beats.is_empty() {
            return;
        }

        let beat_interval_ms = 60000.0 / tempo_bpm as f64;
        let current_beat = &pattern.beats[self.current_beat_index];

        // Calculate milliseconds from beat 1 to this beat position
        let ms_from_beat_1 = (current_beat.beat_position - 1.0) as f64 * beat_interval_ms;

        // Schedule absolute trigger time
        self.next_beat_time = Some(Instant::now() + Duration::from_millis(ms_from_beat_1 as u64));
    }

    /// Advance to next beat with timing reset to prevent drift
    fn advance_to_next_beat(&mut self, tempo_bpm: f32) {
        let Some(ref pattern) = self.current_pattern else {
            return;
        };

        if pattern.beats.is_empty() {
            return;
        }

        // Advance beat index with pattern looping
        let was_at_end = self.current_beat_index == pattern.beats.len() - 1;
        self.current_beat_index = (self.current_beat_index + 1) % pattern.beats.len();

        if was_at_end {
            self.stats.loops_completed += 1;
        }

        let beat_interval_ms = 60000.0 / tempo_bpm as f64;
        let current_beat = &pattern.beats[self.current_beat_index];
        let next_beat_position = current_beat.beat_position;

        // Calculate interval to next beat
        let current_time = Instant::now();
        let interval_ms = if self.current_beat_index == 0 {
            // Looped back to start of pattern
            let last_beat = &pattern.beats[pattern.beats.len() - 1];
            let measure_length = pattern.time_signature.beats_per_measure as f32;
            let remaining_time =
                (measure_length + 1.0 - last_beat.beat_position) as f64 * beat_interval_ms;
            let next_beat_time = (next_beat_position - 1.0) as f64 * beat_interval_ms;
            remaining_time + next_beat_time
        } else {
            // Normal advance within pattern
            let prev_beat = &pattern.beats[self.current_beat_index - 1];
            (next_beat_position - prev_beat.beat_position) as f64 * beat_interval_ms
        };

        // Reset timing base to prevent drift accumulation
        self.next_beat_time = Some(current_time + Duration::from_millis(interval_ms as u64));
    }
}

impl PatternStats {
    /// Create new pattern statistics
    fn new() -> Self {
        Self {
            beats_played: 0,
            loops_completed: 0,
            start_time: None,
            last_beat_time: None,
        }
    }

    /// Get playback duration
    pub fn playback_duration(&self) -> Option<Duration> {
        self.start_time.map(|start| {
            self.last_beat_time
                .unwrap_or_else(Instant::now)
                .duration_since(start)
        })
    }

    /// Get average tempo based on playback statistics
    pub fn average_tempo(&self) -> Option<f32> {
        if let Some(duration) = self.playback_duration() {
            if self.beats_played > 0 {
                let minutes = duration.as_secs_f32() / 60.0;
                Some(self.beats_played as f32 / minutes)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Reset statistics
    pub fn reset(&mut self) {
        self.beats_played = 0;
        self.loops_completed = 0;
        self.start_time = None;
        self.last_beat_time = None;
    }
}

impl Default for PatternState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::types::DrumPatternBeat;
    use crate::timing::TimeSignature;

    fn create_test_pattern() -> DrumPattern {
        DrumPattern::new("test", TimeSignature::new(4, 4))
            .with_beat(
                DrumPatternBeat::new(1.0)
                    .with_sample(ClickType::AcousticKick)
                    .with_accent(true),
            )
            .with_beat(DrumPatternBeat::new(2.0).with_sample(ClickType::AcousticSnare))
            .with_beat(DrumPatternBeat::new(3.0).with_sample(ClickType::AcousticKick))
            .with_beat(DrumPatternBeat::new(4.0).with_sample(ClickType::AcousticSnare))
    }

    #[test]
    fn test_pattern_state_creation() {
        let state = PatternState::new();
        assert!(!state.is_playing());
        assert_eq!(state.current_beat_number(), 1);
        assert_eq!(state.current_beat_position(), 1.0);
    }

    #[test]
    fn test_pattern_loading() {
        let mut state = PatternState::new();
        let pattern = create_test_pattern();

        state.set_pattern(pattern);
        assert!(state.current_pattern().is_some());
        assert_eq!(state.current_pattern().unwrap().name, "test");
    }

    #[test]
    fn test_pattern_playback_control() {
        let mut state = PatternState::new();
        let pattern = create_test_pattern();

        state.set_pattern(pattern);
        assert!(!state.is_playing());

        state.start();
        assert!(state.is_playing());

        state.pause();
        assert!(!state.is_playing());

        state.resume();
        assert!(state.is_playing());

        state.stop();
        assert!(!state.is_playing());
    }

    #[test]
    fn test_pattern_clear() {
        let mut state = PatternState::new();
        let pattern = create_test_pattern();

        state.set_pattern(pattern);
        state.start();
        assert!(state.is_playing());

        state.clear_pattern();
        assert!(!state.is_playing());
        assert!(state.current_pattern().is_none());
    }

    #[test]
    fn test_pattern_stats() {
        let mut state = PatternState::new();
        let pattern = create_test_pattern();

        state.set_pattern(pattern);
        let stats = state.stats();

        assert_eq!(stats.beats_played, 0);
        assert_eq!(stats.loops_completed, 0);
        assert!(stats.start_time.is_none());
    }

    #[test]
    fn test_pattern_trigger_collection() {
        let mut state = PatternState::new();
        let pattern = create_test_pattern();

        state.set_pattern(pattern);

        let triggers = state.collect_triggers_at_position(1.0);
        assert_eq!(triggers.len(), 1);
        assert_eq!(triggers[0].click_type, ClickType::AcousticKick);
        assert!(triggers[0].is_accent);
    }

    #[test]
    fn test_beat_position_and_number() {
        let mut state = PatternState::new();
        let pattern = create_test_pattern();

        state.set_pattern(pattern);

        assert_eq!(state.current_beat_position(), 1.0);
        assert_eq!(state.current_beat_number(), 1);
    }
}
