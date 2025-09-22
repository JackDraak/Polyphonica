/// Sample loading and caching with intelligent memory management
///
/// This module provides efficient sample loading with caching, supporting
/// multiple file formats and intelligent memory management for real-time
/// audio applications.

use std::collections::HashMap;
use std::path::Path;
use crate::SampleData;

/// High-level sample library with intelligent caching
///
/// The SampleLibrary manages sample loading, caching, and memory management.
/// It provides lazy loading to minimize memory usage and intelligent caching
/// with LRU eviction when memory limits are approached.
///
/// # Design Features
///
/// - **Lazy Loading**: Samples are loaded only when first requested
/// - **LRU Caching**: Least recently used samples are evicted when memory fills
/// - **Format Support**: Supports WAV, FLAC, and other audio formats
/// - **Path Resolution**: Intelligent path resolution with fallbacks
/// - **Memory Limits**: Configurable memory limits to prevent OOM issues
///
/// # Memory Management
///
/// The library tracks memory usage and automatically evicts least recently
/// used samples when configured limits are approached. This ensures reliable
/// operation even with large sample libraries.
pub struct SampleLibrary {
    /// Loaded samples cache
    cache: HashMap<String, CachedSample>,

    /// Maximum memory usage in bytes (0 = unlimited)
    max_memory_bytes: usize,

    /// Current memory usage in bytes
    current_memory_bytes: usize,

    /// Sample search paths
    search_paths: Vec<String>,
}

/// Cached sample with metadata for LRU eviction
#[derive(Debug, Clone)]
struct CachedSample {
    /// The actual sample data
    data: SampleData,

    /// Memory usage in bytes
    memory_bytes: usize,

    /// Access counter for LRU tracking
    access_count: u64,

    /// Last access time for LRU tracking
    last_access: std::time::Instant,
}

