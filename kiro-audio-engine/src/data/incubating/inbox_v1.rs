use std::collections::HashMap;

use thiserror::Error;

use kiro_audio_graph::NodeRef;
use std::cell::RefCell;


type DataIndex = usize;

#[derive(Error, Debug, PartialEq)]
pub enum InboxError {
  #[error("Out of Queues")]
  OutOfQueues,

  #[error("Out of Slots")]
  OutOfSlots,

  #[error("Internal error: Free of a non empty slot")]
  FreeNonEmptySlot,
}

pub type Result<T> = core::result::Result<T, InboxError>;

#[derive(Debug, Clone, PartialEq)]
struct QueueIndices {
  head: DataIndex,
  tail: DataIndex,
}

impl Default for QueueIndices {
  fn default() -> Self {
    Self {
      head: 0,
      tail: 0,
    }
  }
}

impl QueueIndices {
  pub fn from_index(head_and_tail: DataIndex) -> Self {
    Self {
      head: head_and_tail,
      tail: head_and_tail,
    }
  }

  pub fn new(head: DataIndex, tail: DataIndex) -> Self {
    Self { head, tail }
  }
}

#[derive(Debug, Clone, PartialEq)]
struct Slot<T> {
  next: Option<DataIndex>,
  data: RefCell<Option<T>>,
}

impl<T> Slot<T> {
  pub fn new(next: Option<DataIndex>, data: Option<T>) -> Self {
    Self {
      next,
      data: RefCell::new(data),
    }
  }
}

pub struct Inbox<T> {
  queues: HashMap<NodeRef, QueueIndices>,
  slots: Vec<Slot<T>>,
  free_slots: Option<QueueIndices>,
}

impl<T> Inbox<T> {
  pub fn new(queues_capacity: usize, slots_capacity: usize) -> Self {
    assert!(slots_capacity > 0);

    let mut slots = Vec::with_capacity(slots_capacity);
    for index in 0..slots_capacity - 1 {
      slots.push(Slot::new(Some(index + 1), None));
    }
    slots.push(Slot::new(None, None));

    Self {
      queues: HashMap::with_capacity(queues_capacity),
      slots,
      free_slots: Some(QueueIndices::new(0, slots_capacity - 1)),
    }
  }

  fn alloc(&mut self) -> Result<DataIndex> {
    let (index, tail) = self.free_slots
        .take()
        .map(|queue| (queue.head, queue.tail))
        .ok_or(InboxError::OutOfSlots)?;

    let slot = &self.slots[index];

    self.free_slots = slot.next.map(|next| QueueIndices::new(next, tail));

    Ok(index)
  }

  fn free(&mut self, index: DataIndex) -> Result<()> {
    let slot = &mut self.slots[index];
    if slot.data.borrow().is_some() {
      Err(InboxError::FreeNonEmptySlot)
    }
    else {
      self.free_slots = match self.free_slots.take() {
        None => {
          slot.next = None;
          Some(QueueIndices::from_index(index))
        }
        Some(mut queue) => {
          let tail_slot = &mut self.slots[queue.tail];
          tail_slot.next = Some(index);
          queue.tail = index;
          Some(queue)
        }
      };
      Ok(())
    }
  }

  pub fn insert(&mut self, node_ref: NodeRef, message: T) -> Result<()> {
    if self.queues.len() == self.queues.capacity() && !self.queues.contains_key(&node_ref) {
      Err(InboxError::OutOfQueues)
    }
    else {
      let free_slot_index = self.alloc()?;
      let free_slot = &mut self.slots[free_slot_index];
      free_slot.next = None;
      free_slot.data = RefCell::new(Some(message));
      let slots = &mut self.slots;
      self.queues
          .entry(node_ref)
          .and_modify(|queue| {
            slots[queue.tail].next = Some(free_slot_index);
            queue.tail = free_slot_index;
          })
          .or_insert_with(|| QueueIndices::from_index(free_slot_index));
      Ok(())
    }
  }

  pub fn consumer(&self, node_ref: NodeRef) -> Consumer<T> {
    let maybe_queue = self.queues.get(&node_ref);
    let head = maybe_queue.map(|queue| queue.head);

    Consumer {
      node_ref,
      slots: &self.slots,
      next: head,
    }
  }

  pub fn update(&mut self, consumer: Consumer<T>) -> Result<()> {
    unimplemented!()
  }
}

pub struct Consumer<'a, T> {
  node_ref: NodeRef,
  slots: &'a [Slot<T>],
  next: Option<DataIndex>,
}

impl<'a, T> Iterator for Consumer<'a, T> {
  type Item = T;

