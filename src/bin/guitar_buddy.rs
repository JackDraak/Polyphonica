use polyphonica::audio::accents::get_legacy_accent_sound;
use polyphonica::audio::synthesis::{get_legacy_sound_params, LegacySampleAdapter};
use polyphonica::patterns::{DrumPattern, PatternLibrary, PatternState};
use polyphonica::timing::{
    BeatClock, BeatEvent, BeatTracker, ClickType, Metronome as NewMetronome, TimeSignature,
};
/// Guitar Buddy - Musical Practice Companion
///
/// Phase 1: Advanced metronome with multiple click sounds and time signatures
/// Phase 2: Full accompaniment with drums, bass lines, and chord progressions
///
/// Uses Polyphonica real-time synthesis engine for precise, low-latency audio generation.
use polyphonica::{AdsrEnvelope, RealtimeEngine, Waveform};
// Audio stream available for future use
// Visualization imports available for future use
// Configuration management available for future use
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use eframe::egui;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
// HashMap no longer needed with modular architecture

/// Drum pattern system for Phase 2 - now using polyphonica::patterns module

/// Pattern playback state - now using polyphonica::patterns::PatternState
/// (local implementation removed)

// DrumSampleManager replaced with modular audio system

// ClickTypeAudioExt trait moved to polyphonica::audio::synthesis module
// Legacy functions available as get_legacy_sound_params() and get_legacy_accent_sound()

/// Metronome state and timing control
struct MetronomeState {
    is_playing: bool,
    tempo_bpm: f32,
    time_signature: TimeSignature,
    click_type: ClickType,
    accent_first_beat: bool,
    volume: f32,
    current_beat: u8,
    last_beat_time: Option<Instant>,
    // Phase 2: Pattern support
    pattern_state: PatternState,
    pattern_mode: bool, // true = pattern mode, false = metronome mode
    pattern_library: PatternLibrary,
    // Phase 2: Beat tracking for event-driven visualizer coupling
    beat_tracker: BeatTracker,
    // New timing module (Phase 1 refactoring)
    new_metronome: NewMetronome,
    // Audio synthesis adapter (Phase 7 refactoring)
    audio_samples: LegacySampleAdapter,
}

impl MetronomeState {
    fn new() -> Self {
        let mut instance = Self {
            is_playing: false,
            tempo_bpm: 120.0,
            time_signature: TimeSignature::new(4, 4),
            click_type: ClickType::WoodBlock,
            accent_first_beat: true,
            volume: 0.7,
            current_beat: 0,
            last_beat_time: None,
            pattern_state: PatternState::new(),
            pattern_mode: false,
            pattern_library: PatternLibrary::with_defaults(),
            beat_tracker: BeatTracker::new(),
            new_metronome: NewMetronome::new(TimeSignature::new(4, 4)),
            audio_samples: {
                let mut adapter = LegacySampleAdapter::new();
                if let Err(e) = adapter.load_drum_samples() {
                    println!("⚠️  Error loading drum samples: {}", e);
                }
                adapter
            },
        };
        // Sync initial settings to new metronome
        instance.sync_to_new_metronome();
        instance
    }

    /// Sync settings to new metronome
    fn sync_to_new_metronome(&mut self) {
        self.new_metronome.set_time_signature(self.time_signature);
        self.new_metronome.set_click_type(self.click_type);
        self.new_metronome
            .set_accent_first_beat(self.accent_first_beat);
    }

    /// Calculate time between beats in milliseconds
    fn beat_interval_ms(&self) -> f64 {
        60000.0 / self.tempo_bpm as f64
    }

    /// Check if it's time for the next beat (using new timing module)
    fn should_trigger_beat(&mut self) -> bool {
        if !self.is_playing {
            return false;
        }

        // Use new metronome timing with discrete scheduling
        let events = self.new_metronome.check_triggers(self.tempo_bpm);
        if !events.is_empty() {
            // Update current beat to match new metronome
            self.current_beat = events[0].beat_number;
            self.last_beat_time = Some(events[0].timestamp);
            true
        } else {
            false
        }
    }

    /// Set drum pattern and switch to pattern mode
    fn set_pattern(&mut self, pattern: DrumPattern) {
        self.pattern_state.set_pattern(pattern);
        self.pattern_mode = true;
    }

