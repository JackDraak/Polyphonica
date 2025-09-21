/// Guitar Buddy - Musical Practice Companion
///
/// Phase 1: Advanced metronome with multiple click sounds and time signatures
/// Phase 2: Full accompaniment with drums, bass lines, and chord progressions
///
/// Uses Polyphonica real-time synthesis engine for precise, low-latency audio generation.

use polyphonica::{RealtimeEngine, Waveform, AdsrEnvelope, SampleData};
use eframe::egui;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Time signature representation
#[derive(Debug, Clone, Copy, PartialEq)]
struct TimeSignature {
    beats_per_measure: u8,
    note_value: u8, // 4 = quarter note, 8 = eighth note, etc.
}

impl TimeSignature {
    fn new(beats: u8, note_value: u8) -> Self {
        Self {
            beats_per_measure: beats,
            note_value,
        }
    }

    fn common_signatures() -> Vec<(&'static str, TimeSignature)> {
        vec![
            ("4/4", TimeSignature::new(4, 4)),
            ("3/4", TimeSignature::new(3, 4)),
            ("2/4", TimeSignature::new(2, 4)),
            ("6/8", TimeSignature::new(6, 8)),
            ("9/8", TimeSignature::new(9, 8)),
            ("12/8", TimeSignature::new(12, 8)),
            ("5/4", TimeSignature::new(5, 4)),
            ("7/8", TimeSignature::new(7, 8)),
        ]
    }

    fn display(&self) -> String {
        format!("{}/{}", self.beats_per_measure, self.note_value)
    }
}

/// Drum sample manager for loading and storing samples
#[derive(Debug, Clone)]
struct DrumSampleManager {
    samples: HashMap<ClickType, SampleData>,
}

impl DrumSampleManager {
    fn new() -> Self {
        Self {
            samples: HashMap::new(),
        }
    }

    fn load_drum_samples(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Load acoustic drum kit samples using relative paths from project root
        let sample_paths = vec![
            (ClickType::AcousticKick, "samples/drums/acoustic/kit_01/drumkit-kick.wav"),
            (ClickType::AcousticSnare, "samples/drums/acoustic/kit_01/drumkit-snare.wav"),
            (ClickType::HiHatClosed, "samples/drums/acoustic/kit_01/drumkit-hihat.wav"),
            (ClickType::HiHatOpen, "samples/drums/acoustic/kit_01/drumkit-hihat-open.wav"),
            (ClickType::RimShot, "samples/drums/acoustic/kit_01/drumkit-rimshot.wav"), // Dedicated rimshot sample
            (ClickType::Stick, "samples/drums/acoustic/kit_01/drumkit-hihat.wav"),    // Reuse hi-hat with short envelope
        ];

        for (click_type, path) in sample_paths {
            match SampleData::from_file(path, 440.0) {
                Ok(sample_data) => {
                    println!("✅ Loaded drum sample: {} from {}", click_type.name(), path);
                    self.samples.insert(click_type, sample_data);
                }
                Err(e) => {
                    println!("⚠️  Could not load {}: {} (falling back to synthetic)", path, e);
                    // Continue without the sample - will fall back to synthetic sound
                }
            }
        }

        Ok(())
    }

    fn get_sample(&self, click_type: &ClickType) -> Option<&SampleData> {
        self.samples.get(click_type)
    }

    fn has_sample(&self, click_type: &ClickType) -> bool {
        self.samples.contains_key(click_type)
    }
}

/// Different metronome click sound types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ClickType {
    // Synthetic sounds
    WoodBlock,      // Sharp percussive click
    DigitalBeep,    // Clean sine wave beep
    Cowbell,        // Metallic ring
    ElectroClick,   // Electronic click
    // Real drum samples
    AcousticKick,   // Acoustic kick drum
    AcousticSnare,  // Acoustic snare drum
    HiHatClosed,    // Closed hi-hat
    HiHatOpen,      // Open hi-hat
    RimShot,        // Snare rim (using snare sample with envelope)
    Stick,          // Drumstick click (using hi-hat)
}

impl ClickType {
    fn all() -> Vec<ClickType> {
        vec![
            // Synthetic sounds
            ClickType::WoodBlock,
            ClickType::DigitalBeep,
            ClickType::Cowbell,
            ClickType::ElectroClick,
            // Real drum samples
            ClickType::AcousticKick,
            ClickType::AcousticSnare,
            ClickType::HiHatClosed,
            ClickType::HiHatOpen,
            ClickType::RimShot,
            ClickType::Stick,
        ]
    }

