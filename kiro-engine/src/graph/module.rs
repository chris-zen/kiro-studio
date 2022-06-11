use crate::graph::node::NodeKey;
use crate::graph::port::{
  AudioDescriptor, DescriptorPorts, EventsDescriptor, GenericDescriptorPorts, NodeLike, Ports,
};
use crate::key_gen::Key;
use std::collections::HashSet;

pub type ModuleKey = Key<Module>;

#[derive(Debug)]
pub struct Module {
  pub name: String,
  pub descriptor: ModuleDescriptor,
  pub parent: Option<ModuleKey>,
  pub path: String,
  pub nodes: HashSet<NodeKey>,
  pub ports: Ports,
}

impl Module {
  pub fn new<S: Into<String>>(
    name: S,
    descriptor: ModuleDescriptor,
    parent: Option<ModuleKey>,
    path: String,
  ) -> Self {
    let ports = Ports::new(
      descriptor.ports.audio.static_inputs.as_slice(),
      descriptor.ports.audio.static_outputs.as_slice(),
      descriptor.ports.events.static_inputs.as_slice(),
      descriptor.ports.events.static_outputs.as_slice(),
    );

    Self {
      name: name.into(),
      descriptor,
      parent,
      path,
      nodes: HashSet::new(),
      ports,
    }
  }
}

impl NodeLike for Module {
  fn full_name(&self) -> String {
    format!("{}/{}", self.path, self.name)
  }

  fn get_descriptor_ports(&self) -> &DescriptorPorts {
    &self.descriptor.ports
  }

  fn get_ports(&self) -> &Ports {
    &self.ports
  }

  fn get_ports_mut(&mut self) -> &mut Ports {
    &mut self.ports
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleDescriptor {
  pub ports: DescriptorPorts,
}

impl ModuleDescriptor {
  pub fn new() -> Self {
    Self {
      ports: DescriptorPorts::new(),
    }
  }

  pub fn with_audio_ports<F>(mut self, f: F) -> Self
  where
    F: FnOnce(GenericDescriptorPorts<AudioDescriptor>) -> GenericDescriptorPorts<AudioDescriptor>,
  {
    self.ports.audio = f(self.ports.audio);
    self
  }

  pub fn with_events_ports<F>(mut self, f: F) -> Self
  where
    F: FnOnce(GenericDescriptorPorts<EventsDescriptor>) -> GenericDescriptorPorts<EventsDescriptor>,
  {
    self.ports.events = f(self.ports.events);
    self
  }
}
