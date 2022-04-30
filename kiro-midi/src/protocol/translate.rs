use std::collections::VecDeque;
use thiserror::Error;

use crate::Filter;

const NULL_STATUS: u8 = 0;
const SYSEX_START_STATUS: u8 = 0xf0;
const SYSEX_END_STATUS: u8 = 0xf7;
const SYSEX_MAX_LEN: usize = 6;

#[derive(Debug, Error)]
pub enum Error {
  #[error("Data buffer overflow")]
  DataOverflow,

  #[error("UMP buffer overflow")]
  UmpOverflow,
}

#[derive(Debug, Clone, Copy, Default)]
struct Data14 {
  msb: Option<u8>,
  lsb: Option<u8>,
}

impl Data14 {
  pub fn set_msb(&mut self, msb: u8) {
    self.msb = Some(msb);
  }

  pub fn set_lsb(&mut self, lsb: u8) {
    self.lsb = Some(lsb);
  }

  pub fn is_empty(&self) -> bool {
    self.msb.is_none() && self.lsb.is_none()
  }

  pub fn reset(&mut self) {
    self.msb = None;
    self.lsb = None;
  }

  pub fn get_value14(&self) -> u16 {
    let msb = self.msb.unwrap_or(0) as u16;
    let lsb = self.lsb.unwrap_or(0) as u16;
    msb << 7 | lsb
  }

