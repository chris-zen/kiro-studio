pub mod codec;
pub mod messages;
pub mod translate;

pub trait Decode {
  fn decode(ump: &[u32]) -> Option<Self>
  where
    Self: Sized;
}

pub trait Encode<const N: usize> {
  fn encode(&self) -> [u32; N];
}
