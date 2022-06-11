use ringbuf::RingBuffer;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use crate::config::EngineConfig;
use crate::error::Result;
use crate::graph::{Graph, ModuleDescriptor};
use crate::node::ProcessorNode;
use crate::processor::Processor;
use crate::rendering::controller::{AudioBufferKey, Controller};
use crate::rendering::renderer::Renderer;
use crate::{Error, Module};

pub(crate) struct InnerEngine {
  pub(crate) graph: Graph,
  pub(crate) controller: Controller,
}

pub struct Engine {
  inner: Rc<RefCell<InnerEngine>>,
  renderer: Option<Renderer>,
  audio_input_buffer_key: AudioBufferKey,
}

impl Engine {
  pub fn new(config: EngineConfig) -> Self {
    let ring_buffer_capacity = config.ring_buffer_capacity;
    let (forward_tx, forward_rx) = RingBuffer::new(ring_buffer_capacity).split();
    let (backward_tx, backward_rx) = RingBuffer::new(ring_buffer_capacity).split();
    let graph = Graph::new(config.audio_input_channels, config.audio_output_channels);
    let mut controller = Controller::new(forward_tx, backward_rx, config.clone());
    let audio_input_buffer_key = controller.add_audio_buffer();
    let inner = Rc::new(RefCell::new(InnerEngine { graph, controller }));
    let renderer = Some(Renderer::new(backward_tx, forward_rx, config));

    Self {
      inner,
      renderer,
      audio_input_buffer_key,
    }
  }

  pub fn take_renderer(&mut self) -> Option<Renderer> {
    self.renderer.take()
  }

  pub fn create_module(&mut self, name: &str, descriptor: ModuleDescriptor) -> Result<Module> {
    self.root_module().create_module(name, descriptor)
  }

  pub fn create_processor<P>(&mut self, name: &str, processor: P) -> Result<ProcessorNode>
  where
    P: Processor + 'static,
  {
    self.root_module().create_processor(name, processor)
  }

  pub fn update_render_plan(&mut self) -> Result<()> {
    let mut engine = self.inner.borrow_mut();
    engine
      .controller
      .send_render_plan(
        vec![],
        vec![self.audio_input_buffer_key],
        vec![],
        vec![],
        vec![],
      )
      .map_err(Error::Controller)?;
    todo!()
  }

  #[inline]
  fn root_module(&self) -> Module {
    Module {
      engine: self.inner.clone(),
      key: self.inner.deref().borrow().graph.get_root_module(),
    }
  }
}

impl Default for Engine {
  fn default() -> Self {
    Self::new(EngineConfig::default())
  }
}
