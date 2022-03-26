use crate::endpoints::SourceId;
use crate::source_match::SourceMatches;

pub struct InputInfo {
  pub name: String,
  pub sources: SourceMatches,
  pub connected_sources: Vec<SourceId>,
}
