use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use crate::engine::InnerEngine;
use crate::error::Result;
use crate::graph::NodeKey;
use crate::module::Module;
use crate::ports::{NodeIn, NodeOut};
use crate::rendering::controller::{ParamKey, ProcessorKey};
use crate::{AudioNodeIn, AudioNodeOut, EventsNodeIn, EventsNodeOut, NodeDescriptor};

pub struct ProcessorNode {
  pub(crate) engine: Rc<RefCell<InnerEngine>>,
  pub(crate) node_key: NodeKey,
  pub(crate) processor_key: ProcessorKey,
  pub(crate) param_keys: Vec<ParamKey>,
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
