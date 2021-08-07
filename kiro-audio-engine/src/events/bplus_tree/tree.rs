use std::ops::{Deref, DerefMut};

use thiserror::Error;

use crate::events::allocator::{AllocError, AllocRef, Allocator};
use crate::events::bplus_tree::internal_node::InternalNode;
use crate::events::bplus_tree::leaf_node::LeafNode;
use std::cell::RefMut;

pub type Result<T> = core::result::Result<T, BplusTreeError>;

pub type NodeRef<K, V, const O: usize> = AllocRef<Node<K, V, O>>;

// struct NodeBorrowMut<'a, K, V, const O: usize>(RefMut<'a, Node<K, V, O>>);

// impl<'a, K, V, const O: usize> Deref for NodeBorrowMut<'a, K, V, O>
// where
//   K: Clone + PartialOrd,
//   V: Clone,
// {
//   type Target = Option<&'a LeafNode<K, V, O>>;

//   fn deref(&self) -> &Self::Target {
//     &self.0.as_leaf()
//   }
// }

// impl<'a, K, V, const O: usize> DerefMut for NodeBorrowMut<'a, K, V, O>
// where
//   K: Clone + PartialOrd,
//   V: Clone,
// {
//   fn deref_mut(&mut self) -> &mut Self::Target {
//     todo!()
//   }
// }

// impl<K, V, const O: usize> NodeRef<K, V, O> {
//   pub fn as_leaf_mut(&self) -> NodeBorrowMut<K, V, O> {
//     NodeBorrowMut(self.borrow_mut())
//   }
// }

#[derive(Debug, Clone)]
pub enum Node<K, V, const O: usize> {
  Uninit,
  Internal(InternalNode<K, V, O>),
  Leaf(LeafNode<K, V, O>),
}

impl<K, V, const O: usize> Node<K, V, O>
where
  K: Clone + PartialOrd,
  V: Clone,
{
  pub fn size(&self) -> usize {
    match self {
      Self::Uninit => 0,
      Self::Internal(node) => node.size(),
      Self::Leaf(node) => node.len(),
    }
  }

  pub fn set_parent(&mut self, parent: NodeRef<K, V, O>) {
    match self {
      Self::Uninit => (),
      Self::Internal(node) => node.set_parent(Some(parent)),
      Self::Leaf(node) => node.set_parent(Some(parent)),
    }
  }

  pub fn is_leaf(&self) -> bool {
    match self {
      Self::Uninit | Self::Internal(_) => false,
      Self::Leaf(_) => true,
    }
  }

  pub fn as_leaf(&self) -> Option<&LeafNode<K, V, O>> {
    match self {
      Self::Uninit | Self::Internal(_) => None,
      Self::Leaf(leaf_node) => Some(leaf_node),
    }
  }

  pub fn as_leaf_mut(&mut self) -> Option<&mut LeafNode<K, V, O>> {
    match self {
      Self::Uninit | Self::Internal(_) => None,
      Self::Leaf(leaf_node) => Some(leaf_node),
    }
  }

  pub fn lookup_node(&self, key: &K) -> Option<NodeRef<K, V, O>> {
    match self {
      Self::Uninit | Self::Leaf(_) => None,
      Self::Internal(node) => node.lookup_node(key),
    }
  }

  pub fn first_key(&self) -> Option<&K> {
    match self {
      Self::Uninit => None,
      Self::Internal(node) => node.first_key(),
      Self::Leaf(node) => node.first_key(),
    }
  }
}

#[derive(Error, Debug, PartialEq)]
pub enum BplusTreeError {
  #[error("Error allocating a node: {0}")]
  NodeAllocation(AllocError),

  #[error("Error allocating a data block: {0}")]
  DataBlockAllocation(AllocError),

  #[error("Leaf not found")]
  LeafNotFound,

  #[error("Leaf overflow")]
  LeafOverflow,

  #[error("Internal Error")]
  InternalError,
}

pub struct BplusTree<K, V, const O: usize> {
  root: Option<NodeRef<K, V, O>>,
}

