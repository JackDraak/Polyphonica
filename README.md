# Polyphonica 🎵

A comprehensive Rust library for real-time audio synthesis with polyphonic mixing capabilities and sample-based synthesis. Polyphonica provides a complete audio synthesis pipeline from basic waveform generation to complex multi-voice compositions using both mathematical waveforms and custom audio samples.

## Features

- **Multiple Waveform Types**: Sine, Square, Sawtooth, Triangle, Pulse (with duty cycle), and White Noise
- **Sample-Based Synthesis**: Load and use custom WAV recordings as waveforms with pitch shifting
- **Sample Catalog System**: JSON-based organization and discovery of sample collections
- **ADSR Envelope Shaping**: Attack, Decay, Sustain, Release envelope control
- **Frequency Sweeping**: Linear interpolation between start and end frequencies
- **Polyphonic Mixing**: Timeline-based rendering of multiple overlapping sound events
- **Real-time Audio Playback**: Live audio output with volume control via CLI test tool
- **CLI Test Tool**: Comprehensive command-line interface for testing and development
- **Clipping Prevention**: Automatic audio level clamping to prevent distortion

## Architecture

Polyphonica is built in four progressive phases:

### Phase 1: Waveform Generation
Advanced oscillator that generates mathematical waveforms or loads custom samples.

```rust
use polyphonica::{Waveform, generate_sample};

// Mathematical waveforms
let samples = generate_sample(Waveform::Sine, 0.0, 440.0, 1.0, 44100);

// Pulse wave with duty cycle
let pulse = generate_sample(Waveform::Pulse { duty_cycle: 0.25 }, 0.0, 440.0, 1.0, 44100);

// White noise
let noise = generate_sample(Waveform::Noise, 0.0, 440.0, 1.0, 44100);

// Custom samples (loaded from WAV files)
let sample_data = SampleData::from_wav_file("path/to/sample.wav")?;
let custom = generate_sample(Waveform::Sample(sample_data), 0.0, 440.0, 1.0, 44100);
```

### Phase 2: ADSR Envelope
Amplitude envelope that shapes the volume of generated waveforms over time.

```rust
use polyphonica::{AdsrEnvelope, apply_envelope};

let envelope = AdsrEnvelope {
    attack_secs: 0.1,
    decay_secs: 0.2,
    sustain_level: 0.7,
    release_secs: 0.3,
};

apply_envelope(&mut samples, &envelope, 44100);
```

### Phase 3: Sound Events
Complete sound abstraction that combines waveforms, frequency sweeps, and envelopes.

```rust
use polyphonica::{SoundEvent, render_event};

let event = SoundEvent {
    waveform: Waveform::Sine,
    start_frequency: 440.0,
    end_frequency: 880.0,  // Frequency sweep
    duration_secs: 2.0,
    envelope,
};

let samples = render_event(&event, 44100);
```

### Phase 4: Polyphonic Timeline
Multi-voice mixer that combines multiple sound events with precise timing.

```rust
use polyphonica::render_timeline;

let events = &[
    (0.0, bass_note),      // Start immediately
    (0.5, melody_note),    // Start at 0.5 seconds
    (1.0, harmony_note),   // Start at 1.0 seconds
];

let final_audio = render_timeline(events, 3.0, 44100);
```

## API Reference

### Core Types

#### `Waveform`
```rust
pub enum Waveform {
    Sine,
    Square,
    Sawtooth,
    Triangle,
    Pulse { duty_cycle: f32 },  // 0.0-1.0 duty cycle
    Noise,                      // White noise generator
    Sample(SampleData),         // Custom WAV samples
}
```

#### `AdsrEnvelope`
```rust
pub struct AdsrEnvelope {
    pub attack_secs: f32,      // Time to reach peak volume
    pub decay_secs: f32,       // Time to decay to sustain level
    pub sustain_level: f32,    // Sustained volume level (0.0-1.0)
    pub release_secs: f32,     // Time to fade to silence
}
```

#### `SoundEvent`
```rust
pub struct SoundEvent {
    pub waveform: Waveform,
    pub start_frequency: f32,
    pub end_frequency: f32,
    pub duration_secs: f32,
    pub envelope: AdsrEnvelope,
}
```

### Core Functions

#### `generate_sample`
```rust
pub fn generate_sample(
    waveform: Waveform,
    time_secs: f32,
    target_frequency: f32,
    duration_secs: f32,
    sample_rate: u32,
) -> Vec<f32>
```
Generates waveform samples with time offset and pitch shifting support for samples.

#### `apply_envelope`
```rust
pub fn apply_envelope(
    samples: &mut Vec<f32>,
    envelope: &AdsrEnvelope,
    sample_rate: u32,
)
```
Applies ADSR envelope to existing samples in-place.

#### `render_event`
```rust
pub fn render_event(
    event: &SoundEvent,
    sample_rate: u32,
) -> Vec<f32>
```
Renders a complete sound event with frequency sweep and envelope.

