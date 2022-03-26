use crate::filter::MidiFilter;
use crate::source_match::{MidiSourceMatch, MidiSourceMatches};

#[derive(Debug, Clone)]
pub struct MidiInputConfig {
  pub name: String,
  pub sources: MidiSourceMatches,
}

impl MidiInputConfig {
  pub fn new<N>(name: N) -> Self
  where
    N: Into<String>,
  {
    Self {
      name: name.into(),
      sources: MidiSourceMatches::default(),
    }
  }

  pub fn with_source<M>(mut self, source_match: M, filter: MidiFilter) -> Self
  where
    M: Into<MidiSourceMatch>,
  {
    self.sources.add_source(source_match, filter);
    self
  }

  pub fn with_all_sources(mut self, filter: MidiFilter) -> Self {
    self
      .sources
      .add_source(MidiSourceMatch::regex(".*").expect("regex"), filter);
    self
  }
}
