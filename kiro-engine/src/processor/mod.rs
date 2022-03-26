pub(crate) mod context;
pub mod ports;

use std::fmt::Formatter;

pub use context::ProcessorContext;

pub type BoxedProcessor = Box<dyn Processor + 'static>;

pub trait Processor {
  fn render(&mut self, context: &mut ProcessorContext);
}

impl std::fmt::Debug for dyn Processor {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.write_str("Processor()")
  }
}
