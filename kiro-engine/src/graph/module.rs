use std::collections::HashSet;

use crate::graph::node::NodeKey;
use crate::graph::port::{AudioDescriptor, DescriptorPorts, EventsDescriptor, HasPorts, Ports};
use crate::key_gen::Key;

pub type ModuleKey = Key<Module>;

#[derive(Debug, PartialEq)]
pub struct Module {
  pub name: String,
  pub descriptor: ModuleDescriptor,
  pub parent: Option<ModuleKey>,
  pub nodes: HashSet<NodeKey>,
  pub ports: Ports,
}

impl Module {
  pub fn new<S: Into<String>>(
    name: S,
    descriptor: ModuleDescriptor,
    parent: Option<ModuleKey>,
  ) -> Self {
    let ports = Ports::new(
      descriptor.audio_ports.static_inputs.as_slice(),
      descriptor.audio_ports.static_outputs.as_slice(),
      descriptor.events_ports.static_inputs.as_slice(),
      descriptor.events_ports.static_outputs.as_slice(),
    );

    Self {
      name: name.into(),
      descriptor,
      parent,
      nodes: HashSet::new(),
      ports,
    }
  }
}

impl HasPorts for Module {
  fn get_audio_descriptor_ports(&self) -> &DescriptorPorts<AudioDescriptor> {
    &self.descriptor.audio_ports
  }

  fn get_events_descriptor_ports(&self) -> &DescriptorPorts<EventsDescriptor> {
    &self.descriptor.events_ports
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
  pub audio_ports: DescriptorPorts<AudioDescriptor>,
  pub events_ports: DescriptorPorts<EventsDescriptor>,
}

impl ModuleDescriptor {
  pub fn new() -> Self {
    Self {
      audio_ports: DescriptorPorts::new(),
      events_ports: DescriptorPorts::new(),
    }
  }

  pub fn with_audio_ports<F>(mut self, f: F) -> Self
  where
    F: FnOnce(DescriptorPorts<AudioDescriptor>) -> DescriptorPorts<AudioDescriptor>,
  {
    self.audio_ports = f(self.audio_ports);
    self
  }

  pub fn with_events_ports<F>(mut self, f: F) -> Self
  where
    F: FnOnce(DescriptorPorts<EventsDescriptor>) -> DescriptorPorts<EventsDescriptor>,
  {
    self.events_ports = f(self.events_ports);
    self
  }
}
