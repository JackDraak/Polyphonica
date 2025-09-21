use clap::{Parser, Subcommand, ValueEnum};
use hound::{WavSpec, WavWriter};
use polyphonica::*;
use rodio::{OutputStream, Sink, Source};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "polyphonica-test")]
#[command(about = "Test tool for the Polyphonica audio synthesis library")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a single waveform
    Generate {
        /// Waveform type
        #[arg(value_enum)]
        waveform: WaveformArg,
        /// Frequency in Hz
        #[arg(short, long, default_value = "440.0")]
        frequency: f32,
        /// Duration in seconds
        #[arg(short, long, default_value = "1.0")]
        duration: f32,
        /// Sample rate
        #[arg(short, long, default_value = "44100")]
        sample_rate: u32,
        /// Output file path
        #[arg(short, long, default_value = "output.wav")]
        output: PathBuf,
        /// Play audio immediately through speakers
        #[arg(short, long)]
        play: bool,
        /// Volume level (0.0-1.0)
        #[arg(short, long, default_value = "0.5")]
        volume: f32,
    },
    /// Test ADSR envelope on a waveform
    Envelope {
        /// Waveform type
        #[arg(value_enum)]
        waveform: WaveformArg,
        /// Frequency in Hz
        #[arg(short, long, default_value = "440.0")]
        frequency: f32,
        /// Duration in seconds
        #[arg(short, long, default_value = "2.0")]
        duration: f32,
        /// Attack time in seconds
        #[arg(long, default_value = "0.1")]
        attack: f32,
        /// Decay time in seconds
        #[arg(long, default_value = "0.2")]
        decay: f32,
        /// Sustain level (0.0-1.0)
        #[arg(long, default_value = "0.7")]
        sustain: f32,
        /// Release time in seconds
        #[arg(long, default_value = "0.5")]
        release: f32,
        /// Sample rate
        #[arg(short, long, default_value = "44100")]
        sample_rate: u32,
        /// Output file path
        #[arg(short, long, default_value = "envelope_test.wav")]
        output: PathBuf,
        /// Play audio immediately through speakers
        #[arg(short, long)]
        play: bool,
        /// Volume level (0.0-1.0)
        #[arg(short, long, default_value = "0.5")]
        volume: f32,
    },
    /// Generate polyphonic composition with multiple voices
    Polyphonic {
        /// Number of voices (1-16)
        #[arg(short = 'n', long, default_value = "4")]
        voices: u8,
        /// Total duration in seconds
        #[arg(short, long, default_value = "3.0")]
        duration: f32,
        /// Sample rate
        #[arg(short, long, default_value = "44100")]
        sample_rate: u32,
        /// Composition type
        #[arg(value_enum, long, default_value = "chord")]
        composition: CompositionType,
        /// Output file path
        #[arg(short, long, default_value = "polyphonic_test.wav")]
        output: PathBuf,
        /// Play audio immediately through speakers
        #[arg(short, long)]
        play: bool,
        /// Volume level (0.0-1.0)
        #[arg(short, long, default_value = "0.5")]
        volume: f32,
    },
    /// Run comprehensive test suite and generate report
    TestSuite {
        /// Output directory for test files
        #[arg(short, long, default_value = "test_output")]
        output_dir: PathBuf,
        /// Sample rate for all tests
        #[arg(short, long, default_value = "44100")]
        sample_rate: u32,
        /// Play generated test files immediately
        #[arg(short, long)]
        play: bool,
        /// Volume level for playback (0.0-1.0)
        #[arg(short, long, default_value = "0.3")]
        volume: f32,
    },
    /// Report an issue with the library
    ReportIssue {
        /// Issue description
        description: String,
        /// Expected behavior
        #[arg(short, long)]
        expected: Option<String>,
        /// Actual behavior
        #[arg(short, long)]
        actual: Option<String>,
        /// Test parameters that caused the issue
        #[arg(short, long)]
        parameters: Option<String>,
    },
}

#[derive(ValueEnum, Clone, Debug)]
enum WaveformArg {
    Sine,
    Square,
    Sawtooth,
    Triangle,
}

#[derive(ValueEnum, Clone, Debug)]
enum CompositionType {
    Chord,
    Arpeggio,
    Scale,
    Random,
}

