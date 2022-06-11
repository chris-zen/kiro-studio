use std::marker::PhantomData;
use std::ops::Deref;

use crate::processor::ports::{Input, Output};
use crate::rendering::buffers::audio::AudioBuffer;
use crate::rendering::owned_data::Ref;

pub struct AudioRenderBuffer<IO> {
  num_samples: usize,
  buffer: Ref<AudioBuffer>,
  _mode: PhantomData<IO>,
}

impl<IO> AudioRenderBuffer<IO> {
  pub fn new(num_samples: usize, buffer: Ref<AudioBuffer>) -> Self {
    Self {
      num_samples,
      buffer,
      _mode: PhantomData,
    }
  }

  pub fn len(&self) -> usize {
    self.num_samples
  }

  pub fn is_empty(&self) -> bool {
    self.num_samples == 0
  }
}

impl AudioRenderBuffer<Input> {
  pub fn iter(&self) -> impl Iterator<Item = &f32> {
    self.buffer.deref().iter().take(self.num_samples)
  }

  pub fn as_slice(&self) -> &[f32] {
    self.buffer.deref().as_slice()[0..self.num_samples].as_ref()
  }
}

impl AudioRenderBuffer<Output> {
  pub fn fill(&mut self, value: f32) {
    self.buffer.get_mut().fill_first(self.num_samples, value);
  }

  pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut f32> {
    self.buffer.get_mut().iter_mut().take(self.num_samples)
  }

  pub fn as_mut_slice(&mut self) -> &mut [f32] {
    self.buffer.get_mut().as_mut_slice()[0..self.num_samples].as_mut()
  }
}

#[derive(Debug, Clone)]
pub struct AudioPort<IO> {
  num_samples: usize,
  channels: Vec<Ref<AudioBuffer>>,
  _mode: PhantomData<IO>,
}

impl<IO> AudioPort<IO> {
  pub fn new(channels: Vec<Ref<AudioBuffer>>) -> Self {
    Self {
      num_samples: 0,
      channels,
      _mode: PhantomData,
    }
  }

  pub(crate) fn set_num_samples(&mut self, num_samples: usize) {
    self.num_samples = num_samples
  }

  pub fn len(&self) -> usize {
    self.channels.len()
  }

  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }
}

impl AudioPort<Input> {
  pub fn channel(&self, index: usize) -> AudioRenderBuffer<Input> {
    AudioRenderBuffer::new(self.num_samples, self.channels[index].clone())
  }
}

impl AudioPort<Output> {
  pub fn channel_mut(&self, index: usize) -> AudioRenderBuffer<Output> {
    AudioRenderBuffer::new(self.num_samples, self.channels[index].clone())
  }
}
