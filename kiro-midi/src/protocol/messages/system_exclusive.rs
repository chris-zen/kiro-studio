type Payload6 = Payload<6>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SystemExclusive {
  Complete(Payload6),
  Start(Payload6),
  Continue(Payload6),
  End(Payload6),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Payload<const N: usize> {
  len: u8,
  data: [u8; N],
}

impl<const N: usize> Payload<N> {
  pub fn new(source: &[u8]) -> Result<Self, ()> {
    if source.len() <= N {
      let mut data = [0; N];
      data[0..source.len()].copy_from_slice(source);
      Ok(Self {
        len: source.len() as u8,
        data,
      })
    } else {
      Err(())
    }
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.len as usize
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    self.len == 0
  }

  pub fn push(&mut self, data: u8) -> Result<(), ()> {
    if self.len() < N {
      self.data[self.len()] = data;
      self.len += 1;
      Ok(())
    } else {
      Err(())
    }
  }

  pub fn extend(&mut self, data: &[u8]) -> Result<(), ()> {
    let start = self.len();
    if start + data.len() <= N {
      let end = start + data.len();
      self.data[start..end].copy_from_slice(data);
      self.len = end as u8;
      Ok(())
    } else {
      Err(())
    }
  }

  pub fn as_slice(&self) -> &[u8] {
    &self.data[0..self.len()]
  }
}

impl<const N: usize> Default for Payload<N> {
  fn default() -> Self {
    Self {
      len: 0,
      data: [0; N],
    }
  }
}

impl<const N: usize> From<[u8; N]> for Payload<N> {
  fn from(data: [u8; N]) -> Self {
    Self { len: N as u8, data }
  }
}

impl<const N: usize> TryFrom<&[u8]> for Payload<N> {
  type Error = ();
  fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
    if data.len() <= N {
      Payload::new(data)
    } else {
      Err(())
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::messages::system_exclusive::Payload;

  #[test]
  fn new_success() {
    let result = Payload::<2>::new(&[1, 2]);
    assert!(matches!(result, Ok(payload) if payload.as_slice() == &[1, 2]));
  }

  #[test]
  fn new_overflow() {
    let result = Payload::<2>::new(&[1, 2, 3]);
    assert!(matches!(result, Err(())));
  }

  #[test]
  fn empty() {
    let payload = Payload::<2>::default();
    assert_eq!(payload.len(), 0);
    assert!(payload.is_empty());
  }

  #[test]
  fn non_empty() {
    let payload = Payload::<4>::new(&[1, 2]).expect("payload");
    assert_eq!(payload.len(), 2);
    assert!(!payload.is_empty());
  }

  #[test]
  fn push_success() {
    let mut payload = Payload::<2>::default();
    assert_eq!(payload.push(1), Ok(()));
    assert_eq!(payload.push(2), Ok(()));
    assert_eq!(payload.as_slice(), &[1, 2]);
  }

  #[test]
  fn push_overflow() {
    let mut payload = Payload::<2>::default();
    assert_eq!(payload.push(1), Ok(()));
    assert_eq!(payload.push(2), Ok(()));
    assert_eq!(payload.push(3), Err(()));
    assert_eq!(payload.as_slice(), &[1, 2]);
  }

  #[test]
  fn extend_success() {
    let mut payload = Payload::<4>::new(&[1, 2]).expect("payload");
    assert_eq!(payload.extend(&[3, 4]), Ok(()));
    assert_eq!(payload.as_slice(), &[1, 2, 3, 4]);
  }

  #[test]
  fn extend_overflow() {
    let mut payload = Payload::<4>::new(&[1, 2]).expect("payload");
    assert_eq!(payload.extend(&[3, 4, 5]), Err(()));
    assert_eq!(payload.as_slice(), &[1, 2]);
  }
}
