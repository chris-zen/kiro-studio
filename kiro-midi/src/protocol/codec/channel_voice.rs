use crate::messages::channel_voice::{ChannelMode, ChannelVoice, ChannelVoiceMessage};

pub fn decode_channel_voice(ump: &[u32]) -> Option<ChannelVoice> {
  if ump.len() == 2 {
    let channel = ((ump[0] >> 16) & 0x0f) as u8;
    let status = ((ump[0] >> 20) & 0x0f) as u8;
    match status {
      0b1000 => Some(ChannelVoice {
        channel,
        message: ChannelVoiceMessage::NoteOff {
          note: ((ump[0] >> 8) & 0x7f) as u8,
          attr_type: (ump[0] & 0xff) as u8,
          velocity: ((ump[1] >> 16) & 0xffff) as u16,
          attr_data: (ump[1] & 0xffff) as u16,
        },
      }),
      0b1001 => Some(ChannelVoice {
        channel,
        message: ChannelVoiceMessage::NoteOn {
          note: ((ump[0] >> 8) & 0x7f) as u8,
          attr_type: (ump[0] & 0xff) as u8,
          velocity: ((ump[1] >> 16) & 0xffff) as u16,
          attr_data: (ump[1] & 0xffff) as u16,
        },
      }),
      0b1010 => Some(ChannelVoice {
        channel,
        message: ChannelVoiceMessage::PolyPressure {
          note: ((ump[0] >> 8) & 0x7f) as u8,
          pressure: ump[1],
        },
      }),
      0b0000 => Some(ChannelVoice {
        channel,
        message: ChannelVoiceMessage::RegisteredPerNoteController {
          note: ((ump[0] >> 8) & 0x7f) as u8,
          index: (ump[0] & 0xff) as u8,
          data: ump[1],
        },
      }),
      0b0001 => Some(ChannelVoice {
        channel,
        message: ChannelVoiceMessage::AssignablePerNoteController {
          note: ((ump[0] >> 8) & 0x7f) as u8,
          index: (ump[0] & 0xff) as u8,
          data: ump[1],
        },
      }),
      0b1111 => Some(ChannelVoice {
        channel,
        message: ChannelVoiceMessage::PerNoteManagement {
          note: ((ump[0] >> 8) & 0x7f) as u8,
          detach: ump[0] & 0b00000010 != 0,
          reset: ump[0] & 0b00000001 != 0,
        },
      }),
      0b1011 => {
        let index = ((ump[0] >> 8) & 0x7f) as u8;
        let data = ump[1];
        if index < 120 {
          Some(ChannelVoice {
            channel,
            message: ChannelVoiceMessage::ControlChange { index, data },
          })
        } else {
          match index {
            120 if data == 0 => Some(ChannelVoice::channel_mode(
              channel,
              ChannelMode::AllSoundOff,
            )),
            121 if data == 0 => Some(ChannelVoice::channel_mode(
              channel,
              ChannelMode::ResetAllControllers,
            )),
            122 if data == 0 || data == 127 => Some(ChannelVoice::channel_mode(
              channel,
              ChannelMode::LocalControl(data == 127),
            )),
            123 if data == 0 => Some(ChannelVoice::channel_mode(
              channel,
              ChannelMode::AllNotesOff,
            )),
            124 if data == 0 => Some(ChannelVoice::channel_mode(
              channel,
              ChannelMode::OmniMode(false),
            )),
            125 if data == 0 => Some(ChannelVoice::channel_mode(
              channel,
              ChannelMode::OmniMode(true),
            )),
            126 if data == 0 => Some(ChannelVoice::channel_mode(
              channel,
              ChannelMode::MonoModeOnForNumberOfVoices,
            )),
            126 if data > 0 && data <= 16 => Some(ChannelVoice::channel_mode(
              channel,
              ChannelMode::MonoModeOnForNumberOfChannels(data as u8),
            )),
            127 if data == 0 => Some(ChannelVoice::channel_mode(channel, ChannelMode::PolyModeOn)),
            _ => None,
          }
        }
      }
      0b0010 => Some(ChannelVoice {
        channel,
        message: ChannelVoiceMessage::RegisteredController {
          bank: ((ump[0] >> 8) & 0x7f) as u8,
          index: (ump[0] & 0x7f) as u8,
          data: ump[1],
        },
      }),
      0b0011 => Some(ChannelVoice {
        channel,
        message: ChannelVoiceMessage::AssignableController {
          bank: ((ump[0] >> 8) & 0x7f) as u8,
          index: (ump[0] & 0x7f) as u8,
          data: ump[1],
        },
      }),
      0b0100 => Some(ChannelVoice {
        channel,
        message: ChannelVoiceMessage::RelativeRegisteredController {
          bank: ((ump[0] >> 8) & 0x7f) as u8,
          index: (ump[0] & 0x7f) as u8,
          data: ump[1] as i32,
        },
      }),
      0b0101 => Some(ChannelVoice {
        channel,
        message: ChannelVoiceMessage::RelativeAssignableController {
          bank: ((ump[0] >> 8) & 0x7f) as u8,
          index: (ump[0] & 0x7f) as u8,
          data: ump[1] as i32,
        },
      }),
      0b1100 => Some(ChannelVoice {
        channel,
        message: ChannelVoiceMessage::ProgramChange {
          program: ((ump[1] >> 24) & 0x7f) as u8,
          bank: (ump[0] & 0b00000001 != 0)
            .then(|| (((ump[1] & 0x7f00) >> 1) | (ump[1] & 0x7f)) as u16),
        },
      }),
      0b1101 => Some(ChannelVoice {
        channel,
        message: ChannelVoiceMessage::ChannelPressure { pressure: ump[1] },
      }),
      0b1110 => Some(ChannelVoice {
        channel,
        message: ChannelVoiceMessage::PitchBend { data: ump[1] },
      }),
      0b0110 => Some(ChannelVoice {
        channel,
        message: ChannelVoiceMessage::PerNotePitchBend {
          note: ((ump[0] >> 8) & 0x7f) as u8,
          data: ump[1],
        },
      }),
      _ => unreachable!(),
    }
  } else {
    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::messages::channel_voice::ChannelVoice;

  #[test]
  fn decode_wrong_length_failure() {
    assert_eq!(decode_channel_voice(&[0x4182bc03]), None,);

    assert_eq!(
      decode_channel_voice(&[0x4182bc03, 0xabcd1234, 0x4182bc03]),
      None,
    );
  }

  #[test]
  fn decode_note_off() {
    assert_eq!(
      decode_channel_voice(&[0x4182bc03, 0xabcd1234]),
      Some(ChannelVoice {
        channel: 2,
        message: ChannelVoiceMessage::NoteOff {
          note: 0x3c,
          attr_type: 0x03,
          velocity: 0xabcd,
          attr_data: 0x1234,
        }
      })
    );
  }

  #[test]
  fn decode_note_on() {
    assert_eq!(
      decode_channel_voice(&[0x4192bc03, 0xabcd1234]),
      Some(ChannelVoice {
        channel: 2,
        message: ChannelVoiceMessage::NoteOn {
          note: 0x3c,
          attr_type: 0x03,
          velocity: 0xabcd,
          attr_data: 0x1234,
        }
      })
    );
  }

  #[test]
  fn decode_poly_pressure() {
    assert_eq!(
      decode_channel_voice(&[0x41a2bcff, 0x12345678]),
      Some(ChannelVoice {
        channel: 2,
        message: ChannelVoiceMessage::PolyPressure {
          note: 0x3c,
          pressure: 0x12345678,
        }
      })
    );
  }

  #[test]
  fn decode_registered_per_note_controller() {
    assert_eq!(
      decode_channel_voice(&[0x4102bca5, 0x12345678]),
      Some(ChannelVoice {
        channel: 2,
        message: ChannelVoiceMessage::RegisteredPerNoteController {
          note: 0x3c,
          index: 0xa5,
          data: 0x12345678,
        }
      })
    );
  }

  #[test]
  fn decode_assignable_per_note_controller() {
    assert_eq!(
      decode_channel_voice(&[0x4112bca5, 0x12345678]),
      Some(ChannelVoice {
        channel: 2,
        message: ChannelVoiceMessage::AssignablePerNoteController {
          note: 0x3c,
          index: 0xa5,
          data: 0x12345678,
        }
      })
    );
  }

  #[test]
  fn decode_per_note_management() {
    assert_eq!(
      decode_channel_voice(&[0x41f2bcfd, 0x12345678]),
      Some(ChannelVoice {
        channel: 2,
        message: ChannelVoiceMessage::PerNoteManagement {
          note: 0x3c,
          detach: false,
          reset: true,
        }
      })
    );

    assert_eq!(
      decode_channel_voice(&[0x41f2bcfe, 0x12345678]),
      Some(ChannelVoice {
        channel: 2,
        message: ChannelVoiceMessage::PerNoteManagement {
          note: 0x3c,
          detach: true,
          reset: false,
        }
      })
    );
  }

  #[test]
  fn decode_control_change() {
    assert_eq!(
      decode_channel_voice(&[0x41b2f7ff, 0x12345678]),
      Some(ChannelVoice {
        channel: 2,
        message: ChannelVoiceMessage::ControlChange {
          index: 0x77,
          data: 0x12345678,
        }
      })
    );
  }

  #[test]
  fn decode_control_mode() {
    let test_cases = vec![
      (vec![0x41b27800, 0x00000000], Some(ChannelMode::AllSoundOff)),
      (vec![0x41b27800, 0x00000001], None),
      (
        vec![0x41b27900, 0x00000000],
        Some(ChannelMode::ResetAllControllers),
      ),
      (vec![0x41b27900, 0x00000001], None),
      (
        vec![0x41b27a00, 0x00000000],
        Some(ChannelMode::LocalControl(false)),
      ),
      (
        vec![0x41b27a00, 0x0000007f],
        Some(ChannelMode::LocalControl(true)),
      ),
      (vec![0x41b27a00, 0x00000001], None),
      (vec![0x41b27b00, 0x00000000], Some(ChannelMode::AllNotesOff)),
      (vec![0x41b27b00, 0x00000001], None),
      (
        vec![0x41b27c00, 0x00000000],
        Some(ChannelMode::OmniMode(false)),
      ),
      (vec![0x41b27c00, 0x00000001], None),
      (
        vec![0x41b27d00, 0x00000000],
        Some(ChannelMode::OmniMode(true)),
      ),
      (vec![0x41b27d00, 0x00000001], None),
      (
        vec![0x41b27e00, 0x00000000],
        Some(ChannelMode::MonoModeOnForNumberOfVoices),
      ),
      (
        vec![0x41b27e00, 0x00000010],
        Some(ChannelMode::MonoModeOnForNumberOfChannels(16)),
      ),
      (vec![0x41b27e00, 0x00000011], None),
      (vec![0x41b27f00, 0x00000000], Some(ChannelMode::PolyModeOn)),
      (vec![0x41b27f00, 0x00000001], None),
    ];

    for (data, expected_mode) in test_cases {
      assert_eq!(
        decode_channel_voice(data.as_slice()),
        expected_mode.map(|mode| ChannelVoice {
          channel: 2,
          message: ChannelVoiceMessage::ChannelMode(mode)
        })
      );
    }
  }

  #[test]
  fn decode_registered_controller() {
    assert_eq!(
      decode_channel_voice(&[0x4122a5ff, 0x12345678]),
      Some(ChannelVoice {
        channel: 2,
        message: ChannelVoiceMessage::RegisteredController {
          bank: 0x25,
          index: 0x7f,
          data: 0x12345678,
        }
      })
    );
  }

  #[test]
  fn decode_assignable_controller() {
    assert_eq!(
      decode_channel_voice(&[0x4132a5ff, 0x12345678]),
      Some(ChannelVoice {
        channel: 2,
        message: ChannelVoiceMessage::AssignableController {
          bank: 0x25,
          index: 0x7f,
          data: 0x12345678,
        }
      })
    );
  }

  #[test]
  fn decode_relative_registered_controller() {
    assert_eq!(
      decode_channel_voice(&[0x4142a5ff, 0x80000000]),
      Some(ChannelVoice {
        channel: 2,
        message: ChannelVoiceMessage::RelativeRegisteredController {
          bank: 0x25,
          index: 0x7f,
          data: i32::MIN,
        }
      })
    );
  }

  #[test]
  fn decode_relative_assignable_controller() {
    assert_eq!(
      decode_channel_voice(&[0x4152a5ff, 0x7fffffff]),
      Some(ChannelVoice {
        channel: 2,
        message: ChannelVoiceMessage::RelativeAssignableController {
          bank: 0x25,
          index: 0x7f,
          data: i32::MAX,
        }
      })
    );
  }

  #[test]
  fn decode_program_change() {
    assert_eq!(
      decode_channel_voice(&[0x41c2ffff, 0xffffcfa5]),
      Some(ChannelVoice {
        channel: 2,
        message: ChannelVoiceMessage::ProgramChange {
          program: 0x7f,
          bank: Some(0x27a5),
        }
      })
    );

    assert_eq!(
      decode_channel_voice(&[0x41c2fffe, 0xffffcfa5]),
      Some(ChannelVoice {
        channel: 2,
        message: ChannelVoiceMessage::ProgramChange {
          program: 0x7f,
          bank: None,
        }
      })
    );
  }

  #[test]
  fn decode_channel_pressure() {
    assert_eq!(
      decode_channel_voice(&[0x41d2ffff, 0x87654321]),
      Some(ChannelVoice {
        channel: 2,
        message: ChannelVoiceMessage::ChannelPressure {
          pressure: 0x87654321
        }
      })
    );
  }

  #[test]
  fn decode_pitch_bend() {
    assert_eq!(
      decode_channel_voice(&[0x41e2ffff, 0x87654321]),
      Some(ChannelVoice {
        channel: 2,
        message: ChannelVoiceMessage::PitchBend { data: 0x87654321 }
      })
    );
  }

  #[test]
  fn decode_per_note_pitch_bend() {
    assert_eq!(
      decode_channel_voice(&[0x4162ffaa, 0x87654321]),
      Some(ChannelVoice {
        channel: 2,
        message: ChannelVoiceMessage::PerNotePitchBend {
          note: 0x7f,
          data: 0x87654321,
        }
      })
    );
  }
}
