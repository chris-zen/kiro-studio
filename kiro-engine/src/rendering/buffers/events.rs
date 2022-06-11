use kiro_midi as midi;
use kiro_time::{BarsTime, ClockTime, Signature, Tempo, TicksTime};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Event {
  pub timestamp: midi::TimestampNanos,
  pub data: EventData,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventData {
  Transport(TransportMessage),
  Midi(midi::messages::Message),
  Automation(), // TODO
}

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

pub struct EventsBuffer {
  data: Vec<Event>,
  sorted: bool,
}

impl EventsBuffer {
  pub fn with_capacity(capacity: usize) -> Self {
    Self {
      data: Vec::with_capacity(capacity),
      sorted: true,
    }
  }

  pub fn capacity(&self) -> usize {
    self.data.capacity()
  }

  pub fn len(&self) -> usize {
    self.data.len()
  }

  pub fn is_empty(&self) -> bool {
    self.data.is_empty()
  }

  pub fn is_sorted(&self) -> bool {
    self.sorted
  }

  pub fn clear(&mut self) {
    self.data.clear();
    self.sorted = true;
  }

  pub fn push(&mut self, event: Event) -> Result<(), Event> {
    if self.data.len() < self.data.capacity() {
      self.sorted = self
        .data
        .last()
        .map_or(true, |last_event| event.timestamp >= last_event.timestamp);
      self.data.push(event);
      Ok(())
    } else {
      Err(event)
    }
  }

  pub fn iter(&self) -> Iter<'_> {
    Iter(self.data.iter())
  }
}

pub struct Iter<'a>(std::slice::Iter<'a, Event>);

impl<'a> Iterator for Iter<'a> {
  type Item = &'a Event;

  fn next(&mut self) -> Option<Self::Item> {
    self.0.next()
  }
}
