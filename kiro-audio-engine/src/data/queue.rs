use thiserror::Error;

use crate::events::slab_pool::SlabRef;
use std::fmt::Debug;

#[derive(Error, Debug, PartialEq)]
pub enum QueueError {
  #[error("Error allocating an slab")]
  AllocSlab,

  #[error("Error releasing an slab")]
  ReleaseSlab,
}

pub trait QueueAllocator<T> {
  fn acquire(&mut self) -> Result<SlabRef<T>, QueueError>;
  fn release(&mut self, slab: SlabRef<T>) -> Result<(), SlabRef<T>>;
}

#[derive(Debug)]
struct SlotRef<T> {
  slab_ref: SlabRef<T>,
  index: usize,
}

pub struct Queue<T> {
  reads: Option<SlotRef<T>>,
  writes: Option<SlotRef<T>>,
  len: usize,
}

impl<T> Queue<T> {
  pub fn new() -> Self {
    Self {
      reads: None,
      writes: None,
      len: 0,
    }
  }

  pub fn len(&self) -> usize {
    self.len
  }

  pub fn push<A: QueueAllocator<T>>(&mut self, data: T, alloc: &mut A) -> Result<(), T> {
    let writes_and_data = match self.writes.take() {
      None => match alloc.acquire() {
        Ok(slab_ref) => {
          let writes = SlotRef { slab_ref, index: 0 };
          Ok((writes, data))
        }
        Err(_) => Err(data),
      },
      Some(writes) => {
        if writes.index >= writes.slab_ref.borrow().data.len() {
          match alloc.acquire() {
            Ok(new_slab_ref) => {
              let SlotRef {
                slab_ref: prev_slab_ref,
                ..
              } = writes;
              prev_slab_ref.borrow_mut().next = Some(new_slab_ref.clone());
              let writes = SlotRef {
                slab_ref: new_slab_ref,
                index: 0,
              };
              Ok((writes, data))
            }
            Err(_) => {
              self.writes = Some(writes);
              Err(data)
            }
          }
        } else {
          Ok((writes, data))
        }
      }
    };

    let (mut writes, data) = writes_and_data?;

    let mut slab = writes.slab_ref.borrow_mut();
    slab.data[writes.index] = Some(data);
    writes.index += 1;
    self.len += 1;
    drop(slab);

    if self.reads.is_none() {
      self.reads = Some(SlotRef {
        slab_ref: writes.slab_ref.clone(),
        index: 0,
      })
    }
    self.writes = Some(writes);

    Ok(())
  }

  pub fn pop<A: QueueAllocator<T>>(&mut self, alloc: &mut A) -> Result<Option<T>, QueueError>
  where
    T: Debug,
  {
    let result = match self.reads.take() {
      None => Ok((None, None)),
      Some(mut reads) => {
        if reads.index >= reads.slab_ref.borrow().data.len() {
          let SlotRef {
            slab_ref: prev_slab_ref,
            ..
          } = reads;
          let next = prev_slab_ref.borrow_mut().next.take();

          match alloc.release(prev_slab_ref) {
            Ok(()) => match next {
              None => Ok((None, None)),
              Some(next_slab_ref) => {
                let data = next_slab_ref.borrow_mut().data[0].take();
                let reads = Some(SlotRef {
                  slab_ref: next_slab_ref,
                  index: 1,
                });
                self.len -= 1;
                Ok((reads, data))
              }
            },
            Err(prev_slab_ref) => {
              prev_slab_ref.borrow_mut().next = next;
              let len = prev_slab_ref.borrow().data.len();
              self.reads = Some(SlotRef {
                slab_ref: prev_slab_ref,
                index: len,
              });
              Err(QueueError::ReleaseSlab)
            }
          }
        } else if self.len > 0 {
          let mut slab = reads.slab_ref.borrow_mut();
          let data = slab.data[reads.index].take();
          drop(slab);
          reads.index += 1;
          self.len -= 1;
          Ok((Some(reads), data))
        } else {
          let SlotRef { slab_ref, index } = reads;
          match alloc.release(slab_ref) {
            Ok(()) => Ok((None, None)),
            Err(slab_ref) => {
              self.reads = Some(SlotRef { slab_ref, index });
              Err(QueueError::ReleaseSlab)
            }
          }
        }
      }
    };

    let (reads, data) = result?;

    self.reads = reads;
    if self.reads.is_none() {
      self.writes = None;
    }

    Ok(data)
  }
}

  #[test]
  fn pop_with_some_slab_release() {
    let mut q = Queue::new();
    let mut alloc = TestAllocator::new(2, 2);
    assert!(q.push(1, &mut alloc).is_ok());
    assert!(q.push(2, &mut alloc).is_ok());
    assert!(q.push(3, &mut alloc).is_ok());
    assert_eq!(alloc.0.len(), 0);
    assert_eq!(q.pop(&mut alloc), Ok(Some(1)));
    assert_eq!(q.pop(&mut alloc), Ok(Some(2)));
    assert_eq!(q.pop(&mut alloc), Ok(Some(3)));
    assert_eq!(q.len(), 0);
    assert_eq!(alloc.0.len(), 1);
    assert_eq!(q.pop(&mut alloc), Ok(None));
    assert_eq!(alloc.0.len(), 2);
    assert!(q.writes.is_none());
    assert!(q.reads.is_none());
  }

