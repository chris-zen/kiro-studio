use thiserror::Error;

use crate::filter::MidiFilter;
use crate::protocol::messages::channel_voice::ChannelVoice;
use crate::protocol::messages::utility::Utility;
use crate::protocol::messages::{Message, MessageType};
use crate::protocol::Decode;

#[derive(Debug, Error)]
pub enum Error {
  #[error("Found reserved encoding")]
  Reserved,
}

#[derive(Default)]
pub struct DecoderProtocol2 {
  ump: [u32; 4],
  index: usize,
  len: usize,
}

impl DecoderProtocol2 {
  pub fn new() -> Self {
    Self {
      ump: [0; 4],
      index: 0,
      len: 0,
    }
  }

  pub fn next(&mut self, data: u32, filter: &MidiFilter) -> Result<Option<Message>, Error> {
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

  fn decode(&mut self, mtype: u8, group: u8, filter: &MidiFilter) -> Option<Message> {
    match mtype {
      0x00 => Some(Message {
        group,
        mtype: MessageType::Utility(Utility::decode(&self.ump[0..1])),
      }),
      0x04 => {
        let channel_voice = ChannelVoice::decode(&self.ump[0..2]);
        filter
          .channel(group, channel_voice.channel)
          .then(|| Message {
            group,
            mtype: MessageType::ChannelVoice(channel_voice),
          })
      }
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
  use crate::protocol::decoder::DecoderProtocol2;
  use crate::protocol::messages::channel_voice::ChanelVoiceMessage;

  #[test]
  fn first_word_does_not_emit() {
    let filter = MidiFilter::none();
    let mut decoder = DecoderProtocol2::default();

    let result = decoder.next(0x40903c00, &filter);

    assert!(
      matches!(result, Ok(None)),
      "Unexpected result: {:?}",
      result
    );
  }

  #[test]
  fn last_word_emits() {
    let filter = MidiFilter::none();
    let mut decoder = DecoderProtocol2::default();

    decoder.next(0x41923c00, &filter);
    let result = decoder.next(0xabcd0000, &filter);
    assert!(
      matches!(&result, Ok(Some(message)) if message == &Message {
        group: 1,
        mtype: MessageType::ChannelVoice(ChannelVoice {
          channel: 2,
          message: ChanelVoiceMessage::NoteOn {
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
    let filter = MidiFilter::none();
    let mut decoder = DecoderProtocol2::default();

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
          message: ChanelVoiceMessage::NoteOff {
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
}