  pub fn get_value16(&self) -> u16 {
    let msb = self.msb.unwrap_or(0) as u16;
    let lsb = self.lsb.unwrap_or(0) as u16;
    msb << 8 | lsb
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ControllerKind {
  None,
  Registered,
  NonRegistered,
}

#[derive(Debug, Clone, Copy)]
struct ControllerState {
  kind: ControllerKind,
  param: Data14,
  data: Data14,
}

impl ControllerState {
  pub fn new() -> Self {
    Self {
      kind: ControllerKind::None,
      param: Data14::default(),
      data: Data14::default(),
    }
  }

  pub fn is_empty(&self) -> bool {
    self.kind == ControllerKind::None && self.param.is_empty() && self.data.is_empty()
  }

  pub fn reset(&mut self) {
    self.kind = ControllerKind::None;
    self.param.reset();
    self.data.reset();
  }

  pub fn reset_data(&mut self) {
    self.data.reset();
  }

  fn update_kind(&mut self, kind: ControllerKind) {
    if self.kind != kind {
      self.reset()
    }
    self.kind = kind;
  }

  pub fn set_param_msb(&mut self, kind: ControllerKind, value: u8) {
    self.update_kind(kind);
    self.param.set_msb(value);
  }

  pub fn set_param_lsb(&mut self, kind: ControllerKind, value: u8) {
    self.update_kind(kind);
    self.param.set_lsb(value);
  }

  pub fn set_data_msb(&mut self, value: u8) {
    self.data.set_msb(value);
  }

  pub fn set_data_lsb(&mut self, value: u8) {
    self.data.set_lsb(value);
  }

  pub fn completed(&self) -> bool {
    self.data.lsb.is_some() && self.data.msb.is_some()
  }

  pub fn get_bank_and_index(&self) -> u16 {
    let bank = self.param.msb.unwrap_or(0) as u16;
    let index = self.param.lsb.unwrap_or(0) as u16;
    bank << 8 | index
  }

  pub fn get_data(&self) -> u32 {
    let value14 = self.data.get_value14();
    convert14to32(value14)
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SysexStatus {
  Start,
  Continue,
}

pub struct Translator {
  group: u8,
  status: u8,
  len: usize,
  data: Vec<u8>,
  ump: VecDeque<u32>,
  banks: [Data14; 16],
  controllers: [ControllerState; 16],
  sysex: SysexStatus,
}

impl Translator {
  const DATA_CAPACITY: usize = 4;
  const UMP_CAPACITY: usize = 4;

  pub fn new(group: u8) -> Self {
    Self {
      group: group & 0x0f,
      status: NULL_STATUS,
      len: 0,
      data: Vec::with_capacity(Self::DATA_CAPACITY),
      ump: VecDeque::with_capacity(Self::UMP_CAPACITY),
      banks: [Data14::default(); 16],
      controllers: [ControllerState::new(); 16],
      sysex: SysexStatus::Start,
    }
  }

  pub fn push(&mut self, byte: u8, filter: &Filter) -> Result<(), Error> {
    if Self::is_status(byte) {
      if Self::is_real_time(byte) {
        self.handle_real_time(byte, filter)
      } else {
        self.handle_status(byte, filter)
      }
    } else {
      self.handle_data(byte, filter)
    }
  }

  pub fn pop(&mut self) -> Option<u32> {
    self.ump.pop_front()
  }

  fn handle_real_time(&mut self, status: u8, filter: &Filter) -> Result<(), Error> {
    let result = self.emit_system_common(status, 0x00, 0x00);
    if status == 0xff {
      self.status = NULL_STATUS;
      self.len = 0;
      self.data.clear();
      self.reset_controllers();
    }
    result
  }

  fn handle_status(&mut self, status: u8, filter: &Filter) -> Result<(), Error> {
    if self.status == SYSEX_START_STATUS {
      self.handle_sysex(true)?;
    }
    self.status = if status == SYSEX_END_STATUS {
      NULL_STATUS
    } else {
      status
    };
    self.data.clear();
    self.len = 0;
    match status {
      // System Exclusive
      SYSEX_START_STATUS => {
        self.sysex = SysexStatus::Start;
        Ok(())
      }
      // Reserved
      0xf4 | 0xf5 => {
        self.status = NULL_STATUS;
        Ok(())
      }
      // Tune Request
      0xf6 => self.emit_system_common(self.status, 0x00, 0x00),
      SYSEX_END_STATUS => Ok(()),
      _ => {
        self.len = Self::expected_len(status);
        // Wait for the next data bytes
        Ok(())
      }
    }
  }

  fn handle_data(&mut self, data: u8, filter: &Filter) -> Result<(), Error> {
    // Handle data byte
    if self.status == NULL_STATUS {
      // Skip data byte
      Ok(())
    } else if self.status == SYSEX_START_STATUS {
      self.data.push(data);
      if self.data.len() == SYSEX_MAX_LEN {
        self.handle_sysex(false)
      } else {
        Ok(())
      }
    } else if self.data.len() < Self::DATA_CAPACITY {
      self.data.push(data);
      if self.data.len() == self.len {
        self.handle_message(filter)
      } else {
        // Wait for the next data bytes
        Ok(())
      }
    } else {
      Err(Error::DataOverflow)
    }
  }

  fn handle_sysex(&mut self, end_of_sysex: bool) -> Result<(), Error> {
    let data_at =
      |index: usize, shift: usize| self.data.get(index).map_or(0u32, |b| *b as u32) << shift;

    let mut ump = [
      Self::ump_type_and_group(0x3, self.group)
        | Self::ump_byte(self.data.len() as u8, 16)
        | data_at(0, 8)
        | data_at(1, 0),
      data_at(2, 24) | data_at(3, 16) | data_at(4, 8) | data_at(5, 0),
    ];

    match self.sysex {
      SysexStatus::Start => {
        if !end_of_sysex {
          ump[0] |= 0x1 << 20;
        }
      }
      SysexStatus::Continue => {
        if !end_of_sysex {
          ump[0] |= 0x2 << 20;
        } else {
          ump[0] |= 0x3 << 20;
        }
      }
    }

    let result = self.emit(&ump);
    self.data.clear();
    self.sysex = SysexStatus::Continue;
    result
  }

  fn handle_message(&mut self, filter: &Filter) -> Result<(), Error> {
    let result = match self.status & 0xf0 {
      // Note Off
      0x80 => {
        let note = self.data[0] & 0x7f;
        let velocity = convert7to16(self.data[1] & 0x7f);
        self.emit_channel_voice(self.status, (note as u16) << 8, (velocity as u32) << 16)
      }
      // Note On
      0x90 => {
        let note = self.data[0] & 0x7f;
        let velocity7 = self.data[1] & 0x7f;
        if velocity7 == 0 {
          let channel = self.status & 0x0f;
          self.emit_channel_voice(0x80 | channel, (note as u16) << 8, 0)
        } else {
          let velocity = convert7to16(velocity7);
          self.emit_channel_voice(self.status, (note as u16) << 8, (velocity as u32) << 16)
        }
      }
      // Polyphonic Key Pressure (Aftertouch)
      0xa0 => {
        let note = self.data[0] & 0x7f;
        let pressure = convert7to32(self.data[1] & 0x7f);
        self.emit_channel_voice(self.status, (note as u16) << 8, pressure)
      }
      // Control Change & Channel Mode messages
      0xb0 => {
        let channel = self.status & 0x0f;
        let index = self.data[0] & 0x7f;
        let data7 = self.data[1] & 0x7f;
        match index {
          // Bank MSB
          0 => {
            self.banks[channel as usize].set_msb(data7);
            Ok(())
          }
          // Data entry MSB
          6 => {
            let controller = &mut self.controllers[channel as usize];
            controller.set_data_msb(data7);
            if controller.completed() {
              let bank_and_index = controller.get_bank_and_index();
              let data = controller.get_data();
              controller.reset_data();
              self.emit_channel_voice(self.status, bank_and_index, data)?;
            }
            Ok(())
          }
          // Bank LSB
          32 => {
            self.banks[channel as usize].set_lsb(data7);
            Ok(())
          }
          // Data entry LSB
          38 => {
            let controller = &mut self.controllers[channel as usize];
            controller.set_data_lsb(data7);
            if controller.completed() {
              let bank_and_index = controller.get_bank_and_index();
              let data = controller.get_data();
              controller.reset_data();
              self.emit_channel_voice(self.status, bank_and_index, data)?;
            }
            Ok(())
          }
          // NRPN LSB
          98 => {
            self.controllers[channel as usize].set_param_lsb(ControllerKind::NonRegistered, data7);
            Ok(())
          }
          // NRPN MSB
          99 => {
            self.controllers[channel as usize].set_param_msb(ControllerKind::NonRegistered, data7);
            Ok(())
          }
          // RPN LSB
          100 => {
            self.controllers[channel as usize].set_param_lsb(ControllerKind::Registered, data7);
            Ok(())
          }
          // RPN MSB
          101 => {
            self.controllers[channel as usize].set_param_msb(ControllerKind::Registered, data7);
            Ok(())
          }
          _ => {
            let data = convert7to32(self.data[1] & 0x7f);
            let result = self.emit_channel_voice(self.status, (index as u16) << 8, data);
            if index == 121 {
              self.banks[channel as usize].reset();
              self.controllers[channel as usize].reset();
            }
            result
          }
        }
      }
      // Program Change
      0xc0 => {
        let channel = self.status & 0x0f;
        let program = self.data[0] & 0x7f;
        let bank = &self.banks[channel as usize];
        let bank_flag = bank.is_empty().then(|| 0).unwrap_or(1);
        let bank_value = bank.get_value16();
        self.emit_channel_voice(
          self.status,
          bank_flag,
          ((program as u32) << 24) | bank_value as u32,
        )
      }
      // Channel Pressure (Aftertouch)
      0xd0 => {
        let pressure = convert7to32(self.data[0] & 0x7f);
        self.emit_channel_voice(self.status, 0, pressure)
      }
      // Pitch Bend Change
      0xe0 => {
        let lsb = (self.data[0] & 0x7f) as u16;
        let msb = (self.data[1] & 0x7f) as u16;
        let data = convert14to32((msb << 7) | lsb);
        self.emit_channel_voice(self.status, 0, data)
      }
      0xf0 => {
        match self.status {
          // System Exclusive
          0xf0 => {
            todo!()
          }
          // MIDI Time Code Quarter Frame
          0xf1 => self.emit_system_common(self.status, self.data[0] & 0x7f, 0x00),
          // Song Position Pointer
          0xf2 => self.emit_system_common(self.status, self.data[0] & 0x7f, self.data[1] & 0x7f),
          // Song Select
          0xf3 => self.emit_system_common(self.status, self.data[0] & 0x7f, 0x00),
          _ => unreachable!(),
        }
      }
      _ => todo!(),
    };
    self.data.clear();
    result
  }

  #[inline]
  fn is_status(data: u8) -> bool {
    data & 0x80 != 0
  }

  #[inline]
  fn is_real_time(status: u8) -> bool {
    status >= 0xf8
  }

  #[inline]
  fn expected_len(status: u8) -> usize {
    match status & 0xf0 {
      0xc0 | 0xd0 => 1,
      0xf0 => match status {
        0xf0 | 0xf7 => 0,
        0xf1 | 0xf3 => 1,
        0xf2 => 2,
        _ => unreachable!(),
      },
      _ => 2,
    }
  }

  fn reset_controllers(&mut self) {
    self.banks.iter_mut().for_each(|bank| bank.reset());
    self
      .controllers
      .iter_mut()
      .for_each(|controller| controller.reset());
  }

  fn emit_system_common(&mut self, status: u8, data0: u8, data1: u8) -> Result<(), Error> {
    self.emit(&[Self::ump_type_and_group(0x1, self.group)
      | Self::ump_byte(status, 16)
      | Self::ump_byte(data0, 8)
      | Self::ump_byte(data1, 0)])
  }

  fn emit_channel_voice(&mut self, status: u8, index: u16, data: u32) -> Result<(), Error> {
    self.emit(&[
      Self::ump_type_and_group(0x4, self.group) | Self::ump_byte(status, 16) | (index as u32),
      data,
    ])
  }

  fn emit(&mut self, ump: &[u32]) -> Result<(), Error> {
    if self.ump.len() + ump.len() <= Self::UMP_CAPACITY {
      self.ump.extend(ump.iter());
      Ok(())
    } else {
      Err(Error::UmpOverflow)
    }
  }

  #[inline]
  fn ump_type_and_group(mtype: u8, group: u8) -> u32 {
    assert!(mtype <= 0x7f);
    assert!(group <= 0x7f);
    Self::ump_byte((mtype << 4) | group, 24)
  }

  #[inline]
  fn ump_byte(value: u8, shift: u8) -> u32 {
    (value as u32) << shift
  }
}

#[inline]
fn convert7to16(value7: u8) -> u16 {
  let bit_shifted_value = (value7 as u16) << 9;
  if value7 <= 0x40 {
    bit_shifted_value
  } else {
    let repeat_value = (value7 as u16) & 0x3f;
    bit_shifted_value | repeat_value << 3 | repeat_value >> 3
  }
}

#[inline]
fn convert7to32(value7: u8) -> u32 {
  let bit_shifted_value = (value7 as u32) << 25;
  if value7 <= 0x40 {
    bit_shifted_value
  } else {
    let repeat_value = (value7 as u32) & 0x3f;
    bit_shifted_value
      | repeat_value << 19
      | repeat_value << 13
      | repeat_value << 7
      | repeat_value << 1
      | repeat_value >> 5
  }
}

#[inline]
fn convert14to32(value14: u16) -> u32 {
  let bit_shifted_value = (value14 as u32) << 18;
  if value14 <= 0x2000 {
    bit_shifted_value
  } else {
    let repeat_value = (value14 as u32) & 0x1fff;
    bit_shifted_value | repeat_value << 5 | repeat_value >> 8
  }
}

#[cfg(test)]
mod tests {
  use crate::protocol::translate::{
    convert14to32, convert7to16, convert7to32, Error, Translator, NULL_STATUS,
  };
  use crate::Filter;

  #[test]
  fn ump_overflow() {
    let mut translator = Translator::new(0);
    let filter = Filter::new();
    for byte in vec![0x89, 0x40, 0x7f, 0x41, 0x40, 0x82, 0x18] {
      assert!(matches!(translator.push(byte, &filter), Ok(())));
    }
    assert!(matches!(
      translator.push(0, &filter),
      Err(Error::UmpOverflow)
    ));
  }

  #[test]
  fn incomplete_message() {
    assert_decodes(
      vec![0x89, 0x40, 0x82, 0x18, 0],
      vec![0x40821800, 0x00000000],
    );
  }

  #[test]
  fn system_common() {
    assert_decodes(
      vec![
        0xf1, 0x7f, 0xf2, 0x7f, 0x7f, 0xf3, 0x7f, 0xf6, 0xf8, 0xfa, 0xfb, 0xfc, 0xfe, 0xff,
      ],
      vec![
        0x10f17f00, 0x10f27f7f, 0x10f37f00, 0x10f60000, 0x10f80000, 0x10fa0000, 0x10fb0000,
        0x10fc0000, 0x10fe0000, 0x10ff0000,
      ],
    );
  }

  #[test]
  fn system_reset() {
    let translator = assert_decodes(vec![0x82, 0x18, 0xff, 0x00], vec![0x10ff0000]);

    assert_eq!(translator.status, NULL_STATUS);
    assert_eq!(translator.len, 0);
    assert!(translator.data.is_empty());
    assert!(translator.banks.iter().all(|bank| bank.is_empty()));
    assert!(translator
      .controllers
      .iter()
      .all(|controller| controller.is_empty()));
  }

  #[test]
  fn system_real_time_interleave() {
    assert_decodes(
      vec![0x89, 0x40, 0xfa, 0x7f, 0xfb, 0x41, 0xfc, 0x40],
      vec![
        0x10fa0000, 0x40894000, 0xffff0000, 0x10fb0000, 0x10fc0000, 0x40894100, 0x80000000,
      ],
    );
  }

  #[test]
  fn system_exclusive_complete() {
    assert_decodes(
      vec![0xf0, 0x01, 0x02, 0x03, 0x04, 0xf7],
      vec![0x30040102, 0x03040000],
    );
  }

  #[test]
  fn system_exclusive_start_end() {
    assert_decodes(
      vec![0xf0, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0xf7],
      vec![0x30160102, 0x03040506, 0x30300000, 0x00000000],
    );

    assert_decodes(
      vec![
        0xf0, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0xf7,
      ],
      vec![0x30160102, 0x03040506, 0x30330708, 0x09000000],
    );
  }

  #[test]
  fn system_exclusive_start_continue_end() {
    assert_decodes(
      vec![
        0xf0, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0xf7,
      ],
      vec![
        0x30160102, 0x03040506, 0x30260708, 0x090a0b0c, 0x30300000, 0x00000000,
      ],
    );

    assert_decodes(
      vec![
        0xf0, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0xf7,
      ],
      vec![
        0x30160102, 0x03040506, 0x30260708, 0x090a0b0c, 0x30310d00, 0x00000000,
      ],
    );
  }

  #[test]
  fn channel_voice_note_off() {
    assert_decodes(
      vec![0x89, 0x40, 0x7f, 0x41, 0x40, 0x82, 0x18, 0],
      vec![
        0x40894000, 0xffff0000, 0x40894100, 0x80000000, 0x40821800, 0x00000000,
      ],
    );
  }

  #[test]
  fn channel_voice_note_on() {
    assert_decodes(
      vec![0x99, 0x40, 0x7f, 0x41, 0x40, 0x92, 0x18, 0],
      vec![
        0x40994000, 0xffff0000, 0x40994100, 0x80000000, 0x40821800, 0x00000000,
      ],
    );
  }

  #[test]
  fn channel_voice_poly_pressure() {
    assert_decodes(
      vec![0xa9, 0x40, 0x7f, 0x41, 0x40, 0xa2, 0x18, 0],
      vec![
        0x40a94000, 0xffffffff, 0x40a94100, 0x80000000, 0x40a21800, 0x00000000,
      ],
    );
  }

  #[test]
  fn channel_voice_control_change() {
    assert_decodes(
      vec![0xb9, 0x40, 0x7f, 0x41, 0x40, 0xb2, 0x18, 0],
      vec![
        0x40b94000, 0xffffffff, 0x40b94100, 0x80000000, 0x40b21800, 0x00000000,
      ],
    );
  }

  #[test]
  fn channel_voice_controllers() {
    assert_decodes(
      vec![
        0xb9, 0x63, 0x13, 0x62, 0x57, // incomplete NRPN parameter on channel 9
        0xb9, 0x65, 0x12, 0x64, 0x34, // RPN parameter 0x1234 on channel 9
        0xb2, 0x63, 0x56, 0x62, 0x78, // NRPN parameter 0x5678 on channel 2
        0xb9, 0x06, 0x7f, 0x26, 0x7f, // Data on channel 9
        0xb2, 0x06, 0x40, 0x26, 0x00, // Data on channel 2
        0xb1, 0x65, 0x34, // MSB partial RPN parameter on channel 1
        0xb1, 0x06, 0x00, 0x26, 0x01, // Data on channel 1
      ],
      vec![
        0x40b91234, 0xffffffff, 0x40b25678, 0x80000000, 0x40b13400, 0x00040000,
      ],
    );
  }

  #[test]
  fn channel_voice_program_change() {
    assert_decodes(
      vec![
        0xb9, 0x00, 0x12, // set bank MSB for channel 9
        0xb9, 0x20, 0x34, // set bank LSB for channel 9
        0xc9, 0x40, 0x41, // program changes for channel 9
        0xc2, 0x18, // program change for channel 2
      ],
      vec![
        0x40c90001, 0x40001234, 0x40c90001, 0x41001234, 0x40c20000, 0x18000000,
      ],
    );
  }

  #[test]
  fn channel_voice_channel_pressure() {
    assert_decodes(
      vec![0xd9, 0x7f, 0x40, 0xd2, 0],
      vec![
        0x40d90000, 0xffffffff, 0x40d90000, 0x80000000, 0x40d20000, 0x00000000,
      ],
    );
  }

  #[test]
  fn channel_voice_pitch_bend() {
    assert_decodes(
      vec![0xe9, 0x7f, 0x7f, 0x00, 0x40, 0xe2, 0x00, 0x00],
      vec![
        0x40e90000, 0xffffffff, 0x40e90000, 0x80000000, 0x40e20000, 0x00000000,
      ],
    );
  }

  #[test]
  fn channel_mode_reset_all_controllers() {
    let translator = assert_decodes(
      vec![
        0xb9, 0x00, 0x12, // set bank MSB for channel 9
        0xb9, 0x20, 0x34, // set bank LSB for channel 9
        0xb2, 0x00, 0x56, // set bank MSB for channel 2
        0xb2, 0x20, 0x78, // set bank LSB for channel 2
        0xb9, 0x65, 0x12, 0x64, 0x34, // RPN parameter 0x1234 on channel 9
        0xb2, 0x63, 0x56, 0x62, 0x78, // NRPN parameter 0x5678 on channel 2
        0xb9, 0x79, 0x00, // Reset All Controllers
      ],
      vec![0x40b97900, 0x00000000],
    );

    assert!(translator.banks[9].is_empty());
    assert!(!translator.banks[2].is_empty());

    assert!(translator.controllers[9].is_empty());
    assert!(!translator.controllers[2].is_empty());
  }

  #[test]
  fn convert_7to16() {
    let max_value: u32 = 1 << 7;
    for src_value in 0..max_value {
      assert_eq!(
        convert7to16(src_value as u8) as u32,
        scale_up(7, 16, src_value, false),
      )
    }
  }

  #[test]
  fn convert_7to32() {
    let max_value: u32 = 1 << 7;
    for src_value in 0..max_value {
      assert_eq!(
        convert7to32(src_value as u8) as u32,
        scale_up(7, 32, src_value, false),
      )
    }
  }

  #[test]
  fn convert_14to32() {
    let max_value: u32 = 1 << 14;
    for src_value in 0..max_value {
      assert_eq!(
        convert14to32(src_value as u16),
        scale_up(14, 32, src_value, false),
      )
    }
  }

  fn assert_decodes(bytes: Vec<u8>, expected: Vec<u32>) -> Translator {
    let mut translator = Translator::new(0);
    let filter = Filter::new();
    let mut bytes = bytes.into_iter();
    let mut ump = Vec::new();

    while let Some(byte) = bytes.next() {
      match translator.push(byte, &filter) {
        Ok(()) => {
          while let Some(data) = translator.pop() {
            ump.push(data);
          }
        }
        Err(err) => panic!("Unexpected error: {:?}", err),
      }
    }

    let found_ump = ump
      .iter()
      .map(|word| format!("0x{:08x}", word))
      .collect::<Vec<String>>();
    let expected_ump = expected
      .iter()
      .map(|word| format!("0x{:08x}", word))
      .collect::<Vec<String>>();
    assert_eq!(
      ump, expected,
      "Unexpected result:\n    found: {:?}\n expected: {:?}\n",
      found_ump, expected_ump
    );

    translator
  }

  fn scale_up(src_bits: u8, dst_bits: u8, src_val: u32, debug: bool) -> u32 {
    // simple bit shift
    let scale_bits = dst_bits - src_bits;
    let mut bit_shifted_value = src_val << scale_bits;
    let src_center = (1 << (src_bits - 1)) as u32;
    if debug {
      println!(
        "scale_bits={}, src_center={} (0x{:0x})",
        scale_bits, src_center, src_center
      );
    }
    if src_val <= src_center {
      return bit_shifted_value;
    }
    // expanded bit repeat scheme
    let repeat_bits = src_bits - 1;
    let repeat_mask = ((1 << repeat_bits) - 1) as u32;
    let mut repeat_value = src_val & repeat_mask;
    let mut shift_bits;
    if debug {
      println!(
        "repeat_bits={}, repeat_mask=0x{:0x}\n",
        repeat_bits, repeat_mask
      );
    }
    if scale_bits > repeat_bits {
      repeat_value <<= scale_bits - repeat_bits;
      shift_bits = -((scale_bits - repeat_bits) as i32);
    } else {
      repeat_value >>= repeat_bits - scale_bits;
      shift_bits = (repeat_bits - scale_bits) as i32;
    }
    while repeat_value != 0 {
      if debug {
        println!(
          "repeat_value {} [{}]",
          if shift_bits < 0 { "<<" } else { ">>" },
          shift_bits.abs()
        );
      }
      bit_shifted_value |= repeat_value;
      repeat_value >>= repeat_bits;
      shift_bits += repeat_bits as i32;
    }

    if debug {
      println!(
        "::: {} ==> {} (0x{:0x})",
        src_val, bit_shifted_value, bit_shifted_value
      );
    }

    return bit_shifted_value;
  }
}
