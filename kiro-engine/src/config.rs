#[derive(Debug, Clone)]
pub struct EngineConfig {
  pub ring_buffer_capacity: usize,
  pub audio_buffer_size: usize,
  pub event_buffer_size: usize,
}

impl EngineConfig {
  const DEFAULT_RING_BUFFER_CAPACITY: usize = 1024;
  const DEFAULT_AUDIO_BUFFER_SIZE: usize = 256;
  const DEFAULT_EVENT_BUFFER_SIZE: usize = 4096;
}

impl Default for EngineConfig {
  fn default() -> Self {
    Self {
      ring_buffer_capacity: EngineConfig::DEFAULT_RING_BUFFER_CAPACITY,
      audio_buffer_size: EngineConfig::DEFAULT_AUDIO_BUFFER_SIZE,
      event_buffer_size: EngineConfig::DEFAULT_EVENT_BUFFER_SIZE,
    }
  }
}
