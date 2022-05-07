#![allow(clippy::wrong_self_convention)]

use crate::graph::connection::{
  AudioConnection, EventsConnection, ModuleAudioIn, ModuleAudioOut, ModuleEventsIn,
  ModuleEventsOut, ModuleIn, ModuleOut, NodeAudioIn, NodeAudioOut, NodeEventsIn, NodeEventsOut,
  NodeIn, NodeOut,
};
use crate::graph::error::{Error, Result};
use crate::graph::module::{Module, ModuleDescriptor, ModuleKey};
use crate::graph::node::{Node, NodeDescriptor, NodeKey};
use crate::graph::port::{
  port_path, AudioDescriptor, AudioInputPortKey, AudioOutputPortKey, DescriptorPortAccessor,
  DescriptorPorts, DynamicPorts, EventsDescriptor, EventsInputPortKey, EventsOutputPortKey,
  InputSource, NodeLike, OutputPort, OutputSource, PortAccessor, Ports,
};
use crate::graph::port::{InputPort, PortDescriptor};
use crate::key_store::KeyStore;

pub struct InnerGraph {
  root_module: ModuleKey,
  modules: KeyStore<Module>,
  nodes: KeyStore<Node>,
}

impl InnerGraph {
  /// Create a new inner graph with a root module
  pub fn new() -> Self {
    let mut modules = KeyStore::new();
    let root_module = modules.add(Module::new(
      "root",
      ModuleDescriptor::new(),
      None,
      String::new(),
    ));
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
    parent_key: ModuleKey,
    name: &str,
    descriptor: ModuleDescriptor,
  ) -> Result<ModuleKey> {
    if self.modules.contains_key(parent_key) {
      let parent = self.get_module(parent_key)?;
      let path = format!("{}/{}", parent.path, parent.name);
      let module = Module::new(name, descriptor, Some(parent_key), path);
      Ok(self.modules.add(module))
    } else {
      Err(Error::ModuleNotFound(parent_key))
    }
  }

