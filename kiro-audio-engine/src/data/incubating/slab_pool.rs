use std::sync::Arc;

use std::cell::RefCell;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum SlabPoolError {
  #[error("Out of Slabs")]
  OutOfSlabs,

  #[error("Non empty Slab has been released")]
  NonEmptySlabRelease,
}

pub type Result<T> = core::result::Result<T, SlabPoolError>;

#[derive(Debug)]
pub struct SlabRef<T>(Arc<RefCell<Slab<T>>>);

impl<T> SlabRef<T> {
  pub fn new(slab: Arc<RefCell<Slab<T>>>) -> Self {
    Self(slab)
  }

  pub fn from(slab: Slab<T>) -> Self {
    Self(Arc::new(RefCell::new(slab)))
  }

  pub fn borrow(&self) -> std::cell::Ref<Slab<T>> {
    (*self.0).borrow()
  }

  pub fn borrow_mut(&self) -> std::cell::RefMut<Slab<T>> {
    (*self.0).borrow_mut()
  }
}

impl<T> Clone for SlabRef<T> {
  fn clone(&self) -> Self {
    Self::new(self.0.clone())
  }
}

#[derive(Debug)]
pub struct Slab<T> {
  pub(crate) next: Option<SlabRef<T>>,
  pub(crate) data: Vec<Option<T>>,
}

impl<T> Slab<T> {
  pub fn new(capacity: usize, next: Option<SlabRef<T>>) -> Self {
    let mut data = Vec::with_capacity(capacity);
    data.resize_with(capacity, || None);
    Self { next, data }
  }

  pub fn len(&self) -> usize {
    self.data.len()
  }
}

pub struct SlabPool<T> {
  head: Option<SlabRef<T>>,
}

impl<T> SlabPool<T> {
  pub fn new(pool_capacity: usize, slab_capacity: usize) -> Self {
    let mut head = None;
    for _ in 0..pool_capacity {
      head = Some(SlabRef::from(Slab::new(slab_capacity, head.take())));
    }
    Self { head }
  }

  pub fn acquire(&mut self) -> Result<SlabRef<T>> {
    let mut slot = self.head.take();
    let mut next = slot
      .as_mut()
      .and_then(|head| (*head).borrow_mut().next.take());
    self.head = next.take();
    slot.ok_or(SlabPoolError::OutOfSlabs)
  }

  pub fn release(&mut self, slab: SlabRef<T>) -> Result<()> {
    let is_empty = slab
      .borrow()
      .data
      .iter()
      .all(|slab_data| slab_data.is_none());
    if is_empty {
      slab.borrow_mut().next = self.head.take();
      self.head = Some(slab);
      Ok(())
    } else {
      Err(SlabPoolError::NonEmptySlabRelease)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn slab_new() {
    let slab1 = Slab::<i32>::new(2, None);
    assert!(slab1.next.is_none());
    assert_eq!(slab1.data, vec![None, None]);

    let slab2 = Slab::<i32>::new(2, Some(SlabRef::from(slab1)));
    assert!(slab2.next.is_some());
  }

  #[test]
  fn pool_new() {
    let mut pool = SlabPool::<i32>::new(3, 2);
    assert_eq!(count_slabs(pool.head.take()), 3);
  }

  #[test]
  fn pool_acquire_when_empty() {
    let mut pool = SlabPool::<i32>::new(0, 2);
    assert_eq!(pool.acquire().unwrap_err(), SlabPoolError::OutOfSlabs);
  }

  #[test]
  fn pool_acquire_success() {
    let mut pool = SlabPool::<i32>::new(3, 2);
    let result = pool.acquire();
    assert!(result.is_ok());
    assert_eq!(count_slabs(pool.head.take()), 2);
  }

  #[test]
  fn pool_release_empty_slab() {
    let mut pool = SlabPool::<i32>::new(3, 2);
    let slab = pool.acquire().unwrap();
    pool.release(slab).unwrap();
    assert_eq!(count_slabs(pool.head.take()), 3);
  }

  #[test]
  fn pool_release_non_empty_slab() {
    let mut pool = SlabPool::<i32>::new(3, 2);
    let slab = pool.acquire().unwrap();
    slab.borrow_mut().data[0] = Some(1);
    assert_eq!(
      pool.release(slab).unwrap_err(),
      SlabPoolError::NonEmptySlabRelease
    );
  }

  fn count_slabs<T>(mut head: Option<SlabRef<T>>) -> usize {
    let mut count = 0;
    while let Some(slab) = head.take() {
      count += 1;
      head = slab.borrow_mut().next.take();
    }
    count
  }
}
