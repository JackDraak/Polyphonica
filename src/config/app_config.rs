/// Application configuration structures and management
///
/// This module contains the configuration structures extracted from the
/// scattered settings throughout guitar_buddy.rs, providing a centralized
/// and type-safe configuration management system.
use crate::timing::{ClickType, TimeSignature};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Metronome-specific settings
    pub metronome: MetronomeConfig,
    /// Audio system settings
    pub audio: AudioConfig,
    /// Pattern playback settings
    pub pattern: PatternConfig,
    /// User interface settings
    pub ui: UiConfig,
    /// Configuration file version for backwards compatibility
    #[serde(default = "default_config_version")]
    pub version: String,
}

/// Metronome configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetronomeConfig {
    /// Tempo in beats per minute (40-200)
    #[serde(default = "default_tempo")]
    pub tempo_bpm: f32,
    /// Time signature
    #[serde(default = "default_time_signature")]
    pub time_signature: TimeSignature,
    /// Click sound type
    #[serde(default = "default_click_type")]
    pub click_type: ClickType,
    /// Whether to accent the first beat
    #[serde(default = "default_accent_first_beat")]
    pub accent_first_beat: bool,
    /// Metronome volume (0.0-1.0)
    #[serde(default = "default_volume")]
    pub volume: f32,
}

/// Audio system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Master audio volume (0.0-1.0)
    #[serde(default = "default_master_volume")]
    pub master_volume: f32,
    /// Preferred audio device name (None = default device)
    #[serde(default)]
    pub preferred_device: Option<String>,
    /// Audio buffer size (None = default)
    #[serde(default)]
    pub buffer_size: Option<u32>,
    /// Sample rate preference (None = device default)
    #[serde(default)]
    pub sample_rate: Option<u32>,
}

/// Pattern playback configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternConfig {
    /// Whether pattern mode is enabled
    #[serde(default)]
    pub pattern_mode: bool,
    /// Last selected pattern name
    #[serde(default)]
    pub last_pattern: Option<String>,
    /// Pattern volume adjustment (-1.0 to 1.0)
    #[serde(default)]
    pub pattern_volume_adjustment: f32,
    /// Whether to loop patterns
    #[serde(default = "default_loop_patterns")]
    pub loop_patterns: bool,
}

/// User interface configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Window width in pixels
    #[serde(default = "default_window_width")]
    pub window_width: f32,
    /// Window height in pixels
    #[serde(default = "default_window_height")]
    pub window_height: f32,
    /// Whether window is maximized
    #[serde(default)]
    pub window_maximized: bool,
    /// UI theme name
    #[serde(default = "default_theme")]
    pub theme: String,
    /// Beat visualization color scheme
    #[serde(default = "default_color_scheme")]
    pub beat_color_scheme: String,
}

/// Configuration error types
#[derive(Debug)]
pub enum ConfigError {
    /// IO error during file operations
    IoError(std::io::Error),
    /// Serialization/deserialization error
    SerdeError(toml::de::Error),
    /// Validation error
    ValidationError(String),
    /// Path error
    PathError(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "IO error: {}", e),
            ConfigError::SerdeError(e) => write!(f, "Serialization error: {}", e),
            ConfigError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            ConfigError::PathError(msg) => write!(f, "Path error: {}", msg),
        }
    }
}

impl Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(error: std::io::Error) -> Self {
        ConfigError::IoError(error)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(error: toml::de::Error) -> Self {
        ConfigError::SerdeError(error)
    }
}

impl AppConfig {
    /// Load configuration from file or create default if file doesn't exist
    pub fn load_or_default() -> Result<Self, ConfigError> {
        let config_path = Self::get_config_path()?;

        if config_path.exists() {
            Self::load_from_file(&config_path)
        } else {
            Ok(Self::default())
        }
    }

    /// Load configuration from specific file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Save configuration to default location
    pub fn save(&self) -> Result<(), ConfigError> {
        let config_path = Self::get_config_path()?;
        self.save_to_file(&config_path)
    }

