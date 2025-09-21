# Polyphonica 🎵

A Rust library for real-time audio synthesis with polyphonic mixing capabilities. Polyphonica provides a complete audio synthesis pipeline from basic waveform generation to complex multi-voice compositions.

## Features

- **Multiple Waveform Types**: Sine, Square, Sawtooth, and Triangle waves
- **ADSR Envelope Shaping**: Attack, Decay, Sustain, Release envelope control
- **Frequency Sweeping**: Linear interpolation between start and end frequencies
- **Polyphonic Mixing**: Timeline-based rendering of multiple overlapping sound events
- **Clipping Prevention**: Automatic audio level clamping to prevent distortion

## Architecture

Polyphonica is built in four progressive phases:

### Phase 1: Waveform Generation
Basic oscillator that generates raw waveforms at specified frequencies.

```rust
use polyphonica::{Waveform, generate_wave};

let samples = generate_wave(Waveform::Sine, 440.0, 1.0, 44100);
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

#### `generate_wave`
```rust
pub fn generate_wave(
    waveform: Waveform,
    frequency: f32,
    duration_secs: f32,
    sample_rate: u32,
) -> Vec<f32>
```
Generates raw waveform samples at a constant frequency.

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

## Examples

### Basic Sine Wave
```rust
use polyphonica::{Waveform, generate_wave};

// Generate 1 second of 440Hz sine wave at 44.1kHz sample rate
let samples = generate_wave(Waveform::Sine, 440.0, 1.0, 44100);
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
let samples = generate_wave(Waveform::Sine, 440.0, 1.0, 44100);

// Or create complex compositions
let events = &[(0.0, your_sound_event)];
let audio = render_timeline(events, 5.0, 44100);
```

## Technical Details

- **Sample Format**: 32-bit floating point (`f32`)
- **Sample Range**: -1.0 to 1.0 (automatically clamped)
- **Mixing**: Additive with clipping prevention
- **Frequency Interpolation**: Linear between start and end frequencies
- **Envelope Curves**: Linear transitions between ADSR phases

## Testing

Run the comprehensive test suite:
```bash
cargo test
```

The tests cover:
- Waveform generation accuracy
- ADSR envelope behavior
- Frequency sweep correctness
- Polyphonic mixing validation
- Edge cases and error handling

## License

This project is licensed under the MIT License.