use kiro_midi_core::messages::Message as MidiMessage;

use crate::time::{BarsTime, ClockTime, Signature, Tempo, TicksTime};

mod allocator;
// mod bplus_tree;
mod events_buffer;
mod queue;

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

pub enum EventData {
  Transport(TransportMessage),
  Midi(MidiMessage),
  Automation(), // TODO
}

pub struct Event {
  timestamp: TicksTime,
  data: EventData,
}
