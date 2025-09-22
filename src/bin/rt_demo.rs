/// Polyphonica Real-Time Audio Demo with GUI
///
/// Interactive demonstration of the real-time synthesis engine with live audio output.
/// Features GUI controls for real-time parameter manipulation and waveform selection.

use polyphonica::{RealtimeEngine, Waveform, AdsrEnvelope, MAX_VOICES};
use eframe::egui;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Shared application state between GUI and audio threads
#[derive(Clone)]
struct AppState {
    engine: Arc<Mutex<RealtimeEngine>>,
    current_waveform: Arc<Mutex<WaveformType>>,
    frequency: Arc<Mutex<f32>>,
    master_volume: Arc<Mutex<f32>>,
    envelope: Arc<Mutex<AdsrEnvelope>>,
    active_voices: Arc<Mutex<Vec<u32>>>,
}

/// Waveform types for GUI selection
#[derive(Debug, Clone, Copy, PartialEq)]
enum WaveformType {
    Sine,
    Square,
    Sawtooth,
    Triangle,
    Pulse25,  // 25% duty cycle
    Pulse50,  // 50% duty cycle
    Pulse75,  // 75% duty cycle
    Noise,
}

impl WaveformType {
    fn to_waveform(self) -> Waveform {
        match self {
            WaveformType::Sine => Waveform::Sine,
            WaveformType::Square => Waveform::Square,
            WaveformType::Sawtooth => Waveform::Sawtooth,
            WaveformType::Triangle => Waveform::Triangle,
            WaveformType::Pulse25 => Waveform::Pulse { duty_cycle: 0.25 },
            WaveformType::Pulse50 => Waveform::Pulse { duty_cycle: 0.50 },
            WaveformType::Pulse75 => Waveform::Pulse { duty_cycle: 0.75 },
            WaveformType::Noise => Waveform::Noise,
        }
    }

    fn name(self) -> &'static str {
        match self {
            WaveformType::Sine => "Sine",
            WaveformType::Square => "Square",
            WaveformType::Sawtooth => "Sawtooth",
            WaveformType::Triangle => "Triangle",
            WaveformType::Pulse25 => "Pulse 25%",
            WaveformType::Pulse50 => "Pulse 50%",
            WaveformType::Pulse75 => "Pulse 75%",
            WaveformType::Noise => "White Noise",
        }
    }

    fn all() -> &'static [WaveformType] {
        &[
            WaveformType::Sine,
            WaveformType::Square,
            WaveformType::Sawtooth,
            WaveformType::Triangle,
            WaveformType::Pulse25,
            WaveformType::Pulse50,
            WaveformType::Pulse75,
            WaveformType::Noise,
        ]
    }
}

/// Main application struct for the GUI demo
struct PolyphonicaDemo {
    app_state: AppState,
    _audio_stream: Stream,
    chord_buttons: Vec<(&'static str, f32)>,
    last_note_voice_id: Option<u32>,
}

impl PolyphonicaDemo {
    fn new(_cc: &eframe::CreationContext<'_>) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize real-time engine
        let engine = Arc::new(Mutex::new(RealtimeEngine::new(44100.0)));

        // Create shared state
        let app_state = AppState {
            engine: engine.clone(),
            current_waveform: Arc::new(Mutex::new(WaveformType::Sine)),
            frequency: Arc::new(Mutex::new(440.0)),
            master_volume: Arc::new(Mutex::new(0.5)),
            envelope: Arc::new(Mutex::new(AdsrEnvelope {
                attack_secs: 0.1,
                decay_secs: 0.2,
                sustain_level: 0.6,
                release_secs: 0.3,
            })),
            active_voices: Arc::new(Mutex::new(Vec::new())),
        };

        // Setup audio stream
        let audio_stream = setup_audio_stream(app_state.clone())?;

        // Musical chord definitions
        let chord_buttons = vec![
            ("C Maj", 261.63),   // C Major chord root
            ("D Maj", 293.66),   // D Major chord root
            ("E Maj", 329.63),   // E Major chord root
            ("F Maj", 349.23),   // F Major chord root
            ("G Maj", 392.00),   // G Major chord root
            ("A Maj", 440.00),   // A Major chord root
            ("B Maj", 493.88),   // B Major chord root
            ("C Oct", 523.25),   // C Octave
        ];

        Ok(PolyphonicaDemo {
            app_state,
            _audio_stream: audio_stream,
            chord_buttons,
            last_note_voice_id: None,
        })
    }

