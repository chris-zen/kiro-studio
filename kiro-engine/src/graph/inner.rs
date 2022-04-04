#![allow(clippy::wrong_self_convention)]

use thiserror::Error;

use crate::graph::module::{Module, ModuleDescriptor, ModuleKey};
use crate::graph::node::{Node, NodeDescriptor, NodeKey};
use crate::graph::port::{
  AudioDescriptor, DynamicPorts, EventsDescriptor, HasPorts, InputPortKey, OutputPort,
  OutputPortKey, PortType,
};
use crate::graph::port::{InputPort, PortDescriptor};
use crate::key_store::KeyStore;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
  #[error("Module not found: {0}")]
  ModuleNotFound(ModuleKey),

  #[error("Node not found: {0}")]
  NodeNotFound(NodeKey),

  #[error("Dynamic ports not available")]
  DynamicPortsNotAvailable,
}

pub struct InnerGraph {
  root_module: ModuleKey,
  modules: KeyStore<Module>,
  nodes: KeyStore<Node>,
}

impl InnerGraph {
  /// Create a new inner graph with a root module
  pub fn new() -> Self {
    let mut modules = KeyStore::new();
    let root_module = modules.add(Module::new("root", ModuleDescriptor::new(), None));
    Self {
      root_module,
      modules,
      nodes: KeyStore::new(),
    }
  }

  /// Get the root module
  pub fn get_root_module(&self) -> ModuleKey {
    self.root_module
  }

  /// Create a new module in the graph.
  /// It will create all the ports declared in the descriptor as static ports.
  pub fn create_module(
    &mut self,
    parent: ModuleKey,
    name: &str,
    descriptor: ModuleDescriptor,
  ) -> Result<ModuleKey> {
    if self.modules.contains_key(parent) {
      let module = Module::new(name, descriptor, Some(parent));
      Ok(self.modules.add(module))
    } else {
      Err(Error::ModuleNotFound(parent))
    }
  }

  /// Remove a module from the graph.
  /// It will remove all the children modules, nodes and connections recursively.
  pub fn remove_module(&mut self, module_key: ModuleKey) -> Result<()> {
    // TODO remove recursively
    let module_nodes = self
      .nodes
      .iter()
      .filter_map(|(node_key, node)| (node.module == module_key).then(|| node_key))
      .collect::<Vec<NodeKey>>();

    for node_key in module_nodes {
      self.nodes.remove(node_key);
    }

    self
      .modules
      .remove(module_key)
      .map(|_| ())
      .ok_or(Error::ModuleNotFound(module_key))
  }

  /// Add a new dynamic audio input to the module.
  /// It will check the dynamic ports constrains declared in the port descriptor.
  pub fn create_module_audio_input(
    &mut self,
    module_key: ModuleKey,
    descriptor: AudioDescriptor,
  ) -> Result<ModuleAudioIn> {
    let module = self.get_module_mut(module_key)?;
    Self::enough_dynamic_input_ports(module, &descriptor)
      .then(|| {
        let port_key = module.ports.audio_input_ports.add(InputPort {
          descriptor,
          source: None,
        });
        ModuleIn(module_key, port_key)
      })
      .ok_or(Error::DynamicPortsNotAvailable)
  }

  /// Add a new dynamic audio output to the module.
  /// It will check the dynamic ports constrains declared in the port descriptor.
  pub fn create_module_audio_output(
    &mut self,
    module_key: ModuleKey,
    descriptor: AudioDescriptor,
  ) -> Result<ModuleAudioOut> {
    let module = self.get_module_mut(module_key)?;
    Self::enough_dynamic_output_ports(module, &descriptor)
      .then(|| {
        let port_key = module.ports.audio_output_ports.add(OutputPort {
          descriptor,
          destinations: Vec::new(),
        });
        ModuleOut(module_key, port_key)
      })
      .ok_or(Error::DynamicPortsNotAvailable)
  }

  /// Add a new dynamic events input to the module.
  /// It will check the dynamic ports constrains declared in the port descriptor.
  pub fn create_module_events_input(
    &mut self,
    module_key: ModuleKey,
    descriptor: EventsDescriptor,
  ) -> Result<ModuleEventsIn> {
    let module = self.get_module_mut(module_key)?;
    Self::enough_dynamic_input_ports(module, &descriptor)
      .then(|| {
        let port_key = module.ports.events_input_ports.add(InputPort {
          descriptor,
          source: None,
        });
        ModuleIn(module_key, port_key)
      })
      .ok_or(Error::DynamicPortsNotAvailable)
  }

