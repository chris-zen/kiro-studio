use ringbuf::RingBuffer;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use thiserror::Error;

use crate::callback::renderer::Renderer;
use crate::config::EngineConfig;
use crate::controller::controller::{ParamKey, ProcessorKey};
use crate::controller::Controller;
use crate::graph::connection::{self, Connection};
use crate::graph::port::{
  AudioDescriptor, EventsDescriptor, InputPortKey, OutputPortKey, PortDescriptor,
};
use crate::graph::{self, InnerGraph, ModuleDescriptor, ModuleKey, NodeDescriptor, NodeKey};
use crate::processor::Processor;

type Result<T> = core::result::Result<T, Error>;

type AudioModuleIn = ModuleIn<AudioDescriptor>;
type AudioModuleOut = ModuleOut<AudioDescriptor>;
type EventsModuleIn = ModuleIn<EventsDescriptor>;
type EventsModuleOut = ModuleOut<EventsDescriptor>;

type AudioNodeIn = NodeIn<AudioDescriptor>;
type AudioNodeOut = NodeOut<AudioDescriptor>;
type EventsNodeIn = NodeIn<EventsDescriptor>;
type EventsNodeOut = NodeOut<EventsDescriptor>;

#[derive(Debug, Error)]
pub enum Error {
  #[error("Graph: {0}")]
  Graph(#[from] graph::Error),
}

struct InnerEngine {
  graph: InnerGraph,
  controller: Controller,
}

pub struct Engine {
  inner: Rc<RefCell<InnerEngine>>,
  renderer: Option<Renderer>,
}

impl Engine {
  pub fn with_config(config: EngineConfig) -> Self {
    let ring_buffer_capacity = config.ring_buffer_capacity;
    let (forward_tx, forward_rx) = RingBuffer::new(ring_buffer_capacity).split();
    let (backward_tx, backward_rx) = RingBuffer::new(ring_buffer_capacity).split();
    let graph = InnerGraph::new();
    let controller = Controller::new(forward_tx, backward_rx, config.clone());
    let inner = Rc::new(RefCell::new(InnerEngine { graph, controller }));
    let renderer = Some(Renderer::new(backward_tx, forward_rx, config));

    Self { inner, renderer }
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
    Self::with_config(EngineConfig::default())
  }
}

pub struct Module {
  engine: Rc<RefCell<InnerEngine>>,
  key: ModuleKey,
}

impl Module {
  pub fn parent(&self) -> Result<Option<Self>> {
    let engine = self.engine.deref().borrow();
    let module = engine.graph.get_module(self.key)?;
    let maybe_parent = module.parent.map(|key| Module {
      engine: self.engine.clone(),
      key,
    });
    Ok(maybe_parent)
  }

  pub fn name(&self) -> Result<String> {
    let engine = self.engine.deref().borrow();
    let module = engine.graph.get_module(self.key)?;
    Ok(module.name.clone())
  }

  pub fn path(&self) -> Result<String> {
    let engine = self.engine.deref().borrow();
    let module = engine.graph.get_module(self.key)?;
    Ok(module.path.clone())
  }

  pub fn descriptor(&self) -> Result<ModuleDescriptor> {
    let engine = self.engine.deref().borrow();
    let module = engine.graph.get_module(self.key)?;
    Ok(module.descriptor.clone())
  }

  pub fn create_module(&mut self, name: &str, descriptor: ModuleDescriptor) -> Result<Module> {
    let mut engine = self.engine.borrow_mut();
    let key = engine.graph.create_module(self.key, name, descriptor)?;
    Ok(Module {
      engine: self.engine.clone(),
      key,
    })
  }

  pub fn create_processor<P>(&mut self, name: &str, processor: P) -> Result<ProcessorNode>
  where
    P: Processor + 'static,
  {
    let mut engine = self.engine.borrow_mut();
    let descriptor = processor.descriptor();

    let processor_key = engine.controller.add_processor(processor);
    let initial_values = descriptor
      .parameters
      .iter()
      .map(|param_descriptor| param_descriptor.initial)
      .collect::<Vec<f32>>();
    let param_keys = engine.controller.add_parameters(initial_values.as_slice());

    let node_key = engine.graph.create_node(self.key, name, descriptor)?;

    Ok(ProcessorNode {
      engine: self.engine.clone(),
      node_key,
      processor_key,
      param_keys,
    })
  }

  pub fn audio_input(&self, name: &str) -> Result<AudioModuleIn> {
    let engine = self.engine.deref().borrow();
    let port_key = engine.graph.module_audio_input(self.key, name)?;
    Ok(ModuleIn {
      engine: self.engine.clone(),
      module_key: self.key,
      port_key,
    })
  }

  pub fn audio_output(&self, name: &str) -> Result<AudioModuleOut> {
    let engine = self.engine.deref().borrow();
    let port_key = engine.graph.module_audio_output(self.key, name)?;
    Ok(ModuleOut {
      engine: self.engine.clone(),
      module_key: self.key,
      port_key,
    })
  }

  pub fn events_input(&self, name: &str) -> Result<EventsModuleIn> {
    let engine = self.engine.deref().borrow();
    let port_key = engine.graph.module_events_input(self.key, name)?;
    Ok(ModuleIn {
      engine: self.engine.clone(),
      module_key: self.key,
      port_key,
    })
  }

