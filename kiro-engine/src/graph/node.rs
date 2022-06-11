use crate::graph::module::ModuleKey;
use crate::graph::param::ParamDescriptor;
use crate::graph::port::{
  AudioDescriptor, DescriptorPorts, EventsDescriptor, GenericDescriptorPorts, NodeLike, Ports,
};
use crate::key_gen::Key;

pub type NodeKey = Key<Node>;

#[derive(Debug)]
pub struct Node {
  pub name: String,
  pub descriptor: NodeDescriptor,
  pub parent: ModuleKey,
  pub path: String,
  pub ports: Ports,
}

impl Node {
  pub fn new<S: Into<String>>(
    name: S,
    descriptor: NodeDescriptor,
    module: ModuleKey,
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
      parent: module,
      path,
      ports,
    }
  }
}

impl NodeLike for Node {
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
pub struct NodeDescriptor {
  pub parameters: Vec<ParamDescriptor>,
  pub ports: DescriptorPorts,
}

impl NodeDescriptor {
  pub fn new() -> Self {
    Self {
      parameters: Vec::new(),
      ports: DescriptorPorts::new(),
    }
  }

  pub fn with_parameters(mut self, params: Vec<ParamDescriptor>) -> Self {
    self.parameters = params;
    self
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