  /// Add a new dynamic events output to the module.
  /// It will check the dynamic ports constrains declared in the port descriptor.
  pub fn create_module_events_output(
    &mut self,
    module_key: ModuleKey,
    descriptor: EventsDescriptor,
  ) -> Result<ModuleEventsOut> {
    let module = self.get_module_mut(module_key)?;
    Self::enough_dynamic_output_ports(module, &descriptor)
      .then(|| {
        let port_key = module.ports.events_output_ports.add(OutputPort {
          descriptor,
          destinations: Vec::new(),
        });
        ModuleOut(module_key, port_key)
      })
      .ok_or(Error::DynamicPortsNotAvailable)
  }

  /// Return all the audio inputs in the same order as they were declared and created
  pub fn module_audio_inputs(&self, module_key: ModuleKey) -> Result<Vec<ModuleAudioIn>> {
    let module = self.get_module(module_key)?;
    Ok(
      module
        .ports
        .audio_input_ports
        .keys()
        .map(|port_key| ModuleIn(module_key, *port_key))
        .collect(),
    )
  }

  /// Return all the audio outputs in the same order as they were declared and created
  pub fn module_audio_outputs(&self, module_key: ModuleKey) -> Result<Vec<ModuleAudioOut>> {
    let module = self.get_module(module_key)?;
    Ok(
      module
        .ports
        .audio_output_ports
        .keys()
        .map(|port_key| ModuleOut(module_key, *port_key))
        .collect(),
    )
  }

  /// Return all the events inputs in the same order as they were declared and created
  pub fn module_events_inputs(&self, module_key: ModuleKey) -> Result<Vec<ModuleEventsIn>> {
    let module = self.get_module(module_key)?;
    Ok(
      module
        .ports
        .events_input_ports
        .keys()
        .map(|port_key| ModuleIn(module_key, *port_key))
        .collect(),
    )
  }

  /// Return all the events outputs in the same order as they were declared and created
  pub fn module_events_outputs(&self, module_key: ModuleKey) -> Result<Vec<ModuleEventsOut>> {
    let module = self.get_module(module_key)?;
    Ok(
      module
        .ports
        .events_output_ports
        .keys()
        .map(|port_key| ModuleOut(module_key, *port_key))
        .collect(),
    )
  }

  /// Create a new node in the graph.
  /// It will create all the ports declared in the descriptor as static ports.
  pub fn create_node(
    &mut self,
    parent: ModuleKey,
    name: &str,
    descriptor: NodeDescriptor,
  ) -> Result<NodeKey> {
    if self.modules.contains_key(parent) {
      let node = Node::new(name, descriptor, parent);
      Ok(self.nodes.add(node))
    } else {
      Err(Error::ModuleNotFound(parent))
    }
  }

  /// Remove a node from the graph.
  /// It will remove all the connections.
  pub fn remove_node(&mut self, key: NodeKey) -> Result<()> {
    // TODO remove connections
    self
      .nodes
      .remove(key)
      .map(|_| ())
      .ok_or(Error::NodeNotFound(key))
  }

  /// Return all the audio inputs in the same order as they were declared and created
  pub fn node_audio_inputs(&self, node_key: NodeKey) -> Result<Vec<NodeAudioIn>> {
    let node = self.get_node(node_key)?;
    Ok(
      node
        .ports
        .audio_input_ports
        .keys()
        .map(|port_key| NodeIn(node_key, *port_key))
        .collect(),
    )
  }

  /// Return all the audio outputs in the same order as they were declared and created
  pub fn node_audio_outputs(&self, node_key: NodeKey) -> Result<Vec<NodeAudioOut>> {
    let node = self.get_node(node_key)?;
    Ok(
      node
        .ports
        .audio_output_ports
        .keys()
        .map(|port_key| NodeOut(node_key, *port_key))
        .collect(),
    )
  }

