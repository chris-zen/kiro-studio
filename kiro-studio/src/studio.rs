use kiro_midi as midi;

use crate::config::Config;
use crate::errors::{Error, Result};

pub struct Studio {
  config: Config,
}

impl Studio {
  pub fn new(config: Config) -> Result<Self> {
    Ok(Self { config })
  }
}