  /// Remove a module from the graph.
  /// It will remove all the children modules, nodes and connections recursively.
  pub fn remove_module(&mut self, module_key: ModuleKey) -> Result<()> {
    // TODO remove recursively
    let module_nodes = self
      .nodes
      .iter()
      .filter_map(|(node_key, node)| (node.parent == module_key).then(|| node_key))
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
    Self::enough_dynamic_input_ports::<_, AudioDescriptor>(module)
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
    Self::enough_dynamic_output_ports::<_, AudioDescriptor>(module)
      .then(|| {
        let port_key = module.ports.audio_output_ports.add(OutputPort {
          descriptor,
          source: None,
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
    Self::enough_dynamic_input_ports::<_, EventsDescriptor>(module)
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
    Self::enough_dynamic_output_ports::<_, EventsDescriptor>(module)
      .then(|| {
        let port_key = module.ports.events_output_ports.add(OutputPort {
          descriptor,
          source: None,
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

  /// Return the module audio input key from the port name
  pub fn module_audio_input(&self, module_key: ModuleKey, name: &str) -> Result<AudioInputPortKey> {
    let module = self.get_module(module_key)?;
    module
      .ports
      .audio_input_ports
      .key_from_id(name)
      .ok_or_else(|| Error::PortNotFound(port_path(module, &name)))
  }

  /// Return the module audio output key from the port name
  pub fn module_audio_output(
    &self,
    module_key: ModuleKey,
    name: &str,
  ) -> Result<AudioOutputPortKey> {
    let module = self.get_module(module_key)?;
    module
      .ports
      .audio_output_ports
      .key_from_id(name)
      .ok_or_else(|| Error::PortNotFound(port_path(module, &name)))
  }

  /// Return the module events input key from the port name
  pub fn module_events_input(
    &self,
    module_key: ModuleKey,
    name: &str,
  ) -> Result<EventsInputPortKey> {
    let module = self.get_module(module_key)?;
    module
      .ports
      .events_input_ports
      .key_from_id(name)
      .ok_or_else(|| Error::PortNotFound(port_path(module, &name)))
  }

  /// Return the module events output key from the port name
  pub fn module_events_output(
    &self,
    module_key: ModuleKey,
    name: &str,
  ) -> Result<EventsOutputPortKey> {
    let module = self.get_module(module_key)?;
    module
      .ports
      .events_output_ports
      .key_from_id(name)
      .ok_or_else(|| Error::PortNotFound(port_path(module, &name)))
  }

  /// Create a new node in the graph.
  /// It will create all the ports declared in the descriptor as static ports.
  pub fn create_node(
    &mut self,
    parent_key: ModuleKey,
    name: &str,
    descriptor: NodeDescriptor,
  ) -> Result<NodeKey> {
    if self.modules.contains_key(parent_key) {
      let parent = self.get_module(parent_key)?;
      let path = format!("{}/{}", parent.path, parent.name);
      let node = Node::new(name, descriptor, parent_key, path);
      Ok(self.nodes.add(node))
    } else {
      Err(Error::ModuleNotFound(parent_key))
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
    Self::enough_dynamic_input_ports::<_, AudioDescriptor>(node)
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
    Self::enough_dynamic_output_ports::<_, AudioDescriptor>(node)
      .then(|| {
        let port_key = node.ports.audio_output_ports.add(OutputPort {
          descriptor,
          source: None,
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
    Self::enough_dynamic_input_ports::<_, EventsDescriptor>(node)
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
    Self::enough_dynamic_output_ports::<_, EventsDescriptor>(node)
      .then(|| {
        let port_key = node.ports.events_output_ports.add(OutputPort {
          descriptor,
          source: None,
        });
        NodeOut(node_key, port_key)
      })
      .ok_or(Error::DynamicPortsNotAvailable)
  }

  /// Return the node audio input key from the port name
  pub fn node_audio_input(&self, node_key: NodeKey, name: &str) -> Result<AudioInputPortKey> {
    let node = self.get_node(node_key)?;
    node
      .ports
      .audio_input_ports
      .key_from_id(name)
      .ok_or_else(|| Error::PortNotFound(port_path(node, &name)))
  }

  /// Return the node audio output key from the port name
  pub fn node_audio_output(&self, node_key: NodeKey, name: &str) -> Result<AudioOutputPortKey> {
    let node = self.get_node(node_key)?;
    node
      .ports
      .audio_output_ports
      .key_from_id(name)
      .ok_or_else(|| Error::PortNotFound(port_path(node, &name)))
  }

  /// Return the node events input key from the port name
  pub fn node_events_input(&self, node_key: NodeKey, name: &str) -> Result<EventsInputPortKey> {
    let node = self.get_node(node_key)?;
    node
      .ports
      .events_input_ports
      .key_from_id(name)
      .ok_or_else(|| Error::PortNotFound(port_path(node, &name)))
  }

  /// Return the node events output key from the port name
  pub fn node_events_output(&self, node_key: NodeKey, name: &str) -> Result<EventsOutputPortKey> {
    let node = self.get_node(node_key)?;
    node
      .ports
      .events_output_ports
      .key_from_id(name)
      .ok_or_else(|| Error::PortNotFound(port_path(node, &name)))
  }

  pub fn connect_audio(&mut self, connection: AudioConnection) -> Result<()> {
    match connection {
      AudioConnection::ModuleOutBindModuleOut(src_module_out, dst_module_out) => {
        self.connect_module_out_bind_module_out(src_module_out, dst_module_out)
      }
      AudioConnection::NodeOutBindModuleOut(src_node_out, dst_module_out) => {
        self.connect_node_out_bind_module_out(src_node_out, dst_module_out)
      }
      AudioConnection::ModuleInBindModuleIn(src_module_in, dst_module_in) => {
        self.connect_module_in_bind_module_in(src_module_in, dst_module_in)
      }
      AudioConnection::ModuleInBindNodeIn(src_module_in, dst_node_in) => {
        self.connect_module_in_bind_node_in(src_module_in, dst_node_in)
      }
      AudioConnection::ModuleOutToNodeIn(src_module_out, dst_node_in) => {
        self.connect_module_out_to_node_in(src_module_out, dst_node_in)
      }
      AudioConnection::ModuleOutToModuleIn(src_module_out, dst_module_in) => {
        self.connect_module_out_to_module_in(src_module_out, dst_module_in)
      }
      AudioConnection::NodeOutToNodeIn(src_node_out, dst_node_in) => {
        self.connect_node_out_to_node_in(src_node_out, dst_node_in)
      }
      AudioConnection::NodeOutToModuleIn(src_node_out, dst_module_in) => {
        self.connect_node_out_to_module_in(src_node_out, dst_module_in)
      }
    }
  }

  pub fn connect_events(&mut self, connection: EventsConnection) -> Result<()> {
    match connection {
      EventsConnection::ModuleOutBindModuleOut(src_module_out, dst_module_out) => {
        self.connect_module_out_bind_module_out(src_module_out, dst_module_out)
      }
      EventsConnection::NodeOutBindModuleOut(src_node_out, dst_module_out) => {
        self.connect_node_out_bind_module_out(src_node_out, dst_module_out)
      }
      EventsConnection::ModuleInBindModuleIn(src_module_in, dst_module_in) => {
        self.connect_module_in_bind_module_in(src_module_in, dst_module_in)
      }
      EventsConnection::ModuleInBindNodeIn(src_module_in, dst_node_in) => {
        self.connect_module_in_bind_node_in(src_module_in, dst_node_in)
      }
      EventsConnection::ModuleOutToNodeIn(src_module_out, dst_node_in) => {
        self.connect_module_out_to_node_in(src_module_out, dst_node_in)
      }
      EventsConnection::ModuleOutToModuleIn(src_module_out, dst_module_in) => {
        self.connect_module_out_to_module_in(src_module_out, dst_module_in)
      }
      EventsConnection::NodeOutToNodeIn(src_node_out, dst_node_in) => {
        self.connect_node_out_to_node_in(src_node_out, dst_node_in)
      }
      EventsConnection::NodeOutToModuleIn(src_node_out, dst_module_in) => {
        self.connect_node_out_to_module_in(src_node_out, dst_module_in)
      }
    }
  }

  fn connect_module_out_bind_module_out<D>(
    &mut self,
    src_module_out: ModuleOut<D>,
    dst_module_out: ModuleOut<D>,
  ) -> Result<()>
  where
    D: PortDescriptor,
    Ports: PortAccessor<D>,
  {
    let src_module = self.get_module(src_module_out.module_key())?;
    let src_port = src_module.get_output_port(src_module_out.output_port_key())?;

    let dst_module = self.get_module(dst_module_out.module_key())?;
    let dst_port = dst_module.get_output_port(dst_module_out.output_port_key())?;

    if src_module.parent != Some(dst_module_out.module_key()) {
      if dst_module.parent == Some(src_module_out.module_key()) {
        self.connect_module_out_bind_module_out(dst_module_out, src_module_out)
      } else {
        let src_path = port_path(src_module, src_port);
        let dst_path = port_path(dst_module, dst_port);
        Err(Error::BindingOutOfScope(src_path, dst_path))?
      }
    } else if dst_port.source.is_some() {
      let dst_path = port_path(dst_module, dst_port);
      Err(Error::OutputSourceAlreadyDefined(
        D::port_type().name().to_string(),
        dst_path,
      ))
    } else {
      let dst_module = self.get_module_mut(dst_module_out.module_key())?;
      let dst_port = dst_module.get_output_port_mut(dst_module_out.output_port_key())?;
      dst_port.source = Some(OutputSource::ModuleBinding(src_module_out));
      Ok(())
    }
  }

  fn connect_node_out_bind_module_out<D>(
    &mut self,
    src_node_out: NodeOut<D>,
    dst_module_out: ModuleOut<D>,
  ) -> Result<()>
  where
    D: PortDescriptor,
    Ports: PortAccessor<D>,
  {
    let src_node = self.get_node(src_node_out.node_key())?;
    let src_port = src_node.get_output_port(src_node_out.output_port_key())?;

    let dst_module = self.get_module(dst_module_out.module_key())?;
    let dst_port = dst_module.get_output_port(dst_module_out.output_port_key())?;

    if src_node.parent != dst_module_out.module_key() {
      let src_path = port_path(src_node, src_port);
      let dst_path = port_path(dst_module, dst_port);
      Err(Error::BindingOutOfScope(src_path, dst_path))
    } else if dst_port.source.is_some() {
      let dst_path = port_path(dst_module, dst_port);
      Err(Error::OutputSourceAlreadyDefined(
        D::port_type().name().to_string(),
        dst_path,
      ))
    } else {
      let dst_module = self.get_module_mut(dst_module_out.module_key())?;
      let dst_port = dst_module.get_output_port_mut(dst_module_out.output_port_key())?;
      dst_port.source = Some(OutputSource::NodeBinding(src_node_out));
      Ok(())
    }
  }

  fn connect_module_in_bind_module_in<D>(
    &mut self,
    src_module_in: ModuleIn<D>,
    dst_module_in: ModuleIn<D>,
  ) -> Result<()>
  where
    D: PortDescriptor,
    Ports: PortAccessor<D>,
  {
    let src_module = self.get_module(src_module_in.module_key())?;
    let src_port = src_module.get_input_port(src_module_in.input_port_key())?;

    let dst_module = self.get_module(dst_module_in.module_key())?;
    let dst_port = dst_module.get_input_port(dst_module_in.input_port_key())?;

    if dst_module.parent != Some(src_module_in.module_key()) {
      if src_module.parent == Some(dst_module_in.module_key()) {
        self.connect_module_in_bind_module_in(dst_module_in, src_module_in)
      } else {
        let src_path = port_path(src_module, src_port);
        let dst_path = port_path(dst_module, dst_port);
        Err(Error::BindingOutOfScope(src_path, dst_path))
      }
    } else if dst_port.source.is_some() {
      let dst_path = port_path(dst_module, dst_port);
      Err(Error::InputSourceAlreadyDefined(
        D::port_type().name().to_string(),
        dst_path,
      ))
    } else {
      let dst_module = self.get_module_mut(dst_module_in.module_key())?;
      let dst_port = dst_module.get_input_port_mut(dst_module_in.input_port_key())?;
      dst_port.source = Some(InputSource::ModuleBinding(src_module_in));
      Ok(())
    }
  }

  fn connect_module_in_bind_node_in<D>(
    &mut self,
    src_module_in: ModuleIn<D>,
    dst_node_in: NodeIn<D>,
  ) -> Result<()>
  where
    D: PortDescriptor,
    Ports: PortAccessor<D>,
  {
    let src_module = self.get_module(src_module_in.module_key())?;
    let src_port = src_module.get_input_port(src_module_in.input_port_key())?;

    let dst_node = self.get_node(dst_node_in.node_key())?;
    let dst_port = dst_node.get_input_port(dst_node_in.input_port_key())?;

    if dst_node.parent != src_module_in.module_key() {
      let src_path = port_path(src_module, src_port);
      let dst_path = port_path(dst_node, dst_port);
      Err(Error::BindingOutOfScope(src_path, dst_path))
    } else if dst_port.source.is_some() {
      let dst_path = port_path(dst_node, dst_port);
      Err(Error::InputSourceAlreadyDefined(
        D::port_type().name().to_string(),
        dst_path,
      ))
    } else {
      let dst_node = self.get_node_mut(dst_node_in.node_key())?;
      let dst_port = dst_node.get_input_port_mut(dst_node_in.input_port_key())?;
      dst_port.source = Some(InputSource::ModuleBinding(src_module_in));
      Ok(())
    }
  }

  fn connect_module_out_to_node_in<D>(
    &mut self,
    src_module_out: ModuleOut<D>,
    dst_node_in: NodeIn<D>,
  ) -> Result<()>
  where
    D: PortDescriptor,
    Ports: PortAccessor<D>,
  {
    let src_module = self.get_module(src_module_out.module_key())?;
    let src_port = src_module.get_output_port(src_module_out.output_port_key())?;

    let dst_node = self.get_node(dst_node_in.node_key())?;
    let dst_port = dst_node.get_input_port(dst_node_in.input_port_key())?;

    if src_module.parent != Some(dst_node.parent) {
      let src_path = port_path(src_module, src_port);
      let dst_path = port_path(dst_node, dst_port);
      Err(Error::ConnectionOutOfScope(src_path, dst_path))
    } else if dst_port.source.is_some() {
      let dst_path = port_path(dst_node, dst_port);
      Err(Error::InputSourceAlreadyDefined(
        D::port_type().name().to_string(),
        dst_path,
      ))
    } else {
      let dst_node = self.get_node_mut(dst_node_in.node_key())?;
      let dst_port = dst_node.get_input_port_mut(dst_node_in.input_port_key())?;
      dst_port.source = Some(InputSource::ModuleConnection(src_module_out));
      Ok(())
    }
  }

  fn connect_module_out_to_module_in<D>(
    &mut self,
    src_module_out: ModuleOut<D>,
    dst_module_in: ModuleIn<D>,
  ) -> Result<()>
  where
    D: PortDescriptor,
    Ports: PortAccessor<D>,
  {
    let src_module = self.get_module(src_module_out.module_key())?;
    let src_port = src_module.get_output_port(src_module_out.output_port_key())?;

    let dst_module = self.get_module(dst_module_in.module_key())?;
    let dst_port = dst_module.get_input_port(dst_module_in.input_port_key())?;

    if src_module.parent != dst_module.parent {
      let src_path = port_path(src_module, src_port);
      let dst_path = port_path(dst_module, dst_port);
      Err(Error::ConnectionOutOfScope(src_path, dst_path))
    } else if dst_port.source.is_some() {
      let dst_path = port_path(dst_module, dst_port);
      Err(Error::InputSourceAlreadyDefined(
        D::port_type().name().to_string(),
        dst_path,
      ))
    } else {
      let dst_module = self.get_module_mut(dst_module_in.module_key())?;
      let mut dst_port = dst_module.get_input_port_mut(dst_module_in.input_port_key())?;
      dst_port.source = Some(InputSource::ModuleConnection(src_module_out));
      Ok(())
    }
  }

  fn connect_node_out_to_node_in<D>(
    &mut self,
    src_node_out: NodeOut<D>,
    dst_node_in: NodeIn<D>,
  ) -> Result<()>
  where
    D: PortDescriptor,
    Ports: PortAccessor<D>,
  {
    let src_node = self.get_node(src_node_out.node_key())?;
    let src_port = src_node.get_output_port(src_node_out.output_port_key())?;

    let dst_node = self.get_node(dst_node_in.node_key())?;
    let dst_port = dst_node.get_input_port(dst_node_in.input_port_key())?;

    if src_node.parent != dst_node.parent {
      let src_path = port_path(src_node, src_port);
      let dst_path = port_path(dst_node, dst_port);
      Err(Error::ConnectionOutOfScope(src_path, dst_path))
    } else if dst_port.source.is_some() {
      let dst_path = port_path(dst_node, dst_port);
      Err(Error::InputSourceAlreadyDefined(
        D::port_type().name().to_string(),
        dst_path,
      ))
    } else {
      let dst_node = self.get_node_mut(dst_node_in.node_key())?;
      let dst_port = dst_node.get_input_port_mut(dst_node_in.input_port_key())?;
      dst_port.source = Some(InputSource::NodeConnection(src_node_out));
      Ok(())
    }
  }

  fn connect_node_out_to_module_in<D>(
    &mut self,
    src_node_out: NodeOut<D>,
    dst_module_in: ModuleIn<D>,
  ) -> Result<()>
  where
    D: PortDescriptor,
    Ports: PortAccessor<D>,
  {
    let src_node = self.get_node(src_node_out.node_key())?;
    let src_port = src_node.get_output_port(src_node_out.output_port_key())?;

    let dst_module = self.get_module(dst_module_in.module_key())?;
    let dst_port = dst_module.get_input_port(dst_module_in.input_port_key())?;

    if Some(src_node.parent) != dst_module.parent {
      let src_path = port_path(src_node, src_port);
      let dst_path = port_path(dst_module, dst_port);
      Err(Error::ConnectionOutOfScope(src_path, dst_path))
    } else if dst_port.source.is_some() {
      let dst_path = port_path(dst_module, dst_port);
      Err(Error::InputSourceAlreadyDefined(
        D::port_type().name().to_string(),
        dst_path,
      ))
    } else {
      let dst_module = self.get_module_mut(dst_module_in.module_key())?;
      let mut dst_port = dst_module.get_input_port_mut(dst_module_in.input_port_key())?;
      dst_port.source = Some(InputSource::NodeConnection(src_node_out));
      Ok(())
    }
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

  fn enough_dynamic_input_ports<P, D>(entity: &mut P) -> bool
  where
    P: NodeLike,
    D: PortDescriptor,
    Ports: PortAccessor<D>,
    DescriptorPorts: DescriptorPortAccessor<D>,
  {
    let dynamic_port = &entity.get_descriptor_port::<D>().dynamic_inputs;

    match dynamic_port {
      DynamicPorts::None => false,
      DynamicPorts::Limited(limit) => {
        let static_len = entity.get_descriptor_port().static_inputs.len();
        let current_len = entity.get_ports().get_input().len();
        if current_len < static_len {
          true
        } else {
          current_len - static_len < *limit
        }
      }
      DynamicPorts::Unlimited => true,
    }
  }

  fn enough_dynamic_output_ports<P, D>(entity: &mut P) -> bool
  where
    P: NodeLike,
    D: PortDescriptor,
    Ports: PortAccessor<D>,
    DescriptorPorts: DescriptorPortAccessor<D>,
  {
    let dynamic_port = &entity.get_descriptor_port::<D>().dynamic_outputs;

    match dynamic_port {
      DynamicPorts::None => false,
      DynamicPorts::Limited(limit) => {
        let static_len = entity.get_descriptor_port().static_inputs.len();
        let current_len = entity.get_ports().get_output().len();
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

    let test_node_descriptor = NodeDescriptor::new().with_audio_ports(|ports| {
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

    g.connect_audio(m2_audio_out[0].bind(m1_audio_out[0]))
      .unwrap();
  }
}
