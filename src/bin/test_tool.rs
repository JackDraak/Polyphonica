use clap::{Parser, Subcommand, ValueEnum};
use hound::{WavSpec, WavWriter};
use polyphonica::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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
        /// Duty cycle for pulse wave (0.0-1.0)
        #[arg(long, default_value = "0.5")]
        duty_cycle: f32,
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
        /// Duty cycle for pulse wave (0.0-1.0)
        #[arg(long, default_value = "0.5")]
        duty_cycle: f32,
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
    /// Test WAV sample loading and playback
    Sample {
        /// Path to WAV file
        wav_file: PathBuf,
        /// Base frequency of the sample (Hz)
        #[arg(short, long)]
        base_frequency: f32,
        /// Target playback frequency (Hz)
        #[arg(short, long, default_value = "440.0")]
        target_frequency: f32,
        /// Duration to play (seconds)
        #[arg(short, long, default_value = "2.0")]
        duration: f32,
        /// Sample rate for playback
        #[arg(short, long, default_value = "44100")]
        sample_rate: u32,
        /// Output file path
        #[arg(short, long, default_value = "sample_test.wav")]
        output: PathBuf,
        /// Play audio immediately through speakers
        #[arg(short, long)]
        play: bool,
        /// Volume level (0.0-1.0)
        #[arg(short, long, default_value = "0.5")]
        volume: f32,
    },
    /// Create sample-based sound event with ADSR
    SampleEvent {
        /// Path to WAV file
        wav_file: PathBuf,
        /// Base frequency of the sample (Hz)
        #[arg(short, long)]
        base_frequency: f32,
        /// Target playback frequency (Hz)
        #[arg(short, long, default_value = "440.0")]
        target_frequency: f32,
        /// Duration in seconds
        #[arg(short, long, default_value = "2.0")]
        duration: f32,
        /// Attack time in seconds
        #[arg(long, default_value = "0.01")]
        attack: f32,
        /// Decay time in seconds
        #[arg(long, default_value = "0.1")]
        decay: f32,
        /// Sustain level (0.0-1.0)
        #[arg(long, default_value = "0.0")]
        sustain: f32,
        /// Release time in seconds
        #[arg(long, default_value = "0.3")]
        release: f32,
        /// Sample rate
        #[arg(short, long, default_value = "44100")]
        sample_rate: u32,
        /// Output file path
        #[arg(short, long, default_value = "sample_event.wav")]
        output: PathBuf,
        /// Play audio immediately through speakers
        #[arg(short, long)]
        play: bool,
        /// Volume level (0.0-1.0)
        #[arg(short, long, default_value = "0.5")]
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
    Pulse,
    Noise,
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

fn waveform_from_arg(arg: WaveformArg, duty_cycle: f32) -> Waveform {
    match arg {
        WaveformArg::Sine => Waveform::Sine,
        WaveformArg::Square => Waveform::Square,
        WaveformArg::Sawtooth => Waveform::Sawtooth,
        WaveformArg::Triangle => Waveform::Triangle,
        WaveformArg::Pulse => Waveform::Pulse { duty_cycle },
        WaveformArg::Noise => Waveform::Noise,
    }
}

// Audio playback using CPAL for consistency with main applications

fn play_audio(
    samples: &[f32],
    _sample_rate: u32,
    volume: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎵 Playing audio...");

    // Clamp volume to safe range
    let volume = volume.clamp(0.0, 1.0);

    // Get default audio device
    let host = cpal::default_host();
    let device = match host.default_output_device() {
        Some(device) => device,
        None => {
            println!("⚠️  No audio output device available");
            println!("   Make sure your system has audio output available.");
            return Ok(()); // Don't error, just skip playback
        }
    };

    let config = match device.default_output_config() {
        Ok(config) => config,
        Err(e) => {
            println!("⚠️  Could not get audio device config: {}", e);
            return Ok(());
        }
    };

    // Prepare audio data
    let audio_samples = Arc::new(Mutex::new(samples.to_vec()));
    let position = Arc::new(Mutex::new(0));
    let finished = Arc::new(Mutex::new(false));

    // Create completion channel
    let (tx, rx) = mpsc::channel();

    // Create audio stream
    let audio_samples_clone = audio_samples.clone();
    let position_clone = position.clone();
    let finished_clone = finished.clone();

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => {
            create_stream::<f32>(&device, &config.into(), audio_samples_clone, position_clone, finished_clone, volume, tx)
        }
        cpal::SampleFormat::I16 => {
            create_stream::<i16>(&device, &config.into(), audio_samples_clone, position_clone, finished_clone, volume, tx)
        }
        cpal::SampleFormat::U16 => {
            create_stream::<u16>(&device, &config.into(), audio_samples_clone, position_clone, finished_clone, volume, tx)
        }
        _ => {
            println!("⚠️  Unsupported audio format");
            return Ok(());
        }
    }?;

    // Start playback
    stream.play()?;

    // Wait for completion
    rx.recv()?;

    println!("✅ Playback completed");
    Ok(())
}

