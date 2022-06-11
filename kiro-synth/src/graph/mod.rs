mod voice;

use thiserror::Error;

use kiro_engine::Engine;

use crate::graph::voice::VoiceNode;

#[derive(Debug, Error)]
pub enum Error {
  #[error("Engine: {0}")]
  Engine(#[from] kiro_engine::Error),
}

pub type Result<T> = core::result::Result<T, Error>;

pub struct SynthGraph {
  voices: Vec<VoiceNode>,
}

impl SynthGraph {
  pub fn try_new(engine: &mut Engine, sample_rate: u32, num_voices: usize) -> Result<Self> {
    let mut voices = Vec::new();

    for index in 0..num_voices {
      let name = format!("voice-{index}");
      let voice = VoiceNode::try_new(engine, name.as_str(), sample_rate)?;
      voices.push(voice);
    }

    Ok(Self { voices })
  }
}
