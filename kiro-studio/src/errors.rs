use thiserror::Error;

use kiro_audio as audio;
use kiro_midi as midi;

#[derive(Debug, Error)]
pub enum Error {
  #[error("Midi: {0}")]
  Midi(#[from] midi::drivers::Error),

  #[error("Audio: {0}")]
  Audio(#[from] audio::AudioError),
}

pub type Result<T> = core::result::Result<T, Error>;
