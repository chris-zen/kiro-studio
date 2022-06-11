pub(crate) mod context;
pub mod ports;

use std::fmt::Formatter;

use crate::graph::NodeDescriptor;
pub use context::ProcessorContext;

pub type BoxedProcessor = Box<dyn Processor + 'static>;

pub trait Processor {
  /// Return the static node descriptor.
  fn static_descriptor() -> NodeDescriptor
  where
    Self: Sized,
  {
    NodeDescriptor::new()
  }

  /// Return the node descriptor for the instance.
  /// By default it delegates to the static node descriptor.
  fn descriptor(&self) -> NodeDescriptor
  where
    Self: Sized,
  {
    Self::static_descriptor()
  }

  // TODO fn prepare(&mut self, );

  /// Render the next period.
  /// Running in the audio context.
  fn render(&mut self, context: &mut ProcessorContext);
}

impl std::fmt::Debug for dyn Processor {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.write_str("Processor()")
  }
}
