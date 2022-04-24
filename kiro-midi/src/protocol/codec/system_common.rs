use crate::messages::system_common::{MidiTimeCode, SystemCommon};

pub fn decode_system_common(ump: &[u32]) -> Option<SystemCommon> {
  if ump.len() == 1 {
    let status = ((ump[0] >> 16) & 0xff) as u8;
    match status {
      // MIDI Time Code
      0xf1 => {
        let message_type = (ump[0] >> 12) & 0x07;
        let value = ((ump[0] >> 8) & 0x0f) as u8;
        let code = match message_type {
          0 => MidiTimeCode::FrameLessSignificantNibble(value),
          1 => MidiTimeCode::FrameMostSignificantNibble(value),
          2 => MidiTimeCode::SecondsLessSignificantNibble(value),
          3 => MidiTimeCode::SecondsMostSignificantNibble(value),
          4 => MidiTimeCode::MinutesLessSignificantNibble(value),
          5 => MidiTimeCode::MinutesMostSignificantNibble(value),
          6 => MidiTimeCode::HoursLessSignificantNibble(value),
          7 => MidiTimeCode::HoursMostSignificantNibble(value),
          _ => unreachable!(),
        };
        Some(SystemCommon::MidiTimeCode(code))
      }
      // Song Position Pointer
      0xf2 => {
        let lsb = (ump[0] >> 8) & 0x7f;
        let msb = ump[0] & 0x7f;
        let value = (msb << 7 | lsb) as u16;
        Some(SystemCommon::SongPositionPointer(value))
      }
      // Song Select
      0xf3 => {
        let value = ((ump[0] >> 8) & 0x7f) as u8;
        Some(SystemCommon::SongSelect(value))
      }
      // Tune Request
      0xf6 => Some(SystemCommon::TuneRequest),
      // Timing Clock
      0xf8 => Some(SystemCommon::TimingClock),
      // Start
      0xfa => Some(SystemCommon::Start),
      // Continue
      0xfb => Some(SystemCommon::Continue),
      // Stop
      0xfc => Some(SystemCommon::Stop),
      // Active Sensing
      0xfe => Some(SystemCommon::ActiveSensing),
      // Reset
      0xff => Some(SystemCommon::Reset),
      _ => None,
    }
  } else {
    None
  }
}

#[cfg(test)]
mod tests {
  use crate::messages::system_common::{MidiTimeCode, SystemCommon};
  use crate::protocol::codec::system_common::decode_system_common;

  #[test]
  fn decode_midi_time_code() {
    let test_cases = vec![
      (
        0x10f10100,
        Some(SystemCommon::MidiTimeCode(
          MidiTimeCode::FrameLessSignificantNibble(1),
        )),
      ),
      (
        0x10f11200,
        Some(SystemCommon::MidiTimeCode(
          MidiTimeCode::FrameMostSignificantNibble(2),
        )),
      ),
      (
        0x10f12300,
        Some(SystemCommon::MidiTimeCode(
          MidiTimeCode::SecondsLessSignificantNibble(3),
        )),
      ),
      (
        0x10f13400,
        Some(SystemCommon::MidiTimeCode(
          MidiTimeCode::SecondsMostSignificantNibble(4),
        )),
      ),
      (
        0x10f14500,
        Some(SystemCommon::MidiTimeCode(
          MidiTimeCode::MinutesLessSignificantNibble(5),
        )),
      ),
      (
        0x10f15600,
        Some(SystemCommon::MidiTimeCode(
          MidiTimeCode::MinutesMostSignificantNibble(6),
        )),
      ),
      (
        0x10f16700,
        Some(SystemCommon::MidiTimeCode(
          MidiTimeCode::HoursLessSignificantNibble(7),
        )),
      ),
      (
        0x10f17800,
        Some(SystemCommon::MidiTimeCode(
          MidiTimeCode::HoursMostSignificantNibble(8),
        )),
      ),
    ];

    for (ump, expected) in test_cases {
      assert_eq!(decode_system_common(&[ump]), expected,)
    }
  }

  #[test]
  fn decode_song_position_pointer() {
    assert_eq!(
      decode_system_common(&[0x10f27f7f]),
      Some(SystemCommon::SongPositionPointer(0x3fff)),
    )
  }

  #[test]
  fn decode_song_select() {
    assert_eq!(
      decode_system_common(&[0x10f37f00]),
      Some(SystemCommon::SongSelect(0x7f)),
    )
  }

  #[test]
  fn decode_tune_request() {
    assert_eq!(
      decode_system_common(&[0x10f60000]),
      Some(SystemCommon::TuneRequest),
    )
  }

  #[test]
  fn decode_timing_clock() {
    assert_eq!(
      decode_system_common(&[0x10f80000]),
      Some(SystemCommon::TimingClock),
    )
  }

  #[test]
  fn decode_start() {
    assert_eq!(
      decode_system_common(&[0x10fa0000]),
      Some(SystemCommon::Start),
    )
  }

  #[test]
  fn decode_continue() {
    assert_eq!(
      decode_system_common(&[0x10fb0000]),
      Some(SystemCommon::Continue),
    )
  }

  #[test]
  fn decode_stop() {
    assert_eq!(
      decode_system_common(&[0x10fc0000]),
      Some(SystemCommon::Stop),
    )
  }

  #[test]
  fn decode_active_sensing() {
    assert_eq!(
      decode_system_common(&[0x10fe0000]),
      Some(SystemCommon::ActiveSensing),
    )
  }

  #[test]
  fn decode_reset() {
    assert_eq!(
      decode_system_common(&[0x10ff0000]),
      Some(SystemCommon::Reset),
    )
  }
}
