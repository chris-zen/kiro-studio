use crate::protocol::{Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChannelVoice {
  pub channel: u8,
  pub message: ChanelVoiceMessage,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChanelVoiceMessage {
  NoteOff {
    note: u8,
    velocity: u16,
    attr_type: u8,
    attr_data: u16,
  },
  NoteOn {
    note: u8,
    velocity: u16,
    attr_type: u8,
    attr_data: u16,
  },
  PolyPressure {
    note: u8,
    data: u32,
  },
  RegisteredPerNoteController {
    note: u8,
    index: u8,
    data: u32,
  },
  AssignablePerNoteController {
    note: u8,
    index: u8,
    data: u32,
  },
  PerNoteManagement {
    note: u8,
    detach: bool,
    reset: bool,
  },
  ControlChange {
    index: u8,
    data: u32,
  },
  RegisteredController {
    bank: u8,
    index: u8,
    data: u32,
  },
  AssignableController {
    bank: u8,
    index: u8,
    data: u32,
  },
  RelativeRegisteredController {
    bank: u8,
    index: u8,
    data: i32,
  },
  RelativeAssignableController {
    bank: u8,
    index: u8,
    data: i32,
  },
  ProgramChange {
    program: u8,
    bank: Option<u16>,
  },
  ChannelPressure {
    data: u32,
  },
  PitchBend {
    /// unsigned bipolar value centered at 0x80000000
    data: u32,
  },
  PerNotePitchBend {
    note: u8,
    /// unsigned bipolar value centered at 0x80000000
    data: u32,
  },
}

impl Decode for ChannelVoice {
  fn decode(ump: &[u32]) -> Self {
    assert_eq!(ump.len(), 2);
    let channel = ((ump[0] >> 16) & 0x0f) as u8;
    let status = ((ump[0] >> 20) & 0x0f) as u8;
    match status {
      0b1000 => Self {
        channel,
        message: ChanelVoiceMessage::NoteOff {
          note: ((ump[0] >> 8) & 0x7f) as u8,
          attr_type: (ump[0] & 0xff) as u8,
          velocity: ((ump[1] >> 16) & 0xffff) as u16,
          attr_data: (ump[1] & 0xffff) as u16,
        },
      },
      0b1001 => Self {
        channel,
        message: ChanelVoiceMessage::NoteOn {
          note: ((ump[0] >> 8) & 0x7f) as u8,
          attr_type: (ump[0] & 0xff) as u8,
          velocity: ((ump[1] >> 16) & 0xffff) as u16,
          attr_data: (ump[1] & 0xffff) as u16,
        },
      },
      0b1010 => Self {
        channel,
        message: ChanelVoiceMessage::PolyPressure {
          note: ((ump[0] >> 8) & 0x7f) as u8,
          data: ump[1],
        },
      },
      0b0000 => Self {
        channel,
        message: ChanelVoiceMessage::RegisteredPerNoteController {
          note: ((ump[0] >> 8) & 0x7f) as u8,
          index: (ump[0] & 0xff) as u8,
          data: ump[1],
        },
      },
      0b0001 => Self {
        channel,
        message: ChanelVoiceMessage::AssignablePerNoteController {
          note: ((ump[0] >> 8) & 0x7f) as u8,
          index: (ump[0] & 0xff) as u8,
          data: ump[1],
        },
      },
      0b1111 => Self {
        channel,
        message: ChanelVoiceMessage::PerNoteManagement {
          note: ((ump[0] >> 8) & 0x7f) as u8,
          detach: ump[0] & 0b00000010 != 0,
          reset: ump[0] & 0b00000001 != 0,
        },
      },
      0b1011 => Self {
        channel,
        message: ChanelVoiceMessage::ControlChange {
          index: ((ump[0] >> 8) & 0x7f) as u8,
          data: ump[1],
        },
      },
      0b0010 => Self {
        channel,
        message: ChanelVoiceMessage::RegisteredController {
          bank: ((ump[0] >> 8) & 0x7f) as u8,
          index: (ump[0] & 0x7f) as u8,
          data: ump[1],
        },
      },
      0b0011 => Self {
        channel,
        message: ChanelVoiceMessage::AssignableController {
          bank: ((ump[0] >> 8) & 0x7f) as u8,
          index: (ump[0] & 0x7f) as u8,
          data: ump[1],
        },
      },
      0b0100 => Self {
        channel,
        message: ChanelVoiceMessage::RelativeRegisteredController {
          bank: ((ump[0] >> 8) & 0x7f) as u8,
          index: (ump[0] & 0x7f) as u8,
          data: ump[1] as i32,
        },
      },
      0b0101 => Self {
        channel,
        message: ChanelVoiceMessage::RelativeAssignableController {
          bank: ((ump[0] >> 8) & 0x7f) as u8,
          index: (ump[0] & 0x7f) as u8,
          data: ump[1] as i32,
        },
      },
      0b1100 => Self {
        channel,
        message: ChanelVoiceMessage::ProgramChange {
          program: ((ump[1] >> 24) & 0x7f) as u8,
          bank: (ump[0] & 0b00000001 != 0)
            .then(|| (((ump[1] & 0x7f00) >> 1) | (ump[1] & 0x7f)) as u16),
        },
      },
      0b1101 => Self {
        channel,
        message: ChanelVoiceMessage::ChannelPressure { data: ump[1] },
      },
      0b1110 => Self {
        channel,
        message: ChanelVoiceMessage::PitchBend { data: ump[1] },
      },
      0b0110 => Self {
        channel,
        message: ChanelVoiceMessage::PerNotePitchBend {
          note: ((ump[0] >> 8) & 0x7f) as u8,
          data: ump[1],
        },
      },
      _ => unreachable!(),
    }
  }
}

impl Encode<2> for ChannelVoice {
  fn encode(&self) -> [u32; 2] {
    todo!()
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttributeType {
  Ignore,
  ManufacturerSpecific,
  ProfileSpecific,
  Pitch7_9,
  Reserved(u8),
}

impl From<u8> for AttributeType {
  fn from(data: u8) -> Self {
    match data {
      0x00 => Self::Ignore,
      0x01 => Self::ManufacturerSpecific,
      0x02 => Self::ProfileSpecific,
      0x03 => Self::Pitch7_9,
      _ => Self::Reserved(data),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn decode_note_off() {
    let channel_voice = ChannelVoice::decode(&[0x4182bc03, 0xabcd1234]);

    assert_eq!(
      channel_voice,
      ChannelVoice {
        channel: 2,
        message: ChanelVoiceMessage::NoteOff {
          note: 0x3c,
          attr_type: 0x03,
          velocity: 0xabcd,
          attr_data: 0x1234,
        }
      }
    );
  }

  #[test]
  fn decode_note_on() {
    let channel_voice = ChannelVoice::decode(&[0x4192bc03, 0xabcd1234]);

    assert_eq!(
      channel_voice,
      ChannelVoice {
        channel: 2,
        message: ChanelVoiceMessage::NoteOn {
          note: 0x3c,
          attr_type: 0x03,
          velocity: 0xabcd,
          attr_data: 0x1234,
        }
      }
    );
  }

  #[test]
  fn decode_poly_pressure() {
    let channel_voice = ChannelVoice::decode(&[0x41a2bcff, 0x12345678]);

    assert_eq!(
      channel_voice,
      ChannelVoice {
        channel: 2,
        message: ChanelVoiceMessage::PolyPressure {
          note: 0x3c,
          data: 0x12345678,
        }
      }
    );
  }

  #[test]
  fn decode_registered_per_note_controller() {
    let channel_voice = ChannelVoice::decode(&[0x4102bca5, 0x12345678]);

    assert_eq!(
      channel_voice,
      ChannelVoice {
        channel: 2,
        message: ChanelVoiceMessage::RegisteredPerNoteController {
          note: 0x3c,
          index: 0xa5,
          data: 0x12345678,
        }
      }
    );
  }

  #[test]
  fn decode_assignable_per_note_controller() {
    let channel_voice = ChannelVoice::decode(&[0x4112bca5, 0x12345678]);

    assert_eq!(
      channel_voice,
      ChannelVoice {
        channel: 2,
        message: ChanelVoiceMessage::AssignablePerNoteController {
          note: 0x3c,
          index: 0xa5,
          data: 0x12345678,
        }
      }
    );
  }

  #[test]
  fn decode_per_note_management() {
    let channel_voice = ChannelVoice::decode(&[0x41f2bcfd, 0x12345678]);

    assert_eq!(
      channel_voice,
      ChannelVoice {
        channel: 2,
        message: ChanelVoiceMessage::PerNoteManagement {
          note: 0x3c,
          detach: false,
          reset: true,
        }
      }
    );

    let channel_voice = ChannelVoice::decode(&[0x41f2bcfe, 0x12345678]);

    assert_eq!(
      channel_voice,
      ChannelVoice {
        channel: 2,
        message: ChanelVoiceMessage::PerNoteManagement {
          note: 0x3c,
          detach: true,
          reset: false,
        }
      }
    );
  }

  #[test]
  fn decode_control_change() {
    let channel_voice = ChannelVoice::decode(&[0x41b2ffff, 0x12345678]);

    assert_eq!(
      channel_voice,
      ChannelVoice {
        channel: 2,
        message: ChanelVoiceMessage::ControlChange {
          index: 0x7f,
          data: 0x12345678,
        }
      }
    );
  }

  #[test]
  fn decode_registered_controller() {
    let channel_voice = ChannelVoice::decode(&[0x4122a5ff, 0x12345678]);

    assert_eq!(
      channel_voice,
      ChannelVoice {
        channel: 2,
        message: ChanelVoiceMessage::RegisteredController {
          bank: 0x25,
          index: 0x7f,
          data: 0x12345678,
        }
      }
    );
  }

  #[test]
  fn decode_assignable_controller() {
    let channel_voice = ChannelVoice::decode(&[0x4132a5ff, 0x12345678]);

    assert_eq!(
      channel_voice,
      ChannelVoice {
        channel: 2,
        message: ChanelVoiceMessage::AssignableController {
          bank: 0x25,
          index: 0x7f,
          data: 0x12345678,
        }
      }
    );
  }

  #[test]
  fn decode_relative_registered_controller() {
    let channel_voice = ChannelVoice::decode(&[0x4142a5ff, 0x80000000]);

    assert_eq!(
      channel_voice,
      ChannelVoice {
        channel: 2,
        message: ChanelVoiceMessage::RelativeRegisteredController {
          bank: 0x25,
          index: 0x7f,
          data: i32::MIN,
        }
      }
    );
  }

  #[test]
  fn decode_relative_assignable_controller() {
    let channel_voice = ChannelVoice::decode(&[0x4152a5ff, 0x7fffffff]);

    assert_eq!(
      channel_voice,
      ChannelVoice {
        channel: 2,
        message: ChanelVoiceMessage::RelativeAssignableController {
          bank: 0x25,
          index: 0x7f,
          data: i32::MAX,
        }
      }
    );
  }

  #[test]
  fn decode_program_change() {
    let channel_voice = ChannelVoice::decode(&[0x41c2ffff, 0xffffcfa5]);

    assert_eq!(
      channel_voice,
      ChannelVoice {
        channel: 2,
        message: ChanelVoiceMessage::ProgramChange {
          program: 0x7f,
          bank: Some(0x27a5),
        }
      }
    );

    let channel_voice = ChannelVoice::decode(&[0x41c2fffe, 0xffffcfa5]);

    assert_eq!(
      channel_voice,
      ChannelVoice {
        channel: 2,
        message: ChanelVoiceMessage::ProgramChange {
          program: 0x7f,
          bank: None,
        }
      }
    );
  }

  #[test]
  fn decode_channel_pressure() {
    let channel_voice = ChannelVoice::decode(&[0x41d2ffff, 0x87654321]);

    assert_eq!(
      channel_voice,
      ChannelVoice {
        channel: 2,
        message: ChanelVoiceMessage::ChannelPressure { data: 0x87654321 }
      }
    );
  }

  #[test]
  fn decode_pitch_bend() {
    let channel_voice = ChannelVoice::decode(&[0x41e2ffff, 0x87654321]);

    assert_eq!(
      channel_voice,
      ChannelVoice {
        channel: 2,
        message: ChanelVoiceMessage::PitchBend { data: 0x87654321 }
      }
    );
  }

  #[test]
  fn decode_per_note_pitch_bend() {
    let channel_voice = ChannelVoice::decode(&[0x4162ffaa, 0x87654321]);

    assert_eq!(
      channel_voice,
      ChannelVoice {
        channel: 2,
        message: ChanelVoiceMessage::PerNotePitchBend {
          note: 0x7f,
          data: 0x87654321,
        }
      }
    );
  }
}
