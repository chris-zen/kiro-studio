use crate::graph::module::ModuleKey;
use crate::graph::param::ParamDescriptor;
use crate::graph::port::{AudioDescriptor, DescriptorPorts, EventsDescriptor, HasPorts, Ports};
use crate::key_gen::Key;

pub type NodeKey = Key<Node>;

#[derive(Debug, PartialEq)]
pub struct Node {
  pub name: String,
  pub descriptor: NodeDescriptor,
  pub module: ModuleKey,
  pub ports: Ports,
}

impl Node {
  pub fn new<S: Into<String>>(name: S, descriptor: NodeDescriptor, module: ModuleKey) -> Self {
    let ports = Ports::new(
      descriptor.audio_ports.static_inputs.as_slice(),
      descriptor.audio_ports.static_outputs.as_slice(),
      descriptor.events_ports.static_inputs.as_slice(),
      descriptor.events_ports.static_outputs.as_slice(),
    );

    Self {
      name: name.into(),
      descriptor,
      module,
      ports,
    }
  }
}

impl HasPorts for Node {
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
pub struct NodeDescriptor {
  pub class: String,
  pub parameters: Vec<ParamDescriptor>,
  pub audio_ports: DescriptorPorts<AudioDescriptor>,
  pub events_ports: DescriptorPorts<EventsDescriptor>,
}

impl NodeDescriptor {
  pub fn new<S: Into<String>>(class: S) -> Self {
    Self {
      class: class.into(),
      parameters: Vec::new(),
      audio_ports: DescriptorPorts::new(),
      events_ports: DescriptorPorts::new(),
    }
  }

  pub fn class(&self) -> &str {
    self.class.as_str()
  }

  pub fn parameters(mut self, params: Vec<ParamDescriptor>) -> Self {
    self.parameters = params;
    self
  }
}
