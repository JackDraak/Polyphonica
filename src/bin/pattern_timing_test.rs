/// Unit test for discrete beat scheduling timing precision
use std::time::{Duration, Instant};

// Copy the pattern structures for testing - using discrete beat scheduling
#[derive(Debug, Clone)]
struct DrumPatternBeat {
    beat_position: f32,
    accent: bool,
}

#[derive(Debug, Clone)]
struct TimeSignature {
    beats_per_measure: u8,
}

impl TimeSignature {
    fn new(beats: u8, _note_value: u8) -> Self {
        Self {
            beats_per_measure: beats,
        }
    }
}

#[derive(Debug, Clone)]
struct DrumPattern {
    time_signature: TimeSignature,
    beats: Vec<DrumPatternBeat>,
}

impl DrumPattern {
    fn basic_rock() -> Self {
        Self {
            time_signature: TimeSignature::new(4, 4),
            beats: vec![
                DrumPatternBeat {
                    beat_position: 1.0,
                    accent: true,
                },
                DrumPatternBeat {
                    beat_position: 1.5,
                    accent: false,
                },
                DrumPatternBeat {
                    beat_position: 2.0,
                    accent: false,
                },
                DrumPatternBeat {
                    beat_position: 2.5,
                    accent: false,
                },
                DrumPatternBeat {
                    beat_position: 3.0,
                    accent: false,
                },
                DrumPatternBeat {
                    beat_position: 3.5,
                    accent: false,
                },
                DrumPatternBeat {
                    beat_position: 4.0,
                    accent: false,
                },
                DrumPatternBeat {
                    beat_position: 4.5,
                    accent: false,
                },
            ],
        }
    }
}

// Discrete beat scheduling version (eliminates timing drift)
#[derive(Debug, Clone)]
struct PatternState {
    current_pattern: Option<DrumPattern>,
    current_beat_index: usize,
    next_beat_time: Option<Instant>,
    pattern_enabled: bool,
}

impl PatternState {
    fn new() -> Self {
        Self {
            current_pattern: None,
            current_beat_index: 0,
            next_beat_time: None,
            pattern_enabled: false,
        }
    }

    fn set_pattern(&mut self, pattern: DrumPattern) {
        self.current_pattern = Some(pattern);
        self.current_beat_index = 0;
        self.next_beat_time = None;
    }

    fn start(&mut self) {
        self.pattern_enabled = true;
        self.current_beat_index = 0;
        self.next_beat_time = None;
    }

    fn check_pattern_triggers(&mut self, tempo_bpm: f32) -> Vec<(bool, f32)> {
        if !self.pattern_enabled {
            return vec![];
        }

        let Some(ref pattern) = self.current_pattern else {
            return vec![];
        };

        let now = Instant::now();

        match self.next_beat_time {
            None => {
                // Start pattern
                if pattern.beats.is_empty() {
                    return vec![];
                }

                // Find beats at position 1.0
                let first_beat_triggers: Vec<(bool, f32)> = pattern
                    .beats
                    .iter()
                    .filter(|beat| (beat.beat_position - 1.0).abs() < 0.01)
                    .map(|beat| (beat.accent, beat.beat_position))
                    .collect();

                if !first_beat_triggers.is_empty() {
                    self.schedule_next_beat(tempo_bpm);
                    first_beat_triggers
                } else {
                    self.current_beat_index = 0;
                    self.schedule_next_beat(tempo_bpm);
                    vec![]
                }
            }
            Some(next_time) => {
                if now >= next_time {
                    // Trigger current beat
                    let current_beat = &pattern.beats[self.current_beat_index];
                    let triggers = vec![(current_beat.accent, current_beat.beat_position)];

                    // Advance to next beat with timing reset (prevents drift)
                    self.advance_to_next_beat(tempo_bpm);

                    triggers
                } else {
                    vec![]
                }
            }
        }
    }

    fn schedule_next_beat(&mut self, tempo_bpm: f32) {
        let Some(ref pattern) = self.current_pattern else {
            return;
        };

        if pattern.beats.is_empty() {
            return;
        }

        let beat_interval_ms = 60000.0 / tempo_bpm as f64;
        let current_beat = &pattern.beats[self.current_beat_index];

        let ms_from_beat_1 = (current_beat.beat_position - 1.0) as f64 * beat_interval_ms;
        self.next_beat_time = Some(Instant::now() + Duration::from_millis(ms_from_beat_1 as u64));
    }