    fn name(self) -> &'static str {
        match self {
            // Synthetic sounds
            ClickType::WoodBlock => "Wood Block",
            ClickType::DigitalBeep => "Digital Beep",
            ClickType::Cowbell => "Cowbell",
            ClickType::ElectroClick => "Electro Click",
            // Real drum samples
            ClickType::AcousticKick => "Acoustic Kick",
            ClickType::AcousticSnare => "Acoustic Snare",
            ClickType::HiHatClosed => "Hi-Hat Closed",
            ClickType::HiHatOpen => "Hi-Hat Open",
            ClickType::RimShot => "Rim Shot",
            ClickType::Stick => "Drum Stick",
        }
    }

    /// Generate the waveform and parameters for this click type
    fn get_sound_params(self, sample_manager: &DrumSampleManager) -> (Waveform, f32, AdsrEnvelope) {
        // Check if we have a sample for this click type
        if let Some(sample_data) = sample_manager.get_sample(&self) {
            return (
                Waveform::DrumSample(sample_data.clone()),
                440.0, // Frequency is ignored for drum samples
                self.get_sample_envelope()
            );
        }

        // Fall back to synthetic sound
        self.get_synthetic_params()
    }

    /// Get ADSR envelope for sample-based sounds
    /// For drums, we use minimal envelope shaping to preserve natural character
    fn get_sample_envelope(self) -> AdsrEnvelope {
        match self {
            ClickType::AcousticKick => AdsrEnvelope {
                attack_secs: 0.001,  // Instant attack
                decay_secs: 1.0,     // Let natural sample decay
                sustain_level: 0.0,  // No sustain - one-shot sample
                release_secs: 0.001, // Minimal release
            },
            ClickType::AcousticSnare => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 0.5,     // Let natural snare ring
                sustain_level: 0.0,
                release_secs: 0.001,
            },
            ClickType::HiHatClosed => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 0.2,     // Natural hi-hat decay
                sustain_level: 0.0,
                release_secs: 0.001,
            },
            ClickType::HiHatOpen => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 1.0,     // Let open hi-hat ring naturally
                sustain_level: 0.0,
                release_secs: 0.001,
            },
            ClickType::RimShot => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 0.3,     // Natural rim shot decay
                sustain_level: 0.0,
                release_secs: 0.001,
            },
            ClickType::Stick => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 0.1,     // Short stick click
                sustain_level: 0.0,
                release_secs: 0.001,
            },
            // For synthetic sounds, use default
            _ => AdsrEnvelope {
                attack_secs: 0.001,
                decay_secs: 0.1,
                sustain_level: 0.0,
                release_secs: 0.05,
            },
        }
    }

    /// Get synthetic sound parameters (fallback when no sample available)
    fn get_synthetic_params(self) -> (Waveform, f32, AdsrEnvelope) {
        match self {
            ClickType::WoodBlock => (
                Waveform::Noise,
                800.0, // High frequency for sharp click
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.05,
                    sustain_level: 0.0,
                    release_secs: 0.02,
                }
            ),
            ClickType::DigitalBeep => (
                Waveform::Sine,
                1000.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.08,
                    sustain_level: 0.0,
                    release_secs: 0.05,
                }
            ),
            ClickType::Cowbell => (
                Waveform::Square,
                800.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.15,
                    sustain_level: 0.0,
                    release_secs: 0.1,
                }
            ),
            ClickType::RimShot => (
                Waveform::Pulse { duty_cycle: 0.1 },
                400.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.03,
                    sustain_level: 0.0,
                    release_secs: 0.02,
                }
            ),
            ClickType::Stick => (
                Waveform::Triangle,
                2000.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.02,
                    sustain_level: 0.0,
                    release_secs: 0.01,
                }
            ),
            ClickType::ElectroClick => (
                Waveform::Pulse { duty_cycle: 0.25 },
                1200.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.04,
                    sustain_level: 0.0,
                    release_secs: 0.03,
                }
            ),
            // For drum samples without sample data, provide synthetic alternatives
            ClickType::AcousticKick => (
                Waveform::Sine,
                60.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.3,
                    sustain_level: 0.0,
                    release_secs: 0.1,
                }
            ),
            ClickType::AcousticSnare => (
                Waveform::Noise,
                800.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.15,
                    sustain_level: 0.0,
                    release_secs: 0.05,
                }
            ),
            ClickType::HiHatClosed => (
                Waveform::Pulse { duty_cycle: 0.1 },
                8000.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.08,
                    sustain_level: 0.0,
                    release_secs: 0.02,
                }
            ),
            ClickType::HiHatOpen => (
                Waveform::Pulse { duty_cycle: 0.1 },
                6000.0,
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.25,
                    sustain_level: 0.0,
                    release_secs: 0.1,
                }
            ),
        }
    }
}

