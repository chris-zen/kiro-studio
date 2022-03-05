use crate::events::slab_pool::{Slab, SlabRef};

pub struct SlabQueue<T> {
  head: Option<SlabRef<T>>,
  last: Option<SlabRef<T>>,
}

impl<T> SlabQueue<T> {
  pub fn new() -> Self {
    Self {
      head: None,
      last: None,
    }
  }

  pub fn push(&mut self, slab: SlabRef<T>) -> Result<(), SlabRef<T>> {
    if slab.borrow().next.is_some() {
      Err(slab)
    } else {
      let prev_last = self.last.take();
      self.last = Some(slab.clone());
      if let Some(last) = prev_last.as_ref() {
        (*last).borrow_mut().next = Some(slab);
      }
      if self.head.is_none() {
        self.head = self.last.clone();
      }
      Ok(())
    }
  }

  pub fn pop(&mut self) -> Option<SlabRef<T>> {
    let slab = self.head.take();
    self.head = slab
      .as_ref()
      .and_then(|slab| (*slab).borrow_mut().next.take());
    if self.head.is_none() {
      self.last = None;
    }
    slab
  }

  pub fn head(&self) -> Option<std::cell::Ref<Slab<T>>> {
    self.head.as_ref().map(|slab| (*slab).borrow())
  }

  pub fn head_mut(&self) -> Option<std::cell::RefMut<Slab<T>>> {
    self.head.as_ref().map(|slab| (*slab).borrow_mut())
  }

  pub fn last(&self) -> Option<std::cell::Ref<Slab<T>>> {
    self.last.as_ref().map(|slab| (*slab).borrow())
  }

  pub fn last_mut(&self) -> Option<std::cell::RefMut<Slab<T>>> {
    self.last.as_ref().map(|slab| (*slab).borrow_mut())
  }
}

#[cfg(test)]
mod tests {
  use crate::events::slab_pool::{Slab, SlabRef};
  use crate::events::slab_queue::SlabQueue;
  use std::fmt::Debug;

  #[test]
  fn push_to_empty_queue() {
    let mut q = SlabQueue::<i32>::new();
    push(&mut q, vec![Some(1), Some(2)]);

    assert_eq!(q.head.unwrap().borrow().data, vec![Some(1), Some(2)]);
    assert_eq!(q.last.unwrap().borrow().data, vec![Some(1), Some(2)]);
  }

  #[test]
  fn push_to_non_empty_queue() {
    let mut q = SlabQueue::<i32>::new();
    push(&mut q, vec![Some(1), Some(2)]);
    push(&mut q, vec![Some(3), Some(4)]);

    assert_eq!(q.head.unwrap().borrow().data, vec![Some(1), Some(2)]);
    assert_eq!(q.last.unwrap().borrow().data, vec![Some(3), Some(4)]);
  }

  #[test]
  fn push_slab_with_non_empty_next() {
    let mut q = SlabQueue::<i32>::new();
    let mut slab = Slab::new(2, None);
    let mut slab = Slab::new(2, Some(SlabRef::from(slab)));
    slab.data = vec![Some(1), Some(2)];
    let result = q.push(SlabRef::from(slab));
    assert_eq!(result.unwrap_err().borrow().data, vec![Some(1), Some(2)]);
  }

  #[test]
  fn pop_from_empty_queue() {
    let mut q = SlabQueue::<i32>::new();
    assert!(q.pop().is_none());
  }

  #[test]
  fn pop_from_non_empty_queue() {
    let mut q = SlabQueue::<i32>::new();
    push(&mut q, vec![Some(1), Some(2)]);
    push(&mut q, vec![Some(3), Some(4)]);

    assert_eq!(q.pop().unwrap().borrow().data, vec![Some(1), Some(2)]);
    assert_eq!(q.pop().unwrap().borrow().data, vec![Some(3), Some(4)]);
  }

  #[test]
  fn head_and_tail() {
    let mut q = SlabQueue::<i32>::new();
    push(&mut q, vec![Some(1), Some(2)]);
    push(&mut q, vec![Some(3), Some(4)]);

    let head = q.head().unwrap();
    assert_eq!(head.data, vec![Some(1), Some(2)]);
    let last = q.last().unwrap();
    assert_eq!(last.data, vec![Some(3), Some(4)]);
  }

  #[test]
  fn head_and_tail_mut() {
    let mut q = SlabQueue::<i32>::new();
    push(&mut q, vec![Some(1), Some(2)]);
    push(&mut q, vec![Some(3), Some(4)]);

    let mut head = q.head_mut().unwrap();
    head.data[0] = Some(10);
    drop(head);
    let mut head = q.head_mut().unwrap();
    assert_eq!(head.data, vec![Some(10), Some(2)]);
    let mut last = q.last_mut().unwrap();
    last.data[0] = Some(30);
    drop(last);
    let mut last = q.last_mut().unwrap();
    assert_eq!(last.data, vec![Some(30), Some(4)]);
  }

  fn push<T: Debug>(queue: &mut SlabQueue<T>, data: Vec<Option<T>>) {
    let mut slab = Slab::new(data.capacity(), None);
    slab.data = data;
    queue.push(SlabRef::from(slab)).unwrap()
  }
}
