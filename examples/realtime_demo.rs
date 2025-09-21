/// Real-time Polyphonic Synthesis Engine Demonstration
///
/// This example shows how to use Polyphonica's real-time engine for streaming audio synthesis.
/// Perfect for integration into applications requiring procedural music generation.

use polyphonica::{RealtimeEngine, Waveform, AdsrEnvelope};
use std::time::Duration;

fn main() {
    println!("🎵 Polyphonica Real-Time Engine Demo");
    println!("====================================");

    // Create a real-time synthesis engine
    let mut engine = RealtimeEngine::new(44100.0);

    // Define a musical envelope
    let envelope = AdsrEnvelope {
        attack_secs: 0.1,   // Quick attack
        decay_secs: 0.2,    // Medium decay
        sustain_level: 0.6, // 60% sustain
        release_secs: 0.5,  // Smooth release
    };

    println!("🚀 Engine initialized with {} max voices", polyphonica::MAX_VOICES);

    // Trigger individual notes
    println!("\n🎹 Triggering individual notes...");
    let voice_a = engine.trigger_note(Waveform::Sine, 440.0, envelope.clone()).unwrap();
    let voice_c = engine.trigger_note(Waveform::Square, 523.25, envelope.clone()).unwrap();

    println!("   ♪ A4 (440Hz) - Voice ID: {}", voice_a);
    println!("   ♪ C5 (523Hz) - Voice ID: {}", voice_c);
    println!("   Active voices: {}", engine.get_active_voice_count());

    // Process some audio buffers
    let mut buffer = vec![0.0; 1024];
    for frame in 0..10 {
        engine.process_buffer(&mut buffer);

        let max_amplitude = buffer.iter().map(|s| s.abs()).fold(0.0, f32::max);
        println!("   Frame {}: max amplitude = {:.3}", frame, max_amplitude);

        // Simulate real-time processing delay
        std::thread::sleep(Duration::from_millis(20));
    }

    // Trigger a chord
    println!("\n🎼 Triggering C Major chord...");
    let chord_notes = &[
        (Waveform::Sine, 261.63),    // C4
        (Waveform::Sine, 329.63),    // E4
        (Waveform::Sine, 392.00),    // G4
    ];

    let chord_voices = engine.trigger_chord(chord_notes, envelope.clone());
    println!("   Chord voice IDs: {:?}", chord_voices);
    println!("   Total active voices: {}", engine.get_active_voice_count());

    // Process chord audio
    for frame in 0..5 {
        engine.process_buffer(&mut buffer);
        let max_amplitude = buffer.iter().map(|s| s.abs()).fold(0.0, f32::max);
        println!("   Chord frame {}: max amplitude = {:.3}", frame, max_amplitude);
        std::thread::sleep(Duration::from_millis(20));
    }

    // Test parameter updates
    println!("\n🎛️  Testing real-time parameter updates...");
    engine.set_master_volume(0.5);
    println!("   Master volume set to 50%");

    engine.set_voice_frequency(voice_a, 880.0);
    println!("   Voice {} frequency changed to 880Hz", voice_a);

    // Process with new parameters
    for frame in 0..3 {
        engine.process_buffer(&mut buffer);
        let max_amplitude = buffer.iter().map(|s| s.abs()).fold(0.0, f32::max);
        println!("   Updated frame {}: max amplitude = {:.3}", frame, max_amplitude);
        std::thread::sleep(Duration::from_millis(20));
    }

    // Release notes gradually
    println!("\n🔇 Releasing notes...");
    engine.release_note(voice_a);
    println!("   Released voice {}", voice_a);
    std::thread::sleep(Duration::from_millis(100));

    engine.release_all_notes();
    println!("   Released all remaining notes");

    // Process release phase
    for frame in 0..10 {
        engine.process_buffer(&mut buffer);
        let active_voices = engine.get_active_voice_count();
        let max_amplitude = buffer.iter().map(|s| s.abs()).fold(0.0, f32::max);

        println!("   Release frame {}: {} voices, amplitude = {:.3}",
                frame, active_voices, max_amplitude);

        if active_voices == 0 {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    // Test voice stealing
    println!("\n⚡ Testing voice stealing...");
    for i in 0..polyphonica::MAX_VOICES + 5 {
        let freq = 220.0 + (i as f32 * 50.0);
        let voice_id = engine.trigger_note(Waveform::Triangle, freq, envelope.clone());
        if let Some(id) = voice_id {
            println!("   Voice {}: {}Hz (ID: {})", i + 1, freq, id);
        }
    }

    println!("   Final active voices: {} (max: {})",
             engine.get_active_voice_count(), polyphonica::MAX_VOICES);

    // Panic stop test
    println!("\n🛑 Testing panic stop...");
    engine.stop_all_notes();
    println!("   All voices stopped immediately");
    println!("   Active voices: {}", engine.get_active_voice_count());

    // Final silence verification
    engine.process_buffer(&mut buffer);
    let silence_amplitude = buffer.iter().map(|s| s.abs()).fold(0.0, f32::max);
    println!("   Silence verification: max amplitude = {:.6}", silence_amplitude);

    println!("\n✅ Real-time engine demonstration complete!");
    println!("🎵 Engine is ready for real-time audio integration");

    // Performance summary
    println!("\n📊 Engine Capabilities:");
    println!("   • {} concurrent voices with voice stealing", polyphonica::MAX_VOICES);
    println!("   • Lock-free parameter updates");
    println!("   • Zero-allocation audio processing");
    println!("   • CPAL-compatible buffer interface");
    println!("   • All waveforms + samples supported");
    println!("   • Thread-safe shared access ready");
}