/// Metronome state and timing control
#[derive(Debug, Clone)]
struct MetronomeState {
    is_playing: bool,
    tempo_bpm: f32,
    time_signature: TimeSignature,
    click_type: ClickType,
    accent_first_beat: bool,
    volume: f32,
    current_beat: u8,
    last_beat_time: Option<Instant>,
}

impl MetronomeState {
    fn new() -> Self {
        Self {
            is_playing: false,
            tempo_bpm: 120.0,
            time_signature: TimeSignature::new(4, 4),
            click_type: ClickType::WoodBlock,
            accent_first_beat: true,
            volume: 0.7,
            current_beat: 0,
            last_beat_time: None,
        }
    }

    /// Calculate time between beats in milliseconds
    fn beat_interval_ms(&self) -> f64 {
        60000.0 / self.tempo_bpm as f64
    }

    /// Check if it's time for the next beat
    fn should_trigger_beat(&mut self) -> bool {
        if !self.is_playing {
            return false;
        }

        let now = Instant::now();

        match self.last_beat_time {
            None => {
                // First beat
                self.last_beat_time = Some(now);
                self.current_beat = 1;
                true
            }
            Some(last_time) => {
                let elapsed_ms = now.duration_since(last_time).as_millis() as f64;
                if elapsed_ms >= self.beat_interval_ms() {
                    self.last_beat_time = Some(now);
                    self.current_beat = (self.current_beat % self.time_signature.beats_per_measure) + 1;
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Get volume for current beat (accent first beat if enabled)
    fn get_beat_volume(&self) -> f32 {
        if self.accent_first_beat && self.current_beat == 1 {
            (self.volume * 1.5).min(1.0) // 50% louder for first beat
        } else {
            self.volume
        }
    }

    fn start(&mut self) {
        self.is_playing = true;
        self.current_beat = 0;
        self.last_beat_time = None;
    }

    fn stop(&mut self) {
        self.is_playing = false;
        self.current_beat = 0;
        self.last_beat_time = None;
    }

    fn pause(&mut self) {
        self.is_playing = false;
    }

    fn resume(&mut self) {
        self.is_playing = true;
        self.last_beat_time = Some(Instant::now());
    }
}

/// Shared state between GUI and audio threads
#[derive(Clone)]
struct AppState {
    engine: Arc<Mutex<RealtimeEngine>>,
    metronome: Arc<Mutex<MetronomeState>>,
    drum_samples: Arc<Mutex<DrumSampleManager>>,
}

/// Main application for Guitar Buddy
struct GuitarBuddy {
    app_state: AppState,
    _audio_stream: Stream,
}

impl GuitarBuddy {
    fn new(_cc: &eframe::CreationContext<'_>) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize real-time engine
        let engine = Arc::new(Mutex::new(RealtimeEngine::new(44100.0)));

        // Initialize metronome state
        let metronome = Arc::new(Mutex::new(MetronomeState::new()));

        // Initialize and load drum samples
        let mut drum_samples = DrumSampleManager::new();
        drum_samples.load_drum_samples()?;
        let drum_samples = Arc::new(Mutex::new(drum_samples));

        // Create shared state
        let app_state = AppState {
            engine: engine.clone(),
            metronome: metronome.clone(),
            drum_samples: drum_samples.clone(),
        };

        // Setup audio stream
        let audio_stream = setup_audio_stream(app_state.clone())?;

        Ok(GuitarBuddy {
            app_state,
            _audio_stream: audio_stream,
        })
    }

    fn trigger_click(&self, is_accent: bool) {
        let metronome = self.app_state.metronome.lock().unwrap();
        let drum_samples = self.app_state.drum_samples.lock().unwrap();
        let (waveform, frequency, mut envelope) = metronome.click_type.get_sound_params(&drum_samples);
        let volume = if is_accent && metronome.accent_first_beat {
            (metronome.volume * 1.5).min(1.0)
        } else {
            metronome.volume
        };
        drop(metronome);
        drop(drum_samples);

        // Adjust envelope amplitude based on volume
        envelope.sustain_level *= volume;

        let mut engine = self.app_state.engine.lock().unwrap();
        engine.trigger_note(waveform, frequency, envelope);
    }
}

impl eframe::App for GuitarBuddy {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for metronome beats
        {
            let mut metronome = self.app_state.metronome.lock().unwrap();
            if metronome.should_trigger_beat() {
                let is_accent = metronome.current_beat == 1;
                drop(metronome);
                self.trigger_click(is_accent);
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎸 Guitar Buddy - Practice Companion");
            ui.separator();

            // Metronome status display
            let metronome = self.app_state.metronome.lock().unwrap();
            let is_playing = metronome.is_playing;
            let current_beat = metronome.current_beat;
            let time_sig = metronome.time_signature;
            let tempo = metronome.tempo_bpm;
            drop(metronome);

            ui.horizontal(|ui| {
                if is_playing {
                    ui.colored_label(egui::Color32::GREEN, "♪ PLAYING");
                    ui.separator();
                    ui.label(format!("Beat: {}/{}", current_beat, time_sig.beats_per_measure));
                } else {
                    ui.colored_label(egui::Color32::GRAY, "⏸ STOPPED");
                }
                ui.separator();
                ui.label(format!("Tempo: {:.0} BPM", tempo));
                ui.separator();
                ui.label(format!("Time: {}", time_sig.display()));
            });

            ui.separator();

            // Transport controls
            ui.horizontal(|ui| {
                let mut metronome = self.app_state.metronome.lock().unwrap();

                if metronome.is_playing {
                    if ui.button("⏸ Pause").clicked() {
                        metronome.pause();
                    }
                    if ui.button("⏹ Stop").clicked() {
                        metronome.stop();
                    }
                } else {
                    if ui.button("▶ Start").clicked() {
                        metronome.start();
                    }
                    if metronome.last_beat_time.is_some() {
                        if ui.button("▶ Resume").clicked() {
                            metronome.resume();
                        }
                    }
                }
            });

            ui.separator();

            // Tempo control
            ui.horizontal(|ui| {
                ui.label("Tempo (BPM):");
                let mut metronome = self.app_state.metronome.lock().unwrap();
                ui.add(egui::Slider::new(&mut metronome.tempo_bpm, 40.0..=200.0)
                    .step_by(1.0)
                    .suffix(" BPM"));

                // Preset tempo buttons
                ui.separator();
                for &(name, bpm) in &[("Slow", 60.0), ("Med", 120.0), ("Fast", 160.0)] {
                    if ui.button(name).clicked() {
                        metronome.tempo_bpm = bpm;
                    }
                }
            });

            ui.separator();

            // Time signature selection
            ui.horizontal(|ui| {
                ui.label("Time Signature:");
                let mut metronome = self.app_state.metronome.lock().unwrap();
                let mut current_sig = metronome.time_signature;

                for &(name, sig) in &TimeSignature::common_signatures() {
                    if ui.radio_value(&mut current_sig, sig, name).clicked() {
                        metronome.time_signature = current_sig;
                        metronome.current_beat = 0; // Reset beat counter
                    }
                }
            });

            ui.separator();

            // Click sound selection
            ui.horizontal(|ui| {
                ui.label("Click Sound:");
                let mut metronome = self.app_state.metronome.lock().unwrap();
                let mut current_click = metronome.click_type;

                for &click_type in &ClickType::all() {
                    if ui.radio_value(&mut current_click, click_type, click_type.name()).clicked() {
                        metronome.click_type = current_click;
                    }
                }
            });

            ui.separator();

            // Volume and accent controls
            ui.horizontal(|ui| {
                ui.label("Volume:");
                let mut metronome = self.app_state.metronome.lock().unwrap();
                ui.add(egui::Slider::new(&mut metronome.volume, 0.0..=1.0)
                    .step_by(0.01)
                    .suffix("%"));

                ui.separator();
                ui.checkbox(&mut metronome.accent_first_beat, "Accent first beat");
            });

            ui.separator();

            // Test click button
            ui.horizontal(|ui| {
                if ui.button("🔊 Test Click").clicked() {
                    let metronome = self.app_state.metronome.lock().unwrap();
                    let drum_samples = self.app_state.drum_samples.lock().unwrap();
                    let (waveform, frequency, envelope) = metronome.click_type.get_sound_params(&drum_samples);
                    drop(metronome);
                    drop(drum_samples);

                    let mut engine = self.app_state.engine.lock().unwrap();
                    engine.trigger_note(waveform, frequency, envelope);
                }

                if ui.button("🔊 Test Accent").clicked() {
                    self.trigger_click(true);
                }
            });

            ui.separator();

            // Beat visualization
            ui.collapsing("Beat Visualization", |ui| {
                ui.horizontal(|ui| {
                    let metronome = self.app_state.metronome.lock().unwrap();

                    for beat in 1..=metronome.time_signature.beats_per_measure {
                        let is_current = beat == current_beat && is_playing;
                        let is_accent = beat == 1 && metronome.accent_first_beat;

                        let color = if is_current && is_accent {
                            egui::Color32::YELLOW
                        } else if is_current {
                            egui::Color32::GREEN
                        } else if is_accent {
                            egui::Color32::LIGHT_GRAY
                        } else {
                            egui::Color32::GRAY
                        };

                        let symbol = if is_accent { "●" } else { "○" };
                        ui.colored_label(color, symbol);
                    }
                });

                ui.label(format!("Interval: {:.0}ms between beats",
                    self.app_state.metronome.lock().unwrap().beat_interval_ms()));
            });

            ui.separator();

            // Phase 2 preview
            ui.collapsing("Coming in Phase 2", |ui| {
                ui.label("🥁 Drum patterns and backing tracks");
                ui.label("🎹 Piano chord progressions");
                ui.label("🎸 Bass line accompaniment");
                ui.label("🎵 Key and chord change management");
                ui.label("📚 Practice session recording");
                ui.label("🎯 Tempo trainer with gradual speed changes");
            });
        });

        // Request repaint for smooth beat timing
        ctx.request_repaint_after(Duration::from_millis(10));
    }
}

/// Setup CPAL audio stream for real-time metronome output
fn setup_audio_stream(app_state: AppState) -> Result<Stream, Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let device = host.default_output_device()
        .ok_or("No audio output device available")?;

    let config = device.default_output_config()?;

    println!("🎸 Guitar Buddy Audio System");
    println!("Audio device: {}", device.name()?);
    println!("Sample rate: {} Hz", config.sample_rate().0);
    println!("Channels: {}", config.channels());

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => create_stream::<f32>(&device, &config.into(), app_state),
        cpal::SampleFormat::I16 => create_stream::<i16>(&device, &config.into(), app_state),
        cpal::SampleFormat::U16 => create_stream::<u16>(&device, &config.into(), app_state),
        _ => return Err("Unsupported audio format".into()),
    }?;

    stream.play()?;
    Ok(stream)
}

/// Create audio stream for specific sample format
fn create_stream<T>(
    device: &Device,
    config: &StreamConfig,
    app_state: AppState,
) -> Result<Stream, Box<dyn std::error::Error>>
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    let channels = config.channels as usize;

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            let mut f32_buffer = vec![0.0f32; data.len()];

