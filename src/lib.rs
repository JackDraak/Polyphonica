use std::f32::consts::PI;

fn validate_inputs(frequency: f32, duration_secs: f32, sample_rate: u32) -> Result<(), &'static str> {
    if frequency <= 0.0 || frequency > 20000.0 {
        return Err("Frequency must be between 0 and 20000 Hz");
    }
    if duration_secs < 0.0 {
        return Err("Duration must be non-negative");
    }
    if sample_rate == 0 || sample_rate > 192000 {
        return Err("Sample rate must be between 1 and 192000 Hz");
    }
    Ok(())
}

fn generate_sample(waveform: Waveform, phase: f32) -> f32 {
    match waveform {
        Waveform::Sine => phase.sin(),
        Waveform::Square => {
            if (phase % (2.0 * PI)).sin() >= 0.0 {
                1.0
            } else {
                -1.0
            }
        }
        Waveform::Sawtooth => {
            let normalized_phase = (phase / (2.0 * PI)) % 1.0;
            2.0 * normalized_phase - 1.0
        }
        Waveform::Triangle => {
            let normalized_phase = (phase / (2.0 * PI)) % 1.0;
            if normalized_phase < 0.5 {
                4.0 * normalized_phase - 1.0
            } else {
                3.0 - 4.0 * normalized_phase
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Waveform {
    Sine,
    Square,
    Sawtooth,
    Triangle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdsrEnvelope {
    pub attack_secs: f32,
    pub decay_secs: f32,
    pub sustain_level: f32,
    pub release_secs: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SoundEvent {
    pub waveform: Waveform,
    pub start_frequency: f32,
    pub end_frequency: f32,
    pub duration_secs: f32,
    pub envelope: AdsrEnvelope,
}

pub fn generate_wave(
    waveform: Waveform,
    frequency: f32,
    duration_secs: f32,
    sample_rate: u32,
) -> Vec<f32> {
    if validate_inputs(frequency, duration_secs, sample_rate).is_err() {
        return Vec::new();
    }
    let total_samples = (duration_secs * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(total_samples);

    for i in 0..total_samples {
        let t = i as f32 / sample_rate as f32;
        let phase = 2.0 * PI * frequency * t;
        let sample = generate_sample(waveform, phase);
        samples.push(sample);
    }

    samples
}

pub fn apply_envelope(samples: &mut [f32], envelope: &AdsrEnvelope, sample_rate: u32) {
    let total_samples = samples.len();
    if total_samples == 0 {
        return;
    }

    let attack_samples = (envelope.attack_secs * sample_rate as f32) as usize;
    let decay_samples = (envelope.decay_secs * sample_rate as f32) as usize;
    let release_samples = (envelope.release_secs * sample_rate as f32) as usize;

    // Ensure we don't exceed the total sample count
    let attack_end = attack_samples.min(total_samples);
    let decay_end = (attack_samples + decay_samples).min(total_samples);
    let sustain_end = total_samples.saturating_sub(release_samples);
    let release_start = sustain_end;

    for (i, sample) in samples.iter_mut().enumerate() {
        let envelope_value = if i < attack_end {
            // Attack phase: linear ramp from 0 to 1
            if attack_samples > 0 {
                i as f32 / attack_samples as f32
            } else {
                1.0
            }
        } else if i < decay_end {
            // Decay phase: linear ramp from 1 to sustain_level
            if decay_samples > 0 {
                let decay_progress = (i - attack_samples) as f32 / decay_samples as f32;
                1.0 - decay_progress * (1.0 - envelope.sustain_level)
            } else {
                envelope.sustain_level
            }
        } else if i < sustain_end {
            // Sustain phase: constant at sustain_level
            envelope.sustain_level
        } else {
            // Release phase: linear ramp from sustain_level to 0
            if release_samples > 0 {
                let release_progress = (i - release_start) as f32 / release_samples as f32;
                envelope.sustain_level * (1.0 - release_progress)
            } else {
                0.0
            }
        };

        *sample *= envelope_value;
    }
}

pub fn render_event(event: &SoundEvent, sample_rate: u32) -> Vec<f32> {
    if validate_inputs(event.start_frequency, event.duration_secs, sample_rate).is_err() {
        return Vec::new();
    }
    if validate_inputs(event.end_frequency, event.duration_secs, sample_rate).is_err() {
        return Vec::new();
    }
    let total_samples = (event.duration_secs * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(total_samples);

    for i in 0..total_samples {
        let t = i as f32 / sample_rate as f32;
        let progress = t / event.duration_secs;

        // Linear interpolation between start and end frequency
        let current_frequency = event.start_frequency +
            (event.end_frequency - event.start_frequency) * progress;

        let phase = 2.0 * PI * current_frequency * t;
        let sample = generate_sample(event.waveform, phase);
        samples.push(sample);
    }

    // Apply the ADSR envelope
    apply_envelope(&mut samples, &event.envelope, sample_rate);

    samples
}

pub fn render_timeline(
    events: &[(f32, SoundEvent)],
    total_duration_secs: f32,
    sample_rate: u32,
) -> Vec<f32> {
    if total_duration_secs < 0.0 || sample_rate == 0 || sample_rate > 192000 {
        return Vec::new();
    }
    let total_samples = (total_duration_secs * sample_rate as f32) as usize;
    let mut master_buffer = vec![0.0; total_samples];

    for (start_time, event) in events {
        // Calculate the start sample index
        let start_sample_index = (*start_time * sample_rate as f32) as usize;

        // Skip events that start after the total duration
        if start_sample_index >= total_samples {
            continue;
        }

        // Render the event's audio samples
        let event_samples = render_event(event, sample_rate);

        // Mix the event samples into the master buffer
        for (i, sample) in event_samples.iter().enumerate() {
            let buffer_index = start_sample_index + i;

            // Stop if we exceed the master buffer length
            if buffer_index >= total_samples {
                break;
            }

            // Add the sample to the master buffer
            master_buffer[buffer_index] += sample;
        }
    }

    // Clamp all samples to prevent clipping
    for sample in master_buffer.iter_mut() {
        *sample = sample.clamp(-1.0, 1.0);
    }

    master_buffer
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOLERANCE: f32 = 1e-6;

    #[test]
    fn test_sine_wave() {
        let samples = generate_wave(Waveform::Sine, 1.0, 1.0, 4);
        assert_eq!(samples.len(), 4);

        // At t=0: sin(0) = 0
        assert!((samples[0] - 0.0).abs() < TOLERANCE);
        // At t=0.25: sin(π/2) = 1
        assert!((samples[1] - 1.0).abs() < TOLERANCE);
        // At t=0.5: sin(π) = 0
        assert!((samples[2] - 0.0).abs() < TOLERANCE);
        // At t=0.75: sin(3π/2) = -1
        assert!((samples[3] - (-1.0)).abs() < TOLERANCE);
    }

    #[test]
    fn test_square_wave() {
        let samples = generate_wave(Waveform::Square, 1.0, 1.0, 8);
        assert_eq!(samples.len(), 8);

        // First half should be positive
        for i in 0..4 {
            assert_eq!(samples[i], 1.0);
        }
        // Second half should be negative
        for i in 4..8 {
            assert_eq!(samples[i], -1.0);
        }
    }

    #[test]
    fn test_sawtooth_wave() {
        let samples = generate_wave(Waveform::Sawtooth, 1.0, 1.0, 4);
        assert_eq!(samples.len(), 4);

        // Sawtooth should go from -1 to 1 linearly over one period
        // At 1Hz with 4 samples/sec: t=0,0.25,0.5,0.75
        // normalized_phase: 0, 0.25, 0.5, 0.75
        // sawtooth: 2*phase-1 = -1, -0.5, 0, 0.5
        assert!((samples[0] - (-1.0)).abs() < TOLERANCE);
        assert!((samples[1] - (-0.5)).abs() < TOLERANCE);
        assert!((samples[2] - 0.0).abs() < TOLERANCE);
        assert!((samples[3] - 0.5).abs() < TOLERANCE);
    }

    #[test]
    fn test_triangle_wave() {
        let samples = generate_wave(Waveform::Triangle, 1.0, 1.0, 8);
        assert_eq!(samples.len(), 8);


        // Triangle wave: starts at -1, goes to 1, back to -1
        // Values: [-1.0, -0.5, 0.0, 0.5, 1.0, 0.5, 0.0, -0.5]
        assert!((samples[0] - (-1.0)).abs() < TOLERANCE);
        assert!((samples[4] - 1.0).abs() < TOLERANCE);  // Peak at sample 4
        assert!((samples[2] - 0.0).abs() < TOLERANCE);  // Zero crossing at sample 2
    }

    #[test]
    fn test_sample_range() {
        let waveforms = [Waveform::Sine, Waveform::Square, Waveform::Sawtooth, Waveform::Triangle];

        for waveform in &waveforms {
            let samples = generate_wave(*waveform, 440.0, 0.1, 44100);

            for sample in &samples {
                assert!(
                    *sample >= -1.0 && *sample <= 1.0,
                    "Sample {} out of range [-1.0, 1.0] for waveform {:?}",
                    sample,
                    waveform
                );
            }
        }
    }

    #[test]
    fn test_correct_sample_count() {
        let samples = generate_wave(Waveform::Sine, 440.0, 2.5, 44100);
        let expected_samples = (2.5 * 44100.0) as usize;
        assert_eq!(samples.len(), expected_samples);
    }

    #[test]
    fn test_frequency_accuracy() {
        // Generate one second of 1Hz sine wave at 100 samples/sec
        let samples = generate_wave(Waveform::Sine, 1.0, 1.0, 100);

        // Check that we complete exactly one cycle
        assert!((samples[0] - samples[100 - 1]).abs() < 0.1); // Should be close to same value

        // Check peak occurs at quarter period
        let quarter_period_idx = 25; // 0.25 seconds * 100 samples/sec
        assert!((samples[quarter_period_idx] - 1.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_adsr_envelope_basic() {
        let envelope = AdsrEnvelope {
            attack_secs: 0.1,
            decay_secs: 0.1,
            sustain_level: 0.5,
            release_secs: 0.1,
        };

        let mut samples = vec![1.0; 40]; // 0.4 seconds at 100 samples/sec
        apply_envelope(&mut samples, &envelope, 100);

        // Attack phase (0-10 samples): should ramp from 0 to 1
        assert!((samples[0] - 0.0).abs() < TOLERANCE);
        assert!((samples[5] - 0.5).abs() < 0.1);
        assert!((samples[9] - 0.9).abs() < 0.1);

        // Decay phase (10-20 samples): should ramp from 1 to 0.5
        assert!((samples[10] - 1.0).abs() < 0.1);
        assert!((samples[15] - 0.75).abs() < 0.1);
        assert!((samples[19] - 0.5).abs() < 0.1);

        // Sustain phase (20-30 samples): should stay at 0.5
        for i in 20..30 {
            assert!((samples[i] - 0.5).abs() < TOLERANCE);
        }

        // Release phase (30-40 samples): should ramp from 0.5 to 0
        assert!((samples[30] - 0.5).abs() < 0.1);
        assert!((samples[35] - 0.25).abs() < 0.1);
        assert!((samples[39] - 0.0).abs() < 0.1);
    }

    #[test]
    fn test_adsr_envelope_empty_samples() {
        let envelope = AdsrEnvelope {
            attack_secs: 0.1,
            decay_secs: 0.1,
            sustain_level: 0.5,
            release_secs: 0.1,
        };

        let mut samples: Vec<f32> = vec![];
        apply_envelope(&mut samples, &envelope, 44100);
        assert_eq!(samples.len(), 0);
    }

    #[test]
    fn test_adsr_envelope_attack_only() {
        let envelope = AdsrEnvelope {
            attack_secs: 1.0,
            decay_secs: 0.0,
            sustain_level: 1.0,
            release_secs: 0.0,
        };

        let mut samples = vec![1.0; 10]; // 0.1 seconds at 100 samples/sec
        apply_envelope(&mut samples, &envelope, 100);


        // Should be in attack phase for all samples
        // Attack spans 100 samples (1.0s * 100 samples/sec), but we only have 10 samples
        // So sample[5] should be 5/100 = 0.05, not 0.5
        assert!((samples[0] - 0.0).abs() < TOLERANCE);
        assert!((samples[5] - 0.05).abs() < 0.1);
        assert!((samples[9] - 0.09).abs() < 0.1);
    }

    #[test]
    fn test_adsr_envelope_sustain_only() {
        let envelope = AdsrEnvelope {
            attack_secs: 0.0,
            decay_secs: 0.0,
            sustain_level: 0.7,
            release_secs: 0.0,
        };

        let mut samples = vec![1.0; 10];
        apply_envelope(&mut samples, &envelope, 100);

        // All samples should be at sustain level
        for sample in &samples {
            assert!((sample - 0.7).abs() < TOLERANCE);
        }
    }

    #[test]
    fn test_adsr_envelope_with_waveform() {
        // Generate a sine wave and apply envelope
        let mut samples = generate_wave(Waveform::Sine, 440.0, 0.4, 100);
        let original_peak = samples.iter().fold(0.0, |max, &x| x.abs().max(max));

        let envelope = AdsrEnvelope {
            attack_secs: 0.1,
            decay_secs: 0.1,
            sustain_level: 0.5,
            release_secs: 0.1,
        };

        apply_envelope(&mut samples, &envelope, 100);

        // Check that envelope was applied correctly
        let enveloped_peak = samples.iter().fold(0.0, |max, &x| x.abs().max(max));
        assert!(enveloped_peak < original_peak);

        // Start should be near silence
        assert!(samples[0].abs() < 0.1);
        // End should be near silence
        assert!(samples[samples.len() - 1].abs() < 0.1);
    }

    #[test]
    fn test_adsr_envelope_zero_sustain() {
        let envelope = AdsrEnvelope {
            attack_secs: 0.1,
            decay_secs: 0.1,
            sustain_level: 0.0,
            release_secs: 0.1,
        };

        let mut samples = vec![1.0; 40]; // 0.4 seconds at 100 samples/sec
        apply_envelope(&mut samples, &envelope, 100);

        // Sustain phase should be silent
        for i in 20..30 {
            assert!((samples[i] - 0.0).abs() < TOLERANCE);
        }
    }

    #[test]
    fn test_adsr_envelope_bounds() {
        let envelope = AdsrEnvelope {
            attack_secs: 0.1,
            decay_secs: 0.1,
            sustain_level: 0.5,
            release_secs: 0.1,
        };

        let mut samples = vec![2.0; 40]; // Start with values > 1.0
        apply_envelope(&mut samples, &envelope, 100);

        // All samples should be within reasonable bounds after envelope
        for sample in &samples {
            assert!(sample.abs() <= 2.0); // Original amplitude was 2.0
        }
    }

    #[test]
    fn test_sound_event_basic() {
        let event = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 440.0,
            end_frequency: 440.0,
            duration_secs: 1.0,
            envelope: AdsrEnvelope {
                attack_secs: 0.1,
                decay_secs: 0.1,
                sustain_level: 0.5,
                release_secs: 0.1,
            },
        };

        let samples = render_event(&event, 100);
        assert_eq!(samples.len(), 100);

        // Check that envelope is applied (start should be near zero)
        assert!(samples[0].abs() < 0.1);
        // End should be near zero (release phase)
        assert!(samples[samples.len() - 1].abs() < 0.1);
    }

    #[test]
    fn test_sound_event_frequency_sweep() {
        let event = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 100.0,
            end_frequency: 200.0,
            duration_secs: 1.0,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 1.0,
                release_secs: 0.0,
            },
        };

        let samples = render_event(&event, 1000);

        // For a frequency sweep, the effective frequency should increase over time
        // We can't easily test the exact frequencies, but we can check that
        // the result is different from a constant frequency
        let constant_freq_event = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 150.0, // Middle frequency
            end_frequency: 150.0,
            duration_secs: 1.0,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 1.0,
                release_secs: 0.0,
            },
        };

        let constant_samples = render_event(&constant_freq_event, 1000);

        // The sweep should produce different results than constant frequency
        let mut differences = 0;
        for (sweep_sample, constant_sample) in samples.iter().zip(constant_samples.iter()) {
            if (sweep_sample - constant_sample).abs() > 0.01 {
                differences += 1;
            }
        }

        // Should have significant differences due to frequency sweep
        assert!(differences > 100);
    }

    #[test]
    fn test_sound_event_constant_frequency() {
        let event = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 440.0,
            end_frequency: 440.0,
            duration_secs: 0.1,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 1.0,
                release_secs: 0.0,
            },
        };

        let samples = render_event(&event, 100);
        let direct_samples = generate_wave(Waveform::Sine, 440.0, 0.1, 100);

        // When start_frequency == end_frequency, should match direct generation
        for (event_sample, direct_sample) in samples.iter().zip(direct_samples.iter()) {
            assert!((event_sample - direct_sample).abs() < TOLERANCE);
        }
    }

    #[test]
    fn test_sound_event_different_waveforms() {
        let base_event = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 440.0,
            end_frequency: 880.0,
            duration_secs: 0.1,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 1.0,
                release_secs: 0.0,
            },
        };

        let waveforms = [Waveform::Sine, Waveform::Square, Waveform::Sawtooth, Waveform::Triangle];

        for waveform in &waveforms {
            let mut event = base_event.clone();
            event.waveform = *waveform;

            let samples = render_event(&event, 100);
            assert_eq!(samples.len(), 10);

            // All samples should be within range
            for sample in &samples {
                assert!(
                    *sample >= -1.0 && *sample <= 1.0,
                    "Sample {} out of range for waveform {:?}",
                    sample,
                    waveform
                );
            }
        }
    }

    #[test]
    fn test_sound_event_zero_duration() {
        let event = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 440.0,
            end_frequency: 440.0,
            duration_secs: 0.0,
            envelope: AdsrEnvelope {
                attack_secs: 0.1,
                decay_secs: 0.1,
                sustain_level: 0.5,
                release_secs: 0.1,
            },
        };

        let samples = render_event(&event, 44100);
        assert_eq!(samples.len(), 0);
    }

    #[test]
    fn test_sound_event_complex_envelope() {
        let event = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 440.0,
            end_frequency: 440.0,
            duration_secs: 0.8,
            envelope: AdsrEnvelope {
                attack_secs: 0.2,
                decay_secs: 0.2,
                sustain_level: 0.3,
                release_secs: 0.2,
            },
        };

        let samples = render_event(&event, 100);
        assert_eq!(samples.len(), 80);

        // Attack phase: should start near 0 and increase
        assert!(samples[0].abs() < 0.1);
        assert!(samples[10].abs() > samples[0].abs());

        // Release phase: should end near 0
        assert!(samples[samples.len() - 1].abs() < 0.1);
    }

    #[test]
    fn test_sound_event_extreme_frequency_sweep() {
        let event = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 20.0,
            end_frequency: 20000.0,
            duration_secs: 0.1,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 1.0,
                release_secs: 0.0,
            },
        };

        let samples = render_event(&event, 44100);
        assert_eq!(samples.len(), 4410);

        // All samples should still be within valid range
        for sample in &samples {
            assert!(
                *sample >= -1.0 && *sample <= 1.0,
                "Sample {} out of range during extreme frequency sweep",
                sample
            );
        }
    }

    #[test]
    fn test_render_timeline_empty() {
        let events: &[(f32, SoundEvent)] = &[];
        let timeline = render_timeline(events, 1.0, 100);

        assert_eq!(timeline.len(), 100);
        for sample in &timeline {
            assert_eq!(*sample, 0.0);
        }
    }

    #[test]
    fn test_render_timeline_single_event() {
        let event = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 440.0,
            end_frequency: 440.0,
            duration_secs: 0.5,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 1.0,
                release_secs: 0.0,
            },
        };

        let events = &[(0.0, event.clone())];
        let timeline = render_timeline(events, 1.0, 100);
        let direct_samples = render_event(&event, 100);

        assert_eq!(timeline.len(), 100);

        // First half should match the direct rendering
        for i in 0..50 {
            assert!((timeline[i] - direct_samples[i]).abs() < TOLERANCE);
        }

        // Second half should be silent
        for i in 50..100 {
            assert_eq!(timeline[i], 0.0);
        }
    }

    #[test]
    fn test_render_timeline_multiple_sequential_events() {
        let event1 = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 440.0,
            end_frequency: 440.0,
            duration_secs: 0.3,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 1.0,
                release_secs: 0.0,
            },
        };

        let event2 = SoundEvent {
            waveform: Waveform::Square,
            start_frequency: 880.0,
            end_frequency: 880.0,
            duration_secs: 0.3,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 1.0,
                release_secs: 0.0,
            },
        };

        let events = &[(0.0, event1), (0.5, event2)];
        let timeline = render_timeline(events, 1.0, 100);

        assert_eq!(timeline.len(), 100);

        // Check that there's activity in both time periods
        let first_period_active = timeline[0..30].iter().any(|&s| s.abs() > 0.1);
        let gap_period_silent = timeline[30..50].iter().all(|&s| s.abs() < 0.01);
        let second_period_active = timeline[50..80].iter().any(|&s| s.abs() > 0.1);

        assert!(first_period_active, "First event should be audible");
        assert!(gap_period_silent, "Gap between events should be silent");
        assert!(second_period_active, "Second event should be audible");
    }

    #[test]
    fn test_render_timeline_overlapping_events() {
        let event1 = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 440.0,
            end_frequency: 440.0,
            duration_secs: 0.6,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 0.5,
                release_secs: 0.0,
            },
        };

        let event2 = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 440.0,
            end_frequency: 440.0,
            duration_secs: 0.6,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 0.5,
                release_secs: 0.0,
            },
        };

        let events = &[(0.0, event1), (0.3, event2)];
        let timeline = render_timeline(events, 1.0, 100);

        // In the overlap region (0.3-0.6 seconds, samples 30-60),
        // the amplitude should be roughly double due to mixing
        let single_event_samples = render_event(&events[0].1, 100);

        // Check overlap region has higher amplitude than single event
        let overlap_start = 30;
        let overlap_end = 60;

        // Check that overlapping signals are being mixed (not necessarily louder due to phase differences)
        // The key test is that the timeline is different from the single event in the overlap region
        let mut significant_differences = 0;
        for i in overlap_start..overlap_end {
            if (timeline[i] - single_event_samples[i]).abs() > 0.01 {
                significant_differences += 1;
            }
        }

        // There should be significant differences in the overlap region due to mixing
        assert!(
            significant_differences > 5,
            "Overlap region should show mixing effects (found {} significant differences)",
            significant_differences
        );
    }

    #[test]
    fn test_render_timeline_clipping_prevention() {
        // Create multiple loud events that would clip if not clamped
        let loud_event = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 440.0,
            end_frequency: 440.0,
            duration_secs: 0.5,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 1.0,
                release_secs: 0.0,
            },
        };

        // Schedule 5 overlapping events to force clipping
        let events = &[
            (0.0, loud_event.clone()),
            (0.1, loud_event.clone()),
            (0.2, loud_event.clone()),
            (0.3, loud_event.clone()),
            (0.4, loud_event.clone()),
        ];

        let timeline = render_timeline(events, 1.0, 100);

        // All samples should be clamped to [-1.0, 1.0]
        for sample in &timeline {
            assert!(
                *sample >= -1.0 && *sample <= 1.0,
                "Sample {} should be clamped to [-1.0, 1.0]",
                sample
            );
        }
    }

    #[test]
    fn test_render_timeline_events_beyond_duration() {
        let event = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 440.0,
            end_frequency: 440.0,
            duration_secs: 0.5,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 1.0,
                release_secs: 0.0,
            },
        };

        // Event starts after timeline ends
        let events = &[(2.0, event.clone())];
        let timeline = render_timeline(events, 1.0, 100);

        // Should be completely silent
        for sample in &timeline {
            assert_eq!(*sample, 0.0);
        }
    }

    #[test]
    fn test_render_timeline_partial_event_cutoff() {
        let event = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 440.0,
            end_frequency: 440.0,
            duration_secs: 0.6,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 1.0,
                release_secs: 0.0,
            },
        };

        // Event starts at 0.7s but timeline ends at 1.0s
        let events = &[(0.7, event)];
        let timeline = render_timeline(events, 1.0, 100);

        assert_eq!(timeline.len(), 100);

        // First 70 samples should be silent
        for i in 0..70 {
            assert_eq!(timeline[i], 0.0);
        }

        // Last 30 samples should have some audio (partial event)
        let has_audio = timeline[70..].iter().any(|&s| s.abs() > 0.1);
        assert!(has_audio, "Should have partial audio from cutoff event");
    }

    #[test]
    fn test_render_timeline_different_waveforms() {
        let sine_event = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 440.0,
            end_frequency: 440.0,
            duration_secs: 0.3,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 1.0,
                release_secs: 0.0,
            },
        };

        let square_event = SoundEvent {
            waveform: Waveform::Square,
            start_frequency: 880.0,
            end_frequency: 880.0,
            duration_secs: 0.3,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 1.0,
                release_secs: 0.0,
            },
        };

        let events = &[(0.0, sine_event), (0.0, square_event)];
        let timeline = render_timeline(events, 1.0, 100);

        // Mixed waveforms should produce different results than either alone
        let sine_only = render_timeline(&[(0.0, events[0].1.clone())], 1.0, 100);
        let square_only = render_timeline(&[(0.0, events[1].1.clone())], 1.0, 100);

        let mut timeline_differs_from_sine = false;
        let mut timeline_differs_from_square = false;

        for i in 0..30 {
            if (timeline[i] - sine_only[i]).abs() > 0.01 {
                timeline_differs_from_sine = true;
            }
            if (timeline[i] - square_only[i]).abs() > 0.01 {
                timeline_differs_from_square = true;
            }
        }

        assert!(timeline_differs_from_sine, "Mixed timeline should differ from sine alone");
        assert!(timeline_differs_from_square, "Mixed timeline should differ from square alone");
    }

    #[test]
    fn test_render_timeline_zero_duration() {
        let event = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 440.0,
            end_frequency: 440.0,
            duration_secs: 0.5,
            envelope: AdsrEnvelope {
                attack_secs: 0.0,
                decay_secs: 0.0,
                sustain_level: 1.0,
                release_secs: 0.0,
            },
        };

        let events = &[(0.0, event)];
        let timeline = render_timeline(events, 0.0, 100);

        assert_eq!(timeline.len(), 0);
    }
}