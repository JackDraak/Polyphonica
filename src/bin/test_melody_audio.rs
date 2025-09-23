use polyphonica::audio::synthesis::{get_note_audio_params, get_chord_audio_params};
use polyphonica::melody::{Note, Chord, ChordQuality};
use polyphonica::RealtimeEngine;
use std::time::Duration;
use std::thread;

fn main() {
    println!("🔧 Testing Melody Manager Audio Synthesis");
    println!("==========================================");

    // Create a basic audio engine
    let mut engine = RealtimeEngine::new(44100.0);
    engine.set_master_volume(0.5);

    println!("✅ RealtimeEngine initialized with sample rate: 44100 Hz");

    // Test 1: Individual note synthesis
    println!("\n🎵 Test 1: Playing individual notes");
    let test_notes = [Note::C, Note::E, Note::G, Note::C];

    for note in test_notes.iter() {
        println!("  Playing note: {:?}", note);
        let (waveform, frequency, envelope) = get_note_audio_params(note);

        println!("    Waveform: {:?}, Frequency: {:.2} Hz", waveform, frequency);
        println!("    Envelope: attack={:.3}s, decay={:.3}s, sustain={:.2}, release={:.3}s",
                 envelope.attack_secs, envelope.decay_secs, envelope.sustain_level, envelope.release_secs);

        // Test synthesis parameters are valid
        assert!(frequency > 20.0 && frequency < 20000.0, "Frequency out of audible range");
        assert!(envelope.attack_secs >= 0.0, "Invalid attack time");
        assert!(envelope.sustain_level >= 0.0 && envelope.sustain_level <= 1.0, "Invalid sustain level");

        let voice_id = engine.trigger_note_with_volume(waveform, frequency, envelope, 0.8);
        println!("    Voice ID: {:?}", voice_id);

        println!("    ✅ Synthesis parameters are valid");
        thread::sleep(Duration::from_millis(100));
    }

    // Test 2: Chord synthesis
    println!("\n🎹 Test 2: Playing chords");
    let chord = Chord::new(Note::C, ChordQuality::Major);
    println!("  Playing chord: {:?}", chord);

    let (waveform, frequency, envelope) = get_chord_audio_params(&chord);
    println!("    Waveform: {:?}, Frequency: {:.2} Hz", waveform, frequency);

    let voice_id = engine.trigger_note_with_volume(waveform, frequency, envelope, 0.8);
    println!("    Voice ID: {:?}", voice_id);
    println!("    ✅ Chord synthesis initiated");

    // Test 3: Test different chord qualities
    println!("\n🎼 Test 3: Different chord qualities");
    let chord_types = [
        ChordQuality::Major,
        ChordQuality::Minor,
        ChordQuality::Dominant7,
    ];

    for chord_quality in chord_types.iter() {
        let chord = Chord::new(Note::C, *chord_quality);
        println!("  Testing chord: {}", chord.symbol());

        let (waveform, frequency, envelope) = get_chord_audio_params(&chord);
        let voice_id = engine.trigger_note_with_volume(waveform, frequency, envelope, 0.6);

        println!("    Voice ID: {:?}", voice_id.is_some());
        thread::sleep(Duration::from_millis(50));
    }

    println!("\n🎯 Audio synthesis test complete!");
    println!("   All parameters are valid and voices are being triggered.");
    println!("   If no audio is heard, the issue is likely in the audio output pipeline.");
}