            // Process audio with the engine
            {
                let mut engine = app_state.engine.lock().unwrap();
                if channels == 1 {
                    engine.process_buffer(&mut f32_buffer);
                } else {
                    engine.process_stereo_buffer(&mut f32_buffer);
                }
            }

            // Convert back to target format
            for (dst, &src) in data.iter_mut().zip(f32_buffer.iter()) {
                *dst = T::from_sample(src);
            }
        },
        |err| eprintln!("Audio stream error: {}", err),
        None,
    )?;

    Ok(stream)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎸 Guitar Buddy - Musical Practice Companion");
    println!("============================================");
    println!("Phase 1: Advanced Metronome");
    println!("Initializing audio system...");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([700.0, 500.0])
            .with_title("Guitar Buddy - Practice Companion")
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Guitar Buddy",
        options,
        Box::new(|cc| {
            match GuitarBuddy::new(cc) {
                Ok(app) => {
                    println!("✅ Guitar Buddy initialized successfully!");
                    println!("🎵 Audio output active - ready to rock!");
                    Ok(Box::new(app))
                }
                Err(e) => {
                    eprintln!("❌ Failed to initialize Guitar Buddy: {}", e);
                    std::process::exit(1);
                }
            }
        }),
    )
    .map_err(|e| format!("GUI error: {}", e).into())
}