fn create_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    audio_samples: Arc<Mutex<Vec<f32>>>,
    position: Arc<Mutex<usize>>,
    finished: Arc<Mutex<bool>>,
    volume: f32,
    completion_tx: mpsc::Sender<()>,
) -> Result<cpal::Stream, Box<dyn std::error::Error>>
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    let channels = config.channels as usize;

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            let mut pos = position.lock().unwrap();
            let samples = audio_samples.lock().unwrap();
            let mut is_finished = finished.lock().unwrap();

            if *is_finished {
                return;
            }

            let frames = data.len() / channels;

            for frame in 0..frames {
                let sample_value = if *pos < samples.len() {
                    samples[*pos] * volume
                } else {
                    if !*is_finished {
                        *is_finished = true;
                        let _ = completion_tx.send(());
                    }
                    0.0
                };

                // Fill all channels with the same mono sample
                for channel in 0..channels {
                    if let Some(sample) = data.get_mut(frame * channels + channel) {
                        *sample = T::from_sample(sample_value);
                    }
                }

                if *pos < samples.len() {
                    *pos += 1;
                }
            }
        },
        |err| eprintln!("Audio stream error: {}", err),
        None,
    )?;

    Ok(stream)
}

fn write_wav_file(
    samples: &[f32],
    sample_rate: u32,
    output_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
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
            let frequencies = [
                261.63, 329.63, 392.00, 523.25, 659.25, 783.99, 1046.50, 1318.51,
            ];

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
            let frequencies = [
                261.63, 293.66, 329.63, 349.23, 392.00, 440.00, 493.88, 523.25,
            ];
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

fn run_test_suite(
    output_dir: &PathBuf,
    sample_rate: u32,
    play: bool,
    volume: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(output_dir)?;

    println!("Running comprehensive test suite...");

    // Test 1: All waveforms
    println!("Testing individual waveforms...");
    let waveforms = [
        Waveform::Sine,
        Waveform::Square,
        Waveform::Sawtooth,
        Waveform::Triangle,
        Waveform::Pulse { duty_cycle: 0.5 },
        Waveform::Noise,
    ];

    for waveform in &waveforms {
        let samples = generate_wave(waveform.clone(), 440.0, 1.0, sample_rate);
        let filename = match waveform {
            Waveform::Sine => "sine_440hz.wav".to_string(),
            Waveform::Square => "square_440hz.wav".to_string(),
            Waveform::Sawtooth => "sawtooth_440hz.wav".to_string(),
            Waveform::Triangle => "triangle_440hz.wav".to_string(),
            Waveform::Pulse { duty_cycle } => {
                format!("pulse_{:.0}pct_440hz.wav", duty_cycle * 100.0)
            }
            Waveform::Noise => "noise_440hz.wav".to_string(),
            Waveform::Sample(_) => "sample_440hz.wav".to_string(),
            Waveform::DrumSample(_) => "drum_sample_440hz.wav".to_string(),
        };
        write_wav_file(&samples, sample_rate, &output_dir.join(filename))?;
    }

    // Test 2: ADSR Envelope variations
    println!("Testing ADSR envelope variations...");
    let envelope_tests = [
        (
            "piano",
            AdsrEnvelope {
                attack_secs: 0.01,
                decay_secs: 0.3,
                sustain_level: 0.3,
                release_secs: 1.0,
            },
        ),
        (
            "organ",
            AdsrEnvelope {
                attack_secs: 0.1,
                decay_secs: 0.0,
                sustain_level: 0.8,
                release_secs: 0.1,
            },
        ),
        (
            "pluck",
            AdsrEnvelope {
                attack_secs: 0.01,
                decay_secs: 0.5,
                sustain_level: 0.0,
                release_secs: 0.0,
            },
        ),
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
        write_wav_file(
            &samples,
            sample_rate,
            &output_dir.join(format!("envelope_{}.wav", name)),
        )?;
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
    write_wav_file(
        &sweep_samples,
        sample_rate,
        &output_dir.join("frequency_sweep.wav"),
    )?;

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
        write_wav_file(
            &timeline,
            sample_rate,
            &output_dir.join(format!("polyphonic_{}.wav", name)),
        )?;
    }

    println!(
        "Test suite completed! Files written to: {}",
        output_dir.display()
    );

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
        Commands::Generate {
            waveform,
            frequency,
            duration,
            sample_rate,
            output,
            play,
            volume,
            duty_cycle,
        } => {
            println!(
                "Generating {:?} wave at {:.1}Hz for {:.1}s",
                waveform, frequency, duration
            );
            let wave = waveform_from_arg(waveform, duty_cycle);
            let samples = generate_wave(wave, frequency, duration, sample_rate);

            write_wav_file(&samples, sample_rate, &output)?;

            if play {
                play_audio(&samples, sample_rate, volume)?;
            }
        }

        Commands::Envelope {
            waveform,
            frequency,
            duration,
            attack,
            decay,
            sustain,
            release,
            sample_rate,
            output,
            play,
            volume,
            duty_cycle,
        } => {
            println!(
                "Testing ADSR envelope: A={:.2}s D={:.2}s S={:.2} R={:.2}s",
                attack, decay, sustain, release
            );

            let envelope = AdsrEnvelope {
                attack_secs: attack,
                decay_secs: decay,
                sustain_level: sustain,
                release_secs: release,
            };

            let event = SoundEvent {
                waveform: waveform_from_arg(waveform, duty_cycle),
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

        Commands::Polyphonic {
            voices,
            duration,
            sample_rate,
            composition,
            output,
            play,
            volume,
        } => {
            println!(
                "Generating {:?} composition with {} voices for {:.1}s",
                composition, voices, duration
            );
            let events = create_polyphonic_composition(composition, voices, duration);
            let timeline = render_timeline(&events, duration, sample_rate);
            write_wav_file(&timeline, sample_rate, &output)?;

            println!("Composition details:");
            for (i, (start_time, event)) in events.iter().enumerate() {
                println!(
                    "  Voice {}: {:?} at {:.1}Hz starting at {:.2}s",
                    i + 1,
                    event.waveform,
                    event.start_frequency,
                    start_time
                );
            }

            if play {
                play_audio(&timeline, sample_rate, volume)?;
            }
        }

        Commands::Sample {
            wav_file,
            base_frequency,
            target_frequency,
            duration,
            sample_rate,
            output,
            play,
            volume,
        } => {
            println!(
                "Loading sample: {} (base: {:.1}Hz, target: {:.1}Hz)",
                wav_file.display(),
                base_frequency,
                target_frequency
            );

            // Load the sample
            let sample_data = match SampleData::from_file(&wav_file, base_frequency) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Error loading sample: {}", e);
                    return Ok(());
                }
            };

            println!(
                "Sample loaded: {:.2}s, {} channels, {}Hz",
                sample_data.metadata.duration_secs,
                sample_data.metadata.channels,
                sample_data.sample_rate
            );

            // Create sample waveform and generate audio
            let waveform = Waveform::Sample(sample_data);
            let samples = generate_wave(waveform, target_frequency, duration, sample_rate);

            write_wav_file(&samples, sample_rate, &output)?;

            if play {
                play_audio(&samples, sample_rate, volume)?;
            }
        }

        Commands::SampleEvent {
            wav_file,
            base_frequency,
            target_frequency,
            duration,
            attack,
            decay,
            sustain,
            release,
            sample_rate,
            output,
            play,
            volume,
        } => {
            println!(
                "Creating sample event: {} (base: {:.1}Hz, target: {:.1}Hz)",
                wav_file.display(),
                base_frequency,
                target_frequency
            );

            // Load the sample
            let sample_data = match SampleData::from_file(&wav_file, base_frequency) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Error loading sample: {}", e);
                    return Ok(());
                }
            };

            println!(
                "Sample loaded: {:.2}s, {} channels, {}Hz",
                sample_data.metadata.duration_secs,
                sample_data.metadata.channels,
                sample_data.sample_rate
            );

            // Create ADSR envelope
            let envelope = AdsrEnvelope {
                attack_secs: attack,
                decay_secs: decay,
                sustain_level: sustain,
                release_secs: release,
            };

            println!(
                "ADSR: A={:.2}s D={:.2}s S={:.2} R={:.2}s",
                attack, decay, sustain, release
            );

            // Create sample event
            let event = SoundEvent {
                waveform: Waveform::Sample(sample_data),
                start_frequency: target_frequency,
                end_frequency: target_frequency,
                duration_secs: duration,
                envelope,
            };

            let samples = render_event(&event, sample_rate);
            write_wav_file(&samples, sample_rate, &output)?;

            if play {
                play_audio(&samples, sample_rate, volume)?;
            }
        }

        Commands::TestSuite {
            output_dir,
            sample_rate,
            play,
            volume,
        } => {
            run_test_suite(&output_dir, sample_rate, play, volume)?;
        }

        Commands::ReportIssue {
            description,
            expected,
            actual,
            parameters,
        } => {
            report_issue(description, expected, actual, parameters)?;
        }
    }

    Ok(())
}
