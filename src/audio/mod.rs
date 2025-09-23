/// Audio accent management for dynamic rhythm emphasis.
pub mod accents;
/// CPAL integration and audio stream management for real-time output.
pub mod stream;
/// Audio Processing Module for Polyphonica
///
/// This module provides comprehensive audio processing capabilities for musical
/// applications, handling synthesis, stream management, and real-time audio output.
/// The architecture separates concerns into:
///
/// - **Synthesis**: Audio parameter generation and waveform synthesis
/// - **Stream**: CPAL integration and audio stream management
/// - **Accents**: Specialized accent sound generation for metronomes
///
/// # Design Principles
///
/// 1. **Real-time Safe**: Zero-allocation audio processing in hot paths
/// 2. **Modular**: Clear separation between synthesis, streaming, and effects
/// 3. **Cross-platform**: CPAL abstraction for platform independence
/// 4. **Extensible**: Plugin architecture for custom audio processing
/// 5. **Type Safe**: Strong typing for audio parameters and configurations
///
/// # Usage Example
///
/// ```rust
/// use polyphonica::audio::synthesis::{AudioSynthesis, AudioSampleAdapter, get_sound_params};
/// use polyphonica::audio::stream::AudioStream;
/// use polyphonica::timing::ClickType;
/// use std::sync::{Arc, Mutex};
/// use std::collections::HashMap;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create sample adapter for audio parameter generation
/// let adapter = AudioSampleAdapter::new();
///
/// // Get audio parameters for a click type
/// let (waveform, frequency, envelope) = get_sound_params(ClickType::WoodBlock, &adapter);
///
/// // Create app state for audio stream
/// let engine = Arc::new(Mutex::new(polyphonica::RealtimeEngine::new(44100.0)));
/// let app_state = polyphonica::audio::stream::AppState::new(engine);
///
/// // Setup audio stream for real-time output
/// let stream = AudioStream::setup_audio_stream(app_state)?;
/// # Ok(())
/// # }
/// ```
pub mod synthesis;

// Re-export core types for convenient access
pub use accents::AccentSoundGenerator;
pub use stream::{AudioStream, PolyphonicaStreamConfig};
pub use synthesis::AudioSynthesis;