    /// Clear pattern and return to metronome mode
    fn clear_pattern(&mut self) {
        self.pattern_state.clear_pattern();
        self.pattern_mode = false;
    }

    /// Check for pattern triggers and return samples to play
    fn check_pattern_triggers(&mut self) -> Vec<(ClickType, bool)> {
        if self.pattern_mode && self.is_playing {
            // Get pattern triggers directly
            self.pattern_state
                .check_pattern_triggers(self.tempo_bpm)
                .into_iter()
                .map(|trigger| (trigger.click_type, trigger.is_accent))
                .collect()
        } else {
            vec![]
        }
    }

    fn start(&mut self) {
        self.is_playing = true;
        self.current_beat = 0;
        self.last_beat_time = None;
        // Start new metronome with updated settings
        self.new_metronome.start();
        if self.pattern_mode {
            self.pattern_state.start();
        }
    }

    fn stop(&mut self) {
        self.is_playing = false;
        self.current_beat = 0;
        self.last_beat_time = None;
        // Stop new metronome
        self.new_metronome.stop();
        if self.pattern_mode {
            self.pattern_state.stop();
        }
    }

    fn pause(&mut self) {
        self.is_playing = false;
        // Pause new metronome
        self.new_metronome.pause();
        if self.pattern_mode {
            self.pattern_state.stop();
        }
    }

    fn resume(&mut self) {
        self.is_playing = true;
        self.last_beat_time = Some(Instant::now());
        // Resume new metronome
        self.new_metronome.resume();
        if self.pattern_mode {
            self.pattern_state.start();
        }
    }
}

/// Shared state between GUI and audio threads
#[derive(Clone)]
struct AppState {
    engine: Arc<Mutex<RealtimeEngine>>,
    metronome: Arc<Mutex<MetronomeState>>,
}

impl AppState {
    fn new(engine: Arc<Mutex<RealtimeEngine>>, metronome: Arc<Mutex<MetronomeState>>) -> Self {
        Self { engine, metronome }
    }
}

/// GUI Components Module
mod gui_components {
    use super::*;
    use egui::{Color32, Slider, Ui};

    /// Status display panel component
    pub struct StatusDisplayPanel;

    impl StatusDisplayPanel {
        pub fn show(ui: &mut Ui, app_state: &AppState) {
            let metronome = app_state.metronome.lock().unwrap();
            let is_playing = metronome.is_playing;
            let current_beat = metronome.current_beat;
            let time_sig = metronome.time_signature;
            let tempo = metronome.tempo_bpm;
            drop(metronome);

            ui.horizontal(|ui| {
                if is_playing {
                    ui.colored_label(Color32::GREEN, "♪ PLAYING");
                    ui.separator();
                    ui.label(format!(
                        "Beat: {}/{}",
                        current_beat, time_sig.beats_per_measure
                    ));
                } else {
                    ui.colored_label(Color32::GRAY, "⏸ STOPPED");
                }
                ui.separator();
                ui.label(format!("Tempo: {:.0} BPM", tempo));
                ui.separator();
                ui.label(format!("Time: {}", time_sig.display()));
            });
        }
    }

    /// Transport controls panel component
    pub struct TransportControlsPanel;