  /// Return all the events inputs in the same order as they were declared and created
  pub fn node_events_inputs(&self, node_key: NodeKey) -> Result<Vec<NodeEventsIn>> {
    let node = self.get_node(node_key)?;
    Ok(
      node
        .ports
        .events_input_ports
        .keys()
        .map(|port_key| NodeIn(node_key, *port_key))
        .collect(),
    )
  }

  /// Return all the events outputs in the same order as they were declared and created
  pub fn node_events_outputs(&self, node_key: NodeKey) -> Result<Vec<NodeEventsOut>> {
    let node = self.get_node(node_key)?;
    Ok(
      node
        .ports
        .events_output_ports
        .keys()
        .map(|port_key| NodeOut(node_key, *port_key))
        .collect(),
    )
  }

  /// Add a new dynamic audio input to the node.
  /// It will check the dynamic ports constrains declared in the port descriptor.
  pub fn create_node_audio_input(
    &mut self,
    node_key: NodeKey,
    descriptor: AudioDescriptor,
  ) -> Result<NodeAudioIn> {
    let node = self.get_node_mut(node_key)?;
    Self::enough_dynamic_input_ports(node, &descriptor)
      .then(|| {
        let port_key = node.ports.audio_input_ports.add(InputPort {
          descriptor,
          source: None,
        });
        NodeIn(node_key, port_key)
      })
      .ok_or(Error::DynamicPortsNotAvailable)
  }

  /// Add a new dynamic audio output to the node.
  /// It will check the dynamic ports constrains declared in the port descriptor.
  pub fn create_node_audio_output(
    &mut self,
    node_key: NodeKey,
    descriptor: AudioDescriptor,
  ) -> Result<NodeAudioOut> {
    let node = self.get_node_mut(node_key)?;
    Self::enough_dynamic_output_ports(node, &descriptor)
      .then(|| {
        let port_key = node.ports.audio_output_ports.add(OutputPort {
          descriptor,
          destinations: Vec::new(),
        });
        NodeOut(node_key, port_key)
      })
      .ok_or(Error::DynamicPortsNotAvailable)
  }

  /// Add a new dynamic events input to the node.
  /// It will check the dynamic ports constrains declared in the port descriptor.
  pub fn create_node_events_input(
    &mut self,
    node_key: NodeKey,
    descriptor: EventsDescriptor,
  ) -> Result<NodeEventsIn> {
    let node = self.get_node_mut(node_key)?;
    Self::enough_dynamic_input_ports(node, &descriptor)
      .then(|| {
        let port_key = node.ports.events_input_ports.add(InputPort {
          descriptor,
          source: None,
        });
        NodeIn(node_key, port_key)
      })
      .ok_or(Error::DynamicPortsNotAvailable)
  }

  /// Add a new dynamic events output to the node.
  /// It will check the dynamic ports constrains declared in the port descriptor.
  pub fn create_node_events_output(
    &mut self,
    node_key: NodeKey,
    descriptor: EventsDescriptor,
  ) -> Result<NodeEventsOut> {
    let node = self.get_node_mut(node_key)?;
    Self::enough_dynamic_output_ports(node, &descriptor)
      .then(|| {
        let port_key = node.ports.events_output_ports.add(OutputPort {
          descriptor,
          destinations: Vec::new(),
        });
        NodeOut(node_key, port_key)
      })
      .ok_or(Error::DynamicPortsNotAvailable)
  }

  pub fn connect_audio(&mut self, connection: AudioConnection) -> Result<()> {
    todo!()
  }

  pub fn connect_events(&mut self, connection: AudioConnection) -> Result<()> {
    todo!()
  }

  pub(crate) fn get_module(&self, key: ModuleKey) -> Result<&Module> {
    self.modules.get(key).ok_or(Error::ModuleNotFound(key))
  }

  pub(crate) fn get_module_mut(&mut self, key: ModuleKey) -> Result<&mut Module> {
    self.modules.get_mut(key).ok_or(Error::ModuleNotFound(key))
  }

  pub(crate) fn get_node(&self, key: NodeKey) -> Result<&Node> {
    self.nodes.get(key).ok_or(Error::NodeNotFound(key))
  }

  pub(crate) fn get_node_mut(&mut self, key: NodeKey) -> Result<&mut Node> {
    self.nodes.get_mut(key).ok_or(Error::NodeNotFound(key))
  }