#### `render_timeline`
```rust
pub fn render_timeline(
    events: &[(f32, SoundEvent)],
    total_duration_secs: f32,
    sample_rate: u32,
) -> Vec<f32>
```
Mixes multiple timed sound events into a single audio buffer.

### Sample System

#### `SampleData`
```rust
pub struct SampleData {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub duration_secs: f32,
    pub metadata: SampleMetadata,
}

impl SampleData {
    pub fn from_wav_file(path: &str) -> Result<Self, SampleError>
}
```

#### `SampleCatalog`
```rust
pub struct SampleCatalog {
    pub collections: HashMap<String, SampleCollection>,
}

impl SampleCatalog {
    pub fn scan_directory(path: &str) -> Result<Self, SampleError>
    pub fn search(&self, query: &str) -> Vec<&SampleEntry>
}
```

## CLI Test Tool

The `polyphonica-test` binary provides comprehensive testing and development capabilities:

```bash
# Install and run the test tool
cargo install --path .
polyphonica-test --help

# Generate basic waveforms with real-time playback
polyphonica-test wave sine --frequency 440 --duration 2 --play --volume 0.5

# Test pulse waves with duty cycle control
polyphonica-test wave pulse --frequency 440 --duty-cycle 0.25 --play

# Create polyphonic compositions (up to 16 voices)
polyphonica-test poly --voices 4 --play

# Load and play custom samples
polyphonica-test sample load path/to/sample.wav --frequency 440 --play

# Sample catalog management
polyphonica-test catalog scan samples/
polyphonica-test catalog search "drum"
polyphonica-test catalog audition samples/drums/kick.wav --play
```

## Examples

### Basic Waveforms
```rust
use polyphonica::{Waveform, generate_sample};

// Generate 1 second of 440Hz sine wave at 44.1kHz sample rate
let samples = generate_sample(Waveform::Sine, 0.0, 440.0, 1.0, 44100);

// Pulse wave with 25% duty cycle
let pulse = generate_sample(Waveform::Pulse { duty_cycle: 0.25 }, 0.0, 440.0, 1.0, 44100);

// White noise
let noise = generate_sample(Waveform::Noise, 0.0, 440.0, 1.0, 44100);
```

### Custom Sample Loading
```rust
use polyphonica::{SampleData, Waveform, generate_sample};

// Load a custom WAV sample
let sample_data = SampleData::from_wav_file("samples/drums/kick.wav")?;

// Use it as a waveform with pitch shifting to 440Hz
let shifted_sample = generate_sample(
    Waveform::Sample(sample_data),
    0.0,
    440.0,  // Target frequency
    1.0,
    44100
);
```

### Piano-like Note with Envelope
```rust
use polyphonica::{Waveform, SoundEvent, AdsrEnvelope, render_event};

let piano_envelope = AdsrEnvelope {
    attack_secs: 0.01,    // Quick attack
    decay_secs: 0.3,      // Medium decay
    sustain_level: 0.3,   // Low sustain
    release_secs: 1.0,    // Long release
};

let note = SoundEvent {
    waveform: Waveform::Sine,
    start_frequency: 261.63, // Middle C
    end_frequency: 261.63,   // No frequency sweep
    duration_secs: 2.0,
    envelope: piano_envelope,
};

let audio = render_event(&note, 44100);
```

### Chord Progression
```rust
use polyphonica::{render_timeline, SoundEvent, Waveform, AdsrEnvelope};

let chord_envelope = AdsrEnvelope {
    attack_secs: 0.1,
    decay_secs: 0.2,
    sustain_level: 0.6,
    release_secs: 0.5,
};

// C Major Chord (C-E-G)
let c_note = SoundEvent {
    waveform: Waveform::Sine,
    start_frequency: 261.63,
    end_frequency: 261.63,
    duration_secs: 2.0,
    envelope: chord_envelope.clone(),
};

let e_note = SoundEvent {
    waveform: Waveform::Sine,
    start_frequency: 329.63,
    end_frequency: 329.63,
    duration_secs: 2.0,
    envelope: chord_envelope.clone(),
};

let g_note = SoundEvent {
    waveform: Waveform::Sine,
    start_frequency: 392.00,
    end_frequency: 392.00,
    duration_secs: 2.0,
    envelope: chord_envelope,
};

let chord_timeline = &[
    (0.0, c_note),
    (0.0, e_note),
    (0.0, g_note),
];

let final_audio = render_timeline(chord_timeline, 3.0, 44100);
```

### Frequency Sweep Effect
```rust
use polyphonica::{SoundEvent, Waveform, AdsrEnvelope, render_event};

let sweep_envelope = AdsrEnvelope {
    attack_secs: 0.0,
    decay_secs: 0.0,
    sustain_level: 1.0,
    release_secs: 0.0,
};

let sweep = SoundEvent {
    waveform: Waveform::Sawtooth,
    start_frequency: 110.0,   // Low A
    end_frequency: 880.0,     // High A (3 octaves up)
    duration_secs: 3.0,
    envelope: sweep_envelope,
};

let sweep_audio = render_event(&sweep, 44100);
```

