use std::fmt::Debug;

use thiserror::Error;

use crate::events::allocator::{AllocError, AllocRef, Allocator};

type DataBlockRef<T, const CAPACITY: usize> = AllocRef<DataBlock<T, CAPACITY>>;

#[derive(Debug, Clone)]
pub struct DataBlock<T, const CAPACITY: usize> {
  pub(crate) next: Option<DataBlockRef<T, CAPACITY>>,
  pub(crate) data: [Option<T>; CAPACITY],
}

impl<T: Clone, const CAPACITY: usize> DataBlock<T, CAPACITY> {
  pub fn new(next: Option<DataBlockRef<T, CAPACITY>>) -> Self {
    Self {
      next,
      data: array_macro::array![None; CAPACITY],
    }
  }

  pub fn len(&self) -> usize {
    self.data.len()
  }
}

#[derive(Error, Debug, PartialEq)]
pub enum QueueError {
  #[error("Allocation error: {0}")]
  Allocation(AllocError),
}

#[derive(Debug)]
struct DataRef<T, const CAPACITY: usize> {
  data_block_ref: DataBlockRef<T, CAPACITY>,
  index: usize,
}

pub struct Queue<T, const BLOCK_CAPACITY: usize> {
  reads: Option<DataRef<T, BLOCK_CAPACITY>>,
  writes: Option<DataRef<T, BLOCK_CAPACITY>>,
  len: usize,
}

impl<T: Clone, const BLOCK_CAPACITY: usize> Queue<T, BLOCK_CAPACITY> {
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

