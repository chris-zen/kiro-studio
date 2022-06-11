use std::marker::PhantomData;

use crate::processor::ports::Output;
use crate::rendering::buffers::events::{EventsBuffer, Iter};
use crate::rendering::owned_data::Ref;

#[derive(Debug)]
pub struct EventsPort<IO> {
  buffer: Ref<EventsBuffer>,
  _mode: PhantomData<IO>,
}

impl<IO> EventsPort<IO> {
  pub fn new(buffer: Ref<EventsBuffer>) -> Self {
    Self {
      buffer,
      _mode: PhantomData,
    }
  }

  pub fn buffer(&self) -> &EventsBuffer {
    &self.buffer
  }

  #[inline]
  pub fn iter(&self) -> Iter<'_> {
    self.buffer.iter()
  }
}

impl EventsPort<Output> {
  pub fn buffer_mut(&mut self) -> &mut EventsBuffer {
    self.buffer.get_mut()
  }
}