    /// Save configuration to specific file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        // Validate before saving
        self.validate()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::ValidationError(e.to_string()))?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Validate configuration parameters
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate tempo
        if self.metronome.tempo_bpm < 40.0 || self.metronome.tempo_bpm > 200.0 {
            return Err(ConfigError::ValidationError(format!(
                "Tempo {} BPM is out of range (40-200)",
                self.metronome.tempo_bpm
            )));
        }

        // Validate volumes
        if self.metronome.volume < 0.0 || self.metronome.volume > 1.0 {
            return Err(ConfigError::ValidationError(format!(
                "Metronome volume {} is out of range (0.0-1.0)",
                self.metronome.volume
            )));
        }

        if self.audio.master_volume < 0.0 || self.audio.master_volume > 1.0 {
            return Err(ConfigError::ValidationError(format!(
                "Master volume {} is out of range (0.0-1.0)",
                self.audio.master_volume
            )));
        }

        // Validate pattern volume adjustment
        if self.pattern.pattern_volume_adjustment < -1.0
            || self.pattern.pattern_volume_adjustment > 1.0
        {
            return Err(ConfigError::ValidationError(format!(
                "Pattern volume adjustment {} is out of range (-1.0-1.0)",
                self.pattern.pattern_volume_adjustment
            )));
        }

        // Validate window dimensions
        if self.ui.window_width < 400.0 || self.ui.window_width > 3840.0 {
            return Err(ConfigError::ValidationError(format!(
                "Window width {} is out of range (400-3840)",
                self.ui.window_width
            )));
        }

        if self.ui.window_height < 300.0 || self.ui.window_height > 2160.0 {
            return Err(ConfigError::ValidationError(format!(
                "Window height {} is out of range (300-2160)",
                self.ui.window_height
            )));
        }

        Ok(())
    }

    /// Get configuration file path
    fn get_config_path() -> Result<PathBuf, ConfigError> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| ConfigError::PathError("Could not find config directory".to_string()))?;

        let app_config_dir = config_dir.join("polyphonica");
        Ok(app_config_dir.join("config.toml"))
    }

    /// Reset to default values
    pub fn reset_to_defaults(&mut self) {
        *self = Self::default();
    }

    /// Update from legacy MetronomeState (for migration)
    pub fn update_from_legacy_metronome_state(
        &mut self,
        tempo_bpm: f32,
        time_signature: TimeSignature,
        click_type: ClickType,
        accent_first_beat: bool,
        volume: f32,
        pattern_mode: bool,
    ) {
        self.metronome.tempo_bpm = tempo_bpm;
        self.metronome.time_signature = time_signature;
        self.metronome.click_type = click_type;
        self.metronome.accent_first_beat = accent_first_beat;
        self.metronome.volume = volume;
        self.pattern.pattern_mode = pattern_mode;
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            metronome: MetronomeConfig::default(),
            audio: AudioConfig::default(),
            pattern: PatternConfig::default(),
            ui: UiConfig::default(),
            version: default_config_version(),
        }
    }
}

impl Default for MetronomeConfig {
    fn default() -> Self {
        Self {
            tempo_bpm: default_tempo(),
            time_signature: default_time_signature(),
            click_type: default_click_type(),
            accent_first_beat: default_accent_first_beat(),
            volume: default_volume(),
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: default_master_volume(),
            preferred_device: None,
            buffer_size: None,
            sample_rate: None,
        }
    }
}

impl Default for PatternConfig {
    fn default() -> Self {
        Self {
            pattern_mode: false,
            last_pattern: None,
            pattern_volume_adjustment: 0.0,
            loop_patterns: default_loop_patterns(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            window_width: default_window_width(),
            window_height: default_window_height(),
            window_maximized: false,
            theme: default_theme(),
            beat_color_scheme: default_color_scheme(),
        }
    }
}

// Default value functions for serde
fn default_config_version() -> String {
    "1.0".to_string()
}
fn default_tempo() -> f32 {
    120.0
}
fn default_time_signature() -> TimeSignature {
    TimeSignature::new(4, 4)
}
fn default_click_type() -> ClickType {
    ClickType::WoodBlock
}
fn default_accent_first_beat() -> bool {
    true
}
fn default_volume() -> f32 {
    0.7
}
fn default_master_volume() -> f32 {
    1.0
}
fn default_loop_patterns() -> bool {
    true
}
fn default_window_width() -> f32 {
    700.0
}
fn default_window_height() -> f32 {
    500.0
}
fn default_theme() -> String {
    "default".to_string()
}
fn default_color_scheme() -> String {
    "default".to_string()
}

/// Configuration manager for easy access to settings
pub struct ConfigManager {
    config: AppConfig,
    auto_save: bool,
}

impl ConfigManager {
    /// Create new configuration manager
    pub fn new(auto_save: bool) -> Result<Self, ConfigError> {
        let config = AppConfig::load_or_default()?;
        Ok(Self { config, auto_save })
    }

