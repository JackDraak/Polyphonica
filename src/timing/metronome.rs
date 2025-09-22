/// Simple metronome implementation using BeatClock
///
/// This module provides a straightforward metronome that plays regular beats
/// at a specified tempo and time signature.
use super::clock::{BeatClock, DiscreteScheduler};
use super::types::{BeatEvent, ClickType, TimeSignature, TriggerResult};

/// Simple metronome for regular beat timing
///
/// The Metronome provides basic timing functionality with configurable
/// time signatures and accent patterns. It implements BeatClock using
/// discrete scheduling for precision.
pub struct Metronome {
    /// Discrete scheduler for precise timing
    scheduler: DiscreteScheduler,

    /// Whether to accent the first beat of each measure
    accent_first_beat: bool,

    /// Click sound type to use
    click_type: ClickType,

    /// Accent click sound type (for downbeats)
    accent_click_type: ClickType,
}

impl Metronome {
    /// Create a new metronome with the given time signature
    pub fn new(time_signature: TimeSignature) -> Self {
        Self {
            scheduler: DiscreteScheduler::new(time_signature),
            accent_first_beat: true,
            click_type: ClickType::WoodBlock,
            accent_click_type: ClickType::Cowbell,
        }
    }

    /// Set the time signature
    pub fn set_time_signature(&mut self, time_signature: TimeSignature) {
        self.scheduler.set_time_signature(time_signature);
    }

    /// Set whether to accent the first beat
    pub fn set_accent_first_beat(&mut self, accent: bool) {
        self.accent_first_beat = accent;
    }

    /// Set the click sound type
    pub fn set_click_type(&mut self, click_type: ClickType) {
        self.click_type = click_type;
    }

    /// Set the accent click sound type (for downbeats)
    pub fn set_accent_click_type(&mut self, accent_click_type: ClickType) {
        self.accent_click_type = accent_click_type;
    }
}

impl BeatClock for Metronome {
    fn start(&mut self) {
        self.scheduler.start();
    }

    fn stop(&mut self) {
        self.scheduler.stop();
    }

    fn pause(&mut self) {
        self.scheduler.pause();
    }

    fn resume(&mut self) {
        self.scheduler.resume();
    }

    fn check_triggers(&mut self, tempo_bpm: f32) -> TriggerResult {
        if self.scheduler.should_trigger(tempo_bpm) {
            let current_beat = self.scheduler.current_beat();
            let is_downbeat = current_beat == 1;
            let should_accent = self.accent_first_beat && is_downbeat;

            // Choose appropriate click sound
            let click_sound = if should_accent {
                self.accent_click_type
            } else {
                self.click_type
            };

            // Create beat event
            let event = BeatEvent::new(
                current_beat,
                should_accent,
                vec![click_sound],
                tempo_bpm,
                self.scheduler.time_signature(),
            );

            // Advance to next beat
            self.scheduler.advance_beat(tempo_bpm);

            vec![event]
        } else {
            vec![]
        }
    }

    fn is_running(&self) -> bool {
        self.scheduler.is_running()
    }

    fn current_beat(&self) -> u8 {
        self.scheduler.current_beat()
    }

    fn time_signature(&self) -> TimeSignature {
        self.scheduler.time_signature()
    }

    fn set_time_signature(&mut self, time_signature: TimeSignature) {
        self.scheduler.set_time_signature(time_signature);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_metronome_creation() {
        let metronome = Metronome::new(TimeSignature::new(4, 4));
        assert_eq!(metronome.time_signature().beats_per_measure, 4);
        assert_eq!(metronome.time_signature().note_value, 4);
        assert!(!metronome.is_running());
        assert_eq!(metronome.current_beat(), 1);
    }

    #[test]
    fn test_metronome_start_stop() {
        let mut metronome = Metronome::new(TimeSignature::new(4, 4));

        assert!(!metronome.is_running());

        metronome.start();
        assert!(metronome.is_running());

        metronome.stop();
        assert!(!metronome.is_running());
        assert_eq!(metronome.current_beat(), 1);
    }

    #[test]
    fn test_metronome_first_beat_trigger() {
        let mut metronome = Metronome::new(TimeSignature::new(4, 4));
        metronome.start();

        // First beat should trigger immediately
        let events = metronome.check_triggers(120.0);
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.beat_number, 1);
        assert!(event.accent); // First beat should be accented by default
        assert_eq!(event.samples.len(), 1);
        assert_eq!(event.tempo_bpm, 120.0);
        assert!(event.is_downbeat());
    }

    #[test]
    fn test_metronome_beat_progression() {
        let mut metronome = Metronome::new(TimeSignature::new(3, 4));
        metronome.start();

        // Trigger first beat
        let events = metronome.check_triggers(240.0); // Fast tempo for testing
        assert_eq!(events[0].beat_number, 1);
        assert!(events[0].accent);

        // Should not trigger again immediately
        assert!(metronome.check_triggers(240.0).is_empty());

        // Wait for next beat (250ms at 240 BPM)
        thread::sleep(Duration::from_millis(260));

        let events = metronome.check_triggers(240.0);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].beat_number, 2); // Should be beat 2, not 3
        assert!(!events[0].accent); // Not first beat
    }

    #[test]
    fn test_metronome_accent_settings() {
        let mut metronome = Metronome::new(TimeSignature::new(4, 4));

        // Disable accent
        metronome.set_accent_first_beat(false);
        metronome.start();

        let events = metronome.check_triggers(120.0);
        assert!(!events[0].accent); // Should not be accented

        // Re-enable accent
        metronome.set_accent_first_beat(true);
        metronome.stop();
        metronome.start();

        let events = metronome.check_triggers(120.0);
        assert!(events[0].accent); // Should be accented again
    }

    #[test]
    fn test_metronome_click_types() {
        let mut metronome = Metronome::new(TimeSignature::new(4, 4));
        metronome.set_click_type(ClickType::DigitalBeep);
        metronome.set_accent_click_type(ClickType::AcousticKick);
        metronome.start();

        // First beat (accented) should use accent sound
        let events = metronome.check_triggers(120.0);
        assert_eq!(events[0].samples[0], ClickType::AcousticKick);

        // Disable accent to test normal click
        metronome.set_accent_first_beat(false);
        metronome.stop();
        metronome.start();

        let events = metronome.check_triggers(120.0);
        assert_eq!(events[0].samples[0], ClickType::DigitalBeep);
    }

    #[test]
    fn test_metronome_pause_resume() {
        let mut metronome = Metronome::new(TimeSignature::new(4, 4));
        metronome.start();

        // Trigger first beat
        let events = metronome.check_triggers(120.0);
        assert_eq!(events[0].beat_number, 1);

        metronome.pause();
        assert!(!metronome.is_running());

        metronome.resume();
        assert!(metronome.is_running());

        // Should maintain beat position after resume
        assert_eq!(metronome.current_beat(), 2);
    }

    #[test]
    fn test_metronome_time_signature_change() {
        let mut metronome = Metronome::new(TimeSignature::new(4, 4));

        metronome.set_time_signature(TimeSignature::new(3, 8));
        assert_eq!(metronome.time_signature().beats_per_measure, 3);
        assert_eq!(metronome.time_signature().note_value, 8);
    }
}
