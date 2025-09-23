use polyphonica::audio::accents::get_accent_sound;
use polyphonica::audio::synthesis::{get_sound_params, AudioSampleAdapter};
use polyphonica::melody::{MelodyAssistantState, Note, KeySelection, GenerationParameters, TimelineConfig};
use polyphonica::patterns::{DrumPattern, MasterCollection, PatternLibrary, PatternState};
use polyphonica::patterns::types::PatternGenre;
use polyphonica::timing::{
    BeatClock, BeatEvent, BeatTracker, ClickType, Metronome as NewMetronome, TimeSignature,
};
/// Guitar Buddy - Musical Practice Companion
///
/// Advanced metronome and drum machine with multiple click sounds, time signatures,
/// and drum patterns. Uses Polyphonica real-time synthesis engine for precise,
/// low-latency audio generation.
///
/// # Features
///
/// **Currently Working:**
/// - High-precision metronome with <1ms timing accuracy
/// - Multiple click sounds (wood block, cowbell, hi-hat, etc.)
/// - Time signature support (4/4, 3/4, 2/4, 6/8, and more)
/// - Tempo control from 60-200 BPM with slider and preset buttons
/// - Pattern mode with drum patterns from 6 genres (Rock, Jazz, Latin, Funk, Pop, Electronic)
/// - Real-time beat visualization with colored circles
/// - Volume control and accent beats on first beat
/// - Test buttons for previewing sounds
/// - Play/pause/stop transport controls
///
/// **Limitations:**
/// - No pattern editing or custom pattern creation in GUI
/// - No MIDI synchronization or export
/// - No recording or playback of practice sessions
/// - No advanced tempo features (gradual tempo changes, tap tempo)
/// - No customizable key bindings
/// - No multiple pattern layers or complex arrangements
///
/// # Usage
///
/// Run with: `cargo run --bin guitar-buddy`
///
/// The GUI provides intuitive controls for all features. Switch between metronome
/// mode (simple click) and pattern mode (drum patterns) using the toggle button.
use polyphonica::RealtimeEngine;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use eframe::egui;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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
    pattern_state: PatternState,
    pattern_mode: bool, // true = pattern mode, false = metronome mode
    pattern_library: PatternLibrary,
    selected_genre: Option<PatternGenre>, // Genre filter for pattern selection
    beat_tracker: BeatTracker,
    new_metronome: NewMetronome,
    audio_samples: AudioSampleAdapter,
    melody_assistant: MelodyAssistantState,
    show_chord_progressions: bool,
    skill_level: f32, // 0.0 = beginner, 1.0 = expert
}