  fn enough_dynamic_input_ports<P, D>(entity: &mut P, descriptor: &D) -> bool
  where
    P: HasPorts,
    D: PortDescriptor,
  {
    let dynamic_port = match descriptor.port_type() {
      PortType::Audio => &entity.get_audio_descriptor_ports().dynamic_inputs,
      PortType::Events => &entity.get_events_descriptor_ports().dynamic_inputs,
    };

    match dynamic_port {
      DynamicPorts::None => false,
      DynamicPorts::Limited(limit) => {
        let (static_len, current_len) = match descriptor.port_type() {
          PortType::Audio => (
            entity.get_audio_descriptor_ports().static_inputs.len(),
            entity.get_ports().audio_input_ports.len(),
          ),
          PortType::Events => (
            entity.get_events_descriptor_ports().static_inputs.len(),
            entity.get_ports().events_input_ports.len(),
          ),
        };
        if current_len < static_len {
          true
        } else {
          current_len - static_len < *limit
        }
      }
      DynamicPorts::Unlimited => true,
    }
  }

  fn enough_dynamic_output_ports<P, D>(entity: &mut P, descriptor: &D) -> bool
  where
    P: HasPorts,
    D: PortDescriptor,
  {
    let dynamic_port = match descriptor.port_type() {
      PortType::Audio => &entity.get_audio_descriptor_ports().dynamic_outputs,
      PortType::Events => &entity.get_events_descriptor_ports().dynamic_outputs,
    };

    match dynamic_port {
      DynamicPorts::None => false,
      DynamicPorts::Limited(limit) => {
        let (static_len, current_len) = match descriptor.port_type() {
          PortType::Audio => (
            entity.get_audio_descriptor_ports().static_outputs.len(),
            entity.get_ports().audio_output_ports.len(),
          ),
          PortType::Events => (
            entity.get_events_descriptor_ports().static_outputs.len(),
            entity.get_ports().events_output_ports.len(),
          ),
        };
        if current_len < static_len {
          true
        } else {
          current_len - static_len < *limit
        }
      }
      DynamicPorts::Unlimited => true,
    }
  }
}

pub type AudioConnection = Connection<AudioDescriptor>;
pub type EventsConnection = Connection<EventsDescriptor>;

pub enum Connection<D> {
  ModuleOutBindModuleOut(ModuleOut<D>, ModuleOut<D>),
  ModuleOutToModuleIn(ModuleOut<D>, ModuleIn<D>),
  ModuleOutToNodeIn(ModuleOut<D>, NodeIn<D>),
  ModuleInBindModuleIn(ModuleIn<D>, ModuleIn<D>),
  ModuleInBindNodeIn(ModuleIn<D>, NodeIn<D>),
  NodeOutBindModuleOut(NodeOut<D>, ModuleOut<D>),
  NodeOutToModuleIn(NodeOut<D>, ModuleIn<D>),
  NodeOutToNodeIn(NodeOut<D>, NodeIn<D>),
}

pub type ModuleAudioIn = ModuleIn<AudioDescriptor>;
pub type ModuleAudioOut = ModuleOut<AudioDescriptor>;
pub type NodeAudioIn = NodeIn<AudioDescriptor>;
pub type NodeAudioOut = NodeOut<AudioDescriptor>;

pub type ModuleEventsIn = ModuleIn<EventsDescriptor>;
pub type ModuleEventsOut = ModuleOut<EventsDescriptor>;
pub type NodeEventsIn = NodeIn<EventsDescriptor>;
pub type NodeEventsOut = NodeOut<EventsDescriptor>;

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleIn<D>(ModuleKey, InputPortKey<D>);

impl<D> Copy for ModuleIn<D> where D: PortDescriptor {}

impl<D> ModuleIn<D>
where
  D: PortDescriptor,
{
  pub fn bind<B>(self, other: B) -> Connection<D>
  where
    B: Into<ModuleInBind<D>>,
  {
    match other.into() {
      ModuleInBind::ModuleIn(module_in) => Connection::ModuleInBindModuleIn(self, module_in),
      ModuleInBind::NodeIn(node_in) => Connection::ModuleInBindNodeIn(self, node_in),
    }
  }

  pub fn from<S>(self, other: S) -> Connection<D>
  where
    S: Into<ModuleInFrom<D>>,
  {
    match other.into() {
      ModuleInFrom::ModuleOut(module_out) => Connection::ModuleOutToModuleIn(module_out, self),
      ModuleInFrom::NodeOut(node_out) => Connection::NodeOutToModuleIn(node_out, self),
    }
  }
}