  pub fn events_output(&self, name: &str) -> Result<EventsModuleOut> {
    let engine = self.engine.deref().borrow();
    let port_key = engine.graph.module_events_output(self.key, name)?;
    Ok(ModuleOut {
      engine: self.engine.clone(),
      module_key: self.key,
      port_key,
    })
  }
}

pub struct ProcessorNode {
  engine: Rc<RefCell<InnerEngine>>,
  node_key: NodeKey,
  processor_key: ProcessorKey,
  param_keys: Vec<ParamKey>,
}

impl ProcessorNode {
  pub fn parent(&self) -> Result<Module> {
    let engine = self.engine.deref().borrow();
    let node = engine.graph.get_node(self.node_key)?;
    Ok(Module {
      engine: self.engine.clone(),
      key: node.parent,
    })
  }

  pub fn name(&self) -> Result<String> {
    let engine = self.engine.deref().borrow();
    let node = engine.graph.get_node(self.node_key)?;
    Ok(node.name.clone())
  }

  pub fn path(&self) -> Result<String> {
    let engine = self.engine.deref().borrow();
    let node = engine.graph.get_node(self.node_key)?;
    Ok(node.path.clone())
  }

  pub fn descriptor(&self) -> Result<NodeDescriptor> {
    let engine = self.engine.deref().borrow();
    let node = engine.graph.get_node(self.node_key)?;
    Ok(node.descriptor.clone())
  }

  pub fn audio_input(&self, name: &str) -> Result<AudioNodeIn> {
    let engine = self.engine.deref().borrow();
    let port_key = engine.graph.node_audio_input(self.node_key, name)?;
    Ok(NodeIn {
      engine: self.engine.clone(),
      node_key: self.node_key,
      port_key,
    })
  }

  pub fn audio_output(&self, name: &str) -> Result<AudioNodeOut> {
    let engine = self.engine.deref().borrow();
    let port_key = engine.graph.node_audio_output(self.node_key, name)?;
    Ok(NodeOut {
      engine: self.engine.clone(),
      node_key: self.node_key,
      port_key,
    })
  }

  pub fn events_input(&self, name: &str) -> Result<EventsNodeIn> {
    let engine = self.engine.deref().borrow();
    let port_key = engine.graph.node_events_input(self.node_key, name)?;
    Ok(NodeIn {
      engine: self.engine.clone(),
      node_key: self.node_key,
      port_key,
    })
  }

  pub fn events_output(&self, name: &str) -> Result<EventsNodeOut> {
    let engine = self.engine.deref().borrow();
    let port_key = engine.graph.node_events_output(self.node_key, name)?;
    Ok(NodeOut {
      engine: self.engine.clone(),
      node_key: self.node_key,
      port_key,
    })
  }
}

/// Module Input Port
#[derive(Clone)]
pub struct ModuleIn<D> {
  engine: Rc<RefCell<InnerEngine>>,
  module_key: ModuleKey,
  port_key: InputPortKey<D>,
}

impl<D> ModuleIn<D>
where
  D: PortDescriptor,
{
  pub fn bind<B>(self, other: B) -> Result<()>
  where
    B: Into<connection::ModuleInBind<D>>,
  {
    let engine = self.engine.deref().borrow();
    let connection = connection::ModuleIn::<D>::bind(self.clone().into(), other);
    // engine.graph.connect_audio(connection).into()
    Ok(()) // FIXME
  }

  pub fn from<S>(self, other: S) -> Connection<D>
  where
    S: Into<connection::ModuleInFrom<D>>,
  {
    connection::ModuleIn::<D>::from(self.into(), other)
  }
}

impl<D> From<ModuleIn<D>> for connection::ModuleIn<D> {
  fn from(module_in: ModuleIn<D>) -> Self {
    connection::ModuleIn(module_in.module_key, module_in.port_key)
  }
}

/// Module Output Port
pub struct ModuleOut<D> {
  engine: Rc<RefCell<InnerEngine>>,
  module_key: ModuleKey,
  port_key: OutputPortKey<D>,
}

impl<D> From<ModuleOut<D>> for connection::ModuleOut<D> {
  fn from(module_out: ModuleOut<D>) -> Self {
    connection::ModuleOut(module_out.module_key, module_out.port_key)
  }
}

/// Node Input Port
pub struct NodeIn<D> {
  engine: Rc<RefCell<InnerEngine>>,
  node_key: NodeKey,
  port_key: InputPortKey<D>,
}

impl<D> From<NodeIn<D>> for connection::NodeIn<D> {
  fn from(node_in: NodeIn<D>) -> Self {
    connection::NodeIn(node_in.node_key, node_in.port_key)
  }
}

/// Node Input Port
pub struct NodeOut<D> {
  engine: Rc<RefCell<InnerEngine>>,
  node_key: NodeKey,
  port_key: OutputPortKey<D>,
}

impl<D> From<NodeOut<D>> for connection::NodeOut<D> {
  fn from(node_out: NodeOut<D>) -> Self {
    connection::NodeOut(node_out.node_key, node_out.port_key)
  }
}
