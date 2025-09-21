# Guitar Buddy - Musical Practice Companion

## Overview

Guitar Buddy is an advanced musical practice companion built on Polyphonica's real-time synthesis engine. Designed specifically for guitar, bass, and other instrumental practice, it provides precision timing tools and will eventually offer full accompaniment features.

## Phase 1: Advanced Metronome (Current)

### Quick Start

```bash
# Launch Guitar Buddy
cargo run --bin guitar-buddy
```

### Features

#### Comprehensive Time Signatures
Support for common and complex time signatures:
- **4/4** - Standard rock/pop time
- **3/4** - Waltz time
- **2/4** - March time
- **6/8** - Compound duple time
- **9/8** - Compound triple time
- **12/8** - Compound quadruple time
- **5/4** - Progressive/odd time
- **7/8** - Complex odd time

#### Multiple Click Sounds
Ten distinct metronome click types including both synthetic and real drum samples:

**Synthetic Sounds:**
- **Wood Block** - Sharp, percussive click (classic metronome sound)
- **Digital Beep** - Clean sine wave tone
- **Cowbell** - Metallic ring with sustain
- **Electro Click** - Modern electronic click

**Real Drum Samples:**
- **Acoustic Kick** - Deep, natural kick drum sound
- **Acoustic Snare** - Crisp snare drum with natural decay
- **Hi-Hat Closed** - Sharp, closed hi-hat click
- **Hi-Hat Open** - Sustained open hi-hat sound
- **Rim Shot** - Snare rim shot with tight envelope
- **Drum Stick** - Drumstick click using hi-hat sample

#### Tempo Control
- **Range**: 40-200 BPM with 1 BPM precision
- **Presets**: Quick access to Slow (60), Medium (120), Fast (160) tempos
- **Real-time adjustment**: Change tempo while playing

#### Beat Accenting
- **First Beat Accent**: Emphasizes the downbeat (50% louder)
- **Visual Beat Indicator**: Shows current beat position in measure
- **Accent Symbols**: Visual distinction between accented (●) and regular (○) beats

#### Transport Controls
- **Start**: Begin metronome from beat 1
- **Pause/Resume**: Pause timing while maintaining beat position
- **Stop**: Complete stop and reset to beginning
- **Test Sounds**: Preview click sounds and accents

#### Beat Visualization
- Real-time display of current beat position
- Visual emphasis for accented beats
- Beat interval timing information (milliseconds between beats)

### Technical Specifications

#### Audio Performance
- **Sample Rate**: 44.1 kHz (professional quality)
- **Latency**: Ultra-low latency using CPAL audio system
- **Timing Precision**: ±1ms accuracy for professional practice
- **Audio Engine**: Polyphonica real-time synthesis with zero allocations

#### Click Sound Generation
Guitar Buddy supports both synthetic waveforms and real drum samples:

**Synthetic Waveforms:**
- **Wood Block**: White noise with sharp attack/decay
- **Digital Beep**: Pure sine wave at 1000Hz
- **Cowbell**: Square wave with medium sustain
- **Electro Click**: Pulse wave with moderate decay

**Real Drum Samples:**
- **Acoustic Kit**: Professional-quality drum samples from `samples/drums/acoustic/kit_01/`
- **Natural Playback**: Drums play at their recorded speed without pitch shifting
- **Minimal Envelopes**: ADSR envelopes preserve natural drum character
- **One-Shot Design**: Samples play through naturally without looping
- **44.1kHz Quality**: High-resolution audio for professional practice
- **Automatic Fallback**: Uses synthetic sounds if samples unavailable

#### GUI Features
- **Real-time Updates**: 10ms refresh rate for smooth operation
- **Responsive Controls**: Immediate parameter changes
- **Status Display**: Current tempo, time signature, and beat position
- **Collapsible Sections**: Organized interface for different control groups

### Usage Guide

#### Basic Practice Session
1. **Set Time Signature**: Choose appropriate time signature for your music
2. **Select Tempo**: Start slower than target tempo for learning
3. **Choose Click Sound**: Pick a click that cuts through your instrument
4. **Enable Accent**: Use first beat accent to maintain downbeat awareness
5. **Start Playing**: Click "Start" and begin practicing

