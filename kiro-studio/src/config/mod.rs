pub mod midi;

use crate::config::midi::MidiConfig;

#[derive(Debug, Clone, Default)]
pub struct Config {
  pub midi: MidiConfig,
}
