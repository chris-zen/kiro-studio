use thiserror::Error;

use crate::graph::module::{Module, ModuleDescriptor, ModuleKey};
use crate::graph::node::{Node, NodeDescriptor, NodeKey};
use crate::graph::port::{
  AudioDescriptor, AudioInputPortKey, AudioOutputPortKey, DynamicPorts, EventsDescriptor,
  EventsInputPortKey, EventsOutputPortKey, HasPorts, OutputPort, PortType,
};
use crate::graph::port::{DestinationConnection, InputPort, PortDescriptor, SourceConnection};
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
  pub fn create_module_audio_input_port(
    &mut self,
    key: ModuleKey,
    descriptor: AudioDescriptor,
  ) -> Result<AudioInputPortKey> {
    let module = self.get_module_mut(key)?;
    Self::enough_dynamic_input_ports(module, &descriptor)
      .then(|| {
        module.ports.audio_input_ports.add(InputPort {
          descriptor,
          source: None,
        })
      })
      .ok_or(Error::DynamicPortsNotAvailable)
  }

  /// Add a new dynamic audio output to the module.
  /// It will check the dynamic ports constrains declared in the port descriptor.
  pub fn create_module_audio_output_port(
    &mut self,
    key: ModuleKey,
    descriptor: AudioDescriptor,
  ) -> Result<AudioOutputPortKey> {
    let module = self.get_module_mut(key)?;
    Self::enough_dynamic_output_ports(module, &descriptor)
      .then(|| {
        module.ports.audio_output_ports.add(OutputPort {
          descriptor,
          destinations: Vec::new(),
        })
      })
      .ok_or(Error::DynamicPortsNotAvailable)
  }

  /// Add a new dynamic events input to the module.
  /// It will check the dynamic ports constrains declared in the port descriptor.
  pub fn create_module_events_input_port(
    &mut self,
    key: ModuleKey,
    descriptor: EventsDescriptor,
  ) -> Result<EventsInputPortKey> {
    let module = self.get_module_mut(key)?;
    Self::enough_dynamic_input_ports(module, &descriptor)
      .then(|| {
        module.ports.events_input_ports.add(InputPort {
          descriptor,
          source: None,
        })
      })
      .ok_or(Error::DynamicPortsNotAvailable)
  }

  /// Add a new dynamic events output to the module.
  /// It will check the dynamic ports constrains declared in the port descriptor.
  pub fn create_module_events_output_port(
    &mut self,
    key: ModuleKey,
    descriptor: EventsDescriptor,
  ) -> Result<EventsOutputPortKey> {
    let module = self.get_module_mut(key)?;
    Self::enough_dynamic_output_ports(module, &descriptor)
      .then(|| {
        module.ports.events_output_ports.add(OutputPort {
          descriptor,
          destinations: Vec::new(),
        })
      })
      .ok_or(Error::DynamicPortsNotAvailable)
  }

  /// Return all the audio input port keys in the same order as they were declared and created
  pub fn module_audio_input_ports(&self, key: ModuleKey) -> Result<Vec<AudioInputPortKey>> {
    let module = self.get_module(key)?;
    Ok(module.ports.audio_input_ports.keys().copied().collect())
  }

  /// Return all the audio output port keys in the same order as they were declared and created
  pub fn module_audio_output_ports(&self, key: ModuleKey) -> Result<Vec<AudioOutputPortKey>> {
    let module = self.get_module(key)?;
    Ok(module.ports.audio_output_ports.keys().copied().collect())
  }

  /// Return all the events input port keys in the same order as they were declared and created
  pub fn module_events_input_ports(&self, key: ModuleKey) -> Result<Vec<EventsInputPortKey>> {
    let module = self.get_module(key)?;
    Ok(module.ports.events_input_ports.keys().copied().collect())
  }

  /// Return all the events output port keys in the same order as they were declared and created
  pub fn module_events_output_ports(&self, key: ModuleKey) -> Result<Vec<EventsOutputPortKey>> {
    let module = self.get_module(key)?;
    Ok(module.ports.events_output_ports.keys().copied().collect())
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

  /// Return all the audio input port keys in the same order as they were declared and created
  pub fn node_audio_input_ports(&self, key: NodeKey) -> Result<Vec<AudioInputPortKey>> {
    let node = self.get_node(key)?;
    Ok(node.ports.audio_input_ports.keys().copied().collect())
  }

  /// Return all the audio output port keys in the same order as they were declared and created
  pub fn node_audio_output_ports(&self, key: NodeKey) -> Result<Vec<AudioOutputPortKey>> {
    let node = self.get_node(key)?;
    Ok(node.ports.audio_output_ports.keys().copied().collect())
  }

  /// Return all the events input port keys in the same order as they were declared and created
  pub fn node_events_input_ports(&self, key: NodeKey) -> Result<Vec<EventsInputPortKey>> {
    let node = self.get_node(key)?;
    Ok(node.ports.events_input_ports.keys().copied().collect())
  }

  /// Return all the events output port keys in the same order as they were declared and created
  pub fn node_events_output_ports(&self, key: NodeKey) -> Result<Vec<EventsOutputPortKey>> {
    let node = self.get_node(key)?;
    Ok(node.ports.events_output_ports.keys().copied().collect())
  }

  /// Add a new dynamic audio input to the node.
  /// It will check the dynamic ports constrains declared in the port descriptor.
  pub fn create_node_audio_input_port(
    &mut self,
    key: NodeKey,
    descriptor: AudioDescriptor,
  ) -> Result<AudioInputPortKey> {
    let node = self.get_node_mut(key)?;
    Self::enough_dynamic_input_ports(node, &descriptor)
      .then(|| {
        node.ports.audio_input_ports.add(InputPort {
          descriptor,
          source: None,
        })
      })
      .ok_or(Error::DynamicPortsNotAvailable)
  }

  /// Add a new dynamic audio output to the node.
  /// It will check the dynamic ports constrains declared in the port descriptor.
  pub fn create_node_audio_output_port<D>(
    &mut self,
    key: NodeKey,
    descriptor: AudioDescriptor,
  ) -> Result<AudioOutputPortKey> {
    let node = self.get_node_mut(key)?;
    Self::enough_dynamic_output_ports(node, &descriptor)
      .then(|| {
        node.ports.audio_output_ports.add(OutputPort {
          descriptor,
          destinations: Vec::new(),
        })
      })
      .ok_or(Error::DynamicPortsNotAvailable)
  }

  /// Add a new dynamic events input to the node.
  /// It will check the dynamic ports constrains declared in the port descriptor.
  pub fn create_node_events_input_port(
    &mut self,
    key: NodeKey,
    descriptor: EventsDescriptor,
  ) -> Result<EventsInputPortKey> {
    let node = self.get_node_mut(key)?;
    Self::enough_dynamic_input_ports(node, &descriptor)
      .then(|| {
        node.ports.events_input_ports.add(InputPort {
          descriptor,
          source: None,
        })
      })
      .ok_or(Error::DynamicPortsNotAvailable)
  }

  /// Add a new dynamic events output to the node.
  /// It will check the dynamic ports constrains declared in the port descriptor.
  pub fn create_node_events_output_port(
    &mut self,
    key: NodeKey,
    descriptor: EventsDescriptor,
  ) -> Result<EventsOutputPortKey> {
    let node = self.get_node_mut(key)?;
    Self::enough_dynamic_output_ports(node, &descriptor)
      .then(|| {
        node.ports.events_output_ports.add(OutputPort {
          descriptor,
          destinations: Vec::new(),
        })
      })
      .ok_or(Error::DynamicPortsNotAvailable)
  }

  pub fn bind_audio_ports() -> Result<()> {
    todo!()
  }

  pub fn connect_audio_ports<S, D>(&mut self, source: S, destination: D) -> Result<()>
  where
    S: Into<SourceConnection<AudioDescriptor>>,
    D: Into<DestinationConnection<AudioDescriptor>>,
  {
    // let source = source.into();
    // let destination = destination.into();

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
