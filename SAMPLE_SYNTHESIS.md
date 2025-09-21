# Sample-Based Synthesis Documentation

## Overview

Polyphonica now supports custom sample-based synthesis, allowing users to load their own WAV recordings (such as drum hits, instrument recordings, or vocal samples) and use them as waveforms alongside the traditional mathematical waveforms (sine, square, sawtooth, triangle).

## Core Features

### Sample Loading
- **WAV File Support**: Load 16-bit WAV files of any sample rate
- **Automatic Conversion**: Samples are automatically converted to the target project sample rate
- **Base Frequency**: Define the original pitch of your sample for accurate pitch shifting
- **Memory Efficient**: Samples are loaded once and reused across multiple events

### Pitch Shifting
- **Real-time Resampling**: Play samples at different pitches using linear interpolation
- **Frequency Mapping**: Specify base frequency and target frequency for accurate transposition
- **Quality Preservation**: Linear interpolation maintains audio quality during pitch changes

### ADSR Integration
- **Envelope Support**: Apply Attack, Decay, Sustain, and Release envelopes to samples
- **Natural Behavior**: Samples blend seamlessly with envelope shaping
- **Mixed Compositions**: Combine samples with mathematical waveforms in polyphonic arrangements

## API Usage

### Basic Sample Loading

```rust
use polyphonica::{SampleData, Waveform, generate_sample};

// Load a sample
let sample_data = SampleData::from_wav_file("kick_drum.wav")?;
let sample_waveform = Waveform::Sample(sample_data);

// Generate with pitch shifting (original: 60Hz, target: 80Hz)
let samples = generate_sample(sample_waveform, 0.0, 80.0, 2.0, 44100);
```

### Sample with Envelope

```rust
use polyphonica::{SampleData, Waveform, SoundEvent, AdsrEnvelope, render_event};

let sample_data = SampleData::from_wav_file("tomtom.wav")?;
let sample_waveform = Waveform::Sample(sample_data);

let envelope = AdsrEnvelope {
    attack_secs: 0.01,
    decay_secs: 0.3,
    sustain_level: 0.7,
    release_secs: 0.8,
};

let event = SoundEvent {
    waveform: sample_waveform,
    start_frequency: 165.0,  // Play at lower pitch than original
    end_frequency: 165.0,
    duration_secs: 1.5,
    envelope,
};

let samples = render_event(&event, 44100);
```

### Polyphonic Sample Mixing

```rust
use polyphonica::{SampleData, Waveform, SoundEvent, AdsrEnvelope, render_timeline};

// Load multiple samples
let kick = SampleData::from_wav_file("kick.wav")?;
let snare = SampleData::from_wav_file("snare.wav")?;
let hihat = SampleData::from_wav_file("hihat.wav")?;

let drum_envelope = AdsrEnvelope {
    attack_secs: 0.01,
    decay_secs: 0.1,
    sustain_level: 0.0,  // Drums don't sustain
    release_secs: 0.2,
};

// Create drum pattern
let events = &[
    (0.0, SoundEvent {
        waveform: Waveform::Sample(kick),
        start_frequency: 60.0,
        end_frequency: 60.0,
        duration_secs: 0.5,
        envelope: drum_envelope.clone(),
    }),
    (0.5, SoundEvent {
        waveform: Waveform::Sample(snare),
        start_frequency: 200.0,
        end_frequency: 200.0,
        duration_secs: 0.3,
        envelope: drum_envelope.clone(),
    }),
    (1.0, SoundEvent {
        waveform: Waveform::Sample(hihat),
        start_frequency: 8000.0,
        end_frequency: 8000.0,
        duration_secs: 0.1,
        envelope: drum_envelope,
    }),
];

let mixed_audio = render_timeline(events, 4.0, 44100);
```

## Test Tool Usage

### Sample Playback

```bash
# Play a sample at its original pitch
polyphonica-test sample load samples/kick.wav \
  --duration 1.0 \
  --play --volume 0.7

# Pitch shift a sample (play at different frequency)
polyphonica-test sample load samples/tomtom.wav \
  --frequency 330.0 \
  --duration 2.0 \
  --output shifted_tomtom.wav \
  --play --volume 0.5
```

### Sample with Envelope

```bash
# Apply envelope shaping to a sample
polyphonica-test sample-event samples/piano_c4.wav \
  --frequency 440.0 \
  --duration 3.0 \
  --attack 0.1 \
  --decay 0.5 \
  --sustain 0.6 \
  --release 1.0 \
  --play --volume 0.4
```

### Creating Drum Patterns

```bash
# Example workflow for creating a drum machine pattern
# (requires creating multiple sample events and mixing them)

# Kick drum pattern
polyphonica-test sample-event samples/kick.wav \
  --frequency 60.0 \
  --duration 0.5 \
  --attack 0.01 \
  --decay 0.1 \
  --sustain 0.8 \
  --release 0.2 \
  --output kick_pattern.wav

# Snare hits
polyphonica-test sample-event samples/snare.wav \
  --frequency 200.0 \
  --duration 0.3 \
  --attack 0.01 \
  --decay 0.05 \
  --sustain 0.3 \
  --release 0.1 \
  --output snare_pattern.wav
```

## Sample Requirements

### File Format
- **Format**: WAV files only (16-bit PCM)
- **Channels**: Mono preferred (stereo files use left channel only)
- **Sample Rate**: Any rate (automatically converted)
- **Bit Depth**: 16-bit signed integer

### Base Frequency Guidelines
- **Drum Samples**: Use fundamental frequency or dominant frequency
  - Kick drums: 40-80 Hz
  - Snare drums: 150-250 Hz
  - Hi-hats: 8000-12000 Hz (use high frequency for metallic samples)
