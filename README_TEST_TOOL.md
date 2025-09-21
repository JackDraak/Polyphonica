# Polyphonica Test Tool 🎵🧪

A comprehensive command-line tool for testing, demonstrating, and validating the Polyphonica audio synthesis library.

## Overview

The `polyphonica-test` tool provides an interactive way to:
- **Test library features** with real audio output
- **Play audio immediately** through your system speakers with volume control
- **Demonstrate polyphonic capabilities** up to 16 concurrent voices
- **Export audio files** for verification and sharing
- **Report issues** to library developers with structured feedback
- **Run comprehensive test suites** for quality assurance

## 🎵 Audio Playback Features

- **Real-time playback**: All commands support `--play` flag for immediate audio output
- **Volume control**: Adjustable volume from 0.0 (silent) to 1.0 (maximum)
- **Automatic device detection**: Uses your system's default audio output
- **Safe fallback**: Graceful degradation if audio hardware is unavailable

## Installation

From the project root directory:

```bash
cargo build --bin polyphonica-test
```

## Usage

### Basic Waveform Generation

Generate individual waveforms and export as WAV files:

```bash
# Generate and play a 440Hz sine wave immediately
polyphonica-test wave sine --frequency 440 --duration 2.0 --play

# Generate with custom volume (0.0-1.0)
polyphonica-test wave sine --frequency 440 --duration 2.0 --play --volume 0.3

# Generate and save to file without playback
polyphonica-test wave sine --frequency 440 --duration 2.0 --output sine_440.wav

# Generate, save, AND play
polyphonica-test wave sawtooth --frequency 220 --duration 1.5 --output sawtooth.wav --play --volume 0.5

# Quick tests of all waveform types with playback
polyphonica-test wave sine --frequency 440 --play --volume 0.4
polyphonica-test wave square --frequency 440 --play --volume 0.4
polyphonica-test wave sawtooth --frequency 440 --play --volume 0.4
polyphonica-test wave triangle --frequency 440 --play --volume 0.4

# Test new waveforms
polyphonica-test wave pulse --frequency 440 --duty-cycle 0.25 --play --volume 0.4
polyphonica-test wave noise --frequency 440 --play --volume 0.2
```

### ADSR Envelope Testing

Test amplitude envelope shaping with customizable parameters:

```bash
# Piano-like attack/decay/sustain/release with immediate playback
polyphonica-test envelope sine \
  --attack 0.01 --decay 0.3 --sustain 0.3 --release 1.0 \
  --frequency 261.63 --duration 3.0 --play --volume 0.6

# Organ-like sustained tone
polyphonica-test envelope square \
  --attack 0.1 --decay 0.0 --sustain 0.8 --release 0.1 \
  --frequency 440 --duration 2.0 --play --volume 0.4

# Plucked string simulation with file output
polyphonica-test envelope triangle \
  --attack 0.01 --decay 0.5 --sustain 0.0 --release 0.0 \
  --frequency 329.63 --duration 1.0 --output pluck_e.wav --play --volume 0.7
```

**ADSR Parameters:**
- `--attack`: Time to reach peak amplitude (seconds)
- `--decay`: Time to decay to sustain level (seconds)
- `--sustain`: Sustained amplitude level (0.0-1.0)
- `--release`: Time to fade to silence (seconds)

### Polyphonic Compositions

Generate multi-voice compositions demonstrating timeline mixing:

```bash
# 4-voice C Major chord with immediate playback
polyphonica-test poly \
  --voices 4 --composition chord --duration 3.0 --play --volume 0.5

# 8-voice ascending arpeggio
polyphonica-test poly \
  --voices 8 --composition arpeggio --duration 4.0 --play --volume 0.4

# 7-note C Major scale
polyphonica-test poly \
  --voices 7 --composition scale --duration 5.0 --play --volume 0.6

# Experimental random composition with maximum 16 voices
polyphonica-test poly \
  --voices 16 --composition random --duration 6.0 --output experimental.wav --play --volume 0.3
```

**💡 Pro Tip**: Start with lower volumes (0.3-0.5) for polyphonic compositions as multiple voices can be quite loud!

**Composition Types:**
- `chord`: Simultaneous harmonic notes (C Major with extensions)
- `arpeggio`: Sequential notes with overlap
- `scale`: Musical scale progression
- `random`: Experimental frequencies and timing

### Sample-Based Synthesis

Load and play custom WAV samples with pitch shifting:

```bash
# Load and play a sample at its original pitch
polyphonica-test sample load samples/drums/kick.wav --play --volume 0.7

# Pitch shift a sample to different frequencies
polyphonica-test sample load samples/drums/kick.wav \
  --frequency 80 --duration 1.0 --play --volume 0.6

# Create a complete sample event with envelope
polyphonica-test sample-event samples/piano/C4.wav \
  --frequency 440 --duration 2.0 \
  --attack 0.1 --decay 0.3 --sustain 0.7 --release 0.8 \
  --play --volume 0.5

# Export processed sample
polyphonica-test sample load samples/vocals/voice.wav \
  --frequency 220 --duration 3.0 --output processed_voice.wav
```

### Sample Catalog Management

Organize and discover sample collections:

```bash
# Scan directory for samples and create catalog
polyphonica-test catalog scan samples/

# Search for specific sample types
polyphonica-test catalog search "drum"
polyphonica-test catalog search "kick"
polyphonica-test catalog search "piano"

# List all available collections
polyphonica-test catalog list

# Audition samples before use
polyphonica-test catalog audition samples/drums/snare.wav --play --volume 0.6

# Show detailed sample information
polyphonica-test catalog info samples/piano/C4.wav
```