  fn next(&mut self) -> Option<Self::Item> {
    self.next.and_then(|index| {
      let slot = &self.slots[index];
      self.next = slot.next;
      slot.data.borrow_mut().take()
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::events::inbox::{Inbox, QueueIndices};
  use kiro_audio_graph::{NodeRef, Key};

  #[test]
  fn insert_when_empty() {
    let mut inbox = Inbox::<i32>::new(1, 2);
    let node = NodeRef::new(Key::new(1));
    let result = inbox.insert(node, 10);
    assert_eq!(result, Ok(()));
    assert_eq!(inbox.queues.len(), 1);
    assert_eq!(*inbox.queues.get(&node).unwrap(), QueueIndices::from_index(0));
    assert_eq!(inbox.slots[0].next, None);
    assert_eq!(inbox.slots[0].data.borrow().as_ref(), Some(&10));
    assert_eq!(inbox.free_slots, Some(QueueIndices::from_index(1)));
  }

  #[test]
  fn insert_when_out_of_queues_with_non_existing_node() {
    let mut inbox = Inbox::<u64>::new(1, 20);
    let mut index = 1;
    while inbox.queues.len() < inbox.queues.capacity() {
      let node = NodeRef::new(Key::new(index));
      let result = inbox.insert(node, index * 10);
      assert_eq!(result, Ok(()));
      index += 1;
    }
    let node = NodeRef::new(Key::new(index));
    let result = inbox.insert(node, index * 10);
    assert_eq!(result, Err(InboxError::OutOfQueues));
  }

  #[test]
  fn insert_when_out_of_queues_with_existing_node() {
    let mut inbox = Inbox::<u64>::new(1, 20);
    let mut index = 1;
    while inbox.queues.len() < inbox.queues.capacity() {
      let node = NodeRef::new(Key::new(index));
      let result = inbox.insert(node, index * 10);
      assert_eq!(result, Ok(()));
      index += 1;
    }
    let node = NodeRef::new(Key::new(1));
    let result = inbox.insert(node, index * 10);
    assert_eq!(result, Ok(()));
  }

  #[test]
  fn insert_when_out_of_slots_with_one_slot_per_queue() {
    let mut inbox = Inbox::<u64>::new(8, 4);

    for index in 0..4 {
      let node = NodeRef::new(Key::new(index));
      let result = inbox.insert(node, index);
      assert_eq!(result, Ok(()));
    }

    let node = NodeRef::new(Key::new(1000));
    let result = inbox.insert(node, 1000);
    assert_eq!(result, Err(InboxError::OutOfSlots));

    assert_queues(
      &inbox,
      vec![
        (NodeRef::new(Key::new(0)), QueueIndices::from_index(0)),
        (NodeRef::new(Key::new(1)), QueueIndices::from_index(1)),
        (NodeRef::new(Key::new(2)), QueueIndices::from_index(2)),
        (NodeRef::new(Key::new(3)), QueueIndices::from_index(3)),
      ]
    );

    assert_eq!(
      inbox.slots,
      vec![
        Slot::new(None, Some(0)),
        Slot::new(None, Some(1)),
        Slot::new(None, Some(2)),
        Slot::new(None, Some(3)),
      ]
    );

    assert_eq!(inbox.free_slots, None);
  }

  #[test]
  fn insert_when_out_of_slots_with_a_single_queue() {
    let mut inbox = Inbox::<u64>::new(8, 4);
    let node = NodeRef::new(Key::new(1));

    for index in 0..4 {
      let result = inbox.insert(node, index);
      assert_eq!(result, Ok(()));
    }

    let result = inbox.insert(node, 1000);
    assert_eq!(result, Err(InboxError::OutOfSlots));

    assert_queues(
      &inbox,
      vec![
        (NodeRef::new(Key::new(1)), QueueIndices { head: 0, tail: 3 }),
      ]
    );

    assert_eq!(
      inbox.slots,
      vec![
        Slot::new(Some(1), Some(0)),
        Slot::new(Some(2), Some(1)),
        Slot::new(Some(3), Some(2)),
        Slot::new(None, Some(3)),
      ]
    );

    assert_eq!(inbox.free_slots, None);
  }

  #[test]
  fn consumer_for_empty_inbox() {
    let mut inbox = Inbox::<u64>::new(4, 4);
    let node_ref = NodeRef::new(Key::new(1));
    let iter = inbox.consumer(node_ref);
    assert_eq!(
      iter.collect::<Vec<u64>>(),
      vec![]
    );
  }

  #[test]
  fn consumer_for_non_empty_inbox() {
    let mut inbox = Inbox::<u64>::new(4, 4);
    let node1 = NodeRef::new(Key::new(1));
    inbox.insert(node1, 1).unwrap();
    inbox.insert(node1, 2).unwrap();
    let node2 = NodeRef::new(Key::new(2));
    inbox.insert(node2, 3).unwrap();
    inbox.insert(node2, 4).unwrap();
    let iter = inbox.consumer(node1);
    assert_eq!(
      iter.collect::<Vec<u64>>(),
      vec![1, 2]
    );
    assert_eq!(
      inbox.slots,
      vec![
        Slot::new(Some(1), None),
        Slot::new(None, None),
        Slot::new(Some(3), Some(3)),
        Slot::new(None, Some(4)),
      ]
    );
  }

  #[test]
  fn queue_clear_for_non_empty_inbox() {
    let mut inbox = Inbox::<u64>::new(4, 4);
    let node1 = NodeRef::new(Key::new(1));
    inbox.insert(node1, 1).unwrap();
    inbox.insert(node1, 2).unwrap();
    let node2 = NodeRef::new(Key::new(2));
    inbox.insert(node2, 3).unwrap();
    inbox.insert(node2, 4).unwrap();
    let consumer = inbox.consumer(node1);
    consumer.collect::<Vec<u64>>();
    inbox.update(consumer);
    assert_eq!(inbox.queues.get(&node1), None);
    assert_eq!(inbox.free_slots, Some(QueueIndices::from_index(0)));
    assert_eq!(
      inbox.slots,
      vec![
        Slot::new(Some(1), None),
        Slot::new(None, None),
        Slot::new(Some(3), Some(3)),
        Slot::new(None, Some(4)),
      ]
    );
  }

  fn assert_queues<T>(inbox: &Inbox<T>, expected_queues: Vec<(NodeRef, QueueIndices)>) {
    let mut queues = inbox.queues
        .iter()
        .map(|(node, queue)| (node.clone(), queue.clone()))
        .collect::<Vec<(NodeRef, QueueIndices)>>();
    queues.sort_by_key(|(_node_ref, queue)| queue.head);

    assert_eq!(queues, expected_queues);
  }
}
