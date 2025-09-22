/// Sample management subsystem for Polyphonica
///
/// This module provides comprehensive sample loading, caching, and playback
/// management for musical applications. The architecture separates concerns:
///
/// - **Library**: Sample loading and caching with efficient memory management
/// - **Manager**: Real-time sample playback and triggering
/// - **Catalog**: Sample metadata and configuration management
/// - **DrumKit**: Specialized drum sample collections and kits
///
/// # Design Principles
///
/// 1. **Lazy Loading**: Samples loaded on-demand to minimize memory usage
/// 2. **Caching**: Intelligent caching with LRU eviction for memory efficiency
/// 3. **Real-time Safe**: Zero-allocation playback path for real-time audio
/// 4. **Modular**: Support for multiple sample formats and sources
/// 5. **Extensible**: Plugin architecture for custom sample sources
///
/// # Usage Example
///
/// ```rust
/// use polyphonica::samples::{SampleLibrary, SampleManager};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut library = SampleLibrary::new();
/// // Load individual samples
/// // library.load_sample_from_path("kick", "samples/kick.wav", 60.0)?;
///
/// let manager = SampleManager::new(library);
/// // Get trigger for real-time use
/// if let Some(trigger) = manager.get_trigger("kick") {
///     // Use trigger in audio callback
/// }
/// # Ok(())
/// # }
/// ```

pub mod library;
pub mod manager;
pub mod catalog;
pub mod drumkit;

// Re-export core types for convenient access
pub use library::SampleLibrary;
pub use manager::SampleManager;
pub use catalog::{SampleCatalog, SampleMetadata};
pub use drumkit::{DrumKit, DrumSample};