impl<K, V, const O: usize> BplusTree<K, V, O>
where
  K: Clone + PartialOrd,
  V: Clone,
{
  pub fn new() -> Self {
    Self { root: None }
  }

  pub fn is_empty(&self) -> bool {
    self.root.is_none()
  }

  pub fn size(&self) -> usize {
    self
      .root
      .as_ref()
      .map(|node| node.borrow().size())
      .unwrap_or(0)
  }

  pub fn insert<Alloc>(&mut self, key: K, value: V, alloc: &mut Alloc) -> Result<()>
  where
    Alloc: Allocator<Node<K, V, O>>,
  {
    if self.is_empty() {
      self.start_new_tree(key, value, alloc)
    } else {
      self.insert_into_leaf(key, value, alloc)
    }
  }

  fn start_new_tree<Alloc>(&mut self, key: K, value: V, alloc: &mut Alloc) -> Result<()>
  where
    Alloc: Allocator<Node<K, V, O>>,
  {
    let leaf_node = LeafNode::from_key_value(key, value);
    let leaf_node_ref = Self::create_leaf_node_ref(leaf_node, alloc)?;
    self.root = Some(leaf_node_ref);

    Ok(())
  }

  fn insert_into_leaf<Alloc>(&mut self, key: K, value: V, alloc: &mut Alloc) -> Result<()>
  where
    Alloc: Allocator<Node<K, V, O>>,
  {
    let node_ref = self
      .find_leaf_node(&key)
      .ok_or(BplusTreeError::LeafNotFound)?;

    let mut node = node_ref.borrow_mut();
    let leaf_node = node.as_leaf_mut().unwrap(); // this can only be a leaf

    let insert_result = leaf_node
      .insert(key, value)
      .map_err(|_| BplusTreeError::LeafOverflow)?;

    if leaf_node.len() == O {
      let mut new_leaf_node = leaf_node.split();

      match self.insert_leaf_into_parent(node_ref.clone(), new_leaf_node, 0, alloc) {
        Ok(new_leaf_node_ref) => {
          let mut new_node = new_leaf_node_ref.borrow_mut();
          let new_leaf_node = new_node.as_leaf_mut().unwrap(); // this can only be a leaf

          new_leaf_node.set_next(leaf_node.take_next());
          leaf_node.set_next(Some(new_leaf_node_ref.clone()));
        }
        Err(new_leaf_node) => {
          // leaf_node.concat(new_leaf_node)
          // match insert_result {
          //   Some(value) => leaf_node.insert(key, value),
          //   None => leaf_node.remove(key),
          // }
          unimplemented!()
        }
      }

      // let first_key = new_leaf_node.records().first().and_then(|first| first.as_ref().map(|(k, _)| k.clone()));
      //
      // let new_leaf_node_ref = Self::create_leaf_node_ref(new_leaf_node, alloc)?;
      // leaf_node.set_next(Some(new_leaf_node_ref.clone()));
      //
      // // recursive call, if it fails, we'll need to undo previous operations
      // //
      // // todo!()
      // // KeyType newKey = newLeaf->firstKey();
      // // insertIntoParent(leafNode, newKey, newLeaf);
      // match leaf_node.get_parent() {
      //   None => {
      //     let new_root_internal_node = InternalNode::new(None);
      //     let new_root_node_ref = Self::create_internal_node_ref(new_root_internal_node, alloc)?;
      //     leaf_node.set_parent(Some(new_root_node_ref.clone()));
      //     // new_leaf_node_ref.borrow_mut().as_leaf_mut().into_iter().for_each(|x| x.set_parent(Some(new_root_node_ref.clone())));
      //     new_leaf_node.set_parent(Some(new_root_node_ref.clone()));
      //     self.root = Some(new_root_node_ref);
      //     todo!()
      //     // parent->populateNewRoot(aOldNode, aKey, aNewNode);
      //   },
      //   Some(parent) => {
      //     todo!()
      //   }
      // }
    }

    Ok(())
  }

  fn insert_leaf_into_parent<Alloc>(
    &mut self,
    prev_leaf_node_ref: NodeRef<K, V, O>,
    new_leaf_node: LeafNode<K, V, O>,
    num_allocations: usize,
    alloc: &mut Alloc,
  ) -> core::result::Result<NodeRef<K, V, O>, LeafNode<K, V, O>>
  where
    Alloc: Allocator<Node<K, V, O>>,
  {
    match new_leaf_node.get_parent() {
      Some(parent) => {
        unimplemented!()
      }
      None => {
        match prev_leaf_node_ref
          .borrow()
          .first_key()
          .zip(new_leaf_node.first_key())
        {
          Some((prev_leaf_key, new_leaf_key)) => {
            // TODO check that there will be enough memory to allocate all the required allocations, or fail
            if alloc.available() <= num_allocations {
              Err(new_leaf_node)
            } else {
              // TODO create a new internal node and add both leafs to it
              let internal_node = InternalNode::new(None);
              // TODO internal_node.insert(prev_leaf_key.clone(), prev_leaf_node_ref.clone());
              let parent_node_ref =
                Self::create_internal_node_ref(internal_node, alloc).map_err(|_| new_leaf_node)?;
              Ok(parent_node_ref)
            }
          }
          None => Err(new_leaf_node), // this is unexpected to happen
        }
      }
    }
  }

  fn find_leaf_node(&self, key: &K) -> Option<NodeRef<K, V, O>> {
    let mut maybe_node_ref = self.root.clone();
    while maybe_node_ref
      .as_ref()
      .map(|node_ref| !node_ref.borrow().is_leaf())
      .unwrap_or(false)
    {
      maybe_node_ref = maybe_node_ref.and_then(|node_ref| node_ref.borrow().lookup_node(key))
    }
    maybe_node_ref
  }

  fn create_leaf_node_ref<Alloc>(
    leaf_node: LeafNode<K, V, O>,
    alloc: &mut Alloc,
  ) -> Result<NodeRef<K, V, O>>
  where
    Alloc: Allocator<Node<K, V, O>>,
  {
    let node = alloc.acquire().map_err(BplusTreeError::NodeAllocation)?;
    *node.borrow_mut() = Node::Leaf(leaf_node);
    Ok(node)
  }

  fn create_internal_node_ref<Alloc>(
    internal_node: InternalNode<K, V, O>,
    alloc: &mut Alloc,
  ) -> Result<NodeRef<K, V, O>>
  where
    Alloc: Allocator<Node<K, V, O>>,
  {
    let node = alloc.acquire().map_err(BplusTreeError::NodeAllocation)?;
    *node.borrow_mut() = Node::Internal(internal_node);
    Ok(node)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn bplustree_is_empty() {
    let tree: BplusTree<i32, i32, 2> = BplusTree::new();
    assert!(tree.is_empty());
    assert_eq!(tree.size(), 0);
  }

  #[test]
  fn bplustree_insert_and_start_new_tree() {
    let mut tree: BplusTree<i32, i32, 2> = BplusTree::new();
    let mut alloc = TestAllocator::new(2);
    let result = tree.insert(1, 10, &mut alloc);
    assert!(result.is_ok());
    assert_eq!(tree.size(), 1);
    // TODO retrieve value associated to 1 and check it is 10
  }

  #[test]
  fn bplustree_insert_into_leaf_no_split() {
    let mut tree: BplusTree<i32, i32, 3> = BplusTree::new();
    let mut alloc = TestAllocator::new(2);

    let result = tree.insert(1, 10, &mut alloc);
    assert!(result.is_ok());
    assert_eq!(tree.size(), 1);

    let result = tree.insert(2, 20, &mut alloc);
    assert!(result.is_ok());
    assert_eq!(tree.size(), 2);

    assert_eq!(
      tree.root.unwrap().borrow().as_leaf().unwrap().records(),
      &[Some((1, 10)), Some((2, 20))]
    )
  }

  #[test]
  fn bplustree_insert_into_leaf_with_split_succeeds() {
    let mut tree: BplusTree<i32, i32, 3> = BplusTree::new();
    let mut alloc = TestAllocator::new(2);

    tree.insert(1, 10, &mut alloc).expect("insert failed");
    tree.insert(2, 20, &mut alloc).expect("insert failed");
    tree.insert(3, 30, &mut alloc).expect("insert failed");
  }

  // --- Helpers ---

  struct TestAllocator<K, V, const O: usize>(Vec<NodeRef<K, V, O>>)
  where
    K: Clone,
    V: Clone;

  impl<K, V, const O: usize> TestAllocator<K, V, O>
  where
    K: Clone,
    V: Clone,
  {
    pub fn new(capacity: usize) -> Self {
      let mut v = Vec::with_capacity(capacity);
      v.resize_with(capacity, || NodeRef::from(Node::Uninit));
      Self(v)
    }
  }

  impl<K, V, const O: usize> Allocator<Node<K, V, O>> for TestAllocator<K, V, O>
  where
    K: Clone,
    V: Clone,
  {
    fn available(&self) -> usize {
      self.0.len()
    }

    fn acquire(&mut self) -> core::result::Result<NodeRef<K, V, O>, AllocError> {
      self.0.pop().ok_or(AllocError::Acquire)
    }

    fn release(&mut self, slab: NodeRef<K, V, O>) -> core::result::Result<(), NodeRef<K, V, O>> {
      self.0.push(slab);
      Ok(())
    }
  }
}
