use crate::endpoints::EndpointId;
use crate::protocol::messages::Message;
use std::fmt::Formatter;

pub type TimestampNanos = u64;

#[derive(Clone, PartialEq)]
pub struct MidiEvent {
  pub timestamp: TimestampNanos,
  pub endpoint: EndpointId,
  pub message: Message,
}

impl std::fmt::Debug for MidiEvent {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "[{:08x}] {:016} {:?}",
      self.endpoint, self.timestamp, self.message
    )
  }
}
