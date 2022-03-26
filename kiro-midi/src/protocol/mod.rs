pub mod decoder;
pub mod messages;

pub trait Decode {
  fn decode(ump: &[u32]) -> Self;
}

pub trait Encode<const N: usize> {
  fn encode(&self) -> [u32; N];
}
