#[derive(Debug, Clone)]
pub struct AudioConfig {
  pub sample_rate: u32,
  pub buffer_size: usize,
}

impl AudioConfig {
  pub const DEFAULT_SAMPLE_RATE: u32 = 44_100;
  pub const DEFAULT_BUFFER_SIZE: usize = 256;
}

impl Default for AudioConfig {
  fn default() -> Self {
    Self {
      sample_rate: AudioConfig::DEFAULT_SAMPLE_RATE,
      buffer_size: AudioConfig::DEFAULT_BUFFER_SIZE,
    }
  }
}
