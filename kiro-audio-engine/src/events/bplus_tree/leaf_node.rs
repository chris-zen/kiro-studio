use crate::events::bplus_tree::tree::NodeRef;

use super::sorted_map::{InsertResult, SortedMap};

#[derive(Debug, Clone)]
pub struct LeafNode<K, V, const O: usize> {
  parent: Option<NodeRef<K, V, O>>,
  next: Option<NodeRef<K, V, O>>,
  records: SortedMap<K, V, O>,
}

impl<K, V, const O: usize> Default for LeafNode<K, V, O>
where
  K: Clone + PartialOrd,
  V: Clone,
{
  fn default() -> Self {
    Self {
      parent: None,
      next: None,
      records: SortedMap::new(),
    }
  }
}

impl<K, V, const O: usize> LeafNode<K, V, O>
where
  K: Clone + PartialOrd,
  V: Clone,
{
  pub fn from_key_value(key: K, value: V) -> Self {
    Self {
      parent: None,
      next: None,
      records: SortedMap::from_key_value(key, value),
    }
  }

  pub fn set_parent(&mut self, parent: Option<NodeRef<K, V, O>>) {
    self.parent = parent;
  }

  pub fn get_parent(&self) -> Option<NodeRef<K, V, O>> {
    self.parent.as_ref().map(|parent| parent.clone())
  }

  pub fn take_next(&mut self) -> Option<NodeRef<K, V, O>> {
    self.next.take()
  }

  pub fn set_next(&mut self, next: Option<NodeRef<K, V, O>>) {
    self.next = next;
  }

  pub fn len(&self) -> usize {
    self.records.len()
  }

  pub fn first_key(&self) -> Option<&K> {
    self.records.first_key()
  }

  pub(super) fn records(&self) -> &[Option<(K, V)>] {
    self.records.data()
  }

  pub fn insert(&mut self, key: K, value: V) -> Result<InsertResult<K, V>, (K, V)> {
    self.records.insert(key, value)
  }

  pub fn split(&mut self) -> LeafNode<K, V, O> {
    let mut new_leaf_node = LeafNode::default();
    new_leaf_node.set_parent(self.parent.clone());
    new_leaf_node.records = self.records.split();
    new_leaf_node
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::events::{allocator::AllocRef, bplus_tree::tree::Node};


  #[test]
  fn from_key_value() {
    let leaf = LeafNode::<u32, u32, 2>::from_key_value(1, 10);
    assert_eq!(leaf.records(), &[Some((1, 10))]);
    assert_eq!(leaf.len(), 1);
  }

  #[test]
  fn first_key() {
    let leaf = LeafNode::<u32, u32, 2>::from_key_value(1, 10);
    assert_eq!(leaf.first_key(), Some(&1));
  }

  #[test]
  fn records() {
    let leaf = LeafNode::<u32, u32, 2>::from_key_value(1, 10);
    assert_eq!(leaf.records(), &[Some((1, 10))]);
  }

  #[test]
  fn insert() {
    let mut leaf = LeafNode::<u32, u32, 2>::default();
    let result = leaf.insert(1, 10);
    assert_eq!(result, Ok(InsertResult::Inserted(0)));
    let result = leaf.insert(1, 20);
    assert_eq!(result, Ok(InsertResult::Replaced(1, 10)));
    let result = leaf.insert(3, 30);
    assert_eq!(result, Ok(InsertResult::Inserted(1)));
    assert_eq!(leaf.records(), [Some((1, 20)), Some((3, 30))]);
    assert_eq!(leaf.len(), 2);
  }

  #[test]
  fn split() {
    let parent = LeafNode::<u32, u32, 3>::from_key_value(123, 321);
    let mut leaf = LeafNode::<u32, u32, 3>::default();
    leaf.set_parent(Some(AllocRef::from(Node::Leaf(parent))));
    leaf.insert(1, 10).ok();
    leaf.insert(2, 20).ok();
    leaf.insert(3, 30).ok();
    let sibling = leaf.split();

    assert_eq!(leaf.records(), &[Some((1, 10))]);
    assert_eq!(sibling.records(), &[Some((2, 20)), Some((3, 30))]);

    let leaf_parent = leaf
      .parent
      .unwrap()
      .borrow()
      .as_leaf()
      .unwrap()
      .records()
      .to_vec();
    let sibling_parent = sibling
      .parent
      .unwrap()
      .borrow()
      .as_leaf()
      .unwrap()
      .records()
      .to_vec();
    assert_eq!(leaf_parent, sibling_parent);
  }
}
