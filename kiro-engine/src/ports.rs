use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use crate::engine::InnerEngine;
use crate::error::Result;
use crate::graph::connection::{self, Connection};
use crate::graph::port::{InputPortKey, OutputPortKey, PortDescriptor};
use crate::graph::{ModuleKey, NodeKey};
use crate::{AudioDescriptor, EventsDescriptor};

pub type AudioModuleIn = ModuleIn<AudioDescriptor>;
pub type AudioModuleOut = ModuleOut<AudioDescriptor>;
pub type EventsModuleIn = ModuleIn<EventsDescriptor>;
pub type EventsModuleOut = ModuleOut<EventsDescriptor>;

pub type AudioNodeIn = NodeIn<AudioDescriptor>;
pub type AudioNodeOut = NodeOut<AudioDescriptor>;
pub type EventsNodeIn = NodeIn<EventsDescriptor>;
pub type EventsNodeOut = NodeOut<EventsDescriptor>;

/// Module Input Port
#[derive(Clone)]
pub struct ModuleIn<D> {
  pub(crate) engine: Rc<RefCell<InnerEngine>>,
  pub(crate) module_key: ModuleKey,
  pub(crate) port_key: InputPortKey<D>,
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
  pub(crate) engine: Rc<RefCell<InnerEngine>>,
  pub(crate) module_key: ModuleKey,
  pub(crate) port_key: OutputPortKey<D>,
}

impl<D> From<ModuleOut<D>> for connection::ModuleOut<D> {
  fn from(module_out: ModuleOut<D>) -> Self {
    connection::ModuleOut(module_out.module_key, module_out.port_key)
  }
}

/// Node Input Port
pub struct NodeIn<D> {
  pub(crate) engine: Rc<RefCell<InnerEngine>>,
  pub(crate) node_key: NodeKey,
  pub(crate) port_key: InputPortKey<D>,
}

impl<D> From<NodeIn<D>> for connection::NodeIn<D> {
  fn from(node_in: NodeIn<D>) -> Self {
    connection::NodeIn(node_in.node_key, node_in.port_key)
  }
}

/// Node Input Port
pub struct NodeOut<D> {
  pub(crate) engine: Rc<RefCell<InnerEngine>>,
  pub(crate) node_key: NodeKey,
  pub(crate) port_key: OutputPortKey<D>,
}

impl<D> From<NodeOut<D>> for connection::NodeOut<D> {
  fn from(node_out: NodeOut<D>) -> Self {
    connection::NodeOut(node_out.node_key, node_out.port_key)
  }
}
