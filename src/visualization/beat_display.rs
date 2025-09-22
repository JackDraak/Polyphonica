use crate::patterns::PatternState;
/// Beat display logic extracted from GUI components
///
/// This module contains the core beat visualization logic that was previously
/// embedded in the BeatVisualizationPanel GUI component. It provides a clean,
/// framework-agnostic API for beat visualization state management.
use crate::timing::{BeatTracker, TimeSignature};

/// Visual representation of a single beat
#[derive(Debug, Clone, PartialEq)]
pub struct BeatVisual {
    /// Beat number (1-based)
    pub beat_number: u8,
    /// Whether this beat is currently active/playing
    pub is_current: bool,
    /// Whether this beat is accented
    pub is_accent: bool,
    /// Visual color for this beat
    pub color: BeatColor,
    /// Visual symbol for this beat
    pub symbol: BeatSymbol,
}

/// Color coding for beat visualization
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BeatColor {
    /// Current beat with accent (yellow)
    CurrentAccent,
    /// Current beat without accent (green)
    Current,
    /// Non-current accent beat (light gray)
    Accent,
    /// Regular non-current beat (gray)
    Regular,
}

/// Symbol representation for beats
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BeatSymbol {
    /// Filled circle for accent beats (●)
    FilledCircle,
    /// Empty circle for regular beats (○)
    EmptyCircle,
}

/// Color scheme configuration for beat visualization
#[derive(Debug, Clone)]
pub struct BeatColorScheme {
    pub current_accent: (u8, u8, u8), // RGB for current accent
    pub current: (u8, u8, u8),        // RGB for current beat
    pub accent: (u8, u8, u8),         // RGB for accent beats
    pub regular: (u8, u8, u8),        // RGB for regular beats
}

impl Default for BeatColorScheme {
    fn default() -> Self {
        Self {
            current_accent: (255, 255, 0), // Yellow
            current: (0, 255, 0),          // Green
            accent: (192, 192, 192),       // Light gray
            regular: (128, 128, 128),      // Gray
        }
    }
}

/// Complete visual state for beat display
#[derive(Debug, Clone)]
pub struct BeatVisualState {
    /// Visual representation for each beat
    pub beats: Vec<BeatVisual>,
    /// Current time signature
    pub time_signature: TimeSignature,
    /// Beat interval in milliseconds
    pub beat_interval_ms: f64,
    /// Additional info for pattern mode
    pub pattern_info: Option<PatternDisplayInfo>,
}

/// Additional display information for pattern mode
#[derive(Debug, Clone)]
pub struct PatternDisplayInfo {
    /// Current position within the pattern (0.0-1.0)
    pub pattern_position: f32,
    /// Pattern name
    pub pattern_name: String,
    /// Whether pattern is currently playing
    pub is_playing: bool,
}

/// Visualization mode configuration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VisualizationMode {
    /// Simple metronome mode
    Metronome,
    /// Pattern mode with additional info
    Pattern,
}

/// Main beat display manager
pub struct BeatDisplay {
    /// Current visualization mode
    mode: VisualizationMode,
    /// Current time signature
    time_signature: TimeSignature,
    /// Color scheme
    color_scheme: BeatColorScheme,
    /// Cached visual state
    cached_state: Option<BeatVisualState>,
    /// Whether cache is dirty
    cache_dirty: bool,
}

impl BeatDisplay {
    /// Create new beat display with given time signature
    pub fn new(time_signature: TimeSignature) -> Self {
        Self {
            mode: VisualizationMode::Metronome,
            time_signature,
            color_scheme: BeatColorScheme::default(),
            cached_state: None,
            cache_dirty: true,
        }
    }

    /// Set visualization mode
    pub fn set_mode(&mut self, mode: VisualizationMode) {
        if self.mode != mode {
            self.mode = mode;
            self.cache_dirty = true;
        }
    }

    /// Update time signature
    pub fn set_time_signature(&mut self, time_signature: TimeSignature) {
        if self.time_signature != time_signature {
            self.time_signature = time_signature;
            self.cache_dirty = true;
        }
    }