#[cfg(test)]
mod tests {
  use crate::events::queue::{Queue, QueueAllocator, QueueError};
  use crate::events::slab_pool::{Slab, SlabRef};

  struct TestAllocator<T>(Vec<SlabRef<T>>, bool);

  impl<T> TestAllocator<T> {
    pub fn new(num_slabs: usize, slab_size: usize) -> Self {
      let mut slabs = Vec::with_capacity(num_slabs);
      slabs.resize_with(num_slabs, || SlabRef::from(Slab::new(slab_size, None)));
      Self(slabs, false)
    }

    pub fn set_failing_release(&mut self, failing: bool) {
      self.1 = failing;
    }
  }

  impl<T> QueueAllocator<T> for TestAllocator<T> {
    fn acquire(&mut self) -> Result<SlabRef<T>, QueueError> {
      self.0.pop().ok_or(QueueError::AllocSlab)
    }

    fn release(&mut self, slab: SlabRef<T>) -> Result<(), SlabRef<T>> {
      if !self.1 {
        self.0.push(slab);
        Ok(())
      } else {
        Err(slab)
      }
    }
  }

  #[test]
  fn push_when_empty() {
    let mut q = Queue::new();
    let mut alloc = TestAllocator::new(2, 2);
    assert!(q.push(1, &mut alloc).is_ok());
    assert_eq!(q.len(), 1);
    let writes_ref_index = q.writes.as_ref().unwrap();
    let writes_slab = writes_ref_index.slab_ref.borrow();
    assert!(writes_slab.next.is_none());
    assert_eq!(writes_slab.data, vec![Some(1), None]);
    assert_eq!(writes_ref_index.index, 1);
    let reads_ref_index = q.reads.as_ref().unwrap();
    let reads_slab = reads_ref_index.slab_ref.borrow();
    assert!(reads_slab.next.is_none());
    assert_eq!(reads_slab.data, vec![Some(1), None]);
    assert_eq!(reads_ref_index.index, 0);
  }

  #[test]
  fn push_when_empty_and_alloc_fails() {
    let mut q = Queue::new();
    let mut alloc = TestAllocator::new(0, 2);
    assert_eq!(q.push(1, &mut alloc), Err(1));
  }

  #[test]
  fn push_until_first_slab_is_full() {
    let mut q = Queue::new();
    let mut alloc = TestAllocator::new(2, 2);
    assert!(q.push(1, &mut alloc).is_ok());
    assert!(q.push(2, &mut alloc).is_ok());
    assert_eq!(q.len(), 2);
    let writes_ref_index = q.writes.as_ref().unwrap();
    let writes_slab = writes_ref_index.slab_ref.borrow();
    assert!(writes_slab.next.is_none());
    assert_eq!(writes_slab.data, vec![Some(1), Some(2)]);
    assert_eq!(writes_ref_index.index, 2);
  }

  #[test]
  fn push_beyond_the_first_slab() {
    let mut q = Queue::new();
    let mut alloc = TestAllocator::new(2, 2);
    assert!(q.push(1, &mut alloc).is_ok());
    assert!(q.push(2, &mut alloc).is_ok());
    assert!(q.push(3, &mut alloc).is_ok());
    let writes_ref_index = q.writes.as_ref().unwrap();
    let writes_slab = writes_ref_index.slab_ref.borrow();
    assert!(writes_slab.next.is_none());
    assert_eq!(writes_slab.data, vec![Some(3), None]);
    assert_eq!(writes_ref_index.index, 1);
    let reads_ref_index = q.reads.as_ref().unwrap();
    let reads_slab = reads_ref_index.slab_ref.borrow();
    assert!(reads_slab.next.is_some());
    assert_eq!(reads_slab.data, vec![Some(1), Some(2)]);
    assert_eq!(reads_ref_index.index, 0);
  }

