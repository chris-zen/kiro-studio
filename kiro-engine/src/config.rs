#[derive(Debug, Clone)]
pub struct EngineConfig {
  pub ring_buffer_capacity: usize,
  pub audio_buffer_size: usize,
  pub audio_input_channels: usize,
  pub audio_output_channels: usize,
  pub event_buffer_size: usize,
}

impl EngineConfig {
  const DEFAULT_RING_BUFFER_CAPACITY: usize = 1024;
  const DEFAULT_AUDIO_BUFFER_SIZE: usize = 256;
  const DEFAULT_AUDIO_INPUT_CHANNELS: usize = 2;
  const DEFAULT_AUDIO_OUTPUT_CHANNELS: usize = 2;
  const DEFAULT_EVENT_BUFFER_SIZE: usize = 4096;
}

impl Default for EngineConfig {
  fn default() -> Self {
    Self {
      ring_buffer_capacity: Self::DEFAULT_RING_BUFFER_CAPACITY,
      audio_buffer_size: Self::DEFAULT_AUDIO_BUFFER_SIZE,
      audio_input_channels: Self::DEFAULT_AUDIO_INPUT_CHANNELS,
      audio_output_channels: Self::DEFAULT_AUDIO_OUTPUT_CHANNELS,
      event_buffer_size: Self::DEFAULT_EVENT_BUFFER_SIZE,
    }
  }
}