    impl TransportControlsPanel {
        pub fn show(ui: &mut Ui, app_state: &AppState) {
            ui.horizontal(|ui| {
                let mut metronome = app_state.metronome.lock().unwrap();

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
        }
    }

    /// Tempo control panel component
    pub struct TempoControlPanel;

    impl TempoControlPanel {
        pub fn show(ui: &mut Ui, app_state: &AppState) {
            ui.horizontal(|ui| {
                ui.label("Tempo (BPM):");
                let mut metronome = app_state.metronome.lock().unwrap();
                ui.add(
                    Slider::new(&mut metronome.tempo_bpm, 40.0..=200.0)
                        .step_by(1.0)
                        .suffix(" BPM"),
                );

                // Preset tempo buttons
                ui.separator();
                for &(name, bpm) in &[("Slow", 60.0), ("Med", 120.0), ("Fast", 160.0)] {
                    if ui.button(name).clicked() {
                        metronome.tempo_bpm = bpm;
                    }
                }
            });
        }
    }

    /// Time signature selection panel component
    pub struct TimeSignaturePanel;

    impl TimeSignaturePanel {
        pub fn show(ui: &mut Ui, app_state: &AppState) {
            ui.horizontal(|ui| {
                ui.label("Time Signature:");
                let mut metronome = app_state.metronome.lock().unwrap();
                let mut current_sig = metronome.time_signature;

                for &(name, sig) in &TimeSignature::common_signatures() {
                    if ui.radio_value(&mut current_sig, sig, name).clicked() {
                        metronome.time_signature = current_sig;
                        metronome.current_beat = 0; // Reset beat counter
                                                    // Sync to new metronome
                        metronome.sync_to_new_metronome();
                    }
                }
            });
        }
    }

    /// Click sound selection panel component
    pub struct ClickSoundPanel;

    impl ClickSoundPanel {
        pub fn show(ui: &mut Ui, app_state: &AppState) {
            ui.horizontal(|ui| {
                ui.label("Click Sound:");
                let mut metronome = app_state.metronome.lock().unwrap();
                let mut current_click = metronome.click_type;

                for &click_type in &ClickType::all() {
                    if ui
                        .radio_value(&mut current_click, click_type, click_type.name())
                        .clicked()
                    {
                        metronome.click_type = current_click;
                        // Sync to new metronome
                        metronome.sync_to_new_metronome();
                    }
                }
            });
        }
    }

    /// Volume controls panel component
    pub struct VolumeControlsPanel;

    impl VolumeControlsPanel {
        pub fn show(ui: &mut Ui, app_state: &AppState) {
            ui.horizontal(|ui| {
                ui.label("Volume:");
                let mut metronome = app_state.metronome.lock().unwrap();
                ui.add(
                    Slider::new(&mut metronome.volume, 0.0..=1.0)
                        .step_by(0.01)
                        .suffix("%"),
                );

                ui.separator();
                ui.checkbox(&mut metronome.accent_first_beat, "Accent first beat");
            });
        }
    }

    /// Test controls panel component
    pub struct TestControlsPanel;

    impl TestControlsPanel {
        pub fn show<F>(ui: &mut Ui, app_state: &AppState, trigger_click_fn: F)
        where
            F: Fn(bool, u8) + Clone,
        {
            ui.horizontal(|ui| {
                if ui.button("🔊 Test Click").clicked() {
                    let metronome = app_state.metronome.lock().unwrap();
                    // drum_samples now accessed via metronome.audio_samples
                    let (waveform, frequency, envelope) =
                        get_legacy_sound_params(metronome.click_type, &metronome.audio_samples);
                    drop(metronome);
                    // No longer need to drop drum_samples

                    let mut engine = app_state.engine.lock().unwrap();
                    engine.trigger_note(waveform, frequency, envelope);
                }

                if ui.button("🔊 Test Accent").clicked() {
                    let trigger_click = trigger_click_fn.clone();
                    trigger_click(true, 1); // Test accent as beat 1
                }
            });
        }
    }

    /// Pattern selection panel component
    pub struct PatternSelectionPanel;

    impl PatternSelectionPanel {
        pub fn show(ui: &mut Ui, app_state: &AppState) {
            ui.collapsing("🥁 Drum Patterns", |ui| {
                let mut metronome = app_state.metronome.lock().unwrap();

                ui.horizontal(|ui| {
                    ui.label("Mode:");
                    let mut pattern_mode = metronome.pattern_mode;
                    if ui
                        .radio_value(&mut pattern_mode, false, "Metronome")
                        .clicked()
                    {
                        metronome.clear_pattern();
                    }
                    if ui
                        .radio_value(&mut pattern_mode, true, "Drum Pattern")
                        .clicked()
                        && !metronome.pattern_mode
                    {
                        // Set default pattern when switching to pattern mode
                        let was_playing = metronome.is_playing;
                        let basic_rock =
                            metronome.pattern_library.get_pattern("basic_rock").cloned();
                        if let Some(pattern) = basic_rock {
                            metronome.set_pattern(pattern);
                        }
                        // If metronome was playing, continue playing in pattern mode
                        if was_playing {
                            metronome.pattern_state.start();
                        }
                    }
                });

                if metronome.pattern_mode {
                    ui.separator();
                    ui.label("Available Patterns:");

                    let mut current_pattern_name = metronome
                        .pattern_state
                        .current_pattern()
                        .map(|p| p.name.clone())
                        .unwrap_or_else(|| "None".to_string());

                    let available_patterns: Vec<_> = metronome
                        .pattern_library
                        .all_patterns()
                        .into_iter()
                        .cloned()
                        .collect();
                    for pattern in available_patterns {
                        if ui
                            .radio_value(
                                &mut current_pattern_name,
                                pattern.name.clone(),
                                &pattern.name,
                            )
                            .clicked()
                        {
                            metronome.set_pattern(pattern);
                        }
                    }

                    ui.separator();
                    if let Some(pattern) = metronome.pattern_state.current_pattern() {
                        ui.label(format!(
                            "Time: {}",
                            Self::format_time_signature(&pattern.time_signature)
                        ));
                        ui.label(format!(
                            "Tempo Range: {}-{} BPM",
                            pattern.tempo_range.0, pattern.tempo_range.1
                        ));

                        // Show current pattern position if playing
                        if metronome.pattern_state.is_playing() {
                            ui.label(format!(
                                "Position: {:.1}",
                                metronome.pattern_state.current_beat_position()
                            ));
                        }
                    }
                }
            });
        }

        fn format_time_signature(time_sig: &TimeSignature) -> String {
            format!("{}/{}", time_sig.beats_per_measure, time_sig.note_value)
        }
    }

    /// Beat visualization panel component
    pub struct BeatVisualizationPanel;

    impl BeatVisualizationPanel {
        pub fn show(ui: &mut Ui, app_state: &AppState) {
            ui.collapsing("Beat Visualization", |ui| {
                ui.horizontal(|ui| {
                    let metronome = app_state.metronome.lock().unwrap();
                    let is_playing = metronome.is_playing;
                    let current_beat = metronome.current_beat;

                    // Use last triggered beat for both metronome and pattern modes (coupled with audio)
                    let (current_beat_display, time_sig) = if metronome.pattern_mode {
                        // For pattern mode, use BeatTracker for last triggered beat
                        let (tracked_beat, _) = metronome.beat_tracker.get_current_beat();
                        let pattern_beat = if metronome.is_playing && tracked_beat > 0 {
                            tracked_beat
                        } else {
                            1 // Default to beat 1 when not playing or no triggers yet
                        };
                        let time_sig = metronome
                            .pattern_state
                            .current_pattern()
                            .map(|p| p.time_signature)
                            .unwrap_or(metronome.time_signature);
                        (pattern_beat, time_sig)
                    } else {
                        // For metronome mode, use BeatTracker for last triggered beat
                        let (tracked_beat, _) = metronome.beat_tracker.get_current_beat();
                        let metronome_beat = if metronome.is_playing && tracked_beat > 0 {
                            tracked_beat
                        } else {
                            current_beat // Fallback to current_beat if no triggers yet
                        };
                        (metronome_beat, metronome.time_signature)
                    };

                    for beat in 1..=time_sig.beats_per_measure {
                        let is_current = beat == current_beat_display && is_playing;

                        // Check accents: use actual triggered accent when showing current beat, pattern definition otherwise
                        let is_accent = if is_current && metronome.is_playing {
                            // For current beat, use BeatTracker accent status (coupled with audio)
                            let (_, tracked_accent) = metronome.beat_tracker.get_current_beat();
                            tracked_accent
                        } else if metronome.pattern_mode {
                            // For non-current beats in pattern mode, show pattern definition
                            if let Some(pattern) = metronome.pattern_state.current_pattern() {
                                pattern.beats.iter().any(|pattern_beat| {
                                    let beat_num = pattern_beat.beat_position.floor() as u8;
                                    beat_num == beat && pattern_beat.accent
                                })
                            } else {
                                false
                            }
                        } else {
                            // For metronome mode, use metronome setting
                            beat == 1 && metronome.accent_first_beat
                        };

                        let color = if is_current && is_accent {
                            Color32::YELLOW
                        } else if is_current {
                            Color32::GREEN
                        } else if is_accent {
                            Color32::LIGHT_GRAY
                        } else {
                            Color32::GRAY
                        };

                        let symbol = if is_accent { "●" } else { "○" };
                        ui.colored_label(color, symbol);
                    }
                });

                let metronome = app_state.metronome.lock().unwrap();
                ui.label(format!(
                    "Interval: {:.0}ms between beats",
                    metronome.beat_interval_ms()
                ));

                // Show additional pattern info if in pattern mode
                if metronome.pattern_mode && metronome.pattern_state.is_playing() {
                    ui.label(format!(
                        "Pattern Position: {:.2}",
                        metronome.pattern_state.current_beat_position()
                    ));
                }
            });
        }
    }
}

use gui_components::*;

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