- **Melodic Samples**: Use the actual musical note frequency
  - Piano C4: 261.63 Hz
  - Guitar A4: 440.0 Hz
- **Unknown Pitch**: Use spectral analysis tools or estimate by ear

### Quality Considerations
- **Sample Length**: Shorter samples pitch-shift more accurately
- **Loop Points**: Samples without natural decay work best for sustained notes
- **Noise Floor**: Clean samples produce better results at extreme pitch shifts
- **Bit Depth**: Higher bit depth source material reduces quantization noise

## Technical Implementation

### Sample Data Structure

```rust
pub struct SampleData {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub duration_secs: f32,
    pub metadata: SampleMetadata,
}

pub struct SampleMetadata {
    pub name: String,
    pub file_path: String,
    pub file_size_bytes: u64,
    pub format_info: String,
}
```

### Pitch Shifting Algorithm

The library uses linear interpolation for real-time pitch shifting:

```rust
// In generate_sample function for Waveform::Sample
let original_frequency = 440.0;  // Assumed base frequency for samples
let pitch_ratio = target_frequency / original_frequency;
let adjusted_time = time_secs * pitch_ratio;
let sample_index = adjusted_time * sample_data.sample_rate as f32;

// Linear interpolation between adjacent samples
let index_floor = sample_index.floor() as usize;
let index_ceil = index_floor + 1;
let fraction = sample_index - index_floor as f32;

if index_ceil < sample_data.samples.len() {
    let sample_a = sample_data.samples[index_floor];
    let sample_b = sample_data.samples[index_ceil];
    sample_a + (sample_b - sample_a) * fraction
} else {
    0.0  // Beyond sample bounds
}
```

### Memory Management
- Samples are stored as `Vec<f32>` normalized to [-1.0, 1.0] range
- Large samples should be used judiciously to avoid memory pressure
- Consider streaming for very long samples (not currently implemented)

## Integration Examples

### Mixing Samples with Mathematical Waveforms

```rust
use polyphonica::{SampleData, Waveform, SoundEvent};

let kick_sample = SampleData::from_wav_file("kick.wav", 60.0)?;

let events = vec![
    // Sample-based kick
    SoundEvent {
        waveform: Waveform::Sample(kick_sample),
        frequency: 60.0,
        start_time: 0.0,
        duration: 0.5,
        // ...
    },
    // Mathematical sine bass
    SoundEvent {
        waveform: Waveform::Sine,
        frequency: 80.0,
        start_time: 0.0,
        duration: 4.0,
        // ...
    },
];
```

### Creating Custom Instruments

Combine multiple samples at different pitches to create playable instruments:

```rust
// Load piano samples at different octaves
let piano_c3 = SampleData::from_wav_file("piano_c3.wav", 130.81)?;
let piano_c4 = SampleData::from_wav_file("piano_c4.wav", 261.63)?;
let piano_c5 = SampleData::from_wav_file("piano_c5.wav", 523.25)?;

// Play melody using appropriate sample for each note's range
// (implementation would select best base sample for target frequency)
```

## Performance Considerations

### CPU Usage
- Linear interpolation is computationally lightweight
- Large samples may impact real-time performance
- Consider pre-processing samples to target sample rate

### Memory Usage
- Each sample consumes `sample_count * 4 bytes` (f32 storage)
- 1 second of 44.1kHz audio ≈ 176KB memory
- Long samples should be used sparingly

### Quality vs Performance
- Linear interpolation: Fast, good quality for moderate pitch shifts
- Future: Cubic interpolation for higher quality (not implemented)
- Future: Time-stretching algorithms for extreme pitch shifts (not implemented)

## Error Handling

### WAV Loading Errors
```rust
// File not found, format errors, etc.
match SampleData::from_wav_file("missing.wav", 440.0) {
    Ok(sample) => { /* use sample */ },
    Err(e) => eprintln!("Failed to load sample: {}", e),
}
```

### Common Issues
- **File Format**: Only 16-bit WAV files supported
- **Missing Files**: Check file paths and permissions
- **Invalid Base Frequency**: Must be positive, non-zero value
- **Memory Limits**: Very large samples may cause allocation failures

## Future Enhancements

### Planned Features
- **Loop Point Support**: Define loop regions for sustained playback
- **Multi-sample Instruments**: Automatically select best sample for target frequency
- **Time Stretching**: Preserve pitch while changing duration
- **Sample Streaming**: Handle very large samples without full memory loading

### Advanced Pitch Algorithms
- **Cubic Interpolation**: Higher quality pitch shifting
- **Phase Vocoder**: Spectral pitch shifting for extreme ratios
- **Granular Synthesis**: Texture-based sample manipulation

## Best Practices

### Sample Preparation
1. **Normalize Audio**: Ensure samples use full dynamic range
2. **Remove DC Offset**: Center audio around zero
3. **Trim Silence**: Remove unnecessary leading/trailing silence
4. **Document Base Frequency**: Keep accurate frequency records

### Performance Optimization
1. **Sample Rate Matching**: Pre-convert samples to target rate when possible
2. **Length Optimization**: Trim samples to minimum necessary length
3. **Memory Management**: Load samples once, reuse across events
4. **Quality Balance**: Use appropriate pitch shift ranges

### Musical Applications
1. **Drum Machines**: Multiple samples with envelope shaping
2. **Melodic Instruments**: Pitch shifting for musical intervals
3. **Texture Pads**: Long samples with slow envelope changes
4. **Percussive Elements**: Short samples with quick envelopes

This documentation covers the complete sample-based synthesis system integrated into Polyphonica. The implementation allows seamless mixing of recorded samples with mathematical waveforms, opening up extensive creative possibilities for audio synthesis applications.