### Sample Catalog Management
```rust
use polyphonica::{SampleCatalog, SampleData};

// Scan a directory tree for samples and create a catalog
let catalog = SampleCatalog::scan_directory("samples/")?;

// Search for specific samples
let drums = catalog.search("drum");
let cymbals = catalog.search("cymbal");

// Load and use a specific sample
if let Some(sample_entry) = catalog.search("kick").first() {
    let sample_data = SampleData::from_wav_file(&sample_entry.path)?;
    let kick_sound = generate_sample(
        Waveform::Sample(sample_data),
        0.0,
        100.0,  // Low frequency for kick drum
        1.0,
        44100
    );
}
```

### Polyphonic Drum Pattern
```rust
use polyphonica::{render_timeline, SoundEvent, Waveform, AdsrEnvelope, SampleData};

// Load drum samples
let kick_data = SampleData::from_wav_file("samples/drums/kick.wav")?;
let snare_data = SampleData::from_wav_file("samples/drums/snare.wav")?;

let drum_envelope = AdsrEnvelope {
    attack_secs: 0.01,
    decay_secs: 0.1,
    sustain_level: 0.0,  // Drums don't sustain
    release_secs: 0.2,
};

// Create drum events
let kick = SoundEvent {
    waveform: Waveform::Sample(kick_data),
    start_frequency: 60.0,   // Low kick frequency
    end_frequency: 60.0,
    duration_secs: 0.5,
    envelope: drum_envelope.clone(),
};

let snare = SoundEvent {
    waveform: Waveform::Sample(snare_data),
    start_frequency: 200.0,  // Higher snare frequency
    end_frequency: 200.0,
    duration_secs: 0.3,
    envelope: drum_envelope,
};

// Create a simple beat pattern
let beat_pattern = &[
    (0.0, kick.clone()),     // Beat 1
    (0.5, snare.clone()),    // Beat 2
    (1.0, kick.clone()),     // Beat 3
    (1.5, snare.clone()),    // Beat 4
];

let drum_loop = render_timeline(beat_pattern, 2.0, 44100);
```

## Getting Started

1. Add polyphonica to your `Cargo.toml`:
```toml
[dependencies]
polyphonica = "0.1.0"
```

2. Import the library:
```rust
use polyphonica::*;
```

3. Start generating audio:
```rust
// Simple sine wave
let samples = generate_sample(Waveform::Sine, 0.0, 440.0, 1.0, 44100);

// Pulse wave with duty cycle
let pulse = generate_sample(Waveform::Pulse { duty_cycle: 0.5 }, 0.0, 440.0, 1.0, 44100);

// Load custom samples
let sample_data = SampleData::from_wav_file("path/to/sample.wav")?;
let custom = generate_sample(Waveform::Sample(sample_data), 0.0, 440.0, 1.0, 44100);

// Or create complex compositions
let events = &[(0.0, your_sound_event)];
let audio = render_timeline(events, 5.0, 44100);
```

4. Use the CLI test tool for development:
```bash
# Install the test tool
cargo install --path .

# Test basic waveforms with real-time playback
polyphonica-test wave sine --frequency 440 --play --volume 0.5

# Test polyphonic capabilities
polyphonica-test poly --voices 8 --play

# Manage sample catalogs
polyphonica-test catalog scan samples/
polyphonica-test catalog search "drum"
```

## Technical Details

- **Sample Format**: 32-bit floating point (`f32`)
- **Sample Range**: -1.0 to 1.0 (automatically clamped)
- **Mixing**: Additive with clipping prevention
- **Frequency Interpolation**: Linear between start and end frequencies
- **Envelope Curves**: Linear transitions between ADSR phases
- **Sample Pitch Shifting**: Linear interpolation with frequency scaling
- **Noise Generation**: Linear congruential generator for deterministic output
- **Pulse Wave Duty Cycle**: Configurable 0.0-1.0 range
- **WAV Support**: Mono and stereo files via hound crate
- **Real-time Audio**: Cross-platform via rodio/cpal

## Testing

Run the comprehensive test suite:
```bash
cargo test
```

The tests cover:
- Mathematical waveform generation accuracy (sine, square, sawtooth, triangle)
- Pulse wave duty cycle validation
- White noise deterministic generation
- Sample loading and pitch shifting
- ADSR envelope behavior
- Frequency sweep correctness
- Polyphonic mixing validation
- Sample catalog system
- Edge cases and error handling

## CLI Test Tool

The included `polyphonica-test` binary provides:
- Real-time audio playback and testing
- Volume control and audio device management
- Polyphonic composition testing (up to 16 voices)
- Sample loading and catalog management
- WAV file export capabilities
- Developer feedback and issue reporting system

## License

This project is licensed under the MIT License.