#### Advanced Features
- **Tempo Training**: Gradually increase BPM as you improve
- **Odd Time Practice**: Use 5/4, 7/8 for progressive music training
- **Volume Adjustment**: Set click volume to blend with your playing
- **Sound Selection**: Different clicks work better with different musical styles
  - Acoustic samples for natural, organic practice
  - Synthetic sounds for electronic/modern music
  - Hi-hat sounds for subtle, unobtrusive timing
  - Kick/snare for pronounced, driving beats

#### Best Practices
- **Start Slow**: Begin 10-20 BPM below comfortable tempo
- **Use Accents**: Downbeat emphasis helps with musical phrasing
- **Match Click to Style**: Wood block for acoustic, electro click for modern
- **Practice Without**: Gradually reduce metronome dependence

## Phase 2: Full Accompaniment (Planned)

### Upcoming Features

#### Drum Patterns
- **Basic Beats**: Rock, pop, ballad, shuffle patterns
- **Genre Styles**: Blues, jazz, latin, reggae, funk
- **Fill Options**: Automatic fills at phrase endings
- **Custom Patterns**: User-defined drum sequences

#### Bass Line Accompaniment
- **Chord-Based**: Automatic bass lines following chord progressions
- **Walking Bass**: Jazz-style walking bass patterns
- **Genre Styles**: Rock, country, jazz, latin bass styles
- **Root/Fifth**: Simple accompaniment patterns

#### Piano/Keyboard Backing
- **Chord Progressions**: Common progressions (I-vi-IV-V, etc.)
- **Voicing Options**: Different chord voicings and inversions
- **Rhythmic Patterns**: Various comping styles
- **Arpeggiation**: Broken chord patterns

#### Key and Chord Management
- **Key Selection**: All major and minor keys
- **Chord Progressions**: Pre-built and custom progressions
- **Key Changes**: Support for modulation during practice
- **Chord Display**: Real-time chord name display

#### Practice Session Management
- **Recording**: Practice session recording and playback
- **Loop Sections**: Repeat difficult passages with backing
- **Tempo Trainer**: Gradual tempo increases with goals
- **Practice Log**: Track practice time and progress

### Technical Architecture for Phase 2

#### Sample Integration
- **Drum Sample Library**: Real acoustic drum samples integrated in Phase 1
- **Natural Playback Engine**: `DrumSample` waveform type bypasses pitch shifting
- **Authentic Sound**: Preserves original drum recordings without frequency distortion
- **Expandable Sample System**: Support for multiple drum kits and styles
- **Future Samples**: Piano sample libraries for authentic keyboard sounds
- **Bass Integration**: Bass sample modeling for natural bass tones
- **Hot-Swappable Kits**: Runtime switching between different drum sample sets

#### Musical Intelligence
- Chord progression analysis and generation
- Automatic arrangement based on selected style
- Intelligent voice leading for piano parts

#### User Interface Expansion
- **Practice Session Panel**: Recording, looping, tempo training
- **Arrangement Panel**: Drum, bass, piano mix controls
- **Chord Panel**: Progression editing and key management
- **Performance Panel**: Live performance mode with minimal UI

## Integration Examples

### CPAL Audio Setup
```rust
// Guitar Buddy uses same pattern as RT Demo
let engine = Arc::new(Mutex::new(RealtimeEngine::new(44100.0)));
let metronome = Arc::new(Mutex::new(MetronomeState::new()));

// Real-time beat triggering
if metronome.should_trigger_beat() {
    let (waveform, frequency, envelope) = click_type.get_sound_params();
    engine.trigger_note(waveform, frequency, envelope);
}
```

### Timing Engine
```rust
// Precise beat scheduling
fn beat_interval_ms(&self) -> f64 {
    60000.0 / self.tempo_bpm as f64
}

// Sample-accurate timing
fn should_trigger_beat(&mut self) -> bool {
    let elapsed_ms = now.duration_since(last_time).as_millis() as f64;
    elapsed_ms >= self.beat_interval_ms()
}
```

