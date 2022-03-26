use ringbuf::RingBuffer;

use crate::{Controller, EngineConfig, Renderer};

pub struct Engine {
  controller: Controller,
  renderer: Renderer,
}

impl Engine {
  pub fn new() -> Self {
    Self::with_config(EngineConfig::default())
  }

  pub fn with_config(config: EngineConfig) -> Self {
    let ring_buffer_capacity = config.ring_buffer_capacity;
    let (forward_tx, forward_rx) = RingBuffer::new(ring_buffer_capacity).split();
    let (backward_tx, backward_rx) = RingBuffer::new(ring_buffer_capacity).split();
    let controller = Controller::new(forward_tx, backward_rx, config.clone());
    let renderer = Renderer::new(backward_tx, forward_rx, config);

    Self {
      controller,
      renderer,
    }
  }

  pub fn split(self) -> (Controller, Renderer) {
    let Self {
      controller,
      renderer,
    } = self;
    (controller, renderer)
  }
}