impl SampleLibrary {
    /// Create a new sample library
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            max_memory_bytes: 0, // Unlimited by default
            current_memory_bytes: 0,
            search_paths: vec![
                "samples/".to_string(),
                "assets/samples/".to_string(),
                "./".to_string(),
            ],
        }
    }

    /// Create a new sample library with memory limit
    pub fn with_memory_limit(max_memory_mb: usize) -> Self {
        let mut library = Self::new();
        library.max_memory_bytes = max_memory_mb * 1024 * 1024;
        library
    }

    /// Add a search path for samples
    pub fn add_search_path<P: AsRef<Path>>(&mut self, path: P) {
        self.search_paths.push(path.as_ref().to_string_lossy().to_string());
    }

    /// Load a sample by name
    ///
    /// Attempts to load the sample from the cache first, then from disk
    /// if not cached. Supports automatic path resolution with fallbacks.
    pub fn load_sample(&mut self, name: &str, base_frequency: f32) -> Result<SampleData, SampleError> {
        // Check cache first
        if let Some(cached) = self.cache.get_mut(name) {
            // Update LRU tracking
            cached.access_count += 1;
            cached.last_access = std::time::Instant::now();
            return Ok(cached.data.clone());
        }

        // Try to load from disk
        let sample_data = self.load_from_disk(name, base_frequency)?;
        let memory_bytes = self.estimate_memory_usage(&sample_data);

        // Check if we need to evict samples to make room
        self.ensure_memory_available(memory_bytes)?;

        // Cache the sample
        let cached = CachedSample {
            data: sample_data.clone(),
            memory_bytes,
            access_count: 1,
            last_access: std::time::Instant::now(),
        };

        self.current_memory_bytes += memory_bytes;
        self.cache.insert(name.to_string(), cached);

        Ok(sample_data)
    }

    /// Load a sample from a specific file path
    pub fn load_sample_from_path<P: AsRef<Path>>(&mut self, name: &str, path: P, base_frequency: f32) -> Result<SampleData, SampleError> {
        // Check cache first
        if let Some(cached) = self.cache.get_mut(name) {
            cached.access_count += 1;
            cached.last_access = std::time::Instant::now();
            return Ok(cached.data.clone());
        }

        // Load from specific path
        let sample_data = SampleData::from_file(path, base_frequency)
            .map_err(|e| SampleError::LoadError(format!("Failed to load {}: {}", name, e)))?;

        let memory_bytes = self.estimate_memory_usage(&sample_data);
        self.ensure_memory_available(memory_bytes)?;

        let cached = CachedSample {
            data: sample_data.clone(),
            memory_bytes,
            access_count: 1,
            last_access: std::time::Instant::now(),
        };

        self.current_memory_bytes += memory_bytes;
        self.cache.insert(name.to_string(), cached);

        Ok(sample_data)
    }

    /// Check if a sample is loaded in cache
    pub fn is_loaded(&self, name: &str) -> bool {
        self.cache.contains_key(name)
    }

    /// Get memory usage statistics
    pub fn memory_stats(&self) -> MemoryStats {
        MemoryStats {
            current_bytes: self.current_memory_bytes,
            max_bytes: self.max_memory_bytes,
            cached_samples: self.cache.len(),
        }
    }

    /// Clear all cached samples
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.current_memory_bytes = 0;
    }

    /// Load sample from disk with path resolution
    fn load_from_disk(&self, name: &str, base_frequency: f32) -> Result<SampleData, SampleError> {
        // Try different file extensions
        let extensions = ["wav", "flac", "ogg"];

        for search_path in &self.search_paths {
            for ext in &extensions {
                let full_path = format!("{}/{}.{}", search_path, name, ext);
                if Path::new(&full_path).exists() {
                    return SampleData::from_file(&full_path, base_frequency)
                        .map_err(|e| SampleError::LoadError(format!("Failed to load {}: {}", full_path, e)));
                }
            }
        }

        Err(SampleError::NotFound(name.to_string()))
    }

    /// Estimate memory usage of a sample
    fn estimate_memory_usage(&self, sample: &SampleData) -> usize {
        // Rough estimate: sample data + overhead
        sample.samples.len() * std::mem::size_of::<f32>() + 1024
    }

    /// Ensure enough memory is available, evicting LRU samples if needed
    fn ensure_memory_available(&mut self, needed_bytes: usize) -> Result<(), SampleError> {
        if self.max_memory_bytes == 0 {
            return Ok(()); // No limit
        }

        while self.current_memory_bytes + needed_bytes > self.max_memory_bytes {
            if !self.evict_lru_sample()? {
                return Err(SampleError::OutOfMemory(needed_bytes));
            }
        }

        Ok(())
    }

    /// Evict the least recently used sample
    fn evict_lru_sample(&mut self) -> Result<bool, SampleError> {
        if self.cache.is_empty() {
            return Ok(false);
        }

        // Find LRU sample
        let mut lru_name = String::new();
        let mut lru_time = std::time::Instant::now();

        for (name, cached) in &self.cache {
            if cached.last_access < lru_time {
                lru_time = cached.last_access;
                lru_name = name.clone();
            }
        }

        // Remove LRU sample
        if let Some(cached) = self.cache.remove(&lru_name) {
            self.current_memory_bytes -= cached.memory_bytes;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl Default for SampleLibrary {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub current_bytes: usize,
    pub max_bytes: usize,
    pub cached_samples: usize,
}

/// Sample loading errors
#[derive(Debug, Clone)]
pub enum SampleError {
    /// Sample file not found
    NotFound(String),

    /// Failed to load sample file
    LoadError(String),

    /// Out of memory
    OutOfMemory(usize),

    /// Invalid sample format
    InvalidFormat(String),
}

impl std::fmt::Display for SampleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SampleError::NotFound(name) => write!(f, "Sample not found: {}", name),
            SampleError::LoadError(msg) => write!(f, "Load error: {}", msg),
            SampleError::OutOfMemory(bytes) => write!(f, "Out of memory (needed {} bytes)", bytes),
            SampleError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
        }
    }
}

impl std::error::Error for SampleError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_library_creation() {
        let library = SampleLibrary::new();
        assert_eq!(library.cache.len(), 0);
        assert_eq!(library.current_memory_bytes, 0);
        assert_eq!(library.max_memory_bytes, 0);
    }

    #[test]
    fn test_memory_limit() {
        let library = SampleLibrary::with_memory_limit(100); // 100MB
        assert_eq!(library.max_memory_bytes, 100 * 1024 * 1024);
    }

    #[test]
    fn test_search_path_management() {
        let mut library = SampleLibrary::new();
        library.add_search_path("custom/path");
        assert!(library.search_paths.contains(&"custom/path".to_string()));
    }

    #[test]
    fn test_memory_stats() {
        let library = SampleLibrary::new();
        let stats = library.memory_stats();
        assert_eq!(stats.current_bytes, 0);
        assert_eq!(stats.cached_samples, 0);
    }
}