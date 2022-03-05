use std::cell::RefCell;
use std::collections::HashMap;

use thiserror::Error;

use crate::events::queue::{Queue, QueueAllocator};
use crate::events::slab_pool::{SlabPool, SlabRef};
use crate::events::slab_queue::SlabQueue;
use kiro_audio_graph::NodeRef;

type DataIndex = usize;

#[derive(Error, Debug, PartialEq)]
pub enum InboxError {
  #[error("Out of Queues")]
  OutOfQueues,

  #[error("Out of Slabs")]
  OutOfSlabs,
}

pub type Result<T> = core::result::Result<T, InboxError>;

const DEFAULT_SLAB_CAPACITY: usize = 32;

pub struct Inbox<T> {
  pool: SlabPool<T>,
  queues: HashMap<NodeRef, Queue<T>>,
}

impl<T> Inbox<T> {
  pub fn new(slots_capacity: usize) -> Self {
    let pool_capacity = (slots_capacity + DEFAULT_SLAB_CAPACITY - 1) / DEFAULT_SLAB_CAPACITY;
    let pool = SlabPool::new(pool_capacity, DEFAULT_SLAB_CAPACITY);
    Self {
      pool,
      queues: HashMap::new(),
    }
  }

  pub fn insert<A>(
    &mut self,
    node_ref: NodeRef,
    message: T,
    alloc: &mut A,
  ) -> std::result::Result<(), T>
  where
    A: QueueAllocator<T>,
  {
    match self.queues.get_mut(&node_ref) {
      None => {
        let mut queue = Queue::new();
        let result = queue.push(message, alloc);
        self.queues.insert(node_ref, queue);
        result
      }
      Some(queue) => queue.push(message, alloc),
    }
  }

  pub fn take_queue(&mut self, node_ref: NodeRef) -> Queue<T> {
    self
      .queues
      .remove(&node_ref)
      .unwrap_or_else(|| Queue::new())
  }