pub enum ModuleInBind<D> {
  ModuleIn(ModuleIn<D>),
  NodeIn(NodeIn<D>),
}

impl<D> From<ModuleIn<D>> for ModuleInBind<D> {
  fn from(value: ModuleIn<D>) -> Self {
    ModuleInBind::ModuleIn(value)
  }
}

impl<D> From<NodeIn<D>> for ModuleInBind<D> {
  fn from(value: NodeIn<D>) -> Self {
    ModuleInBind::NodeIn(value)
  }
}

pub enum ModuleInFrom<D> {
  ModuleOut(ModuleOut<D>),
  NodeOut(NodeOut<D>),
}

impl<D> From<ModuleOut<D>> for ModuleInFrom<D> {
  fn from(value: ModuleOut<D>) -> Self {
    ModuleInFrom::ModuleOut(value)
  }
}

impl<D> From<NodeOut<D>> for ModuleInFrom<D> {
  fn from(value: NodeOut<D>) -> Self {
    ModuleInFrom::NodeOut(value)
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleOut<D>(ModuleKey, OutputPortKey<D>);

impl<D> Copy for ModuleOut<D> where D: PortDescriptor {}

impl<D> ModuleOut<D>
where
  D: PortDescriptor,
{
  pub fn bind(self, other: ModuleOut<D>) -> Connection<D> {
    Connection::ModuleOutBindModuleOut(self, other)
  }

  pub fn to<C>(self, other: C) -> Connection<D>
  where
    C: Into<ModuleOutTo<D>>,
  {
    match other.into() {
      ModuleOutTo::ModuleIn(module_in) => Connection::ModuleOutToModuleIn(self, module_in),
      ModuleOutTo::NodeIn(node_in) => Connection::ModuleOutToNodeIn(self, node_in),
    }
  }

  pub fn bind_from<B>(self, other: B) -> Connection<D>
  where
    B: Into<ModuleOutBindFrom<D>>,
  {
    match other.into() {
      ModuleOutBindFrom::ModuleOut(module_out) => {
        Connection::ModuleOutBindModuleOut(module_out, self)
      }
      ModuleOutBindFrom::NodeOut(node_out) => Connection::NodeOutBindModuleOut(node_out, self),
    }
  }
}

pub enum ModuleOutTo<D> {
  ModuleIn(ModuleIn<D>),
  NodeIn(NodeIn<D>),
}

impl<D> From<ModuleIn<D>> for ModuleOutTo<D> {
  fn from(value: ModuleIn<D>) -> Self {
    ModuleOutTo::ModuleIn(value)
  }
}

impl<D> From<NodeIn<D>> for ModuleOutTo<D> {
  fn from(value: NodeIn<D>) -> Self {
    ModuleOutTo::NodeIn(value)
  }
}

pub enum ModuleOutBindFrom<D> {
  ModuleOut(ModuleOut<D>),
  NodeOut(NodeOut<D>),
}

impl<D> From<ModuleOut<D>> for ModuleOutBindFrom<D> {
  fn from(value: ModuleOut<D>) -> Self {
    ModuleOutBindFrom::ModuleOut(value)
  }
}

impl<D> From<NodeOut<D>> for ModuleOutBindFrom<D> {
  fn from(value: NodeOut<D>) -> Self {
    ModuleOutBindFrom::NodeOut(value)
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeIn<D>(NodeKey, InputPortKey<D>);

impl<D> Copy for NodeIn<D> where D: PortDescriptor {}

impl<D> NodeIn<D>
where
  D: PortDescriptor,
{
  pub fn bind_from(self, other: ModuleIn<D>) -> Connection<D> {
    Connection::ModuleInBindNodeIn(other, self)
  }

  pub fn from<C>(self, other: C) -> Connection<D>
  where
    C: Into<NodeInFrom<D>>,
  {
    match other.into() {
      NodeInFrom::ModuleOut(module_out) => Connection::ModuleOutToNodeIn(module_out, self),
      NodeInFrom::NodeOut(node_out) => Connection::NodeOutToNodeIn(node_out, self),
    }
  }
}

pub enum NodeInFrom<D> {
  ModuleOut(ModuleOut<D>),
  NodeOut(NodeOut<D>),
}

impl<D> From<ModuleOut<D>> for NodeInFrom<D> {
  fn from(value: ModuleOut<D>) -> Self {
    NodeInFrom::ModuleOut(value)
  }
}

impl<D> From<NodeOut<D>> for NodeInFrom<D> {
  fn from(value: NodeOut<D>) -> Self {
    NodeInFrom::NodeOut(value)
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeOut<D>(NodeKey, OutputPortKey<D>);

impl<D> Copy for NodeOut<D> where D: PortDescriptor {}

impl<D> NodeOut<D>
where
  D: PortDescriptor,
{
  pub fn bind(self, other: ModuleOut<D>) -> Connection<D> {
    Connection::NodeOutBindModuleOut(self, other)
  }

  pub fn to<C>(self, other: C) -> Connection<D>
  where
    C: Into<NodeOutTo<D>>,
  {
    match other.into() {
      NodeOutTo::ModuleIn(module_in) => Connection::NodeOutToModuleIn(self, module_in),
      NodeOutTo::NodeIn(node_in) => Connection::NodeOutToNodeIn(self, node_in),
    }
  }
}

pub enum NodeOutTo<D> {
  ModuleIn(ModuleIn<D>),
  NodeIn(NodeIn<D>),
}

impl<D> From<ModuleIn<D>> for NodeOutTo<D> {
  fn from(value: ModuleIn<D>) -> Self {
    NodeOutTo::ModuleIn(value)
  }
}

impl<D> From<NodeIn<D>> for NodeOutTo<D> {
  fn from(value: NodeIn<D>) -> Self {
    NodeOutTo::NodeIn(value)
  }
}

#[cfg(test)]
mod tests {
  use crate::graph::inner::InnerGraph;
  use crate::graph::module::ModuleDescriptor;
  use crate::graph::node::NodeDescriptor;
  use crate::graph::port::AudioDescriptor;

  #[test]
  fn bind() {
    let mut g = InnerGraph::new();

    let test_module_descriptor = ModuleDescriptor::new().with_audio_ports(|ports| {
      ports
        .static_inputs(vec![AudioDescriptor::new("audio-in", 1)])
        .static_outputs(vec![AudioDescriptor::new("audio-out", 1)])
    });

    let m1 = g
      .create_module(g.get_root_module(), "m1", test_module_descriptor.clone())
      .unwrap();

    let m1_audio_in = g.module_audio_inputs(m1).unwrap();
    let m1_audio_out = g.module_audio_outputs(m1).unwrap();

    let m2 = g.create_module(m1, "m2", test_module_descriptor).unwrap();

    let m2_audio_in = g.module_audio_inputs(m2).unwrap();
    let m2_audio_out = g.module_audio_outputs(m2).unwrap();

    let test_node_descriptor = NodeDescriptor::new("test").with_audio_ports(|ports| {
      ports
        .static_inputs(vec![AudioDescriptor::new("audio-in", 1)])
        .static_outputs(vec![AudioDescriptor::new("audio-out", 1)])
    });

    let n1 = g
      .create_node(m2, "n1", test_node_descriptor.clone())
      .unwrap();

    let n1_audio_in = g.node_audio_inputs(n1).unwrap();
    let n1_audio_out = g.node_audio_outputs(n1).unwrap();

    let n2 = g.create_node(m2, "n2", test_node_descriptor).unwrap();

    let n2_audio_in = g.node_audio_inputs(n2).unwrap();
    let n2_audio_out = g.node_audio_outputs(n2).unwrap();

    g.connect_audio(m1_audio_in[0].bind(m2_audio_in[0]))
      .unwrap();

    g.connect_audio(m2_audio_in[0].bind(n1_audio_in[0]))
      .unwrap();

    g.connect_audio(n1_audio_out[0].to(n2_audio_in[0])).unwrap();

    g.connect_audio(n2_audio_out[0].bind(m2_audio_out[0]))
      .unwrap();

    g.connect_audio(m2_audio_out[0].bind(m2_audio_out[0]))
      .unwrap();
  }
}