    fn advance_to_next_beat(&mut self, tempo_bpm: f32) {
        let Some(ref pattern) = self.current_pattern else {
            return;
        };

        if pattern.beats.is_empty() {
            return;
        }

        // Advance beat index with looping
        self.current_beat_index = (self.current_beat_index + 1) % pattern.beats.len();

        let beat_interval_ms = 60000.0 / tempo_bpm as f64;
        let current_beat = &pattern.beats[self.current_beat_index];
        let next_beat_position = current_beat.beat_position;

        // Calculate interval to next beat (handles looping)
        let current_time = Instant::now();
        let interval_ms = if self.current_beat_index == 0 {
            // Looped back to start - calculate time to beat 1 of next measure
            let current_beat_in_pattern = &pattern.beats[pattern.beats.len() - 1];
            let loop_point = pattern.time_signature.beats_per_measure as f32 + 1.0;
            let remaining_time =
                (loop_point - current_beat_in_pattern.beat_position) as f64 * beat_interval_ms;
            let next_beat_time = (next_beat_position - 1.0) as f64 * beat_interval_ms;
            remaining_time + next_beat_time
        } else {
            // Normal advance within measure
            let prev_beat = &pattern.beats[self.current_beat_index - 1];
            (next_beat_position - prev_beat.beat_position) as f64 * beat_interval_ms
        };

        // Reset timing base - this prevents drift accumulation!
        self.next_beat_time = Some(current_time + Duration::from_millis(interval_ms as u64));
    }

    /// Get current beat position for display (1.0-based)
    fn get_current_beat_position(&self) -> f32 {
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
}

fn main() {
    println!("🧪 Testing Drum Pattern Loop Timing");

    let mut pattern_state = PatternState::new();
    pattern_state.set_pattern(DrumPattern::basic_rock());
    pattern_state.start();

    let tempo_bpm = 120.0; // 120 BPM = 500ms per beat, 2000ms per 4/4 measure
    let expected_beat_interval_ms = 60000.0 / tempo_bpm as f64; // 500ms per beat
    let expected_measure_duration_ms = expected_beat_interval_ms * 4.0; // 2000ms per measure

    println!("Expected beat interval: {:.1}ms", expected_beat_interval_ms);
    println!(
        "Expected measure duration: {:.1}ms",
        expected_measure_duration_ms
    );
    println!();

    let start_time = Instant::now();
    let mut measure_count = 0;
    let mut last_beat_1_time: Option<Instant> = None;
    let mut measure_intervals = Vec::new();

    // Run for 10 seconds to capture multiple loops
    while start_time.elapsed().as_millis() < 10000 {
        let triggers = pattern_state.check_pattern_triggers(tempo_bpm);

        if !triggers.is_empty() {
            println!(
                "Triggers: {:?}, Position: {:.3}",
                triggers,
                pattern_state.get_current_beat_position()
            );
        }

        for (is_accent, beat_position) in triggers {
            if is_accent && (beat_position - 1.0).abs() < 0.01 {
                // This is beat 1 (start of measure)
                measure_count += 1;
                let now = Instant::now();

                if let Some(last_time) = last_beat_1_time {
                    let interval = now.duration_since(last_time).as_millis();
                    measure_intervals.push(interval);

                    let error = interval as f64 - expected_measure_duration_ms;
                    println!(
                        "Measure {}: {:.0}ms (error: {:+.1}ms, position: {:.3})",
                        measure_count,
                        interval,
                        error,
                        pattern_state.get_current_beat_position()
                    );

                    // Flag significant timing errors
                    if error.abs() > 10.0 {
                        println!("⚠️  TIMING ERROR: {:.1}ms off expected!", error);
                    }
                }

                last_beat_1_time = Some(now);
            }
        }

        // Small sleep to prevent busy loop
        std::thread::sleep(Duration::from_millis(1));
    }

    println!();
    println!("📊 TIMING ANALYSIS:");
    println!("Measures captured: {}", measure_intervals.len());

    if !measure_intervals.is_empty() {
        let avg_interval: f64 = measure_intervals.iter().map(|&x| x as f64).sum::<f64>()
            / measure_intervals.len() as f64;
        let min_interval = *measure_intervals.iter().min().unwrap();
        let max_interval = *measure_intervals.iter().max().unwrap();

        println!(
            "Average interval: {:.1}ms (expected: {:.1}ms)",
            avg_interval, expected_measure_duration_ms
        );
        println!("Min interval: {}ms", min_interval);
        println!("Max interval: {}ms", max_interval);
        println!("Range: {}ms", max_interval - min_interval);

        let avg_error = avg_interval - expected_measure_duration_ms;
        println!("Average timing error: {:+.1}ms", avg_error);

        if avg_error.abs() > 5.0 {
            println!("❌ SIGNIFICANT TIMING DRIFT DETECTED!");
        } else if measure_intervals
            .iter()
            .any(|&x| (x as f64 - expected_measure_duration_ms).abs() > 10.0)
        {
            println!("⚠️  TIMING SPIKES DETECTED!");
        } else {
            println!("✅ Timing within acceptable range");
        }
    }
}