#[derive(Serialize, Deserialize)]
struct IssueReport {
    timestamp: String,
    description: String,
    expected: Option<String>,
    actual: Option<String>,
    parameters: Option<String>,
    library_version: String,
}

impl From<WaveformArg> for Waveform {
    fn from(arg: WaveformArg) -> Self {
        match arg {
            WaveformArg::Sine => Waveform::Sine,
            WaveformArg::Square => Waveform::Square,
            WaveformArg::Sawtooth => Waveform::Sawtooth,
            WaveformArg::Triangle => Waveform::Triangle,
        }
    }
}

// Simple audio source that wraps our samples for rodio
struct AudioSource {
    samples: Vec<f32>,
    sample_rate: u32,
    position: usize,
}

impl AudioSource {
    fn new(samples: Vec<f32>, sample_rate: u32) -> Self {
        Self {
            samples,
            sample_rate,
            position: 0,
        }
    }
}

impl Iterator for AudioSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.samples.len() {
            let sample = self.samples[self.position];
            self.position += 1;
            Some(sample)
        } else {
            None
        }
    }
}

impl Source for AudioSource {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.samples.len() - self.position)
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f32(
            self.samples.len() as f32 / self.sample_rate as f32,
        ))
    }
}

fn play_audio(samples: &[f32], sample_rate: u32, volume: f32) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎵 Playing audio...");

    // Clamp volume to safe range
    let volume = volume.clamp(0.0, 1.0);

    // Try to create audio output stream
    let (_stream, stream_handle) = match OutputStream::try_default() {
        Ok(output) => output,
        Err(e) => {
            println!("⚠️  Could not initialize audio output: {}", e);
            println!("   Make sure your system has audio output available.");
            return Ok(()); // Don't error, just skip playback
        }
    };

    // Create sink for audio playback
    let sink = match Sink::try_new(&stream_handle) {
        Ok(sink) => sink,
        Err(e) => {
            println!("⚠️  Could not create audio sink: {}", e);
            return Ok(());
        }
    };

    // Set volume
    sink.set_volume(volume);

    // Create audio source and play
    let source = AudioSource::new(samples.to_vec(), sample_rate);
    sink.append(source);

    // Wait for playback to complete
    sink.sleep_until_end();

    println!("✅ Playback completed");
    Ok(())
}

fn write_wav_file(samples: &[f32], sample_rate: u32, output_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create(output_path, spec)?;

    for &sample in samples {
        let amplitude = (sample * i16::MAX as f32) as i16;
        writer.write_sample(amplitude)?;
    }

    writer.finalize()?;
    println!("Audio written to: {}", output_path.display());
    Ok(())
}

