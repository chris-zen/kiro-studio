use crate::events::bplus_tree::tree::NodeRef;
use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct InternalNode<K, V, const O: usize> {
  parent: Option<NodeRef<K, V, O>>,
  size: usize,
  records: [Option<(K, NodeRef<K, V, O>)>; O],
}

impl<K, V, const O: usize> InternalNode<K, V, O>
where
  K: Clone + PartialOrd,
  V: Clone,
{
  pub fn new(parent: Option<NodeRef<K, V, O>>) -> Self {
    Self {
      parent,
      size: 0,
      records: array_macro::array![None; O],
    }
  }

  pub fn set_parent(&mut self, parent: Option<NodeRef<K, V, O>>) {
    self.parent = parent;
  }

  pub fn size(&self) -> usize {
    self.size
  }

  pub fn lookup_node(&self, key: &K) -> Option<NodeRef<K, V, O>> {
    unimplemented!()
  }

  pub fn first_key(&self) -> Option<&K> {
    unimplemented!()
  }
}