**Sample Catalog Features:**
- **Automatic Discovery**: Recursively scans directory trees
- **Metadata Extraction**: Duration, sample rate, and format detection
- **Search Functionality**: Find samples by name, tags, or collection
- **JSON Storage**: Persistent catalog with timestamps and metadata

### Comprehensive Test Suite

Run the full test suite to validate library functionality:

```bash
# Generate complete test battery with audio demo at the end
polyphonica-test test-suite --output test_results/ --play --volume 0.4

# Silent test suite (files only)
polyphonica-test test-suite --output test_results/

# High quality tests with playback
polyphonica-test test-suite --sample-rate 48000 --output high_quality_tests/ --play --volume 0.3
```

**Generated Test Files:**
- `sine_440hz.wav`, `square_440hz.wav`, `sawtooth_440hz.wav`, `triangle_440hz.wav`
- `pulse_25_duty.wav`, `pulse_50_duty.wav`, `pulse_75_duty.wav`
- `white_noise.wav`
- `envelope_piano.wav`, `envelope_organ.wav`, `envelope_pluck.wav`
- `frequency_sweep.wav` (110Hz → 880Hz sawtooth)
- `polyphonic_chord.wav`, `polyphonic_arpeggio.wav`, `polyphonic_scale.wav`
- `sample_tests/` (if samples directory exists)

### Issue Reporting System

Report bugs or unexpected behavior to library developers:

```bash
# Report a simple issue
cargo run --bin polyphonica-test report-issue \
  "Sawtooth wave produces clicking artifacts at high frequencies"

# Detailed issue report
cargo run --bin polyphonica-test report-issue \
  "ADSR envelope not working correctly" \
  --expected "Smooth attack phase over 0.5 seconds" \
  --actual "Immediate jump to full amplitude" \
  --parameters "attack_secs: 0.5, waveform: Sine, frequency: 880Hz"
```

**Issue reports are saved to `issue_reports.json` with:**
- Timestamp
- Description and context
- Expected vs actual behavior
- Test parameters that triggered the issue
- Library version information

## Advanced Examples

### Complex Polyphonic Demo

```bash
# 12-voice chord with mixed waveforms and envelope shaping
cargo run --bin polyphonica-test polyphonic \
  --voices 12 --composition chord \
  --duration 8.0 --sample-rate 48000 \
  -o complex_harmony.wav
```

### Frequency Sweep Analysis

```bash
# Generate individual waveforms across frequency range for analysis
for freq in 110 220 440 880 1760; do
  cargo run --bin polyphonica-test generate sawtooth \
    -f $freq -d 1.0 -o "sawtooth_${freq}hz.wav"
done
```

### Envelope Comparison Study

```bash
# Compare different envelope shapes on same frequency
cargo run --bin polyphonica-test envelope sine -f 440 \
  --attack 0.0 --decay 0.0 --sustain 1.0 --release 0.0 -o raw_sine.wav

cargo run --bin polyphonica-test envelope sine -f 440 \
  --attack 0.1 --decay 0.2 --sustain 0.7 --release 0.5 -o shaped_sine.wav
```

## Performance Notes

- **Voice Limit**: Polyphonic compositions are capped at 16 voices for performance
- **Memory Usage**: ~176KB per second of 44.1kHz audio (per voice)
- **File Sizes**: WAV files are ~172KB per second of 44.1kHz mono audio
- **Real-time**: Generation is much faster than real-time playback

## Development Workflow

### For Library Developers

1. **Test new features**: Use the tool to validate changes
2. **Generate test cases**: Create audio files for manual verification
3. **Monitor issue reports**: Check `issue_reports.json` regularly
4. **Performance testing**: Use test suite for benchmarking

### For Library Users

1. **Validate installation**: Run test suite to ensure everything works
2. **Explore capabilities**: Try different waveforms and compositions
3. **Report problems**: Use issue reporting for bug reports
4. **Create examples**: Generate audio for documentation or demos

## Troubleshooting

### Common Issues

**"No audio output"**: Check that WAV files are being created in the specified location.

**"Validation failed"**: Ensure parameters are within valid ranges:
- Frequency: 0-20,000 Hz
- Duration: ≥ 0 seconds
- Sample rate: 1-192,000 Hz
- Sustain level: 0.0-1.0

**"Distorted audio"**: Multiple overlapping voices can cause clipping. Try:
- Reducing voice count
- Using lower sustain levels
- Different waveform combinations

### Getting Help

1. **Run test suite**: `cargo run --bin polyphonica-test test-suite`
2. **Check issue reports**: Review `issue_reports.json` for similar problems
3. **Report new issues**: Use the issue reporting feature
4. **Verify installation**: Ensure all dependencies are available

## File Formats

- **Output**: 16-bit WAV files, mono channel
- **Sample rates**: Configurable (default 44.1kHz)
- **Bit depth**: 16-bit signed integer
- **Duration**: Limited only by available disk space

## Contributing

When reporting issues or contributing to the test tool:

1. Include specific parameters that cause problems
2. Provide both expected and actual behavior
3. Test with the comprehensive test suite
4. Consider adding new test cases for edge conditions

The test tool is designed to grow with the library - new features should include corresponding test capabilities!