fn create_polyphonic_composition(
    composition_type: CompositionType,
    voices: u8,
    duration: f32,
) -> Vec<(f32, SoundEvent)> {
    let voices = voices.min(16); // Cap at 16 voices
    let mut events = Vec::new();

    // Standard envelope for all voices
    let envelope = AdsrEnvelope {
        attack_secs: 0.1,
        decay_secs: 0.2,
        sustain_level: 0.6,
        release_secs: 0.4,
    };

    match composition_type {
        CompositionType::Chord => {
            // C Major chord with extensions
            let frequencies = [261.63, 329.63, 392.00, 523.25, 659.25, 783.99, 1046.50, 1318.51];

            for i in 0..voices {
                let freq_idx = (i as usize) % frequencies.len();
                let event = SoundEvent {
                    waveform: match i % 4 {
                        0 => Waveform::Sine,
                        1 => Waveform::Triangle,
                        2 => Waveform::Square,
                        _ => Waveform::Sawtooth,
                    },
                    start_frequency: frequencies[freq_idx],
                    end_frequency: frequencies[freq_idx],
                    duration_secs: duration - 0.1,
                    envelope: envelope.clone(),
                };
                events.push((0.0, event));
            }
        }
        CompositionType::Arpeggio => {
            // Ascending C Major arpeggio
            let frequencies = [261.63, 329.63, 392.00, 523.25];
            let note_duration = duration / voices as f32;

            for i in 0..voices {
                let freq_idx = (i as usize) % frequencies.len();
                let start_time = i as f32 * note_duration * 0.8;
                let event = SoundEvent {
                    waveform: Waveform::Sine,
                    start_frequency: frequencies[freq_idx],
                    end_frequency: frequencies[freq_idx],
                    duration_secs: note_duration * 1.5,
                    envelope: envelope.clone(),
                };
                events.push((start_time, event));
            }
        }
        CompositionType::Scale => {
            // C Major scale
            let frequencies = [261.63, 293.66, 329.63, 349.23, 392.00, 440.00, 493.88, 523.25];
            let note_duration = duration / voices as f32;

            for i in 0..voices {
                let freq_idx = (i as usize) % frequencies.len();
                let start_time = i as f32 * note_duration * 0.9;
                let event = SoundEvent {
                    waveform: Waveform::Triangle,
                    start_frequency: frequencies[freq_idx],
                    end_frequency: frequencies[freq_idx],
                    duration_secs: note_duration * 1.2,
                    envelope: envelope.clone(),
                };
                events.push((start_time, event));
            }
        }
        CompositionType::Random => {
            // Random frequencies and timing for experimental purposes
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            for i in 0..voices {
                let mut hasher = DefaultHasher::new();
                i.hash(&mut hasher);
                let hash = hasher.finish();

                let frequency = 200.0 + (hash % 800) as f32; // 200-1000 Hz range
                let start_time = (hash % (duration as u64 * 1000 / 2)) as f32 / 1000.0;
                let note_duration = 0.5 + (hash % 1500) as f32 / 1000.0; // 0.5-2.0 sec

                let event = SoundEvent {
                    waveform: match hash % 4 {
                        0 => Waveform::Sine,
                        1 => Waveform::Square,
                        2 => Waveform::Sawtooth,
                        _ => Waveform::Triangle,
                    },
                    start_frequency: frequency,
                    end_frequency: frequency * 1.1, // Slight frequency sweep
                    duration_secs: note_duration,
                    envelope: envelope.clone(),
                };
                events.push((start_time, event));
            }
        }
    }

    events
}

fn run_test_suite(output_dir: &PathBuf, sample_rate: u32, play: bool, volume: f32) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(output_dir)?;

    println!("Running comprehensive test suite...");

    // Test 1: All waveforms
    println!("Testing individual waveforms...");
    let waveforms = [Waveform::Sine, Waveform::Square, Waveform::Sawtooth, Waveform::Triangle];

    for waveform in &waveforms {
        let samples = generate_wave(*waveform, 440.0, 1.0, sample_rate);
        let filename = format!("{:?}_440hz.wav", waveform).to_lowercase();
        write_wav_file(&samples, sample_rate, &output_dir.join(filename))?;
    }

    // Test 2: ADSR Envelope variations
    println!("Testing ADSR envelope variations...");
    let envelope_tests = [
        ("piano", AdsrEnvelope { attack_secs: 0.01, decay_secs: 0.3, sustain_level: 0.3, release_secs: 1.0 }),
        ("organ", AdsrEnvelope { attack_secs: 0.1, decay_secs: 0.0, sustain_level: 0.8, release_secs: 0.1 }),
        ("pluck", AdsrEnvelope { attack_secs: 0.01, decay_secs: 0.5, sustain_level: 0.0, release_secs: 0.0 }),
    ];

    for (name, envelope) in &envelope_tests {
        let event = SoundEvent {
            waveform: Waveform::Sine,
            start_frequency: 440.0,
            end_frequency: 440.0,
            duration_secs: 2.0,
            envelope: envelope.clone(),
        };
        let samples = render_event(&event, sample_rate);
        write_wav_file(&samples, sample_rate, &output_dir.join(format!("envelope_{}.wav", name)))?;
    }

    // Test 3: Frequency sweeps
    println!("Testing frequency sweeps...");
    let sweep_event = SoundEvent {
        waveform: Waveform::Sawtooth,
        start_frequency: 110.0,
        end_frequency: 880.0,
        duration_secs: 3.0,
        envelope: AdsrEnvelope {
            attack_secs: 0.0,
            decay_secs: 0.0,
            sustain_level: 1.0,
            release_secs: 0.0,
        },
    };
    let sweep_samples = render_event(&sweep_event, sample_rate);
    write_wav_file(&sweep_samples, sample_rate, &output_dir.join("frequency_sweep.wav"))?;

    // Test 4: Polyphonic compositions
    println!("Testing polyphonic compositions...");
    let compositions = [
        (CompositionType::Chord, "chord"),
        (CompositionType::Arpeggio, "arpeggio"),
        (CompositionType::Scale, "scale"),
    ];

    for (comp_type, name) in &compositions {
        let events = create_polyphonic_composition(comp_type.clone(), 8, 4.0);
        let timeline = render_timeline(&events, 4.0, sample_rate);
        write_wav_file(&timeline, sample_rate, &output_dir.join(format!("polyphonic_{}.wav", name)))?;
    }

    println!("Test suite completed! Files written to: {}", output_dir.display());

    if play {
        println!("\n🎵 Playing sample compositions...");

        // Play a quick demo chord
        let demo_events = create_polyphonic_composition(CompositionType::Chord, 4, 2.0);
        let demo_timeline = render_timeline(&demo_events, 2.0, sample_rate);

        println!("▶️  Playing C Major chord demo...");
        play_audio(&demo_timeline, sample_rate, volume)?;

        println!("🎼 Demo playback completed!");
    }

    Ok(())
}

