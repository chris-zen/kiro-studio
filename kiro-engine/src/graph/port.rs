use crate::graph::connection::{ModuleIn, ModuleOut, NodeOut};
use crate::graph::error::{Error, Result};
use crate::key_gen::Key;
use crate::key_store::{HasId, KeyStoreWithId};

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

pub trait PortAccessor<D> {
  fn get_input(&self) -> &KeyStoreWithId<InputPort<D>>;
  fn get_input_mut(&mut self) -> &mut KeyStoreWithId<InputPort<D>>;
  fn get_output(&self) -> &KeyStoreWithId<OutputPort<D>>;
  fn get_output_mut(&mut self) -> &mut KeyStoreWithId<OutputPort<D>>;
}

impl PortAccessor<AudioDescriptor> for Ports {
  fn get_input(&self) -> &KeyStoreWithId<InputPort<AudioDescriptor>> {
    &self.audio_input_ports
  }

  fn get_input_mut(&mut self) -> &mut KeyStoreWithId<InputPort<AudioDescriptor>> {
    &mut self.audio_input_ports
  }

  fn get_output(&self) -> &KeyStoreWithId<OutputPort<AudioDescriptor>> {
    &self.audio_output_ports
  }

  fn get_output_mut(&mut self) -> &mut KeyStoreWithId<OutputPort<AudioDescriptor>> {
    &mut self.audio_output_ports
  }
}

impl PortAccessor<EventsDescriptor> for Ports {
  fn get_input(&self) -> &KeyStoreWithId<InputPort<EventsDescriptor>> {
    &self.events_input_ports
  }

  fn get_input_mut(&mut self) -> &mut KeyStoreWithId<InputPort<EventsDescriptor>> {
    &mut self.events_input_ports
  }

  fn get_output(&self) -> &KeyStoreWithId<OutputPort<EventsDescriptor>> {
    &self.events_output_ports
  }

  fn get_output_mut(&mut self) -> &mut KeyStoreWithId<OutputPort<EventsDescriptor>> {
    &mut self.events_output_ports
  }
}

#[derive(Debug)]
pub struct Ports {
  pub audio_input_ports: KeyStoreWithId<AudioInputPort>,
  pub audio_output_ports: KeyStoreWithId<AudioOutputPort>,
  pub events_input_ports: KeyStoreWithId<EventsInputPort>,
  pub events_output_ports: KeyStoreWithId<EventsOutputPort>,
}

