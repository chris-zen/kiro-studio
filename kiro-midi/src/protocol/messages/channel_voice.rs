/// Channel Voice and Channel Mode Type
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChannelVoice {
  pub channel: u8,
  pub message: ChannelVoiceMessage,
}

impl ChannelVoice {
  pub fn new(channel: u8, message: ChannelVoiceMessage) -> Self {
    Self { channel, message }
  }

  pub fn channel_mode(channel: u8, mode: ChannelMode) -> Self {
    Self {
      channel,
      message: ChannelVoiceMessage::ChannelMode(mode),
    }
  }
}

/// Channel Voice and Channel Mode message
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChannelVoiceMessage {
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
    pressure: u32,
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
    pressure: u32,
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
  ChannelMode(ChannelMode),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChannelMode {
  AllSoundOff,
  ResetAllControllers,
  LocalControl(bool),
  AllNotesOff,
  OmniMode(bool),
  MonoModeOnForNumberOfChannels(u8),
  MonoModeOnForNumberOfVoices,
  PolyModeOn,
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
