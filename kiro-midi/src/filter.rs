use std::fmt::{Debug, Formatter};

#[derive(Clone, Copy)]
pub struct Filter {
  mtypes: u16,
  groups: u16,
  channels: [u16; 16],
}

impl Filter {
  pub fn new() -> Self {
    Self {
      mtypes: 0xffff,
      groups: 0xffff,
      channels: [0xffff; 16],
    }
  }

  #[must_use]
  pub fn with_groups(mut self, groups: &[u8]) -> Self {
    self.groups = 0;
    for group in groups.iter().cloned() {
      if group > 0 && group <= 16 {
        self.groups |= 1 << (group - 1);
      }
    }
    self
  }

  #[must_use]
  pub fn with_channels(mut self, group: u8, channels: &[u8]) -> Self {
    if group > 0 && group <= 16 {
      let group = (group - 1) as usize;
      self.channels[group] = 0;
      for channel in channels.iter().cloned() {
        if channel > 0 && channel <= 16 {
          self.channels[group] |= 1 << (channel - 1);
        }
      }
    }
    self
  }

  #[inline]
  pub fn mtype(&self, mtype: u8) -> bool {
    let mtype = mtype & 0x0f;
    let mask = 1 << mtype;
    (self.mtypes & mask) != 0
  }

  #[inline]
  pub fn group(&self, group: u8) -> bool {
    let group = group & 0x0f;
    let mask = 1 << group;
    (self.groups & mask) != 0
  }

  #[inline]
  pub fn channel(&self, group: u8, channel: u8) -> bool {
    let group = (group & 0x0f) as usize;
    let channel = channel & 0x0f;
    let mask = 1 << channel;
    (self.channels[group] & mask) != 0
  }
}

impl Default for Filter {
  fn default() -> Self {
    Self::new()
  }
}

impl Debug for Filter {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "MidiFilter:")?;
    writeln!(f, "  MT : {:016b}  GR : {:016b}", self.mtypes, self.groups)?;
    for i in 0..8 {
      let j = i * 2;
      writeln!(
        f,
        "  C{:02}: {:016b}  C{:02}: {:016b}",
        j + 1,
        self.channels[j],
        j + 2,
        self.channels[j + 1]
      )?;
    }
    Ok(())
  }
}