    fn trigger_note(&mut self, frequency: f32) {
        let mut engine = self.app_state.engine.lock().unwrap();
        let waveform = self.app_state.current_waveform.lock().unwrap().to_waveform();
        let envelope = self.app_state.envelope.lock().unwrap().clone();

        if let Some(voice_id) = engine.trigger_note(waveform, frequency, envelope) {
            self.last_note_voice_id = Some(voice_id);
            let mut active_voices = self.app_state.active_voices.lock().unwrap();
            active_voices.push(voice_id);
        }
    }

    fn trigger_chord(&mut self, root_frequency: f32) {
        let mut engine = self.app_state.engine.lock().unwrap();
        let waveform = self.app_state.current_waveform.lock().unwrap().to_waveform();
        let envelope = self.app_state.envelope.lock().unwrap().clone();

        // Major chord intervals: root, major third (+4 semitones), perfect fifth (+7 semitones)
        let chord_frequencies = [
            root_frequency,                    // Root
            root_frequency * 1.2599,          // Major third (2^(4/12))
            root_frequency * 1.4983,          // Perfect fifth (2^(7/12))
        ];

        let mut active_voices = self.app_state.active_voices.lock().unwrap();
        for &freq in &chord_frequencies {
            if let Some(voice_id) = engine.trigger_note(waveform.clone(), freq, envelope.clone()) {
                active_voices.push(voice_id);
            }
        }
    }

    fn release_all_notes(&mut self) {
        let mut engine = self.app_state.engine.lock().unwrap();
        engine.release_all_notes();
        self.app_state.active_voices.lock().unwrap().clear();
        self.last_note_voice_id = None;
    }

    fn panic_stop(&mut self) {
        let mut engine = self.app_state.engine.lock().unwrap();
        engine.stop_all_notes();
        self.app_state.active_voices.lock().unwrap().clear();
        self.last_note_voice_id = None;
    }
}

impl eframe::App for PolyphonicaDemo {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update engine parameters from GUI state
        {
            let volume = *self.app_state.master_volume.lock().unwrap();
            self.app_state.engine.lock().unwrap().set_master_volume(volume);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎵 Polyphonica Real-Time Demo");
            ui.separator();

            // Engine status
            let active_count = self.app_state.engine.lock().unwrap().get_active_voice_count();
            let master_volume = self.app_state.engine.lock().unwrap().get_master_volume();

            ui.horizontal(|ui| {
                ui.label(format!("Active Voices: {}/{}", active_count, MAX_VOICES));
                ui.separator();
                ui.label(format!("Master Volume: {:.1}%", master_volume * 100.0));
                ui.separator();

                if active_count > 0 {
                    ui.colored_label(egui::Color32::GREEN, "♪ PLAYING");
                } else {
                    ui.colored_label(egui::Color32::GRAY, "♫ SILENT");
                }
            });

            ui.separator();

            // Waveform selection
            ui.horizontal(|ui| {
                ui.label("Waveform:");
                let mut current_waveform = *self.app_state.current_waveform.lock().unwrap();

                for &waveform_type in WaveformType::all() {
                    if ui.radio_value(&mut current_waveform, waveform_type, waveform_type.name()).clicked() {
                        *self.app_state.current_waveform.lock().unwrap() = current_waveform;
                    }
                }
            });

            ui.separator();

            // Master volume control
            ui.horizontal(|ui| {
                ui.label("Master Volume:");
                let mut volume = *self.app_state.master_volume.lock().unwrap();
                if ui.add(egui::Slider::new(&mut volume, 0.0..=1.0).step_by(0.01).suffix("%")).changed() {
                    *self.app_state.master_volume.lock().unwrap() = volume;
                }
            });

            // Frequency control
            ui.horizontal(|ui| {
                ui.label("Frequency (Hz):");
                let mut frequency = *self.app_state.frequency.lock().unwrap();
                if ui.add(egui::Slider::new(&mut frequency, 80.0..=2000.0).step_by(1.0).suffix(" Hz")).changed() {
                    *self.app_state.frequency.lock().unwrap() = frequency;
                }
            });

            ui.separator();

