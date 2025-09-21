# Sample-Based Synthesis Development Plan 🎵

## Overview

Extend Polyphonica to support custom audio samples (WAV files) as waveform sources, enabling hybrid synthesis combining mathematical waveforms with recorded audio. This transforms the library from pure synthesis to a versatile audio engine supporting drums, instruments, and creative sound design.

## Goals

### Primary Objectives
- **Sample Integration**: Load WAV files as playable waveforms
- **Pitch Shifting**: Play samples at different frequencies with resampling
- **Seamless API**: Maintain existing API compatibility
- **Performance**: Efficient sample playback and memory management

### Secondary Objectives
- **Loop Support**: Sustain samples with loop points
- **Multi-sampling**: Multiple samples across frequency ranges
- **Test Tool Enhancement**: CLI commands for sample testing
- **Advanced Features**: Velocity layers, crossfading

## Technical Architecture

### Core Data Structures

```rust
pub enum Waveform {
    Sine,
    Square,
    Sawtooth,
    Triangle,
    Sample(SampleData),  // 🆕 New variant
}

pub struct SampleData {
    samples: Vec<f32>,           // Audio data
    sample_rate: u32,            // Original sample rate
    base_frequency: f32,         // Fundamental frequency
    loop_start: Option<usize>,   // Loop start point
    loop_end: Option<usize>,     // Loop end point
    metadata: SampleMetadata,    // File info
}

pub struct SampleMetadata {
    filename: String,
    duration_secs: f32,
    channels: u16,
    bits_per_sample: u16,
}
```

### Sample Loading System

```rust
impl SampleData {
    pub fn from_file(path: &str, base_frequency: f32) -> Result<Self, SampleError>;
    pub fn from_bytes(data: &[u8], base_frequency: f32) -> Result<Self, SampleError>;
    pub fn with_loop_points(self, start: usize, end: usize) -> Self;
}
```

### Pitch Shifting Algorithm

- **Linear Interpolation**: Basic resampling for pitch adjustment
- **Frequency Ratio**: `target_freq / base_freq = playback_speed`
- **Sample Index Mapping**: Fractional indexing with interpolation
- **Edge Cases**: Handle sample boundaries and loop points

## Implementation Phases

### Phase 1: Foundation (Core Library)
**Estimated Time**: 2-3 hours
**Files**: `src/lib.rs`

1. **Extend Waveform Enum**
   - Add `Sample(SampleData)` variant
   - Implement Debug, Clone, PartialEq traits

2. **Create SampleData Structure**
   - Basic data container
   - File loading functionality
   - Error handling types

3. **Add WAV Loading Dependency**
   - Use `hound` crate (already available)
   - Handle mono/stereo conversion
   - Sample rate validation

4. **Integrate with generate_sample()**
   - Modify core synthesis function
   - Add sample playback logic
   - Implement basic pitch shifting

### Phase 2: Advanced Synthesis (Core Library)
**Estimated Time**: 2-3 hours
**Files**: `src/lib.rs`

1. **Pitch Shifting Implementation**
   - Linear interpolation resampling
   - Fractional sample indexing
   - Frequency ratio calculations

2. **Loop Point Support**
   - Sustain sample playback
   - Seamless loop transitions
   - ADSR envelope integration

3. **Performance Optimization**
   - Sample caching strategies
   - Memory-efficient playback
   - Reduce allocation overhead

4. **Comprehensive Testing**
   - Unit tests for sample loading
   - Pitch shifting accuracy tests
   - Loop point validation

### Phase 3: Test Tool Enhancement
**Estimated Time**: 2-3 hours
**Files**: `src/bin/test_tool.rs`, `README_TEST_TOOL.md`

1. **Sample Command**
   ```bash
   cargo run --bin polyphonica-test sample drum.wav --base-freq 440 --play
   ```

2. **Sample-Event Command**
   ```bash
   cargo run --bin polyphonica-test sample-event drum.wav \
     --base-freq 440 --target-freq 220 --envelope piano --play
   ```

3. **Drum Pattern Generator**
   ```bash
   cargo run --bin polyphonica-test drum-pattern \
     --samples kick.wav,snare.wav,hihat.wav \
     --pattern "K.S.K.S." --tempo 120 --play
   ```

4. **Sample Analysis Tools**
   - Display sample metadata
   - Frequency analysis
   - Loop point detection

### Phase 4: Advanced Features (Optional)
**Estimated Time**: 3-4 hours
**Files**: Various

1. **Multi-sampling Support**
   - Multiple samples per frequency range
   - Automatic sample selection
   - Crossfading between samples

2. **Velocity Layers**
   - Multiple samples per velocity
   - Dynamic sample selection
   - Realistic instrument response

3. **Sample Processing**
   - Built-in effects (reverb, filtering)
   - Granular synthesis options
   - Time-stretching algorithms

4. **Advanced Test Tool Features**
   - Sample library management
   - Batch processing commands
   - Performance benchmarking

## API Design Examples

