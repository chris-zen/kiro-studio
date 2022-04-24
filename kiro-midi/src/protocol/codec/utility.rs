use crate::messages::utility::Utility;

pub fn decode_utility(ump: &[u32]) -> Option<Utility> {
  (ump.len() == 1).then(|| {
    let status = ((ump[0] >> 20) & 0x0f) as u8;
    match status {
      0b0000 => Utility::Noop,
      _ => unimplemented!(),
    }
  })
}

#[cfg(test)]
mod tests {}