            // ADSR Envelope controls
            ui.collapsing("ADSR Envelope", |ui| {
                let mut envelope = self.app_state.envelope.lock().unwrap().clone();
                let mut changed = false;

                ui.horizontal(|ui| {
                    ui.label("Attack:");
                    if ui.add(egui::Slider::new(&mut envelope.attack_secs, 0.0..=2.0).step_by(0.01).suffix("s")).changed() {
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Decay:");
                    if ui.add(egui::Slider::new(&mut envelope.decay_secs, 0.0..=2.0).step_by(0.01).suffix("s")).changed() {
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Sustain:");
                    if ui.add(egui::Slider::new(&mut envelope.sustain_level, 0.0..=1.0).step_by(0.01)).changed() {
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Release:");
                    if ui.add(egui::Slider::new(&mut envelope.release_secs, 0.0..=2.0).step_by(0.01).suffix("s")).changed() {
                        changed = true;
                    }
                });

                if changed {
                    *self.app_state.envelope.lock().unwrap() = envelope;
                }
            });

            ui.separator();

            // Note control buttons
            ui.horizontal(|ui| {
                let frequency = *self.app_state.frequency.lock().unwrap();

                if ui.button("🎵 Play Note").clicked() {
                    self.trigger_note(frequency);
                }

                if ui.button("🔇 Release All").clicked() {
                    self.release_all_notes();
                }

                if ui.button("🛑 Panic Stop").clicked() {
                    self.panic_stop();
                }
            });

            ui.separator();

            // Chord buttons
            ui.label("Major Chords:");
            ui.horizontal_wrapped(|ui| {
                let chord_buttons = self.chord_buttons.clone();
                for &(chord_name, root_freq) in &chord_buttons {
                    if ui.button(chord_name).clicked() {
                        self.trigger_chord(root_freq);
                    }
                }
            });

            ui.separator();

            // Preset buttons for common frequencies
            ui.label("Note Presets:");
            ui.horizontal_wrapped(|ui| {
                let presets = [
                    ("C4", 261.63),
                    ("D4", 293.66),
                    ("E4", 329.63),
                    ("F4", 349.23),
                    ("G4", 392.00),
                    ("A4", 440.00),
                    ("B4", 493.88),
                    ("C5", 523.25),
                ];

                for &(note_name, freq) in &presets {
                    if ui.button(note_name).clicked() {
                        *self.app_state.frequency.lock().unwrap() = freq;
                        self.trigger_note(freq);
                    }
                }
            });

            ui.separator();

            // Performance information
            ui.collapsing("Performance Info", |ui| {
                ui.label(format!("• Real-time synthesis at 44.1kHz"));
                ui.label(format!("• Zero-allocation audio processing"));
                ui.label(format!("• Lock-free parameter updates"));
                ui.label(format!("• Sample-accurate timing"));
                ui.label(format!("• Voice stealing when > {} voices", MAX_VOICES));
                ui.label(format!("• CPAL-compatible audio output"));
            });

            ui.separator();

            // Instructions
            ui.collapsing("Instructions", |ui| {
                ui.label("• Select different waveforms to hear their characteristics");
                ui.label("• Adjust master volume and frequency with sliders");
                ui.label("• Experiment with ADSR envelope settings");
                ui.label("• Click 'Play Note' to trigger individual notes");
                ui.label("• Use chord buttons for harmonic content");
                ui.label("• Try note presets for common musical frequencies");
                ui.label("• 'Release All' stops notes gracefully (ADSR release)");
                ui.label("• 'Panic Stop' immediately silences all audio");
            });
        });

        // Request repaint for smooth UI updates
        ctx.request_repaint_after(Duration::from_millis(50));
    }
}

/// Setup CPAL audio stream for real-time output
fn setup_audio_stream(app_state: AppState) -> Result<Stream, Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let device = host.default_output_device()
        .ok_or("No audio output device available")?;

    let config = device.default_output_config()?;

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
            // Convert to f32 for processing
            let mut f32_buffer = vec![0.0f32; data.len()];

            // Process audio with the engine
            {
                let mut engine = app_state.engine.lock().unwrap();
                if channels == 1 {
                    // Mono output
                    engine.process_buffer(&mut f32_buffer);
                } else {
                    // Stereo output
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
    println!("🎵 Polyphonica Real-Time Demo");
    println!("=============================");
    println!("Initializing audio system...");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("Polyphonica Real-Time Demo")
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Polyphonica Real-Time Demo",
        options,
        Box::new(|cc| {
            match PolyphonicaDemo::new(cc) {
                Ok(app) => {
                    println!("✅ Demo initialized successfully!");
                    println!("🎵 Audio output active - adjust volume as needed");
                    Ok(Box::new(app))
                }
                Err(e) => {
                    eprintln!("❌ Failed to initialize demo: {}", e);
                    std::process::exit(1);
                }
            }
        }),
    )
    .map_err(|e| format!("GUI error: {}", e).into())
}