### Basic Sample Usage
```rust
// Load a TomTom sample recorded at middle C
let tomtom = SampleData::from_file("tomtom_c4.wav", 261.63)?;

// Play it one octave lower
let low_tom_event = SoundEvent {
    waveform: Waveform::Sample(tomtom),
    start_frequency: 130.81,  // C3
    end_frequency: 130.81,    // No pitch bend
    duration_secs: 1.5,
    envelope: AdsrEnvelope {
        attack_secs: 0.01,
        decay_secs: 0.3,
        sustain_level: 0.0,  // Drum-like: no sustain
        release_secs: 0.0,
    },
};
```

### Hybrid Synthesis
```rust
// Mix samples with mathematical waveforms
let hybrid_timeline = &[
    (0.0, SoundEvent { waveform: Waveform::Sample(kick), ... }),
    (0.0, SoundEvent { waveform: Waveform::Sine, frequency: 60.0, ... }),
    (0.5, SoundEvent { waveform: Waveform::Sample(snare), ... }),
    (0.5, SoundEvent { waveform: Waveform::Square, frequency: 200.0, ... }),
];
```

### Loop-based Instruments
```rust
// Sustained sample with loop points
let piano_c4 = SampleData::from_file("piano_c4.wav", 261.63)?
    .with_loop_points(44100, 88200);  // Loop from 1s to 2s

let sustained_note = SoundEvent {
    waveform: Waveform::Sample(piano_c4),
    start_frequency: 261.63,
    end_frequency: 261.63,
    duration_secs: 5.0,  // Play for 5 seconds using loops
    envelope: organ_envelope,  // Sustained envelope
};
```

## File Structure Changes

```
polyphonica/
├── src/
│   ├── lib.rs                    # Extended with sample support
│   ├── sample.rs                 # New: Sample loading/processing
│   └── bin/
│       └── test_tool.rs          # Enhanced with sample commands
├── tests/
│   └── sample_tests.rs           # New: Sample-specific tests
├── examples/
│   ├── drum_machine.rs           # New: Drum pattern example
│   └── hybrid_synthesis.rs       # New: Mixed synthesis example
├── samples/                      # New: Test sample files
│   ├── kick.wav
│   ├── snare.wav
│   └── tomtom.wav
└── docs/
    ├── SAMPLE_SYNTHESIS.md       # New: Sample usage guide
    └── SAMPLE_SYNTHESIS_PLAN.md  # This document
```

## Dependencies

### New Dependencies
```toml
[dependencies]
hound = "3.5"           # WAV file loading (already available)
# Optional for advanced features:
# rubato = "0.12"       # High-quality resampling
# dasp = "0.11"         # Digital audio signal processing
```

### Test Tool Dependencies (Already Available)
- `clap` - CLI argument parsing
- `rodio` - Audio playback
- `serde` - Sample metadata serialization

## Testing Strategy

### Unit Tests
1. **Sample Loading**
   - Valid WAV file parsing
   - Error handling for invalid files
   - Metadata extraction accuracy

2. **Pitch Shifting**
   - Frequency ratio calculations
   - Interpolation accuracy
   - Edge case handling

3. **Integration**
   - Sample + envelope combination
   - Timeline mixing with samples
   - Performance benchmarking

### Integration Tests
1. **Real-world Samples**
   - Various drum sounds
   - Instrument recordings
   - Different sample rates/formats

2. **Use Case Validation**
   - Drum machine patterns
   - Melodic instrument playback
   - Hybrid synthesis scenarios

## Risk Mitigation

### Technical Risks
1. **Memory Usage**: Large samples could impact performance
   - **Mitigation**: Lazy loading, sample streaming

2. **Pitch Shifting Quality**: Simple interpolation may sound poor
   - **Mitigation**: Implement higher-quality algorithms if needed

3. **File Format Support**: Limited to WAV initially
   - **Mitigation**: Document limitations, plan for future formats

### API Risks
1. **Breaking Changes**: Sample integration could affect existing code
   - **Mitigation**: Maintain backward compatibility in Waveform enum

2. **Complexity**: Sample management could complicate simple use cases
   - **Mitigation**: Keep mathematical waveforms as primary, samples as enhancement

## Success Metrics

### Functional Goals
- [ ] Load WAV files as samples
- [ ] Play samples at different pitches
- [ ] Integrate samples with ADSR envelopes
- [ ] Mix samples in polyphonic timelines
- [ ] Support loop points for sustained playback

### Performance Goals
- [ ] Sample loading < 100ms for typical drum sounds
- [ ] Real-time playback without audio dropouts
- [ ] Memory usage < 2x sample file size
- [ ] Pitch shifting accuracy within 1 cent

### User Experience Goals
- [ ] Simple API for basic sample usage
- [ ] Comprehensive test tool commands
- [ ] Clear documentation with examples
- [ ] Graceful error handling and messaging

## Future Roadmap

### Version 0.2.0: Basic Samples
- Sample loading and playback
- Basic pitch shifting
- Test tool integration

### Version 0.3.0: Advanced Samples
- Loop point support
- Multi-sampling
- Performance optimization

### Version 0.4.0: Production Features
- Velocity layers
- Sample effects
- Advanced resampling algorithms

### Version 1.0.0: Complete Audio Engine
- Full format support (FLAC, OGG, MP3)
- Real-time processing
- Plugin architecture

---

**Next Steps**: Begin Phase 1 implementation with core library extensions for sample support.