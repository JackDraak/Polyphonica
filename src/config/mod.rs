/// Configuration management module for Polyphonica
///
/// This module provides centralized configuration management for musical
/// applications, handling settings persistence, validation, and type-safe
/// access to application parameters. The architecture separates concerns:
///
/// - **AppConfig**: Main application configuration structure
/// - **Persistence**: Configuration loading and saving
/// - **Validation**: Parameter validation and bounds checking
/// - **Defaults**: Sensible default values for all settings
///
/// # Design Principles
///
/// 1. **Type Safety**: Strong typing for all configuration parameters
/// 2. **Validation**: Automatic bounds checking and parameter validation
/// 3. **Persistence**: Automatic loading/saving of configuration
/// 4. **Extensible**: Easy to add new configuration parameters
/// 5. **Backwards Compatible**: Graceful handling of old configuration files
///
/// # Usage Example
///
/// ```rust
/// use polyphonica::config::{AppConfig, MetronomeConfig};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Load configuration from file or use defaults
/// let mut config = AppConfig::load_or_default()?;
///
/// // Modify settings
/// config.metronome.tempo_bpm = 140.0;
/// config.metronome.volume = 0.8;
///
/// // Save configuration
/// config.save()?;
/// # Ok(())
/// # }
/// ```
pub mod app_config;

// Re-export core types for convenient access
pub use app_config::{
    AppConfig, AudioConfig, ConfigError, MetronomeConfig, PatternConfig, UiConfig,
};
