use ringbuf::Consumer;
use thiserror::Error;

use kiro_audio as audio;
use kiro_engine::{Engine, EngineConfig, Event, EventData, Renderer};
use kiro_midi::{self as midi, Driver, DriverSpec};
use kiro_time::SampleRate;

use crate::config::Config;

#[derive(Debug, Error)]
pub enum Error {
  #[error("Midi: {0}")]
  Midi(#[from] midi::drivers::Error),

  #[error("Audio: {0}")]
  Audio(#[from] audio::AudioError),
}

pub type Result<T> = core::result::Result<T, Error>;

pub struct SynthEngine {
  config: Config,
  _midi_driver: Driver,
  audio_driver: audio::AudioDriver,
  engine: Engine,
}

impl SynthEngine {
  pub fn new(config: Config) -> Result<Self> {
    let mut midi_driver = midi::drivers::create("kiro-synth")?;

    let (midi_track_producer, midi_track_consumer) =
      ringbuf::RingBuffer::new(config.midi.ringbuf_size).split();
    midi_driver.create_input(
      midi::InputConfig::new("track").with_all_sources(midi::Filter::default()),
      midi_track_producer,
    )?;

    let audio_output_config = audio::AudioDriver::output_config(&config.audio)?;

    let mut engine_config = EngineConfig::default();
    engine_config.audio_buffer_size = audio_output_config.buffer_size;
    engine_config.audio_output_channels = audio_output_config.channels;

    let mut engine = Engine::new(engine_config);
    // the renderer will always be available just after creating the engine so it is safe to unwrap
    let renderer = engine.take_renderer().unwrap();

    let studio_callack = StudioCallback {
      midi_consumer: midi_track_consumer,
      renderer,
    };

    let audio_driver = audio::AudioDriver::new(config.audio.clone(), studio_callack)?;

    Ok(Self {
      config,
      _midi_driver: midi_driver,
      audio_driver,
      engine,
    })
  }

  pub fn sample_rate(&self) -> SampleRate {
    self.audio_driver.sample_rate()
  }

  pub fn audio_input_channels(&self) -> usize {
    self.audio_driver.num_input_channels()
  }

  pub fn audio_output_channels(&self) -> usize {
    self.audio_driver.num_output_channels()
  }

  pub fn engine(&self) -> &Engine {
    &self.engine
  }

  pub fn engine_mut(&mut self) -> &mut Engine {
    &mut self.engine
  }

  pub fn start(&self) -> Result<()> {
    self.audio_driver.start().map_err(Error::Audio)
  }
}

struct StudioCallback {
  midi_consumer: Consumer<midi::Event>,
  renderer: Renderer,
}

impl StudioCallback {
  fn process_audio_input(&mut self, num_samples: usize) {
    for audio_input in self.renderer.get_audio_inputs() {
      audio_input.get_mut().fill_first(num_samples, 0.0);
    }
  }

  fn process_audio_output(&mut self, output: &mut [f32], channels: usize, num_samples: usize) {
    output.iter_mut().for_each(|s| *s = 0.0);
    let audio_outputs = self.renderer.get_audio_outputs();
    for (channel_index, output_buffer) in audio_outputs.iter().enumerate() {
      let mut output_offset = channel_index;
      for sample in output_buffer.iter().take(num_samples) {
        output[output_offset] = *sample;
        output_offset += channels;
      }
    }
  }

  fn process_midi_input(&mut self) {
    if let Some(buffer) = self.renderer.get_events_inputs().get(0) {
      let buffer = buffer.get_mut();
      buffer.clear();
      for midi_event in self.midi_consumer.iter() {
        let event = Event {
          timestamp: midi_event.timestamp,
          data: EventData::Midi(midi_event.message),
        };
        buffer.push(event).ok();
      }
    }
  }
}

impl audio::AudioHandler for StudioCallback {
  fn process(&mut self, output: &mut [f32], channels: usize) {
    let num_samples = output.len() / channels;

    self.process_audio_input(num_samples);
    self.process_midi_input();

    self.renderer.render(num_samples);

    self.process_audio_output(output, channels, num_samples);
  }
}