    /// Get reference to configuration
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// Get mutable reference to configuration
    pub fn config_mut(&mut self) -> &mut AppConfig {
        &mut self.config
    }

    /// Save configuration if auto-save is enabled
    pub fn maybe_save(&self) -> Result<(), ConfigError> {
        if self.auto_save {
            self.config.save()
        } else {
            Ok(())
        }
    }

    /// Force save configuration
    pub fn save(&self) -> Result<(), ConfigError> {
        self.config.save()
    }

    /// Set metronome tempo with validation and optional auto-save
    pub fn set_tempo(&mut self, tempo_bpm: f32) -> Result<(), ConfigError> {
        if !(40.0..=200.0).contains(&tempo_bpm) {
            return Err(ConfigError::ValidationError(format!(
                "Tempo {} BPM is out of range (40-200)",
                tempo_bpm
            )));
        }
        self.config.metronome.tempo_bpm = tempo_bpm;
        self.maybe_save()
    }

    /// Set metronome volume with validation and optional auto-save
    pub fn set_volume(&mut self, volume: f32) -> Result<(), ConfigError> {
        if !(0.0..=1.0).contains(&volume) {
            return Err(ConfigError::ValidationError(format!(
                "Volume {} is out of range (0.0-1.0)",
                volume
            )));
        }
        self.config.metronome.volume = volume;
        self.maybe_save()
    }

    /// Set click type and optional auto-save
    pub fn set_click_type(&mut self, click_type: ClickType) -> Result<(), ConfigError> {
        self.config.metronome.click_type = click_type;
        self.maybe_save()
    }

    /// Set time signature and optional auto-save
    pub fn set_time_signature(&mut self, time_signature: TimeSignature) -> Result<(), ConfigError> {
        self.config.metronome.time_signature = time_signature;
        self.maybe_save()
    }

    /// Toggle pattern mode and optional auto-save
    pub fn toggle_pattern_mode(&mut self) -> Result<bool, ConfigError> {
        self.config.pattern.pattern_mode = !self.config.pattern.pattern_mode;
        self.maybe_save()?;
        Ok(self.config.pattern.pattern_mode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_config_creation() {
        let config = AppConfig::default();
        assert_eq!(config.metronome.tempo_bpm, 120.0);
        assert_eq!(config.metronome.time_signature.beats_per_measure, 4);
        assert_eq!(config.metronome.volume, 0.7);
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();

        // Valid config should pass
        assert!(config.validate().is_ok());

        // Invalid tempo should fail
        config.metronome.tempo_bpm = 300.0;
        assert!(config.validate().is_err());

        // Reset and try invalid volume
        config.metronome.tempo_bpm = 120.0;
        config.metronome.volume = 2.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_save_load() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        let mut original_config = AppConfig::default();
        original_config.metronome.tempo_bpm = 140.0;
        original_config.metronome.click_type = ClickType::Cowbell;

        // Save config
        assert!(original_config.save_to_file(&config_path).is_ok());

        // Load config
        let loaded_config = AppConfig::load_from_file(&config_path).unwrap();
        assert_eq!(loaded_config.metronome.tempo_bpm, 140.0);
        assert_eq!(loaded_config.metronome.click_type, ClickType::Cowbell);
    }

    #[test]
    fn test_config_manager() {
        let mut manager = ConfigManager::new(false).unwrap();

        // Test tempo setting with validation
        assert!(manager.set_tempo(140.0).is_ok());
        assert_eq!(manager.config().metronome.tempo_bpm, 140.0);

        // Test invalid tempo
        assert!(manager.set_tempo(300.0).is_err());
        assert_eq!(manager.config().metronome.tempo_bpm, 140.0); // Should remain unchanged
    }

    #[test]
    fn test_legacy_migration() {
        let mut config = AppConfig::default();

        config.update_from_legacy_metronome_state(
            150.0,
            TimeSignature::new(3, 4),
            ClickType::DigitalBeep,
            false,
            0.8,
            true,
        );

        assert_eq!(config.metronome.tempo_bpm, 150.0);
        assert_eq!(config.metronome.time_signature.beats_per_measure, 3);
        assert_eq!(config.metronome.click_type, ClickType::DigitalBeep);
        assert_eq!(config.metronome.accent_first_beat, false);
        assert_eq!(config.metronome.volume, 0.8);
        assert_eq!(config.pattern.pattern_mode, true);
    }
}
