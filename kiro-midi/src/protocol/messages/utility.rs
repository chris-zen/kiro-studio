use crate::protocol::Decode;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Utility {
  Noop,
  // TODO ...
}

impl Decode for Utility {
  fn decode(ump: &[u32]) -> Self {
    assert_eq!(ump.len(), 1);
    let status = ((ump[0] >> 20) & 0x0f) as u8;
    match status {
      0b0000 => Self::Noop,
      _ => unimplemented!(),
    }
  }
}