## Drum Sample System

### Current Implementation (Phase 1)
Guitar Buddy includes a comprehensive drum sample management system:

#### Sample Loading
- **Automatic Discovery**: Loads samples from `samples/drums/acoustic/kit_01/`
- **Graceful Fallback**: Uses synthetic sounds if samples unavailable
- **Format Support**: WAV files at various sample rates (auto-converted to 44.1kHz)
- **Natural Playback**: `DrumSample` waveform type preserves original recording characteristics
- **No Pitch Shifting**: Drums maintain their authentic sound without frequency manipulation
- **Minimal Processing**: ADSR envelopes allow natural sample decay

#### Available Samples

**Currently Used in Phase 1 Metronome:**
- `drumkit-kick.wav` - Deep acoustic kick drum
- `drumkit-snare.wav` - Crisp snare drum (mono)
- `drumkit-rimshot.wav` - Sharp snare rim shot
- `drumkit-stick.wav` - Drumstick click sound
- `drumkit-hihat.wav` - Closed hi-hat (tight sound)
- `drumkit-hihat-open.wav` - Open hi-hat (sustained)

**Additional Samples for Phase 2 Development:**
- `drumkit-ride.wav` - Ride cymbal for rhythm patterns
- `drumkit-ride-bell.wav` - Ride bell for accents and highlights
- `drumkit-hihat-lose.wav` - Loose hi-hat (medium decay)
- `drumkit-hihat-vlose.wav` - Very loose hi-hat (long decay)
- `drumkit-cymbol-splash.wav` - Splash cymbal
- `drumkit-cymball-roll.wav` - Cymbal roll/crash

#### Sample Catalog
A complete catalog file (`drum_samples_catalog.json`) documents:
- Sample specifications and recommended usage
- Metronome mapping configurations
- Pre-built drum patterns for Phase 2
- Technical implementation details

## Development Status

### Phase 1 (Complete) ✅
- [x] Advanced metronome with 10 click sounds (4 synthetic + 6 real samples)
- [x] Real drum sample integration with acoustic kit
- [x] Multiple time signatures (4/4, 3/4, 6/8, 5/4, 7/8, etc.)
- [x] Precise tempo control (40-200 BPM)
- [x] Beat accenting and visualization
- [x] Transport controls (start/stop/pause)
- [x] Real-time parameter adjustment
- [x] Professional audio quality (44.1kHz samples)
- [x] Drum sample catalog system for Phase 2 expansion

### Phase 2 (Planned) 🚧
- [ ] Drum pattern library with ride cymbal and bell accents
- [ ] Bass line generation
- [ ] Piano chord accompaniment
- [ ] Key/chord progression management
- [ ] Practice session recording
- [ ] Tempo training features
- [ ] Multi-track mixing controls

**Ride Cymbal Integration:**
- **Steady Patterns**: Ride cymbal for consistent rhythm in jazz, rock, and blues
- **Bell Accents**: Ride bell for phrase endings and musical emphasis
- **Pattern Variations**: Multiple hi-hat options (tight, loose, very loose) for groove diversity

## Troubleshooting

### Audio Issues
- **No Click Sound**: Check system audio and volume settings
- **Timing Drift**: Ensure no other high-CPU applications running
- **Choppy Audio**: Verify audio buffer settings in system

### Performance
- **High CPU**: Guitar Buddy is optimized for real-time performance
- **Memory Usage**: Fixed allocation pattern prevents memory issues
- **Responsiveness**: 10ms GUI updates ensure smooth operation

## Contributing to Phase 2

Guitar Buddy is designed for expansion. Key areas for Phase 2 development:

1. **Musical Pattern Library**: Drum beats, bass lines, chord progressions
2. **Style Engine**: Genre-specific arrangement rules
3. **Practice Analytics**: Session tracking and progress metrics
4. **MIDI Integration**: External controller support
5. **Audio Recording**: Practice session capture and analysis

The existing Phase 1 architecture provides a solid foundation for all planned Phase 2 features, with Polyphonica's real-time engine handling the complex audio processing requirements.