impl Ports {
  pub fn new(
    audio_input: &[AudioDescriptor],
    audio_output: &[AudioDescriptor],
    events_input: &[EventsDescriptor],
    events_output: &[EventsDescriptor],
  ) -> Self {
    let mut audio_input_ports = KeyStoreWithId::new();
    for descriptor in audio_input.iter() {
      audio_input_ports.add(AudioInputPort {
        descriptor: descriptor.clone(),
        source: None,
      });
    }

    let mut audio_output_ports = KeyStoreWithId::new();
    for descriptor in audio_output.iter() {
      audio_output_ports.add(AudioOutputPort {
        descriptor: descriptor.clone(),
        source: None,
      });
    }

    let mut events_input_ports = KeyStoreWithId::new();
    for descriptor in events_input.iter() {
      events_input_ports.add(EventsInputPort {
        descriptor: descriptor.clone(),
        source: None,
      });
    }

    let mut events_output_ports = KeyStoreWithId::new();
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

impl PortType {
  pub fn name(&self) -> &str {
    match self {
      PortType::Audio => "Audio",
      PortType::Events => "Events",
    }
  }
}

pub trait PortDescriptor: HasId + Clone {
  fn with_id<S: Into<String>>(self, id: S) -> Self;
  fn port_type() -> PortType;
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

  fn port_type() -> PortType {
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

  fn port_type() -> PortType {
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

pub trait DescriptorPortAccessor<D> {
  fn get_port(&self) -> &GenericDescriptorPorts<D>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DescriptorPorts {
  pub audio: GenericDescriptorPorts<AudioDescriptor>,
  pub events: GenericDescriptorPorts<EventsDescriptor>,
}

impl DescriptorPorts {
  pub fn new() -> Self {
    Self {
      audio: GenericDescriptorPorts::new(),
      events: GenericDescriptorPorts::new(),
    }
  }
}

impl DescriptorPortAccessor<AudioDescriptor> for DescriptorPorts {
  fn get_port(&self) -> &GenericDescriptorPorts<AudioDescriptor> {
    &self.audio
  }
}

impl DescriptorPortAccessor<EventsDescriptor> for DescriptorPorts {
  fn get_port(&self) -> &GenericDescriptorPorts<EventsDescriptor> {
    &self.events
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericDescriptorPorts<D> {
  pub static_inputs: Vec<D>,
  pub dynamic_inputs: DynamicPorts,
  pub static_outputs: Vec<D>,
  pub dynamic_outputs: DynamicPorts,
}

impl<D> GenericDescriptorPorts<D>
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

  fn get_descriptor_ports(&self) -> &DescriptorPorts;

  fn get_descriptor_port<D>(&self) -> &GenericDescriptorPorts<D>
  where
    D: PortDescriptor,
    DescriptorPorts: DescriptorPortAccessor<D>,
  {
    self.get_descriptor_ports().get_port()
  }

  fn get_ports(&self) -> &Ports;
  fn get_ports_mut(&mut self) -> &mut Ports;

  fn get_input_port<D>(&self, port_key: InputPortKey<D>) -> Result<&InputPort<D>>
  where
    D: PortDescriptor,
    Ports: PortAccessor<D>,
  {
    self
      .get_ports()
      .get_input()
      .get(port_key)
      .ok_or_else(|| Error::InputPortNotFound(self.full_name(), port_key.to_string()))
  }

  fn get_input_port_mut<D>(&mut self, port_key: InputPortKey<D>) -> Result<&mut InputPort<D>>
  where
    D: PortDescriptor,
    Ports: PortAccessor<D>,
  {
    // See this for further info on why this needs to be that convoluted:
    // https://github.com/rust-lang/rfcs/blob/master/text/2094-nll.md#problem-case-3-conditional-control-flow-across-functions
    if self.get_ports().get_input().contains_key(port_key) {
      match self.get_ports_mut().get_input_mut().get_mut(port_key) {
        Some(port) => return Ok(port),
        None => unreachable!(),
      }
    }
    Err(Error::InputPortNotFound(
      self.full_name(),
      port_key.to_string(),
    ))
  }

  fn get_output_port<D>(&self, port_key: OutputPortKey<D>) -> Result<&OutputPort<D>>
  where
    D: PortDescriptor,
    Ports: PortAccessor<D>,
  {
    self
      .get_ports()
      .get_output()
      .get(port_key)
      .ok_or_else(|| Error::OutputPortNotFound(self.full_name(), port_key.to_string()))
  }

  fn get_output_port_mut<D>(&mut self, port_key: OutputPortKey<D>) -> Result<&mut OutputPort<D>>
  where
    D: PortDescriptor,
    Ports: PortAccessor<D>,
  {
    // See this for further info on why this needs to be that convoluted:
    // https://github.com/rust-lang/rfcs/blob/master/text/2094-nll.md#problem-case-3-conditional-control-flow-across-functions
    if self.get_ports().get_output().contains_key(port_key) {
      match self.get_ports_mut().get_output_mut().get_mut(port_key) {
        Some(port) => return Ok(port),
        None => unreachable!(),
      }
    }
    Err(Error::OutputPortNotFound(
      self.full_name(),
      port_key.to_string(),
    ))
  }
}

impl HasId for &str {
  fn id(&self) -> &str {
    *self
  }
}

pub fn port_path<N: NodeLike, I: HasId>(node: &N, port: &I) -> String {
  format!("{}:{}", node.full_name(), port.id())
}
