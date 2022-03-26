use regex::Regex;

use crate::endpoints::SourceId;
use crate::filter::MidiFilter;

#[derive(Debug, Clone)]
pub enum MidiSourceMatch {
  Id(SourceId),
  Name(String),
  Regex(Regex),
}

impl MidiSourceMatch {
  pub fn regex(regex: &str) -> Result<Self, regex::Error> {
    Regex::new(regex).map(Self::Regex)
  }

  pub(crate) fn matches(&self, source_id: SourceId, source_name: &str) -> bool {
    match self {
      Self::Id(id) => source_id == *id,
      Self::Name(name) => source_name == name.as_str(),
      Self::Regex(regex) => regex.is_match(source_name),
    }
  }
}

impl From<SourceId> for MidiSourceMatch {
  fn from(source_id: SourceId) -> Self {
    Self::Id(source_id)
  }
}

impl From<&str> for MidiSourceMatch {
  fn from(name: &str) -> Self {
    Self::Name(name.to_string())
  }
}

#[derive(Debug, Clone, Default)]
pub struct MidiSourceMatches(Vec<(MidiSourceMatch, MidiFilter)>);

impl MidiSourceMatches {
  pub fn new(matches: Vec<(MidiSourceMatch, MidiFilter)>) -> Self {
    Self(matches)
  }

  #[must_use]
  pub fn with_source<M>(mut self, source_match: M, filter: MidiFilter) -> Self
  where
    M: Into<MidiSourceMatch>,
  {
    self.add_source(source_match.into(), filter);
    self
  }

  pub fn add_source<M>(&mut self, source_match: M, filter: MidiFilter)
  where
    M: Into<MidiSourceMatch>,
  {
    self.0.push((source_match.into(), filter));
  }

  pub fn match_filter(&self, id: SourceId, name: &str) -> Option<MidiFilter> {
    self
      .0
      .iter()
      .find_map(|(source_match, filter)| source_match.matches(id, name).then(|| *filter))
  }

  pub fn match_index(&self, id: SourceId, name: &str) -> Option<usize> {
    self
      .0
      .iter()
      .position(|(source_match, _)| source_match.matches(id, name))
  }
}