    /// Set custom color scheme
    pub fn set_color_scheme(&mut self, scheme: BeatColorScheme) {
        self.color_scheme = scheme;
        self.cache_dirty = true;
    }

    /// Generate visual state for current beat configuration
    ///
    /// This is the core logic extracted from BeatVisualizationPanel::show()
    pub fn generate_visual_state(
        &self,
        beat_tracker: &BeatTracker,
        pattern_state: Option<&PatternState>,
        tempo_bpm: f32,
        is_playing: bool,
        accent_first_beat: bool,
    ) -> BeatVisualState {
        // Determine current beat and time signature based on mode
        let (current_beat_display, effective_time_sig) = match self.mode {
            VisualizationMode::Pattern => {
                if let Some(pattern_state) = pattern_state {
                    // For pattern mode, use BeatTracker for last triggered beat
                    let (tracked_beat, _) = beat_tracker.get_current_beat();
                    let pattern_beat = if is_playing && tracked_beat > 0 {
                        tracked_beat
                    } else {
                        1 // Default to beat 1 when not playing or no triggers yet
                    };
                    let time_sig = pattern_state
                        .current_pattern()
                        .map(|p| p.time_signature)
                        .unwrap_or(self.time_signature);
                    (pattern_beat, time_sig)
                } else {
                    (1, self.time_signature)
                }
            }
            VisualizationMode::Metronome => {
                // For metronome mode, use BeatTracker for last triggered beat
                let (tracked_beat, _) = beat_tracker.get_current_beat();
                let metronome_beat = if is_playing && tracked_beat > 0 {
                    tracked_beat
                } else {
                    1 // Fallback to beat 1 if no triggers yet
                };
                (metronome_beat, self.time_signature)
            }
        };

        // Generate beat visuals
        let mut beats = Vec::new();
        for beat in 1..=effective_time_sig.beats_per_measure {
            let is_current = beat == current_beat_display && is_playing;

            // Check accents: use actual triggered accent when showing current beat
            let is_accent = if is_current && is_playing {
                // For current beat, use BeatTracker accent status (coupled with audio)
                let (_, tracked_accent) = beat_tracker.get_current_beat();
                tracked_accent
            } else if self.mode == VisualizationMode::Pattern {
                // For non-current beats in pattern mode, show pattern definition
                if let Some(pattern_state) = pattern_state {
                    if let Some(pattern) = pattern_state.current_pattern() {
                        pattern.beats.iter().any(|pattern_beat| {
                            let beat_num = pattern_beat.beat_position.floor() as u8;
                            beat_num == beat && pattern_beat.accent
                        })
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                // For metronome mode, use metronome setting
                beat == 1 && accent_first_beat
            };

            // Determine color and symbol
            let color = if is_current && is_accent {
                BeatColor::CurrentAccent
            } else if is_current {
                BeatColor::Current
            } else if is_accent {
                BeatColor::Accent
            } else {
                BeatColor::Regular
            };

            let symbol = if is_accent {
                BeatSymbol::FilledCircle
            } else {
                BeatSymbol::EmptyCircle
            };

            beats.push(BeatVisual {
                beat_number: beat,
                is_current,
                is_accent,
                color,
                symbol,
            });
        }

        // Calculate beat interval
        let beat_interval_ms = 60000.0 / tempo_bpm as f64;

        // Generate pattern info if in pattern mode
        let pattern_info = if self.mode == VisualizationMode::Pattern {
            pattern_state.and_then(|ps| {
                ps.current_pattern().map(|pattern| PatternDisplayInfo {
                    pattern_position: ps.current_beat_position(),
                    pattern_name: pattern.name.clone(),
                    is_playing: ps.is_playing(),
                })
            })
        } else {
            None
        };

        BeatVisualState {
            beats,
            time_signature: effective_time_sig,
            beat_interval_ms,
            pattern_info,
        }
    }

    /// Get current visual state (with caching)
    pub fn get_visual_state(
        &mut self,
        beat_tracker: &BeatTracker,
        pattern_state: Option<&PatternState>,
        tempo_bpm: f32,
        is_playing: bool,
        accent_first_beat: bool,
    ) -> &BeatVisualState {
        if self.cache_dirty || self.cached_state.is_none() {
            let state = self.generate_visual_state(
                beat_tracker,
                pattern_state,
                tempo_bpm,
                is_playing,
                accent_first_beat,
            );
            self.cached_state = Some(state);
            self.cache_dirty = false;
        }

        self.cached_state.as_ref().unwrap()
    }

    /// Force cache invalidation (call when beat state changes)
    pub fn invalidate_cache(&mut self) {
        self.cache_dirty = true;
    }

    /// Get RGB color values for a beat color
    pub fn get_color_rgb(&self, color: BeatColor) -> (u8, u8, u8) {
        match color {
            BeatColor::CurrentAccent => self.color_scheme.current_accent,
            BeatColor::Current => self.color_scheme.current,
            BeatColor::Accent => self.color_scheme.accent,
            BeatColor::Regular => self.color_scheme.regular,
        }
    }

    /// Get symbol character for a beat symbol
    pub fn get_symbol_char(&self, symbol: BeatSymbol) -> char {
        match symbol {
            BeatSymbol::FilledCircle => '●',
            BeatSymbol::EmptyCircle => '○',
        }
    }
}

/// Legacy compatibility for transitioning from GUI component
///
/// This provides similar functionality to the old BeatVisualizationPanel::show()
/// method to ease the transition to the modular visualization system.
pub fn get_legacy_beat_visuals(
    beat_tracker: &BeatTracker,
    pattern_state: Option<&PatternState>,
    time_signature: TimeSignature,
    tempo_bpm: f32,
    is_playing: bool,
    accent_first_beat: bool,
    pattern_mode: bool,
) -> Vec<(u8, bool, bool, (u8, u8, u8), char)> {
    let mut display = BeatDisplay::new(time_signature);
    display.set_mode(if pattern_mode {
        VisualizationMode::Pattern
    } else {
        VisualizationMode::Metronome
    });

    let state = display.generate_visual_state(
        beat_tracker,
        pattern_state,
        tempo_bpm,
        is_playing,
        accent_first_beat,
    );

    state
        .beats
        .into_iter()
        .map(|beat| {
            let color_rgb = display.get_color_rgb(beat.color);
            let symbol_char = display.get_symbol_char(beat.symbol);
            (
                beat.beat_number,
                beat.is_current,
                beat.is_accent,
                color_rgb,
                symbol_char,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timing::BeatTracker;

    #[test]
    fn test_beat_display_creation() {
        let time_sig = TimeSignature::new(4, 4);
        let display = BeatDisplay::new(time_sig);
        assert_eq!(display.time_signature, time_sig);
        assert_eq!(display.mode, VisualizationMode::Metronome);
    }

    #[test]
    fn test_mode_switching() {
        let mut display = BeatDisplay::new(TimeSignature::new(4, 4));
        assert_eq!(display.mode, VisualizationMode::Metronome);

        display.set_mode(VisualizationMode::Pattern);
        assert_eq!(display.mode, VisualizationMode::Pattern);
    }

    #[test]
    fn test_color_scheme_defaults() {
        let scheme = BeatColorScheme::default();
        assert_eq!(scheme.current_accent, (255, 255, 0)); // Yellow
        assert_eq!(scheme.current, (0, 255, 0)); // Green
    }

    #[test]
    fn test_symbol_representation() {
        let display = BeatDisplay::new(TimeSignature::new(4, 4));
        assert_eq!(display.get_symbol_char(BeatSymbol::FilledCircle), '●');
        assert_eq!(display.get_symbol_char(BeatSymbol::EmptyCircle), '○');
    }

    #[test]
    fn test_visual_state_generation() {
        let display = BeatDisplay::new(TimeSignature::new(4, 4));
        let beat_tracker = BeatTracker::new();

        let state = display.generate_visual_state(&beat_tracker, None, 120.0, false, true);

        assert_eq!(state.beats.len(), 4);
        assert_eq!(state.time_signature.beats_per_measure, 4);
        assert_eq!(state.beat_interval_ms, 500.0); // 60000 / 120
    }
}