        // Create shared state (drum samples now managed within MetronomeState)
        let app_state = AppState::new(engine.clone(), metronome.clone());

        // Setup audio stream
        let audio_stream = setup_audio_stream(app_state.clone())?;

        Ok(GuitarBuddy {
            app_state,
            _audio_stream: audio_stream,
        })
    }

    fn trigger_click(&self, is_accent: bool, beat_number: u8) {
        let mut metronome = self.app_state.metronome.lock().unwrap();
        // drum_samples now accessed via metronome.audio_samples

        // Use different sound for accents to make them clearly distinct
        let (waveform, frequency, envelope) = if is_accent && metronome.accent_first_beat {
            // For accents, use a more prominent sound
            get_legacy_accent_sound(metronome.click_type, &metronome.audio_samples)
        } else {
            // Regular click
            get_legacy_sound_params(metronome.click_type, &metronome.audio_samples)
        };

        let volume = metronome.volume;
        let click_type = metronome.click_type;

        // Record beat event for visualizer coupling
        let beat_event = BeatEvent::new(
            beat_number,
            is_accent,
            vec![click_type],
            metronome.tempo_bpm,
            metronome.time_signature,
        );
        metronome.beat_tracker.record_beat(beat_event);

        drop(metronome);
        // No longer need to drop drum_samples

        let mut engine = self.app_state.engine.lock().unwrap();
        engine.trigger_note_with_volume(waveform, frequency, envelope, volume);
    }

