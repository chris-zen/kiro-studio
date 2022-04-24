pub mod channel_voice;
pub mod system_common;
pub mod utility;

use crate::messages::channel_voice::ChannelVoiceMessage;
use crate::messages::system_common::SystemCommon;
use crate::protocol::messages::channel_voice::ChannelVoice;
use crate::protocol::messages::utility::Utility;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Message {
  pub group: u8,
  pub mtype: MessageType,
}

impl Message {
  pub fn new(group: u8, mtype: MessageType) -> Self {
    Self { group, mtype }
  }

  pub fn channel_voice(group: u8, channel: u8, message: ChannelVoiceMessage) -> Self {
    Self {
      group,
      mtype: MessageType::ChannelVoice(ChannelVoice { channel, message }),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageType {
  Utility(Utility),
  SystemCommon(SystemCommon),
  // SystemExclusive(SystemExclusive),
  ChannelVoice(ChannelVoice),
  // Data(Data)
}
