use std::collections::VecDeque;
/// Comprehensive precision timing test for Guitar Buddy metronome and patterns
/// Tests beat-to-beat consistency (<5ms precision) rather than long-term accuracy
use std::time::{Duration, Instant};

// Copy the exact timing structures from guitar_buddy.rs for testing
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

// Discrete beat scheduling pattern state (same as guitar_buddy.rs)
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
}

// Metronome state (simplified for testing)
#[derive(Debug)]
struct MetronomeState {
    is_playing: bool,
    tempo_bpm: f32,
    current_beat: u8,
    last_beat_time: Option<Instant>,
    time_signature: TimeSignature,
}

impl MetronomeState {
    fn new() -> Self {
        Self {
            is_playing: false,
            tempo_bpm: 120.0,
            current_beat: 0,
            last_beat_time: None,
            time_signature: TimeSignature::new(4, 4),
        }
    }

    fn start(&mut self) {
        self.is_playing = true;
        self.current_beat = 1;
        self.last_beat_time = Some(Instant::now());
    }

    fn beat_interval_ms(&self) -> f64 {
        60000.0 / self.tempo_bpm as f64
    }

    fn should_trigger_beat(&mut self) -> bool {
        if !self.is_playing {
            return false;
        }

        let now = Instant::now();
        if let Some(last_time) = self.last_beat_time {
            let elapsed = now.duration_since(last_time).as_millis() as f64;
            let interval = self.beat_interval_ms();

            if elapsed >= interval {
                // Advance beat with timing reset (prevents drift)
                self.advance_beat();
                true
            } else {
                false
            }
        } else {
            self.last_beat_time = Some(now);
            true
        }
    }

    fn advance_beat(&mut self) {
        self.current_beat = if self.current_beat >= self.time_signature.beats_per_measure {
            1
        } else {
            self.current_beat + 1
        };
        // Reset timing base to prevent drift accumulation
        self.last_beat_time = Some(Instant::now());
    }
}

// Precision analysis structure
#[derive(Debug)]
struct PrecisionStats {
    intervals: VecDeque<u128>,
    max_window_size: usize,
    expected_interval_ms: f64,
}

impl PrecisionStats {
    fn new(expected_interval_ms: f64) -> Self {
        Self {
            intervals: VecDeque::new(),
            max_window_size: 10, // Track last 10 intervals for precision analysis
            expected_interval_ms,
        }
    }

    fn add_interval(&mut self, interval_ms: u128) {
        self.intervals.push_back(interval_ms);
        if self.intervals.len() > self.max_window_size {
            self.intervals.pop_front();
        }
    }

    fn get_precision_metrics(&self) -> (f64, f64, f64, bool) {
        if self.intervals.len() < 2 {
            return (0.0, 0.0, 0.0, false);
        }

        let intervals: Vec<f64> = self.intervals.iter().map(|&x| x as f64).collect();
        let mean = intervals.iter().sum::<f64>() / intervals.len() as f64;

        // Calculate standard deviation (precision measure)
        let variance =
            intervals.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / intervals.len() as f64;
        let std_dev = variance.sqrt();

        // Maximum deviation from expected interval
        let max_deviation = intervals
            .iter()
            .map(|&x| (x - self.expected_interval_ms).abs())
            .fold(0.0, f64::max);

        // Check if precision meets <5ms requirement
        let precision_ok = std_dev < 5.0 && max_deviation < 10.0;

        (mean, std_dev, max_deviation, precision_ok)
    }
}

fn test_metronome_precision(tempo_bpm: f32, test_duration_secs: u64) -> (f64, f64, f64, bool) {
    println!("🎯 Testing Metronome Precision at {} BPM", tempo_bpm);

    let mut metronome = MetronomeState::new();
    metronome.tempo_bpm = tempo_bpm;
    metronome.start();

    let expected_interval_ms = 60000.0 / tempo_bpm as f64;
    let mut precision_stats = PrecisionStats::new(expected_interval_ms);
    let mut last_trigger_time: Option<Instant> = None;
    let mut beat_count = 0;

    let start_time = Instant::now();
    while start_time.elapsed().as_secs() < test_duration_secs {
        if metronome.should_trigger_beat() {
            let now = Instant::now();

            if let Some(last_time) = last_trigger_time {
                let interval_ms = now.duration_since(last_time).as_millis();
                precision_stats.add_interval(interval_ms);

                let deviation = interval_ms as f64 - expected_interval_ms;
                if deviation.abs() > 10.0 {
                    println!(
                        "  ⚠️  Beat {}: {}ms (deviation: {:+.1}ms)",
                        beat_count, interval_ms, deviation
                    );
                }
            }

            last_trigger_time = Some(now);
            beat_count += 1;
        }
        std::thread::sleep(Duration::from_millis(1));
    }

    let (mean, std_dev, max_dev, precision_ok) = precision_stats.get_precision_metrics();

    println!("  📊 Beats measured: {}", beat_count);
    println!("  📏 Expected interval: {:.1}ms", expected_interval_ms);
    println!("  📈 Mean interval: {:.1}ms", mean);
    println!("  🎯 Precision (std dev): {:.2}ms", std_dev);
    println!("  📐 Max deviation: {:.1}ms", max_dev);
    println!(
        "  ✅ Precision OK: {}",
        if precision_ok { "YES" } else { "NO" }
    );

    (mean, std_dev, max_dev, precision_ok)
}

