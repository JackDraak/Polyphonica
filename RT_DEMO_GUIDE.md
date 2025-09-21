# Polyphonica Real-Time GUI Demo

## Overview

The RT Demo is an interactive GUI demonstration of Polyphonica's real-time synthesis engine. It provides live audio output through your system's speakers with comprehensive controls for experimenting with waveforms, envelopes, and polyphonic synthesis.

## Quick Start

```bash
# Run the interactive GUI demo
cargo run --bin rt-demo
```

The demo will:
1. Initialize CPAL audio system
2. Connect to your default audio output device
3. Launch the interactive GUI
4. Start real-time audio processing

## GUI Controls

### Waveform Selection
Choose from 8 different waveform types:
- **Sine**: Pure harmonic tone
- **Square**: Rich harmonic content with odd harmonics
- **Sawtooth**: Bright sound with all harmonics
- **Triangle**: Mellow tone with reduced harmonics
- **Pulse 25%**: Square wave with 25% duty cycle
- **Pulse 50%**: Standard square wave
- **Pulse 75%**: Square wave with 75% duty cycle
- **White Noise**: Random noise for percussion effects

### Audio Parameters
- **Master Volume**: Global output level (0-100%)
- **Frequency**: Fundamental frequency in Hz (80-2000 Hz)

### ADSR Envelope (Collapsible)
Fine-tune the amplitude envelope for each note:
- **Attack**: Time to reach peak amplitude (0-2 seconds)
- **Decay**: Time to decay to sustain level (0-2 seconds)
- **Sustain**: Sustained amplitude level (0-100%)
- **Release**: Time to fade to silence after note release (0-2 seconds)

### Note Control
- **🎵 Play Note**: Trigger a single note at current frequency
- **🔇 Release All**: Gracefully release all active notes (ADSR release)
- **🛑 Panic Stop**: Immediately silence all audio

### Musical Features

#### Major Chords
Pre-configured chord buttons for instant harmonic content:
- C Maj, D Maj, E Maj, F Maj, G Maj, A Maj, B Maj, C Oct

Each chord triggers a root, major third, and perfect fifth simultaneously.

#### Note Presets
Quick access to common musical frequencies:
- C4 (261.63 Hz) through C5 (523.25 Hz)

Clicking a preset both sets the frequency and triggers the note.

## Technical Information

### Real-Time Performance
- **Sample Rate**: 44.1 kHz (CD quality)
- **Voice Count**: Up to 32 concurrent voices
- **Latency**: Low-latency buffer processing
- **Thread Safety**: Lock-free parameter updates

### Status Display
The GUI shows:
- Active voice count vs. maximum
- Current master volume percentage
- Playing/silent status indicator

### Performance Info Panel
Detailed technical specifications:
- Zero-allocation audio processing
- Sample-accurate timing
- Voice stealing when exceeding 32 voices
- CPAL-compatible audio output

## Usage Tips

### Basic Operation
1. Start with default settings (Sine wave, 440Hz)
2. Click "Play Note" to hear audio output
3. Experiment with different waveforms
4. Adjust volume to comfortable level

### Exploring Waveforms
- **Sine waves** are great for pure tones and testing
- **Square/Sawtooth** provide rich harmonic content
- **Pulse waves** offer duty cycle variations
- **Noise** is perfect for percussion and effects

### Envelope Experimentation
- **Short attack** (0.01s) for percussive sounds
- **Long attack** (1s+) for pad-like swells
- **High sustain** (0.8+) for organ-like tones
- **Long release** (1s+) for ambient effects

### Polyphonic Features
- Trigger multiple notes without releasing previous ones
- Use chord buttons for instant harmonic combinations
- Monitor voice count to understand polyphonic limits
- Use "Release All" for controlled fade-outs

## Troubleshooting

### No Audio Output
1. Check system volume and audio device
2. Verify audio device selection in system settings
3. Ensure no other applications are blocking audio
4. Try restarting the demo

### High CPU Usage
- The real-time engine is optimized for performance
- 32 concurrent voices should run smoothly on modern systems
- Use "Panic Stop" to immediately reduce CPU load

### GUI Responsiveness
- The interface updates at 50ms intervals for smooth operation
- Parameter changes take effect immediately
- Voice management is handled automatically

## Integration Examples

### CPAL Integration Pattern
```rust
use polyphonica::RealtimeEngine;
use std::sync::{Arc, Mutex};

let engine = Arc::new(Mutex::new(RealtimeEngine::new(44100.0)));

// In audio callback
let mut engine = shared_engine.lock().unwrap();
engine.process_stereo_buffer(output_buffer);
```

### Parameter Control
```rust
// Real-time parameter updates
engine.set_master_volume(0.7);
engine.set_voice_frequency(voice_id, 880.0);

// Note triggering
let voice_id = engine.trigger_note(Waveform::Sine, 440.0, envelope);
```

## Development Features

This demo serves as a reference implementation for:
- CPAL audio stream setup
- Real-time parameter control
- GUI integration with audio processing
- Thread-safe engine operation
- Voice management strategies

The source code (`src/bin/rt_demo.rs`) provides a complete example for integrating Polyphonica into interactive applications.