fn report_issue(
    description: String,
    expected: Option<String>,
    actual: Option<String>,
    parameters: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let issue = IssueReport {
        timestamp: chrono::Utc::now().to_rfc3339(),
        description,
        expected,
        actual,
        parameters,
        library_version: env!("CARGO_PKG_VERSION").to_string(),
    };

    let report_file = PathBuf::from("issue_reports.json");
    let mut reports: Vec<IssueReport> = if report_file.exists() {
        let content = fs::read_to_string(&report_file)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    reports.push(issue);
    let json = serde_json::to_string_pretty(&reports)?;
    fs::write(&report_file, json)?;

    println!("Issue reported and saved to: {}", report_file.display());
    println!("Thank you for helping improve the Polyphonica library!");
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { waveform, frequency, duration, sample_rate, output, play, volume } => {
            println!("Generating {:?} wave at {:.1}Hz for {:.1}s", waveform, frequency, duration);
            let samples = generate_wave(waveform.into(), frequency, duration, sample_rate);

            write_wav_file(&samples, sample_rate, &output)?;

            if play {
                play_audio(&samples, sample_rate, volume)?;
            }
        }

        Commands::Envelope { waveform, frequency, duration, attack, decay, sustain, release, sample_rate, output, play, volume } => {
            println!("Testing ADSR envelope: A={:.2}s D={:.2}s S={:.2} R={:.2}s", attack, decay, sustain, release);

            let envelope = AdsrEnvelope {
                attack_secs: attack,
                decay_secs: decay,
                sustain_level: sustain,
                release_secs: release,
            };

            let event = SoundEvent {
                waveform: waveform.into(),
                start_frequency: frequency,
                end_frequency: frequency,
                duration_secs: duration,
                envelope,
            };

            let samples = render_event(&event, sample_rate);
            write_wav_file(&samples, sample_rate, &output)?;

            if play {
                play_audio(&samples, sample_rate, volume)?;
            }
        }

        Commands::Polyphonic { voices, duration, sample_rate, composition, output, play, volume } => {
            println!("Generating {:?} composition with {} voices for {:.1}s", composition, voices, duration);
            let events = create_polyphonic_composition(composition, voices, duration);
            let timeline = render_timeline(&events, duration, sample_rate);
            write_wav_file(&timeline, sample_rate, &output)?;

            println!("Composition details:");
            for (i, (start_time, event)) in events.iter().enumerate() {
                println!("  Voice {}: {:?} at {:.1}Hz starting at {:.2}s",
                    i + 1, event.waveform, event.start_frequency, start_time);
            }

            if play {
                play_audio(&timeline, sample_rate, volume)?;
            }
        }

        Commands::TestSuite { output_dir, sample_rate, play, volume } => {
            run_test_suite(&output_dir, sample_rate, play, volume)?;
        }

        Commands::ReportIssue { description, expected, actual, parameters } => {
            report_issue(description, expected, actual, parameters)?;
        }
    }

    Ok(())
}