    fn get_accent_sound(&self, metronome: &MetronomeState) -> (Waveform, f32, AdsrEnvelope) {
        // Choose accent sound based on the current click type
        match metronome.click_type {
            // For drum samples, use kick drum for accent
            ClickType::AcousticSnare
            | ClickType::HiHatClosed
            | ClickType::HiHatOpen
            | ClickType::RimShot
            | ClickType::Stick
            | ClickType::HiHatLoose
            | ClickType::HiHatVeryLoose
            | ClickType::CymbalSplash
            | ClickType::CymbalRoll
            | ClickType::Ride
            | ClickType::RideBell => {
                get_legacy_sound_params(ClickType::AcousticKick, &metronome.audio_samples)
            }
            // For kick drum variants, use snare for accent
            ClickType::AcousticKick | ClickType::KickTight => {
                get_legacy_sound_params(ClickType::AcousticSnare, &metronome.audio_samples)
            }
            // For synthetic sounds, use higher pitch and different waveform
            ClickType::WoodBlock => (
                Waveform::Square, // Different waveform
                1600.0,           // Higher pitch
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.1,
                    sustain_level: 0.0,
                    release_secs: 0.05,
                },
            ),
            ClickType::DigitalBeep => (
                Waveform::Square, // Different waveform
                2000.0,           // Much higher pitch
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.12,
                    sustain_level: 0.0,
                    release_secs: 0.06,
                },
            ),
            ClickType::Cowbell => (
                Waveform::Triangle, // Different waveform
                1600.0,             // Higher pitch
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.2,
                    sustain_level: 0.0,
                    release_secs: 0.15,
                },
            ),
            ClickType::ElectroClick => (
                Waveform::Sine, // Different waveform
                2400.0,         // Much higher pitch
                AdsrEnvelope {
                    attack_secs: 0.001,
                    decay_secs: 0.06,
                    sustain_level: 0.0,
                    release_secs: 0.04,
                },
            ),
        }
    }

    /// Trigger a specific drum sample for pattern playback
    fn trigger_pattern_sample(
        &self,
        click_type: ClickType,
        is_accent: bool,
        beat_number: u8,
        samples: Vec<ClickType>,
    ) {
        let mut metronome = self.app_state.metronome.lock().unwrap();
        // drum_samples now accessed via metronome.audio_samples

        let (waveform, frequency, envelope) =
            get_legacy_sound_params(click_type, &metronome.audio_samples);

        // Pattern accents need volume boost since they use same samples, unlike metronome which uses different sounds
        let volume = if is_accent {
            (metronome.volume * 1.5).min(1.0) // 50% louder for pattern accents
        } else {
            metronome.volume
        };

        // Record beat event for visualizer coupling (only once per beat, not per sample)
        if click_type == samples[0] {
            // Only record event for the first sample in the group
            let beat_event = BeatEvent::new(
                beat_number,
                is_accent,
                samples.clone(),
                metronome.tempo_bpm,
                metronome.time_signature,
            );
            metronome.beat_tracker.record_beat(beat_event);
        }

        drop(metronome);
        // No longer need to drop drum_samples

        let mut engine = self.app_state.engine.lock().unwrap();
        engine.trigger_note_with_volume(waveform, frequency, envelope, volume);
    }
}

