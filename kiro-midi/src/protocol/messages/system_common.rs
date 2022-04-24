/// System Common and Real Time Type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SystemCommon {
  // System Common
  /// MIDI Time Code
  MidiTimeCode(MidiTimeCode),

  /// Song Position Pointer (14 bits)
  SongPositionPointer(u16),

  /// Song Select (7 bits)
  SongSelect(u8),

  /// Tune Request
  TuneRequest,

  // System Real Time
  /// Timing Clock
  TimingClock,

  /// Start
  Start,

  /// Continue
  Continue,

  /// Stop
  Stop,

  /// Active Sensing
  ActiveSensing,

  /// Reset
  Reset,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MidiTimeCode {
  FrameLessSignificantNibble(u8),
  FrameMostSignificantNibble(u8),
  SecondsLessSignificantNibble(u8),
  SecondsMostSignificantNibble(u8),
  MinutesLessSignificantNibble(u8),
  MinutesMostSignificantNibble(u8),
  HoursLessSignificantNibble(u8),
  HoursMostSignificantNibble(u8),
}
