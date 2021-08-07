#[derive(Debug, PartialEq)]
pub enum InsertResult<K, V> {
  Replaced(K, V),
  Inserted(usize),
}

#[derive(Debug, Clone)]
pub struct SortedMap<K, V, const O: usize> {
  size: usize,
  data: [Option<(K, V)>; O],
}

impl<K, V, const O: usize> SortedMap<K, V, O>
where
  K: Clone + PartialOrd,
  V: Clone,
{
  const MIN_SIZE: usize = O / 2;

  pub fn new() -> Self {
    Self {
      size: 0,
      data: array_macro::array![None; O],
    }
  }

  pub fn from_key_value(key: K, value: V) -> Self {
    assert!(O > 0);
    let mut data = array_macro::array![None; O];
    data[0] = Some((key, value));
    Self { size: 1, data }
  }

  pub fn len(&self) -> usize {
    self.size
  }

  pub fn data(&self) -> &[Option<(K, V)>] {
    &self.data[0..self.size]
  }

  pub fn first_key(&self) -> Option<&K> {
    self
      .data
      .first()
      .and_then(|first| first.as_ref().map(|(k, _)| k))
  }

  pub fn split(&mut self) -> SortedMap<K, V, O> {
    let mut new_sorted_map = SortedMap::new();
    for index in Self::MIN_SIZE..self.size {
      let record = self.data[index].take();
      new_sorted_map.data[new_sorted_map.size] = record;
      new_sorted_map.size += 1;
    }
    self.size = Self::MIN_SIZE;
    new_sorted_map
  }

  pub fn insert(&mut self, key: K, value: V) -> Result<InsertResult<K, V>, (K, V)> {
    if self.size == O {
      Err((key, value))
    } else {
      let mut index = 0;
      while self.key_greater_than_record_at(&key, index) {
        index += 1;
      }

      let same_key = self.data[index]
        .as_ref()
        .map_or(false, |(record_key, _)| key == *record_key);

      if same_key {
        // if it is the same key, then records[index] have some record, then it's safe to use unwrap
        let (prev_key, prev_value) = self.data[index].take().unwrap();
        self.data[index] = Some((key, value));
        Ok(InsertResult::Replaced(prev_key, prev_value))
      } else if index > self.size {
        self.data[index] = Some((key, value));
        self.size = index + 1;
        Ok(InsertResult::Inserted(index))
      } else {
        let insert_index = index;
        let mut insert_record = Some((key, value));
        while insert_record.is_some() {
          let next_record = self.data[index].take();
          self.data[index] = insert_record.take();
          insert_record = next_record;
          index += 1;
        }
        self.size += 1;
        Ok(InsertResult::Inserted(insert_index))
      }
    }
  }

  fn key_greater_than_record_at(&self, key: &K, index: usize) -> bool {
    self.data[index]
      .as_ref()
      .map_or(false, |(record_key, _)| key > record_key)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn insert_new() {
    let mut map = SortedMap::<u32, u32, 2>::new();
    let result = map.insert(1, 10);
    assert_eq!(result, Ok(InsertResult::Inserted(0)));
    assert_eq!(map.data, [Some((1, 10)), None]);
    assert_eq!(map.size, 1);
  }

  #[test]
  fn insert_same_key() {
    let mut map = SortedMap::<u32, u32, 2>::new();
    let result = map.insert(1, 10);
    assert_eq!(result, Ok(InsertResult::Inserted(0)));
    let result = map.insert(1, 20);
    assert_eq!(result, Ok(InsertResult::Replaced(1, 10)));
    assert_eq!(map.data, [Some((1, 20)), None]);
    assert_eq!(map.size, 1);
  }

  #[test]
  fn insert_after_last() {
    let mut map = SortedMap::<u32, u32, 3>::new();
    let result = map.insert(1, 10);
    assert_eq!(result, Ok(InsertResult::Inserted(0)));
    let result = map.insert(2, 20);
    assert_eq!(result, Ok(InsertResult::Inserted(1)));
    let result = map.insert(3, 30);
    assert_eq!(result, Ok(InsertResult::Inserted(2)));
    assert_eq!(map.data, [Some((1, 10)), Some((2, 20)), Some((3, 30))]);
    assert_eq!(map.size, 3);
  }

  #[test]
  fn insert_before_first() {
    let mut map = SortedMap::<u32, u32, 3>::new();
    let result = map.insert(2, 20);
    assert_eq!(result, Ok(InsertResult::Inserted(0)));
    let result = map.insert(1, 10);
    assert_eq!(result, Ok(InsertResult::Inserted(0)));
    assert_eq!(map.data, [Some((1, 10)), Some((2, 20)), None]);
    assert_eq!(map.size, 2);
  }

  #[test]
  fn insert_before_last() {
    let mut map = SortedMap::<u32, u32, 3>::new();
    let result = map.insert(1, 10);
    assert_eq!(result, Ok(InsertResult::Inserted(0)));
    let result = map.insert(3, 30);
    assert_eq!(result, Ok(InsertResult::Inserted(1)));
    let result = map.insert(2, 20);
    assert_eq!(result, Ok(InsertResult::Inserted(1)));
    assert_eq!(map.data, [Some((1, 10)), Some((2, 20)), Some((3, 30))]);
    assert_eq!(map.size, 3);
  }

  #[test]
  fn insert_fails_with_overflow() {
    let mut map = SortedMap::<u32, u32, 3>::new();
    map.insert(1, 10).ok();
    map.insert(2, 20).ok();
    map.insert(4, 40).ok();

    let result = map.insert(3, 30);
    assert_eq!(result, Err((3, 30)));
  }

  #[test]
  fn split_odd() {
    let mut map = SortedMap::<u32, u32, 3>::new();
    map.insert(1, 10).ok();
    map.insert(2, 20).ok();
    map.insert(3, 30).ok();
    let split = map.split();
    assert_eq!(map.data(), &[Some((1, 10))]);
    assert_eq!(split.data(), &[Some((2, 20)), Some((3, 30))]);
  }

  #[test]
  fn split_even() {
    let mut map = SortedMap::<u32, u32, 4>::new();
    map.insert(1, 10).ok();
    map.insert(2, 20).ok();
    map.insert(3, 30).ok();
    map.insert(4, 40).ok();
    let split = map.split();
    assert_eq!(map.data(), &[Some((1, 10)), Some((2, 20))]);
    assert_eq!(split.data(), &[Some((3, 30)), Some((4, 40))]);
  }
}
