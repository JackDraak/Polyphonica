/// Audio stream management and CPAL integration
///
/// This module provides cross-platform audio stream setup and management
/// using the CPAL library. It abstracts audio device selection, format
/// negotiation, and real-time audio callback handling.
use crate::RealtimeEngine;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig as CpalStreamConfig};
use std::sync::{Arc, Mutex};

/// Shared application state for audio processing
#[derive(Clone)]
pub struct AppState {
    pub engine: Arc<Mutex<RealtimeEngine>>,
}

impl AppState {
    pub fn new(engine: Arc<Mutex<RealtimeEngine>>) -> Self {
        Self { engine }
    }
}

/// Audio stream configuration and management
pub struct AudioStream {
    _stream: Stream, // Keep stream alive
}

impl AudioStream {
    /// Setup CPAL audio stream for real-time metronome output
    pub fn setup_audio_stream(
        app_state: AppState,
    ) -> Result<AudioStream, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or("No audio output device available")?;

        let config = device.default_output_config()?;

        println!("🎸 Guitar Buddy Audio System");
        println!("Audio device: {}", device.name()?);
        println!("Sample rate: {} Hz", config.sample_rate().0);
        println!("Channels: {}", config.channels());

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                Self::create_stream::<f32>(&device, &config.into(), app_state)
            }
            cpal::SampleFormat::I16 => {
                Self::create_stream::<i16>(&device, &config.into(), app_state)
            }
            cpal::SampleFormat::U16 => {
                Self::create_stream::<u16>(&device, &config.into(), app_state)
            }
            _ => return Err("Unsupported audio format".into()),
        }?;

        stream.play()?;
        Ok(AudioStream { _stream: stream })
    }

    /// Create audio stream for specific sample format
    fn create_stream<T>(
        device: &Device,
        config: &CpalStreamConfig,
        app_state: AppState,
    ) -> Result<Stream, Box<dyn std::error::Error>>
    where
        T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
    {
        let channels = config.channels as usize;

        let stream = device.build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                let mut f32_buffer = vec![0.0f32; data.len()];

                // Process audio with the engine
                {
                    let mut engine = app_state.engine.lock().unwrap();
                    if channels == 1 {
                        engine.process_buffer(&mut f32_buffer);
                    } else {
                        engine.process_stereo_buffer(&mut f32_buffer);
                    }
                }

                // Convert back to target format
                for (dst, &src) in data.iter_mut().zip(f32_buffer.iter()) {
                    *dst = T::from_sample(src);
                }
            },
            |err| eprintln!("Audio stream error: {}", err),
            None,
        )?;

        Ok(stream)
    }
}

/// Simplified audio stream configuration
#[derive(Debug, Clone)]
pub struct PolyphonicaStreamConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub buffer_size: Option<u32>,
}

impl Default for PolyphonicaStreamConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            channels: 2,
            buffer_size: None,
        }
    }
}

/// Audio stream builder for more flexible configuration
pub struct AudioStreamBuilder {
    config: PolyphonicaStreamConfig,
    device_name: Option<String>,
}

impl Default for AudioStreamBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioStreamBuilder {
    pub fn new() -> Self {
        Self {
            config: PolyphonicaStreamConfig::default(),
            device_name: None,
        }
    }

    pub fn with_sample_rate(mut self, sample_rate: u32) -> Self {
        self.config.sample_rate = sample_rate;
        self
    }

    pub fn with_channels(mut self, channels: u16) -> Self {
        self.config.channels = channels;
        self
    }

    pub fn with_buffer_size(mut self, buffer_size: u32) -> Self {
        self.config.buffer_size = Some(buffer_size);
        self
    }

    pub fn with_device_name(mut self, device_name: String) -> Self {
        self.device_name = Some(device_name);
        self
    }

    pub fn build_with_callback<F>(
        self,
        _callback: F,
    ) -> Result<AudioStream, Box<dyn std::error::Error>>
    where
        F: FnMut(&mut [f32]) + Send + 'static,
    {
        // Implementation would create stream with custom callback
        Err("Custom callback streams not yet implemented".into())
    }
}

/// Get available audio devices
pub fn get_audio_devices() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let mut devices = Vec::new();

    for device in host.output_devices()? {
        if let Ok(name) = device.name() {
            devices.push(name);
        }
    }

    Ok(devices)
}

/// Get default audio device information
pub fn get_default_audio_device_info() -> Result<(String, u32, u16), Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("No audio output device available")?;

    let name = device.name()?;
    let config = device.default_output_config()?;

    Ok((name, config.sample_rate().0, config.channels()))
}
