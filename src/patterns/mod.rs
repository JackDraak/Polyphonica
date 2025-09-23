/// Pattern builder for creating drum patterns with fluent API and notation support.
pub mod builder;
/// Pre-defined pattern collections organized by musical genre.
pub mod collections;
/// JSON import/export functionality for pattern catalogs.
pub mod io;
/// Pattern library management and factory system.
pub mod library;
/// Real-time pattern state management for playback.
pub mod state;
/// Pattern library module for rhythm and drum patterns
///
/// This module provides a comprehensive pattern system for creating and managing
/// drum patterns, rhythms, and musical arrangements. It includes predefined
/// patterns for various genres as well as tools for creating custom patterns.
///
/// # Module Structure
///
/// - `types`: Core pattern data structures and types
/// - `library`: Pattern definitions and collections
/// - `builder`: Tools for creating custom patterns
/// - `state`: Pattern playback state management
/// - `collections`: Genre-specific pattern collections
/// - `io`: JSON import/export functionality
pub mod types;

// Re-export commonly used types
pub use builder::PatternBuilder;
pub use collections::*;
pub use io::{PatternCatalog, PatternIoError};
pub use library::PatternLibrary;
pub use state::PatternState;
pub use types::{DrumPattern, DrumPatternBeat, PatternMetadata};