impl eframe::App for GuitarBuddy {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for metronome beats and drum patterns
        {
            let mut metronome = self.app_state.metronome.lock().unwrap();

            if metronome.pattern_mode {
                // Pattern mode: only play drum patterns
                // Get beat number before mutable borrow
                let beat_number = metronome.pattern_state.current_beat_number();
                let pattern_triggers = metronome.check_pattern_triggers();
                if !pattern_triggers.is_empty() {
                    // Collect beat information for BeatTracker event
                    let _has_accent = pattern_triggers.iter().any(|(_, is_accent)| *is_accent);
                    let all_samples: Vec<ClickType> = pattern_triggers
                        .iter()
                        .map(|(click_type, _)| *click_type)
                        .collect();

                    // Visualizer state is now handled by BeatTracker (no manual updates needed)

                    drop(metronome);
                    for (click_type, is_accent) in pattern_triggers {
                        self.trigger_pattern_sample(
                            click_type,
                            is_accent,
                            beat_number,
                            all_samples.clone(),
                        );
                    }
                }
            } else {
                // Metronome mode: only play metronome clicks
                if metronome.should_trigger_beat() {
                    let is_accent = metronome.current_beat == 1;
                    let beat_number = metronome.current_beat;

                    // Visualizer state is now handled by BeatTracker (no manual updates needed)

                    drop(metronome);
                    self.trigger_click(is_accent, beat_number);
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎸 Guitar Buddy - Practice Companion");
            ui.separator();

            // Status display panel
            StatusDisplayPanel::show(ui, &self.app_state);

            ui.separator();

            // Transport controls panel
            TransportControlsPanel::show(ui, &self.app_state);

            ui.separator();

            // Tempo control panel
            TempoControlPanel::show(ui, &self.app_state);

            ui.separator();

            // Time signature panel
            TimeSignaturePanel::show(ui, &self.app_state);

            ui.separator();

            // Click sound panel
            ClickSoundPanel::show(ui, &self.app_state);

            ui.separator();

            // Volume controls panel
            VolumeControlsPanel::show(ui, &self.app_state);

            ui.separator();

            // Test controls panel
            let trigger_click_fn = |is_accent: bool, beat_number: u8| {
                self.trigger_click(is_accent, beat_number);
            };
            TestControlsPanel::show(ui, &self.app_state, trigger_click_fn);

            ui.separator();

            // Pattern selection panel
            PatternSelectionPanel::show(ui, &self.app_state);

            ui.separator();

            // Beat visualization panel
            BeatVisualizationPanel::show(ui, &self.app_state);

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
    let device = host
        .default_output_device()
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
        Box::new(|cc| match GuitarBuddy::new(cc) {
            Ok(app) => {
                println!("✅ Guitar Buddy initialized successfully!");
                println!("🎵 Audio output active - ready to rock!");
                Ok(Box::new(app))
            }
            Err(e) => {
                eprintln!("❌ Failed to initialize Guitar Buddy: {}", e);
                std::process::exit(1);
            }
        }),
    )
    .map_err(|e| format!("GUI error: {}", e).into())
}
