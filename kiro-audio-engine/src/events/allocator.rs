use std::cell::RefCell;
use std::sync::Arc;

use thiserror::Error;

#[derive(Debug)]
pub struct AllocRef<T>(Arc<RefCell<T>>);

impl<T> AllocRef<T> {
  pub fn new(inner: Arc<RefCell<T>>) -> Self {
    Self(inner)
  }

  pub fn from(data: T) -> Self {
    Self(Arc::new(RefCell::new(data)))
  }

  pub fn borrow(&self) -> std::cell::Ref<T> {
    (*self.0).borrow()
  }

  pub fn borrow_mut(&self) -> std::cell::RefMut<T> {
    (*self.0).borrow_mut()
  }
}

impl<T> Clone for AllocRef<T> {
  fn clone(&self) -> Self {
    Self::new(self.0.clone())
  }
}

#[derive(Error, Debug, PartialEq)]
pub enum AllocError {
  #[error("Error allocating a node")]
  Acquire,

  #[error("Error releasing a node")]
  Release,
}

pub trait Allocator<T> {
  fn available(&self) -> usize;
  fn acquire(&mut self) -> Result<AllocRef<T>, AllocError>;
  fn release(&mut self, slab: AllocRef<T>) -> Result<(), AllocRef<T>>;
}
