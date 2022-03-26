use crate::filter::Filter;
use crate::source_match::{SourceMatch, SourceMatches};

#[derive(Debug, Clone)]
pub struct InputConfig {
  pub name: String,
  pub sources: SourceMatches,
}

impl InputConfig {
  pub fn new<N>(name: N) -> Self
  where
    N: Into<String>,
  {
    Self {
      name: name.into(),
      sources: SourceMatches::default(),
    }
  }

  pub fn with_source<M>(mut self, source_match: M, filter: Filter) -> Self
  where
    M: Into<SourceMatch>,
  {
    self.sources.add_source(source_match, filter);
    self
  }

  pub fn with_all_sources(mut self, filter: Filter) -> Self {
    self
      .sources
      .add_source(SourceMatch::regex(".*").expect("regex"), filter);
    self
  }
}