impl MetronomeState {
    fn new() -> Self {
        let mut instance = Self {
            is_playing: false,
            tempo_bpm: 120.0,
            time_signature: TimeSignature::new(4, 4),
            click_type: ClickType::RimShot,
            accent_first_beat: true,
            volume: 0.33, // default volume
            current_beat: 0,
            last_beat_time: None,
            pattern_state: PatternState::new(),
            pattern_mode: false,
            pattern_library: PatternLibrary::with_defaults(),
            selected_genre: None, // Show all genres by default
            beat_tracker: BeatTracker::new(),
            new_metronome: NewMetronome::new(TimeSignature::new(4, 4)),
            audio_samples: {
                let mut adapter = AudioSampleAdapter::new();
                if let Err(e) = adapter.load_drum_samples() {
                    println!("⚠️  Error loading drum samples: {}", e);
                }
                adapter
            },
            melody_assistant: MelodyAssistantState::new_for_key(Note::C, true),
            show_chord_progressions: false,
            skill_level: 0.3, // Default to medium skill level
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
                    if metronome.last_beat_time.is_some()
                        && ui.button("▶ Resume").clicked() {
                            metronome.resume();
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
                    let (waveform, frequency, envelope) =
                        get_sound_params(metronome.click_type, &metronome.audio_samples);
                    let volume = metronome.volume;
                    drop(metronome);

                    let mut engine = app_state.engine.lock().unwrap();
                    engine.trigger_note_with_volume(waveform, frequency, envelope, volume);
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
                    ui.label("Pattern Selection:");

                    // Genre filter dropdown
                    ui.horizontal(|ui| {
                        ui.label("Genre Filter:");
                        let current_genre_text = metronome
                            .selected_genre
                            .as_ref()
                            .map(|g| g.display_name())
                            .unwrap_or("All Genres");

                        egui::ComboBox::from_label("")
                            .selected_text(current_genre_text)
                            .width(120.0)
                            .show_ui(ui, |ui| {
                                if ui
                                    .selectable_value(&mut metronome.selected_genre, None, "All Genres")
                                    .clicked()
                                {}

                                for genre in PatternGenre::all() {
                                    if ui
                                        .selectable_value(
                                            &mut metronome.selected_genre,
                                            Some(genre.clone()),
                                            genre.display_name(),
                                        )
                                        .clicked()
                                    {}
                                }
                            });
                    });

                    // Get patterns based on genre filter
                    let mut available_patterns: Vec<_> = if let Some(ref genre) = metronome.selected_genre {
                        MasterCollection::by_genre(genre)
                    } else {
                        MasterCollection::all()
                    };
                    // Sort patterns alphabetically by display name
                    available_patterns.sort_by(|a, b| a.display_name.cmp(&b.display_name));

                    let current_pattern_name = metronome
                        .pattern_state
                        .current_pattern()
                        .map(|p| p.display_name.clone())
                        .unwrap_or_else(|| "Select Pattern".to_string());

                    let mut selected_display_name = current_pattern_name.clone();

                    egui::ComboBox::from_label("")
                        .selected_text(&current_pattern_name)
                        .width(200.0)
                        .height(120.0) // Show approximately 5 patterns at once
                        .show_ui(ui, |ui| {
                            for pattern in &available_patterns {
                                if ui
                                    .selectable_value(
                                        &mut selected_display_name,
                                        pattern.display_name.clone(),
                                        &pattern.display_name,
                                    )
                                    .clicked()
                                {
                                    metronome.set_pattern(pattern.clone());
                                }
                            }
                        });

                    ui.separator();
                    if let Some(pattern) = metronome.pattern_state.current_pattern() {
                        ui.horizontal(|ui| {
                            ui.label("Pattern Info:");
                            ui.separator();
                            ui.label(format!(
                                "Time: {}",
                                Self::format_time_signature(&pattern.time_signature)
                            ));
                            ui.separator();
                            ui.label(format!(
                                "Tempo: {}-{} BPM",
                                pattern.tempo_range.0, pattern.tempo_range.1
                            ));
                            ui.separator();
                            ui.label(format!("Difficulty: {}/5", pattern.metadata.difficulty));
                        });

                        if !pattern.metadata.description.is_empty() {
                            ui.label(format!("Description: {}", pattern.metadata.description));
                        }

                        if !pattern.metadata.tags.is_empty() {
                            ui.horizontal(|ui| {
                                ui.label("Tags:");
                                for tag in &pattern.metadata.tags {
                                    ui.small(format!("#{}", tag));
                                }
                            });
                        }

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

    /// Melody assistant panel component
    pub struct MelodyAssistantPanel;

    impl MelodyAssistantPanel {
        pub fn show(ui: &mut Ui, app_state: &AppState) {
            ui.collapsing("🎵 Chord Progressions", |ui| {
                let mut metronome = app_state.metronome.lock().unwrap();

                // Toggle for showing chord progressions
                ui.horizontal(|ui| {
                    ui.checkbox(&mut metronome.show_chord_progressions, "Enable chord progressions");
                    if ui.button("Start/Stop").clicked() {
                        if metronome.show_chord_progressions {
                            if metronome.melody_assistant.is_running() {
                                metronome.melody_assistant.stop();
                            } else {
                                metronome.melody_assistant.start();
                            }
                        }
                    }
                });

                if metronome.show_chord_progressions {
                    ui.separator();

                    // Key selection section
                    ui.horizontal(|ui| {
                        ui.label("Key:");
                        let current_key = metronome.melody_assistant.get_current_key();
                        ui.label(format!("{}", current_key));
                    });

                    // Timeline display
                    let timeline_data = metronome.melody_assistant.get_timeline_display();

                    ui.separator();
                    ui.label("Timeline:");

                    ui.horizontal(|ui| {
                        // Current chord - highlight with background color and audio controls
                        if let Some(ref current_chord) = timeline_data.current_chord {
                            ui.allocate_ui_with_layout(
                                egui::Vec2::new(150.0, 40.0),
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    // Background highlight for active chord
                                    let rect = ui.available_rect_before_wrap();
                                    ui.painter().rect_filled(
                                        rect,
                                        egui::Rounding::same(5.0),
                                        Color32::from_rgb(40, 80, 40), // Dark green background
                                    );

                                    ui.vertical(|ui| {
                                        ui.colored_label(Color32::WHITE, format!("NOW: {}", current_chord.chord.symbol()));
                                        ui.horizontal(|ui| {
                                            if ui.small_button("♪ Root").clicked() {
                                                // Play root note in mid-range
                                                println!("Playing root note: {} Hz", current_chord.chord.root_frequency());
                                            }
                                            if ui.small_button("♫ Chord").clicked() {
                                                // Play full chord with multi-octave voicing
                                                let frequencies = current_chord.chord.chord_frequencies();
                                                println!("Playing chord (bass+mid+treble): {:?}", frequencies);
                                            }
                                            if ui.small_button("🎵 Melody").clicked() {
                                                // Play melody notes in treble range
                                                let melody_freqs = current_chord.chord.melody_frequencies();
                                                println!("Playing melody notes: {:?}", melody_freqs);
                                            }
                                            if ui.small_button("🔄 Arp").clicked() {
                                                // Play arpeggio across 2 octaves
                                                let arp_freqs = current_chord.chord.arpeggio_frequencies(2);
                                                println!("Playing arpeggio: {:?}", arp_freqs);
                                            }
                                        });
                                    });
                                },
                            );
                        } else {
                            ui.colored_label(Color32::GRAY, "NOW: -");
                        }

                        ui.separator();

                        // Next chord
                        if let Some(ref next_chord) = timeline_data.next_chord {
                            ui.vertical(|ui| {
                                ui.colored_label(Color32::YELLOW, format!("NEXT: {}", next_chord.chord.symbol()));
                                ui.horizontal(|ui| {
                                    if ui.small_button("♪").clicked() {
                                        println!("Playing next root: {} Hz", next_chord.chord.root_frequency());
                                    }
                                    if ui.small_button("♫").clicked() {
                                        let frequencies = next_chord.chord.chord_frequencies();
                                        println!("Playing next chord (multi-octave): {:?}", frequencies);
                                    }
                                    if ui.small_button("🎵").clicked() {
                                        let melody_freqs = next_chord.chord.melody_frequencies();
                                        println!("Playing next melody: {:?}", melody_freqs);
                                    }
                                });
                            });
                        } else {
                            ui.colored_label(Color32::GRAY, "NEXT: -");
                        }

                        ui.separator();

                        // Following chord
                        if let Some(ref following_chord) = timeline_data.following_chord {
                            ui.vertical(|ui| {
                                ui.label(format!("FOLLOWING: {}", following_chord.chord.symbol()));
                                ui.horizontal(|ui| {
                                    if ui.small_button("♪").clicked() {
                                        println!("Playing following root: {} Hz", following_chord.chord.root_frequency());
                                    }
                                    if ui.small_button("♫").clicked() {
                                        let frequencies = following_chord.chord.chord_frequencies();
                                        println!("Playing following chord (multi-octave): {:?}", frequencies);
                                    }
                                    if ui.small_button("🎵").clicked() {
                                        let melody_freqs = following_chord.chord.melody_frequencies();
                                        println!("Playing following melody: {:?}", melody_freqs);
                                    }
                                });
                            });
                        } else {
                            ui.label("FOLLOWING: -");
                        }
                    });

                    // Key center info
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label(format!("Key center: {}", timeline_data.current_key_center));
                        if let Some(next_key) = timeline_data.next_key_center {
                            ui.colored_label(Color32::YELLOW, format!("→ {}", next_key));
                        }
                    });

                    // Chromatic note selection checkboxes
                    ui.separator();
                    ui.label("Note Selection:");

                    let mut key_selection = metronome.melody_assistant.get_key_selection().clone();
                    let all_notes = Note::all();
                    let mut selection_changed = false;

                    // Display checkboxes in two rows (6 notes each)
                    ui.horizontal(|ui| {
                        for &note in &all_notes[0..6] {
                            let mut enabled = key_selection.is_note_enabled(note);
                            if ui.checkbox(&mut enabled, note.name()).changed() {
                                key_selection.set_note_enabled(note, enabled);
                                selection_changed = true;
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        for &note in &all_notes[6..12] {
                            let mut enabled = key_selection.is_note_enabled(note);
                            if ui.checkbox(&mut enabled, note.name()).changed() {
                                key_selection.set_note_enabled(note, enabled);
                                selection_changed = true;
                            }
                        }
                    });

                    // Update melody assistant if selection changed
                    if selection_changed {
                        metronome.melody_assistant.update_key_selection(key_selection);
                    }

                    // Quick preset buttons
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Presets:");
                        if ui.button("C Major").clicked() {
                            let c_major = KeySelection::for_major_key(Note::C);
                            metronome.melody_assistant.update_key_selection(c_major);
                        }
                        if ui.button("A Minor").clicked() {
                            let a_minor = KeySelection::for_minor_key(Note::A);
                            metronome.melody_assistant.update_key_selection(a_minor);
                        }
                        if ui.button("All Notes").clicked() {
                            let all_notes = KeySelection::all_notes();
                            metronome.melody_assistant.update_key_selection(all_notes);
                        }
                    });

                    // Skill level slider for note density control
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Skill Level:");
                        if ui.add(egui::Slider::new(&mut metronome.skill_level, 0.0..=1.0)
                            .text("Note Density")
                            .show_value(false)).changed() {
                            // Apply skill level changes to melody generation parameters

                            // Update generation parameters based on skill level
                            let generation_params = GenerationParameters {
                                theory_adherence: 0.9 - (metronome.skill_level * 0.2), // Less strict at higher levels
                                repetition_avoidance: 0.7,
                                voice_leading_weight: 0.6,
                                cadence_strength: 0.8 - (metronome.skill_level * 0.3), // More adventurous at higher levels
                                modulation_tendency: metronome.skill_level * 0.3, // More modulation at higher levels
                                complexity_level: metronome.skill_level, // More complex chords at higher levels
                                rhythm_density: metronome.skill_level,
                            };
                            metronome.melody_assistant.update_generation_params(generation_params);

                            // Update timeline config for chord change frequency
                            let timeline_config = TimelineConfig::default().for_skill_level(metronome.skill_level);
                            metronome.melody_assistant.update_timeline_config(timeline_config);

                            println!("Skill level changed to: {:.2} - chord frequency updated", metronome.skill_level);
                        }

                        let level_text = match metronome.skill_level {
                            x if x < 0.2 => "Beginner (1/4 notes over 4 bars)",
                            x if x < 0.4 => "Easy (1/8 notes, simple patterns)",
                            x if x < 0.6 => "Medium (1/8 + 1/16 notes)",
                            x if x < 0.8 => "Advanced (1/16 notes, dense)",
                            _ => "Expert (complex rhythms, syncopation)"
                        };
                        ui.label(level_text);
                    });

                    // Audio range information
                    ui.separator();
                    ui.label("Audio Playback Info:");
                    ui.horizontal(|ui| {
                        ui.label("♪ Root: Mid-range (175-350 Hz)");
                        ui.separator();
                        ui.label("♫ Chord: Bass+Mid+Treble (87-1400+ Hz)");
                    });
                    ui.horizontal(|ui| {
                        ui.label("🎵 Melody: Treble range (350-1400+ Hz)");
                        ui.separator();
                        ui.label("🔄 Arp: 2-octave span");
                    });
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
            get_accent_sound(metronome.click_type, &metronome.audio_samples)
        } else {
            // Regular click
            get_sound_params(metronome.click_type, &metronome.audio_samples)
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
        metronome.beat_tracker.record_beat(beat_event.clone());

        // Update melody assistant with beat event if chord progressions are enabled
        if metronome.show_chord_progressions {
            metronome.melody_assistant.update_with_beat(&beat_event);
        }

        drop(metronome);
        // No longer need to drop drum_samples

        let mut engine = self.app_state.engine.lock().unwrap();
        engine.trigger_note_with_volume(waveform, frequency, envelope, volume);
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
            get_sound_params(click_type, &metronome.audio_samples);

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
            metronome.beat_tracker.record_beat(beat_event.clone());

            // Update melody assistant with beat event if chord progressions are enabled
            if metronome.show_chord_progressions {
                metronome.melody_assistant.update_with_beat(&beat_event);
            }
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

            // Melody assistant panel
            MelodyAssistantPanel::show(ui, &self.app_state);

            ui.separator();

            ui.collapsing("Upcoming Features", |ui| {
                ui.label("🎹 Piano chord progressions");
                ui.label("🎸 Bass line accompaniment");
                ui.label("🎵 Key and chord change management");
                ui.label("📚 Practice session recording and playback");
                ui.label("🎯 Tempo trainer with gradual speed changes");
                ui.label("🎛️ Custom pattern creation and editing");
                ui.label("🎹 MIDI input/output synchronization");
                ui.label("⌨️ Customizable keyboard shortcuts");
                ui.label("🎨 Advanced visualization modes");
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
    println!("Advanced Metronome with Pattern Support");
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