  #[test]
  fn push_beyond_the_first_slab_and_alloc_fails() {
    let mut q = Queue::new();
    let mut alloc = TestAllocator::new(1, 2);
    assert!(q.push(1, &mut alloc).is_ok());
    assert!(q.push(2, &mut alloc).is_ok());
    assert_eq!(q.push(3, &mut alloc), Err(3));
    assert_eq!(q.len(), 2);
    let writes_ref_index = q.writes.as_ref().unwrap();
    let writes_slab = writes_ref_index.slab_ref.borrow();
    assert!(writes_slab.next.is_none());
    assert_eq!(writes_slab.data, vec![Some(1), Some(2)]);
    assert_eq!(writes_ref_index.index, 2);
  }

  #[test]
  fn pop_when_empty() {
    let mut q = Queue::<i32>::new();
    let mut alloc = TestAllocator::new(0, 2);
    assert_eq!(q.pop(&mut alloc), Ok(None));
    assert_eq!(q.pop(&mut alloc), Ok(None));
    assert_eq!(q.len(), 0);
  }

  #[test]
  fn pop_without_slab_release() {
    let mut q = Queue::new();
    let mut alloc = TestAllocator::new(1, 4);
    assert!(q.push(1, &mut alloc).is_ok());
    assert!(q.push(2, &mut alloc).is_ok());
    assert_eq!(q.pop(&mut alloc), Ok(Some(1)));
    assert_eq!(q.len(), 1);
    assert_eq!(q.pop(&mut alloc), Ok(Some(2)));
    assert_eq!(q.len(), 0);
    assert_eq!(q.pop(&mut alloc), Ok(None));
  }

  #[test]
  fn pop_with_last_slab_release() {
    let mut q = Queue::new();
    let mut alloc = TestAllocator::new(1, 2);
    assert!(q.push(1, &mut alloc).is_ok());
    assert!(q.push(2, &mut alloc).is_ok());
    assert_eq!(q.pop(&mut alloc), Ok(Some(1)));
    assert_eq!(q.pop(&mut alloc), Ok(Some(2)));
    assert_eq!(q.len(), 0);
    assert_eq!(alloc.0.len(), 0);
    assert_eq!(q.pop(&mut alloc), Ok(None));
    assert_eq!(alloc.0.len(), 1);
    assert!(q.writes.is_none());
    assert!(q.reads.is_none());
  }

  #[test]
  fn pop_with_last_slab_release_failing() {
    let mut q = Queue::new();
    let mut alloc = TestAllocator::new(1, 2);
    alloc.set_failing_release(true);
    assert!(q.push(1, &mut alloc).is_ok());
    assert!(q.push(2, &mut alloc).is_ok());
    assert_eq!(q.pop(&mut alloc), Ok(Some(1)));
    assert_eq!(q.pop(&mut alloc), Ok(Some(2)));
    assert_eq!(q.pop(&mut alloc), Err(QueueError::ReleaseSlab));
    assert_eq!(alloc.0.len(), 0);
    alloc.set_failing_release(false);
    assert_eq!(q.pop(&mut alloc), Ok(None));
    assert_eq!(alloc.0.len(), 1);
  }

  #[test]
  fn pop_with_some_slab_release_failing() {
    let mut q = Queue::new();
    let mut alloc = TestAllocator::new(2, 2);
    assert!(q.push(1, &mut alloc).is_ok());
    assert!(q.push(2, &mut alloc).is_ok());
    assert!(q.push(3, &mut alloc).is_ok());
    assert_eq!(alloc.0.len(), 0);
    assert_eq!(q.pop(&mut alloc), Ok(Some(1)));
    assert_eq!(q.pop(&mut alloc), Ok(Some(2)));
    alloc.set_failing_release(true);
    assert_eq!(q.pop(&mut alloc), Err(QueueError::ReleaseSlab));
    alloc.set_failing_release(false);
    assert_eq!(q.pop(&mut alloc), Ok(Some(3)));
    assert_eq!(q.len(), 0);
    assert_eq!(alloc.0.len(), 1);
    alloc.set_failing_release(true);
    assert_eq!(q.pop(&mut alloc), Err(QueueError::ReleaseSlab));
    assert_eq!(alloc.0.len(), 1);
    alloc.set_failing_release(false);
    assert_eq!(q.pop(&mut alloc), Ok(None));
    assert_eq!(alloc.0.len(), 2);
    assert!(q.writes.is_none());
    assert!(q.reads.is_none());
  }
}
