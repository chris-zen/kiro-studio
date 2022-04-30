use crate::messages::system_exclusive::{Payload, SystemExclusive};

pub fn decode_system_exclusive(ump: &[u32]) -> Option<SystemExclusive> {
  if ump.len() == 2 {
    let status = (ump[0] >> 20) & 0x7f;
    let len = (ump[0] >> 16) & 0x0f;
    let mut payload = Payload::default();
    if len > 0 {
      let data0 = ((ump[0] & 0xffff) as u16).to_be_bytes().map(|b| b & 0x7f);
      let end = usize::min(len as usize, 2);
      payload.extend(&data0[0..end]).ok()?;
      if len > 2 {
        let data1 = ump[1].to_be_bytes().map(|b| b & 0x7f);
        let end = usize::min(len as usize - 2, 4);
        payload.extend(&data1[0..end]).ok()?;
      }
    }
    match status {
      0x00 => Some(SystemExclusive::Complete(payload)),
      0x01 => Some(SystemExclusive::Start(payload)),
      0x02 => Some(SystemExclusive::Continue(payload)),
      0x03 => Some(SystemExclusive::End(payload)),
      _ => None,
    }
  } else {
    None
  }
}

#[cfg(test)]
mod tests {
  use crate::messages::system_exclusive::{Payload, SystemExclusive};
  use crate::protocol::codec::system_exclusive::decode_system_exclusive;

  #[test]
  fn payload_empty() {
    assert_eq!(
      decode_system_exclusive(vec![0x30000102, 0x03040506].as_slice()),
      Some(SystemExclusive::Complete(Payload::default()))
    );
  }

  #[test]
  fn payload_full() {
    assert_eq!(
      decode_system_exclusive(vec![0x30060102, 0x03040506].as_slice()),
      Some(SystemExclusive::Complete(Payload::from([
        0x01u8, 0x02, 0x03, 0x04, 0x05, 0x06,
      ])))
    );
  }

  #[test]
  fn payload_7bits() {
    assert_eq!(
      decode_system_exclusive(vec![0x30068182, 0x83848586].as_slice()),
      Some(SystemExclusive::Complete(Payload::from([
        0x01u8, 0x02, 0x03, 0x04, 0x05, 0x06,
      ])))
    );
  }

  #[test]
  fn status_start() {
    assert_eq!(
      decode_system_exclusive(vec![0x30100000, 0x00000000].as_slice()),
      Some(SystemExclusive::Start(Payload::default()))
    );
  }

  #[test]
  fn status_continue() {
    assert_eq!(
      decode_system_exclusive(vec![0x30200000, 0x00000000].as_slice()),
      Some(SystemExclusive::Continue(Payload::default()))
    );
  }

  #[test]
  fn status_end() {
    assert_eq!(
      decode_system_exclusive(vec![0x30300000, 0x00000000].as_slice()),
      Some(SystemExclusive::End(Payload::default()))
    );
  }
}