  pub fn insert_queue(&mut self, node_ref: NodeRef, queue: Queue<T>) {
    assert!(!self.queues.contains_key(&node_ref));
    self.queues.insert(node_ref, queue);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn insert_when_empty() {
  }

  // #[test]
  // fn insert_when_empty() {
  //   let mut inbox = Inbox::<i32>::new(1, 2);
  //   let node = NodeRef::new(Key::new(1));
  //   let result = inbox.insert(node, 10);
  //   assert_eq!(result, Ok(()));
  //   assert_eq!(inbox.queues.len(), 1);
  //   assert_eq!(*inbox.queues.get(&node).unwrap(), QueueIndices::from_index(0));
  //   assert_eq!(inbox.slots[0].next, None);
  //   assert_eq!(inbox.slots[0].data.borrow().as_ref(), Some(&10));
  //   assert_eq!(inbox.free_slots, Some(QueueIndices::from_index(1)));
  // }
  //
  // #[test]
  // fn insert_when_out_of_queues_with_non_existing_node() {
  //   let mut inbox = Inbox::<u64>::new(1, 20);
  //   let mut index = 1;
  //   while inbox.queues.len() < inbox.queues.capacity() {
  //     let node = NodeRef::new(Key::new(index));
  //     let result = inbox.insert(node, index * 10);
  //     assert_eq!(result, Ok(()));
  //     index += 1;
  //   }
  //   let node = NodeRef::new(Key::new(index));
  //   let result = inbox.insert(node, index * 10);
  //   assert_eq!(result, Err(InboxError::OutOfQueues));
  // }
  //
  // #[test]
  // fn insert_when_out_of_queues_with_existing_node() {
  //   let mut inbox = Inbox::<u64>::new(1, 20);
  //   let mut index = 1;
  //   while inbox.queues.len() < inbox.queues.capacity() {
  //     let node = NodeRef::new(Key::new(index));
  //     let result = inbox.insert(node, index * 10);
  //     assert_eq!(result, Ok(()));
  //     index += 1;
  //   }
  //   let node = NodeRef::new(Key::new(1));
  //   let result = inbox.insert(node, index * 10);
  //   assert_eq!(result, Ok(()));
  // }
  //
  // #[test]
  // fn insert_when_out_of_slots_with_one_slot_per_queue() {
  //   let mut inbox = Inbox::<u64>::new(8, 4);
  //
  //   for index in 0..4 {
  //     let node = NodeRef::new(Key::new(index));
  //     let result = inbox.insert(node, index);
  //     assert_eq!(result, Ok(()));
  //   }
  //
  //   let node = NodeRef::new(Key::new(1000));
  //   let result = inbox.insert(node, 1000);
  //   assert_eq!(result, Err(InboxError::OutOfSlots));
  //
  //   assert_queues(
  //     &inbox,
  //     vec![
  //       (NodeRef::new(Key::new(0)), QueueIndices::from_index(0)),
  //       (NodeRef::new(Key::new(1)), QueueIndices::from_index(1)),
  //       (NodeRef::new(Key::new(2)), QueueIndices::from_index(2)),
  //       (NodeRef::new(Key::new(3)), QueueIndices::from_index(3)),
  //     ]
  //   );
  //
  //   assert_eq!(
  //     inbox.slots,
  //     vec![
  //       Slot::new(None, Some(0)),
  //       Slot::new(None, Some(1)),
  //       Slot::new(None, Some(2)),
  //       Slot::new(None, Some(3)),
  //     ]
  //   );
  //
  //   assert_eq!(inbox.free_slots, None);
  // }
  //
  // #[test]
  // fn insert_when_out_of_slots_with_a_single_queue() {
  //   let mut inbox = Inbox::<u64>::new(8, 4);
  //   let node = NodeRef::new(Key::new(1));
  //
  //   for index in 0..4 {
  //     let result = inbox.insert(node, index);
  //     assert_eq!(result, Ok(()));
  //   }
  //
  //   let result = inbox.insert(node, 1000);
  //   assert_eq!(result, Err(InboxError::OutOfSlots));
  //
  //   assert_queues(
  //     &inbox,
  //     vec![
  //       (NodeRef::new(Key::new(1)), QueueIndices { head: 0, tail: 3 }),
  //     ]
  //   );
  //
  //   assert_eq!(
  //     inbox.slots,
  //     vec![
  //       Slot::new(Some(1), Some(0)),
  //       Slot::new(Some(2), Some(1)),
  //       Slot::new(Some(3), Some(2)),
  //       Slot::new(None, Some(3)),
  //     ]
  //   );
  //
  //   assert_eq!(inbox.free_slots, None);
  // }
  //
  // #[test]
  // fn consumer_for_empty_inbox() {
  //   let mut inbox = Inbox::<u64>::new(4, 4);
  //   let node_ref = NodeRef::new(Key::new(1));
  //   let iter = inbox.consumer(node_ref);
  //   assert_eq!(
  //     iter.collect::<Vec<u64>>(),
  //     vec![]
  //   );
  // }
  //
  // #[test]
  // fn consumer_for_non_empty_inbox() {
  //   let mut inbox = Inbox::<u64>::new(4, 4);
  //   let node1 = NodeRef::new(Key::new(1));
  //   inbox.insert(node1, 1).unwrap();
  //   inbox.insert(node1, 2).unwrap();
  //   let node2 = NodeRef::new(Key::new(2));
  //   inbox.insert(node2, 3).unwrap();
  //   inbox.insert(node2, 4).unwrap();
  //   let iter = inbox.consumer(node1);
  //   assert_eq!(
  //     iter.collect::<Vec<u64>>(),
  //     vec![1, 2]
  //   );
  //   assert_eq!(
  //     inbox.slots,
  //     vec![
  //       Slot::new(Some(1), None),
  //       Slot::new(None, None),
  //       Slot::new(Some(3), Some(3)),
  //       Slot::new(None, Some(4)),
  //     ]
  //   );
  // }
  //
  // #[test]
  // fn queue_clear_for_non_empty_inbox() {
  //   let mut inbox = Inbox::<u64>::new(4, 4);
  //   let node1 = NodeRef::new(Key::new(1));
  //   inbox.insert(node1, 1).unwrap();
  //   inbox.insert(node1, 2).unwrap();
  //   let node2 = NodeRef::new(Key::new(2));
  //   inbox.insert(node2, 3).unwrap();
  //   inbox.insert(node2, 4).unwrap();
  //   let consumer = inbox.consumer(node1);
  //   consumer.collect::<Vec<u64>>();
  //   inbox.update(consumer);
  //   assert_eq!(inbox.queues.get(&node1), None);
  //   assert_eq!(inbox.free_slots, Some(QueueIndices::from_index(0)));
  //   assert_eq!(
  //     inbox.slots,
  //     vec![
  //       Slot::new(Some(1), None),
  //       Slot::new(None, None),
  //       Slot::new(Some(3), Some(3)),
  //       Slot::new(None, Some(4)),
  //     ]
  //   );
  // }
  //
  // fn assert_queues<T>(inbox: &Inbox<T>, expected_queues: Vec<(NodeRef, QueueIndices)>) {
  //   let mut queues = inbox.queues
  //       .iter()
  //       .map(|(node, queue)| (node.clone(), queue.clone()))
  //       .collect::<Vec<(NodeRef, QueueIndices)>>();
  //   queues.sort_by_key(|(_node_ref, queue)| queue.head);
  //
  //   assert_eq!(queues, expected_queues);
  // }
}
