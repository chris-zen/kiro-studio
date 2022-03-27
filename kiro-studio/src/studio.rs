use ringbuf::Consumer;

use kiro_audio as audio;
use kiro_engine::{Controller, Engine, EngineConfig, Renderer};
use kiro_engine::events::{Event, EventData};
use kiro_midi::{self as midi, Driver, DriverSpec};

use crate::config::Config;
use crate::errors::{Error, Result};

pub struct Studio {
  config: Config,
  _midi_driver: Driver,
  _audio_driver: audio::AudioDriver,
  controller: Controller,
}

impl Studio {
  pub fn new(config: Config) -> Result<Self> {
    let mut midi_driver = midi::drivers::create("kiro-studio")?;

    let (midi_track_producer, midi_track_consumer) = ringbuf::RingBuffer::new(config.midi.ringbuf_size).split();
    midi_driver.create_input(midi::InputConfig::new("track").with_all_sources(midi::Filter::default()), midi_track_producer)?;

    let audio_config = audio::AudioConfig::default();
    let sample_rate = audio_config.sample_rate as f32;

    let mut engine_config = EngineConfig::default();
    engine_config.audio_buffer_size = audio_config.buffer_size;

    let engine = Engine::with_config(engine_config);
    let (controller, renderer) = engine.split();

    let studio_callack = StudioCallback {
      midi_consumer: midi_track_consumer,
      renderer,
    };

    let audio_driver = audio::AudioDriver::new(audio_config, studio_callack)?;

    Ok(Self {
      config,
      _midi_driver: midi_driver,
      _audio_driver: audio_driver,
      controller,
    })
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