  pub fn push<A>(&mut self, data: T, alloc: &mut A) -> Result<(), T>
  where
    A: Allocator<DataBlock<T, BLOCK_CAPACITY>>,
  {
    let writes_and_data = match self.writes.take() {
      None => match alloc.acquire() {
        Ok(data_block_ref) => {
          let writes = DataRef {
            data_block_ref,
            index: 0,
          };
          Ok((writes, data))
        }
        Err(_) => Err(data),
      },
      Some(writes) => {
        if writes.index >= writes.data_block_ref.borrow().data.len() {
          match alloc.acquire() {
            Ok(new_data_block_ref) => {
              let DataRef {
                data_block_ref: prev_data_block_ref,
                ..
              } = writes;
              prev_data_block_ref.borrow_mut().next = Some(new_data_block_ref.clone());
              let writes = DataRef {
                data_block_ref: new_data_block_ref,
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

    let mut data_block = writes.data_block_ref.borrow_mut();
    data_block.data[writes.index] = Some(data);
    writes.index += 1;
    self.len += 1;
    drop(data_block);

    if self.reads.is_none() {
      self.reads = Some(DataRef {
        data_block_ref: writes.data_block_ref.clone(),
        index: 0,
      })
    }
    self.writes = Some(writes);

    Ok(())
  }

  pub fn pop<A>(&mut self, alloc: &mut A) -> Result<Option<T>, QueueError>
  where
    T: Debug,
    A: Allocator<DataBlock<T, BLOCK_CAPACITY>>,
  {
    let result = match self.reads.take() {
      None => Ok((None, None)),
      Some(mut reads) => {
        if reads.index >= reads.data_block_ref.borrow().data.len() {
          let DataRef {
            data_block_ref: prev_data_block_ref,
            ..
          } = reads;
          let next = prev_data_block_ref.borrow_mut().next.take();

          match alloc.release(prev_data_block_ref) {
            Ok(()) => match next {
              None => Ok((None, None)),
              Some(next_data_block_ref) => {
                let data = next_data_block_ref.borrow_mut().data[0].take();
                let reads = Some(DataRef {
                  data_block_ref: next_data_block_ref,
                  index: 1,
                });
                self.len -= 1;
                Ok((reads, data))
              }
            },
            Err(data_block_ref) => {
              data_block_ref.borrow_mut().next = next;
              let len = data_block_ref.borrow().data.len();
              self.reads = Some(DataRef {
                data_block_ref,
                index: len,
              });
              Err(QueueError::Allocation(AllocError::Release))
            }
          }
        } else if self.len > 0 {
          let mut data_block = reads.data_block_ref.borrow_mut();
          let data = data_block.data[reads.index].take();
          drop(data_block);
          reads.index += 1;
          self.len -= 1;
          Ok((Some(reads), data))
        } else {
          let DataRef {
            data_block_ref,
            index,
          } = reads;
          match alloc.release(data_block_ref) {
            Ok(()) => Ok((None, None)),
            Err(data_block_ref) => {
              self.reads = Some(DataRef {
                data_block_ref,
                index,
              });
              Err(QueueError::Allocation(AllocError::Release))
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

#[cfg(test)]
mod tests {
  use super::*;

  struct TestAllocator<T, const CAPACITY: usize>(Vec<DataBlockRef<T, CAPACITY>>, bool);

  impl<T, const CAPACITY: usize> TestAllocator<T, CAPACITY>
  where
    T: Clone,
  {
    pub fn new(num_data_blocks: usize) -> Self {
      let mut data_blocks = Vec::with_capacity(num_data_blocks);
      data_blocks.resize_with(num_data_blocks, || DataBlockRef::from(DataBlock::new(None)));
      Self(data_blocks, false)
    }

    pub fn set_failing_release(&mut self, failing: bool) {
      self.1 = failing;
    }
  }

  impl<T, const CAPACITY: usize> Allocator<DataBlock<T, CAPACITY>> for TestAllocator<T, CAPACITY> {
    fn available(&self) -> usize {
      self.0.len()
    }

    fn acquire(&mut self) -> Result<DataBlockRef<T, CAPACITY>, AllocError> {
      self.0.pop().ok_or(AllocError::Acquire)
    }

    fn release(
      &mut self,
      data_block: DataBlockRef<T, CAPACITY>,
    ) -> Result<(), DataBlockRef<T, CAPACITY>> {
      if !self.1 {
        self.0.push(data_block);
        Ok(())
      } else {
        Err(data_block)
      }
    }
  }

  #[test]
  fn push_when_empty() {
    let mut q = Queue::<i32, 2>::new();
    let mut alloc = TestAllocator::new(2);
    assert!(q.push(1, &mut alloc).is_ok());
    assert_eq!(q.len(), 1);
    let writes_ref_index = q.writes.as_ref().unwrap();
    let writes_slab = writes_ref_index.data_block_ref.borrow();
    assert!(writes_slab.next.is_none());
    assert_eq!(writes_slab.data, [Some(1), None]);
    assert_eq!(writes_ref_index.index, 1);
    let reads_ref_index = q.reads.as_ref().unwrap();
    let reads_slab = reads_ref_index.data_block_ref.borrow();
    assert!(reads_slab.next.is_none());
    assert_eq!(reads_slab.data, [Some(1), None]);
    assert_eq!(reads_ref_index.index, 0);
  }

  #[test]
  fn push_when_empty_and_alloc_fails() {
    let mut q = Queue::<i32, 2>::new();
    let mut alloc = TestAllocator::new(0);
    assert_eq!(q.push(1, &mut alloc), Err(1));
  }

  #[test]
  fn push_until_first_slab_is_full() {
    let mut q = Queue::<i32, 2>::new();
    let mut alloc = TestAllocator::new(2);
    assert!(q.push(1, &mut alloc).is_ok());
    assert!(q.push(2, &mut alloc).is_ok());
    assert_eq!(q.len(), 2);
    let writes_ref_index = q.writes.as_ref().unwrap();
    let writes_slab = writes_ref_index.data_block_ref.borrow();
    assert!(writes_slab.next.is_none());
    assert_eq!(writes_slab.data, [Some(1), Some(2)]);
    assert_eq!(writes_ref_index.index, 2);
  }

  #[test]
  fn push_beyond_the_first_slab() {
    let mut q = Queue::<i32, 2>::new();
    let mut alloc = TestAllocator::new(2);
    assert!(q.push(1, &mut alloc).is_ok());
    assert!(q.push(2, &mut alloc).is_ok());
    assert!(q.push(3, &mut alloc).is_ok());
    let writes_ref_index = q.writes.as_ref().unwrap();
    let writes_slab = writes_ref_index.data_block_ref.borrow();
    assert!(writes_slab.next.is_none());
    assert_eq!(writes_slab.data, [Some(3), None]);
    assert_eq!(writes_ref_index.index, 1);
    let reads_ref_index = q.reads.as_ref().unwrap();
    let reads_slab = reads_ref_index.data_block_ref.borrow();
    assert!(reads_slab.next.is_some());
    assert_eq!(reads_slab.data, [Some(1), Some(2)]);
    assert_eq!(reads_ref_index.index, 0);
  }

  #[test]
  fn push_beyond_the_first_slab_and_alloc_fails() {
    let mut q = Queue::<i32, 2>::new();
    let mut alloc = TestAllocator::new(1);
    assert!(q.push(1, &mut alloc).is_ok());
    assert!(q.push(2, &mut alloc).is_ok());
    assert_eq!(q.push(3, &mut alloc), Err(3));
    assert_eq!(q.len(), 2);
    let writes_ref_index = q.writes.as_ref().unwrap();
    let writes_slab = writes_ref_index.data_block_ref.borrow();
    assert!(writes_slab.next.is_none());
    assert_eq!(writes_slab.data, [Some(1), Some(2)]);
    assert_eq!(writes_ref_index.index, 2);
  }

  #[test]
  fn pop_when_empty() {
    let mut q = Queue::<i32, 2>::new();
    let mut alloc = TestAllocator::new(0);
    assert_eq!(q.pop(&mut alloc), Ok(None));
    assert_eq!(q.pop(&mut alloc), Ok(None));
    assert_eq!(q.len(), 0);
  }

  #[test]
  fn pop_without_slab_release() {
    let mut q = Queue::<i32, 4>::new();
    let mut alloc = TestAllocator::new(1);
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
    let mut q = Queue::<i32, 2>::new();
    let mut alloc = TestAllocator::new(1);
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
    let mut q = Queue::<i32, 2>::new();
    let mut alloc = TestAllocator::new(1);
    alloc.set_failing_release(true);
    assert!(q.push(1, &mut alloc).is_ok());
    assert!(q.push(2, &mut alloc).is_ok());
    assert_eq!(q.pop(&mut alloc), Ok(Some(1)));
    assert_eq!(q.pop(&mut alloc), Ok(Some(2)));
    assert_eq!(
      q.pop(&mut alloc),
      Err(QueueError::Allocation(AllocError::Release))
    );
    assert_eq!(alloc.0.len(), 0);
    alloc.set_failing_release(false);
    assert_eq!(q.pop(&mut alloc), Ok(None));
    assert_eq!(alloc.0.len(), 1);
  }

  #[test]
  fn pop_with_some_slab_release_failing() {
    let mut q = Queue::<i32, 2>::new();
    let mut alloc = TestAllocator::new(2);
    assert!(q.push(1, &mut alloc).is_ok());
    assert!(q.push(2, &mut alloc).is_ok());
    assert!(q.push(3, &mut alloc).is_ok());
    assert_eq!(alloc.0.len(), 0);
    assert_eq!(q.pop(&mut alloc), Ok(Some(1)));
    assert_eq!(q.pop(&mut alloc), Ok(Some(2)));
    alloc.set_failing_release(true);
    assert_eq!(
      q.pop(&mut alloc),
      Err(QueueError::Allocation(AllocError::Release))
    );
    alloc.set_failing_release(false);
    assert_eq!(q.pop(&mut alloc), Ok(Some(3)));
    assert_eq!(q.len(), 0);
    assert_eq!(alloc.0.len(), 1);
    alloc.set_failing_release(true);
    assert_eq!(
      q.pop(&mut alloc),
      Err(QueueError::Allocation(AllocError::Release))
    );
    assert_eq!(alloc.0.len(), 1);
    alloc.set_failing_release(false);
    assert_eq!(q.pop(&mut alloc), Ok(None));
    assert_eq!(alloc.0.len(), 2);
    assert!(q.writes.is_none());
    assert!(q.reads.is_none());
  }
}
