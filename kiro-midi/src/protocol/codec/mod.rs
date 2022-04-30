mod channel_voice;
mod system_common;
mod system_exclusive;
mod utility;

use thiserror::Error;

use crate::filter::Filter;
use crate::protocol::codec::channel_voice::decode_channel_voice;
use crate::protocol::codec::system_common::decode_system_common;
use crate::protocol::codec::utility::decode_utility;
use crate::protocol::messages::{Message, MessageType};

#[derive(Debug, Error)]
pub enum Error {
  #[error("Found reserved encoding")]
  Reserved,
}

#[derive(Default)]
pub struct Decoder {
  ump: [u32; 4],
  index: usize,
  len: usize,
}

impl Decoder {
  pub fn next(&mut self, data: u32, filter: &Filter) -> Result<Option<Message>, Error> {
    if self.index == 0 {
      self.init(data);
    }
    self.push(data);

    let next_message = if self.is_complete() {
      let (mtype, group) = self.extract_mtype_and_group();
      let message = if filter.mtype(mtype) && filter.group(group) {
        self.decode(mtype, group, filter)
      } else {
        None
      };
      self.reset();
      message
    } else {
      None
    };

    Ok(next_message)
  }

  fn init(&mut self, data: u32) {
    let mtype = (data >> 28) & 0x0f;
    self.len = match mtype {
      0x00 => 1,
      0x01 => 1,
      0x02 => 1,
      0x03 => 2,
      0x04 => 2,
      0x05 => 4,
      _ => 1,
    };
  }

  fn push(&mut self, data: u32) {
    self.ump[self.index] = data;
    self.index += 1;
  }

  fn is_complete(&self) -> bool {
    self.index == self.len
  }

  fn extract_mtype_and_group(&self) -> (u8, u8) {
    let mtype = ((self.ump[0] >> 28) & 0x0f) as u8;
    let group = ((self.ump[0] >> 24) & 0x0f) as u8;
    (mtype, group)
  }

  fn decode(&mut self, mtype: u8, group: u8, filter: &Filter) -> Option<Message> {
    match mtype {
      // Utility
      0x00 => decode_utility(&self.ump[0..1]).map(|utility| Message {
        group,
        mtype: MessageType::Utility(utility),
      }),
      // System Common and Real Time
      0x01 => decode_system_common(&self.ump[0..1]).map(|system_common| Message {
        group,
        mtype: MessageType::SystemCommon(system_common),
      }),
      // Channel Voice
      0x04 => decode_channel_voice(&self.ump[0..2]).and_then(|channel_voice| {
        filter
          .channel(group, channel_voice.channel)
          .then(|| Message {
            group,
            mtype: MessageType::ChannelVoice(channel_voice),
          })
      }),
      _ => None,
    }
  }

  pub fn reset(&mut self) {
    self.index = 0;
    self.len = 0;
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::messages::channel_voice::ChannelVoice;
  use crate::protocol::codec::Decoder;
  use crate::protocol::messages::channel_voice::ChannelVoiceMessage;

  #[test]
  fn first_word_does_not_emit() {
    let filter = Filter::new();
    let mut decoder = Decoder::default();

    let result = decoder.next(0x40903c00, &filter);

    assert!(
      matches!(result, Ok(None)),
      "Unexpected result: {:?}",
      result
    );
  }

  #[test]
  fn last_word_emits() {
    let filter = Filter::new();
    let mut decoder = Decoder::default();

    decoder.next(0x41923c00, &filter);
    let result = decoder.next(0xabcd0000, &filter);
    assert!(
      matches!(&result, Ok(Some(message)) if message == &Message {
        group: 1,
        mtype: MessageType::ChannelVoice(ChannelVoice {
          channel: 2,
          message: ChannelVoiceMessage::NoteOn {
            note: 0x3c,
            velocity: 0xabcd,
            attr_type: 0,
            attr_data: 0,
          }
        })
      }),
      "Unexpected result: {:?}",
      result
    );
  }

  #[test]
  fn two_messages_are_emitted() {
    let filter = Filter::new();
    let mut decoder = Decoder::default();

    decoder.next(0x41923c00, &filter);
    let result = decoder.next(0xabcd0000, &filter);
    assert!(
      matches!(&result, Ok(Some(_))),
      "Unexpected result: {:?}",
      result
    );
    decoder.next(0x43853d00, &filter);
    let result = decoder.next(0x12340000, &filter);
    assert!(
      matches!(&result, Ok(Some(message)) if message == &Message {
        group: 3,
        mtype: MessageType::ChannelVoice(ChannelVoice {
          channel: 5,
          message: ChannelVoiceMessage::NoteOff {
            note: 0x3d,
            velocity: 0x1234,
            attr_type: 0,
            attr_data: 0,
          }
        })
      }),
      "Unexpected result: {:?}",
      result
    );
  }

  #[test]
  fn decode_utility() {
    let filter = Filter::new();
    let mut decoder = Decoder::default();

    let result = decoder.next(0x00000000, &filter);
    assert!(
      matches!(
        result,
        Ok(Some(Message {
          group: _,
          mtype: MessageType::Utility(_)
        }))
      ),
      "Unexpected result: {:?}",
      result
    )
  }

  #[test]
  fn decode_system_common() {
    let filter = Filter::new();
    let mut decoder = Decoder::default();

    let result = decoder.next(0x10f20000, &filter);
    assert!(
      matches!(
        result,
        Ok(Some(Message {
          group: _,
          mtype: MessageType::SystemCommon(_)
        }))
      ),
      "Unexpected result: {:?}",
      result
    )
  }

  #[test]
  fn decode_channel_voice() {
    let filter = Filter::new();
    let mut decoder = Decoder::default();

    decoder.next(0x43853d00, &filter);
    let result = decoder.next(0x00000000, &filter);
    assert!(
      matches!(
        result,
        Ok(Some(Message {
          group: _,
          mtype: MessageType::ChannelVoice(_)
        }))
      ),
      "Unexpected result: {:?}",
      result
    )
  }
}
