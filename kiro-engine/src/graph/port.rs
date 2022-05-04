use crate::graph::connection::{ModuleIn, ModuleOut, NodeOut};
use crate::graph::error::{Error, Result};
use crate::key_gen::Key;
use crate::key_store::{HasId, KeyStore};

pub type InputPortKey<D> = Key<InputPort<D>>;
pub type OutputPortKey<D> = Key<OutputPort<D>>;

pub type AudioInputPortKey = InputPortKey<AudioDescriptor>;
pub type AudioOutputPortKey = OutputPortKey<AudioDescriptor>;

pub type AudioInputPort = InputPort<AudioDescriptor>;
pub type AudioOutputPort = OutputPort<AudioDescriptor>;

pub type EventsInputPortKey = InputPortKey<EventsDescriptor>;
pub type EventsOutputPortKey = OutputPortKey<EventsDescriptor>;

pub type EventsInputPort = InputPort<EventsDescriptor>;
pub type EventsOutputPort = OutputPort<EventsDescriptor>;

#[derive(Debug, Clone, PartialEq)]
pub struct InputPort<D> {
  pub descriptor: D,
  pub source: Option<InputSource<D>>,
}

impl<D> HasId for InputPort<D>
where
  D: HasId,
{
  fn id(&self) -> &str {
    self.descriptor.id()
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputSource<D> {
  ModuleBinding(ModuleIn<D>),
  ModuleConnection(ModuleOut<D>),
  NodeConnection(NodeOut<D>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct OutputPort<D> {
  pub descriptor: D,
  pub source: Option<OutputSource<D>>,
}

impl<D> HasId for OutputPort<D>
where
  D: HasId,
{
  fn id(&self) -> &str {
    self.descriptor.id()
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutputSource<D> {
  ModuleBinding(ModuleOut<D>),
  NodeBinding(NodeOut<D>),
}

#[derive(Debug, PartialEq)]
pub struct Ports {
  pub audio_input_ports: KeyStore<AudioInputPort>,
  pub audio_output_ports: KeyStore<AudioOutputPort>,
  pub events_input_ports: KeyStore<EventsInputPort>,
  pub events_output_ports: KeyStore<EventsOutputPort>,
}

impl Ports {
  pub fn new(
    audio_input: &[AudioDescriptor],
    audio_output: &[AudioDescriptor],
    events_input: &[EventsDescriptor],
    events_output: &[EventsDescriptor],
  ) -> Self {
    let mut audio_input_ports = KeyStore::new();
    for descriptor in audio_input.iter() {
      audio_input_ports.add(AudioInputPort {
        descriptor: descriptor.clone(),
        source: None,
      });
    }

    let mut audio_output_ports = KeyStore::new();
    for descriptor in audio_output.iter() {
      audio_output_ports.add(AudioOutputPort {
        descriptor: descriptor.clone(),
        source: None,
      });
    }

    let mut events_input_ports = KeyStore::new();
    for descriptor in events_input.iter() {
      events_input_ports.add(EventsInputPort {
        descriptor: descriptor.clone(),
        source: None,
      });
    }

    let mut events_output_ports = KeyStore::new();
    for descriptor in events_output.iter() {
      events_output_ports.add(EventsOutputPort {
        descriptor: descriptor.clone(),
        source: None,
      });
    }

    Self {
      audio_input_ports,
      audio_output_ports,
      events_input_ports,
      events_output_ports,
    }
  }
}

pub enum PortType {
  Audio,
  Events,
}

pub trait PortDescriptor: HasId + Clone {
  fn with_id<S: Into<String>>(self, id: S) -> Self;
  fn port_type(&self) -> PortType;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioDescriptor {
  id: String,
  channels: usize,
}

impl AudioDescriptor {
  pub fn new<S: Into<String>>(id: S, channels: usize) -> Self {
    Self {
      id: id.into(),
      channels,
    }
  }

  pub fn channels(&self) -> usize {
    self.channels
  }
}

impl PortDescriptor for AudioDescriptor {
  fn with_id<S: Into<String>>(mut self, id: S) -> Self {
    self.id = id.into();
    self
  }

  fn port_type(&self) -> PortType {
    PortType::Audio
  }
}

impl HasId for AudioDescriptor {
  fn id(&self) -> &str {
    self.id.as_str()
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EventsDescriptor {
  id: String,
}

impl EventsDescriptor {
  pub fn new<S: Into<String>>(id: S) -> Self {
    Self { id: id.into() }
  }
}

impl PortDescriptor for EventsDescriptor {
  fn with_id<S: Into<String>>(mut self, id: S) -> Self {
    self.id = id.into();
    self
  }

  fn port_type(&self) -> PortType {
    PortType::Events
  }
}

impl HasId for EventsDescriptor {
  fn id(&self) -> &str {
    self.id.as_str()
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DynamicPorts {
  None,
  Limited(usize),
  Unlimited,
}

pub type DescriptorAudioPorts = DescriptorPorts<AudioDescriptor>;
pub type DescriptorEventsPorts = DescriptorPorts<EventsDescriptor>;

#[derive(Debug, Clone, PartialEq)]
pub struct DescriptorPorts<D> {
  pub static_inputs: Vec<D>,
  pub dynamic_inputs: DynamicPorts,
  pub static_outputs: Vec<D>,
  pub dynamic_outputs: DynamicPorts,
}

impl<D> DescriptorPorts<D>
where
  D: PortDescriptor,
{
  pub fn new() -> Self {
    Self {
      static_inputs: Vec::new(),
      dynamic_inputs: DynamicPorts::None,
      static_outputs: Vec::new(),
      dynamic_outputs: DynamicPorts::None,
    }
  }

  pub fn static_inputs(mut self, descriptors: Vec<D>) -> Self {
    self.static_inputs = descriptors;
    self
  }

  pub fn static_inputs_cardinality(mut self, cardinality: usize, template_descriptor: D) -> Self {
    let prefix = template_descriptor.id();
    self.static_inputs = (0..cardinality)
      .into_iter()
      .map(|i| {
        template_descriptor
          .clone()
          .with_id(format!("{}-{}", prefix, i))
      })
      .collect();
    self
  }

  pub fn dynamic_inputs(mut self, dynamic_ports: DynamicPorts) -> Self {
    self.dynamic_inputs = dynamic_ports;
    self
  }

  pub fn static_outputs(mut self, descriptors: Vec<D>) -> Self {
    self.static_outputs = descriptors;
    self
  }

  pub fn static_outputs_cardinality(mut self, cardinality: usize, template_descriptor: D) -> Self {
    let prefix = template_descriptor.id();
    self.static_outputs = (0..cardinality)
      .into_iter()
      .map(|i| {
        template_descriptor
          .clone()
          .with_id(format!("{}-{}", prefix, i))
      })
      .collect();
    self
  }

  pub fn dynamic_outputs(mut self, dynamic_ports: DynamicPorts) -> Self {
    self.dynamic_outputs = dynamic_ports;
    self
  }
}

pub trait NodeLike {
  fn full_name(&self) -> String;

  fn get_audio_descriptor_ports(&self) -> &DescriptorPorts<AudioDescriptor>;
  fn get_events_descriptor_ports(&self) -> &DescriptorPorts<EventsDescriptor>;

  fn get_ports(&self) -> &Ports;
  fn get_ports_mut(&mut self) -> &mut Ports;

  fn get_audio_input_port(&self, port_key: AudioInputPortKey) -> Result<&AudioInputPort> {
    self
      .get_ports()
      .audio_input_ports
      .get(port_key)
      .ok_or_else(|| Error::AudioInputPortNotFound(self.full_name(), port_key))
  }

  fn get_audio_input_port_mut(
    &mut self,
    port_key: AudioInputPortKey,
  ) -> Result<&mut AudioInputPort> {
    // See this for further info on why this needs to be that convoluted:
    // https://github.com/rust-lang/rfcs/blob/master/text/2094-nll.md#problem-case-3-conditional-control-flow-across-functions
    if self.get_ports().audio_input_ports.contains_key(port_key) {
      match self.get_ports_mut().audio_input_ports.get_mut(port_key) {
        Some(port) => return Ok(port),
        None => unreachable!(),
      }
    }
    Err(Error::AudioInputPortNotFound(self.full_name(), port_key))
  }

  fn get_audio_output_port(&self, port_key: AudioOutputPortKey) -> Result<&AudioOutputPort> {
    self
      .get_ports()
      .audio_output_ports
      .get(port_key)
      .ok_or_else(|| Error::AudioOutputPortNotFound(self.full_name(), port_key))
  }

  fn get_audio_output_port_mut(
    &mut self,
    port_key: AudioOutputPortKey,
  ) -> Result<&mut AudioOutputPort> {
    // See this for further info on why this needs to be that convoluted:
    // https://github.com/rust-lang/rfcs/blob/master/text/2094-nll.md#problem-case-3-conditional-control-flow-across-functions
    if self.get_ports().audio_output_ports.contains_key(port_key) {
      match self.get_ports_mut().audio_output_ports.get_mut(port_key) {
        Some(port) => return Ok(port),
        None => unreachable!(),
      }
    }
    Err(Error::AudioOutputPortNotFound(self.full_name(), port_key))
  }

  fn get_events_input_port(&self, port_key: EventsInputPortKey) -> Result<&EventsInputPort> {
    self
      .get_ports()
      .events_input_ports
      .get(port_key)
      .ok_or_else(|| Error::EventsInputPortNotFound(self.full_name(), port_key))
  }

  fn get_events_input_port_mut(
    &mut self,
    port_key: EventsInputPortKey,
  ) -> Result<&mut EventsInputPort> {
    // See this for further info on why this needs to be that convoluted:
    // https://github.com/rust-lang/rfcs/blob/master/text/2094-nll.md#problem-case-3-conditional-control-flow-across-functions
    if self.get_ports().events_input_ports.contains_key(port_key) {
      match self.get_ports_mut().events_input_ports.get_mut(port_key) {
        Some(port) => return Ok(port),
        None => unreachable!(),
      }
    }
    Err(Error::EventsInputPortNotFound(self.full_name(), port_key))
  }

  fn get_events_output_port(&self, port_key: EventsOutputPortKey) -> Result<&EventsOutputPort> {
    self
      .get_ports()
      .events_output_ports
      .get(port_key)
      .ok_or_else(|| Error::EventsOutputPortNotFound(self.full_name(), port_key))
  }

  fn get_events_output_port_mut(
    &mut self,
    port_key: EventsOutputPortKey,
  ) -> Result<&mut EventsOutputPort> {
    // See this for further info on why this needs to be that convoluted:
    // https://github.com/rust-lang/rfcs/blob/master/text/2094-nll.md#problem-case-3-conditional-control-flow-across-functions
    if self.get_ports().events_output_ports.contains_key(port_key) {
      match self.get_ports_mut().events_output_ports.get_mut(port_key) {
        Some(port) => return Ok(port),
        None => unreachable!(),
      }
    }
    Err(Error::EventsOutputPortNotFound(self.full_name(), port_key))
  }
}

pub fn port_path<N: NodeLike, I: HasId>(node: &N, port: &I) -> String {
  format!("{}:{}", node.full_name(), port.id())
}
