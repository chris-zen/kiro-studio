use crate::endpoints::SourceId;
use crate::source_match::MidiSourceMatches;

pub struct MidiInputInfo {
  pub name: String,
  pub sources: MidiSourceMatches,
  pub connected_sources: Vec<SourceId>,
}