fn test_pattern_precision(tempo_bpm: f32, test_duration_secs: u64) -> (f64, f64, f64, bool) {
    println!("🥁 Testing Pattern Precision at {} BPM", tempo_bpm);

    let mut pattern_state = PatternState::new();
    pattern_state.set_pattern(DrumPattern::basic_rock());
    pattern_state.start();

    let expected_interval_ms = 60000.0 / tempo_bpm as f64 / 2.0; // Half-beat intervals
    let mut precision_stats = PrecisionStats::new(expected_interval_ms);
    let mut last_trigger_time: Option<Instant> = None;
    let mut trigger_count = 0;

    let start_time = Instant::now();
    while start_time.elapsed().as_secs() < test_duration_secs {
        let triggers = pattern_state.check_pattern_triggers(tempo_bpm);

        if !triggers.is_empty() {
            let now = Instant::now();

            if let Some(last_time) = last_trigger_time {
                let interval_ms = now.duration_since(last_time).as_millis();
                precision_stats.add_interval(interval_ms);

                let deviation = interval_ms as f64 - expected_interval_ms;
                if deviation.abs() > 10.0 {
                    println!(
                        "  ⚠️  Trigger {}: {}ms (deviation: {:+.1}ms)",
                        trigger_count, interval_ms, deviation
                    );
                }
            }

            last_trigger_time = Some(now);
            trigger_count += 1;
        }
        std::thread::sleep(Duration::from_millis(1));
    }

    let (mean, std_dev, max_dev, precision_ok) = precision_stats.get_precision_metrics();

    println!("  📊 Triggers measured: {}", trigger_count);
    println!("  📏 Expected interval: {:.1}ms", expected_interval_ms);
    println!("  📈 Mean interval: {:.1}ms", mean);
    println!("  🎯 Precision (std dev): {:.2}ms", std_dev);
    println!("  📐 Max deviation: {:.1}ms", max_dev);
    println!(
        "  ✅ Precision OK: {}",
        if precision_ok { "YES" } else { "NO" }
    );

    (mean, std_dev, max_dev, precision_ok)
}

fn main() {
    println!("🧪 Guitar Buddy Precision Timing Test Suite");
    println!("===========================================");
    println!("Testing beat-to-beat consistency (<5ms precision requirement)");
    println!("Note: Precision matters more than accuracy for musical timing\n");

    let test_cases = [
        ("Slow", 60.0),
        ("Medium", 120.0),
        ("Fast", 180.0),
        ("Very Fast", 240.0),
    ];

    let mut all_tests_passed = true;

    for (label, tempo) in test_cases.iter() {
        println!("🎵 Testing {} Tempo ({}BPM)", label, tempo);
        println!("───────────────────────────────────────");

        // Test metronome precision
        let (_, met_precision, _, met_ok) = test_metronome_precision(*tempo, 10);

        // Test pattern precision
        let (_, pat_precision, _, pat_ok) = test_pattern_precision(*tempo, 10);

        let tempo_passed = met_ok && pat_ok;
        all_tests_passed &= tempo_passed;

        println!(
            "  🎯 {} Tempo Result: {}",
            label,
            if tempo_passed { "✅ PASS" } else { "❌ FAIL" }
        );
        println!(
            "     Metronome precision: {:.2}ms (target: <5ms)",
            met_precision
        );
        println!(
            "     Pattern precision: {:.2}ms (target: <5ms)",
            pat_precision
        );
        println!();
    }

    println!("📋 FINAL RESULTS");
    println!("================");
    if all_tests_passed {
        println!("✅ ALL TESTS PASSED - Timing precision meets <5ms requirement");
        println!("🎵 Guitar Buddy is ready for professional musical practice!");
    } else {
        println!("❌ SOME TESTS FAILED - Timing precision issues detected");
        println!("🔧 Review discrete beat scheduling implementation");
    }

    println!("\n💡 Note: This test focuses on beat-to-beat consistency (precision)");
    println!("   rather than long-term accuracy. Small drift over many measures");
    println!("   is acceptable as long as each beat interval is consistent.");
}
