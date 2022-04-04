use ringbuf::Producer;
use std::fmt::{Debug, Formatter};

use crate::event::Event;

pub enum InputHandler {
  Callback(Box<dyn FnMut(Event) + Send + 'static>),
  RingBuffer(Producer<Event>),
}

impl InputHandler {
  pub(crate) fn call(&mut self, event: Event) {
    match self {
      InputHandler::Callback(ref mut callback) => (callback)(event),
      InputHandler::RingBuffer(ref mut producer) => {
        producer.push(event).ok();
      }
    };
  }
}

impl<F> From<F> for InputHandler
where
  F: FnMut(Event) + Send + 'static,
{
  fn from(callback: F) -> Self {
    InputHandler::Callback(Box::new(callback))
  }
}

impl From<Producer<Event>> for InputHandler {
  fn from(producer: Producer<Event>) -> Self {
    InputHandler::RingBuffer(producer)
  }
}

impl Debug for InputHandler {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Callback(_) => write!(f, "Callback"),
      Self::RingBuffer(_) => write!(f, "RingBuffer"),
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::messages::Message;
  use std::sync::atomic::{AtomicU8, Ordering};
  use std::sync::Arc;

  use crate::protocol::messages::utility::Utility;
  use crate::protocol::messages::MessageType;

  use super::*;

  #[test]
  fn from_callback() {
    let state = Arc::new(AtomicU8::new(1));
    let state_clone = state.clone();

    let mut handler = InputHandler::from(move |event: Event| {
      state_clone.store(event.message.group, Ordering::Relaxed)
    });

    handler.call(Event {
      timestamp: 0,
      endpoint: 0,
      message: Message {
        group: 8,
        mtype: MessageType::Utility(Utility::Noop),
      },
    });

    assert_eq!(state.load(Ordering::Relaxed), 8);
  }

  #[test]
  fn from_ring_buffer() {
    let (mut producer, mut consumer) = ringbuf::RingBuffer::new(1).split();
    let event = Event {
      timestamp: 0,
      endpoint: 0,
      message: Message {
        group: 8,
        mtype: MessageType::Utility(Utility::Noop),
      },
    };

    let mut handler = InputHandler::from(producer);

    handler.call(event.clone());

    assert_eq!(consumer.pop(), Some(event));
  }
}
