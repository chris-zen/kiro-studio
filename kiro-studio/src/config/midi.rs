#[derive(Debug, Clone)]
pub struct MidiConfig {
  pub endpoints: Vec<EndpointConfig>,
  pub ringbuf_size: usize,
}

impl Default for MidiConfig {
  fn default() -> Self {
    Self {
      endpoints: Default::default(),
      ringbuf_size: 4096,
    }
  }
}

#[derive(Debug, Clone, Default)]
pub struct EndpointConfig {}
