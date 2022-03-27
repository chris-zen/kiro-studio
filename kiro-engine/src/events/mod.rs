// mod allocator;
pub mod buffer;

use kiro_midi as midi;
use kiro_time::{BarsTime, ClockTime, Signature, Tempo, TicksTime};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransportMessage {
  Start,
  Stop,
  Continue,
  Loop,
  Tempo(Tempo),
  Signature(Signature),
  Position {
    bars: BarsTime,
    ticks: TicksTime,
    clock: ClockTime,
  },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventData {
  Transport(TransportMessage),
  Midi(midi::messages::Message),
  Automation(), // TODO
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Event {
  pub timestamp: midi::TimestampNanos,
  pub data: EventData,
}
