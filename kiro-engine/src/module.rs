use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use crate::engine::InnerEngine;
use crate::error::Result;
use crate::graph::ModuleKey;
use crate::node::ProcessorNode;
use crate::ports::{AudioModuleIn, AudioModuleOut, EventsModuleIn, EventsModuleOut};
use crate::ports::{ModuleIn, ModuleOut};
use crate::{ModuleDescriptor, Processor};

pub struct Module {
  pub(crate) engine: Rc<RefCell<InnerEngine>>,
  pub(crate) key: ModuleKey,
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
