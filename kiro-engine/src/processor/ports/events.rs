use std::marker::PhantomData;

use crate::controller::owned_data::Ref;
use crate::events::buffer::{EventsBuffer, Iter};
use crate::processor::ports::Output;

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
  pub fn buffer_mut(&self) -> &mut EventsBuffer {
    self.buffer.get_mut()
  }
}
