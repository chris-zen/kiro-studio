#![allow(clippy::wrong_self_convention)]

use crate::graph::connection::{
  AudioConnection, EventsConnection, ModuleAudioIn, ModuleAudioOut, ModuleEventsIn,
  ModuleEventsOut, ModuleIn, ModuleOut, NodeAudioIn, NodeAudioOut, NodeEventsIn, NodeEventsOut,
  NodeIn, NodeOut,
};
use crate::graph::error::{Error, Result};
use crate::graph::module::{Module, ModuleDescriptor, ModuleKey};
use crate::graph::node::{Node, NodeDescriptor, NodeKey};
use crate::graph::port::{AudioDescriptor, AudioOutputPort, AudioOutputPortKey, DynamicPorts, EventsDescriptor, InputPortKey, InputSource, NodeLike, OutputPort, OutputPortKey, OutputSource, port_path, PortType};
use crate::graph::port::{InputPort, PortDescriptor};
use crate::key_store::{HasId, KeyStore};

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
          source: None,
        });
        NodeOut(node_key, port_key)
      })
      .ok_or(Error::DynamicPortsNotAvailable)
  }

  fn module_path(&self, module_key: ModuleKey) -> Result<Vec<String>> {
    let module = self.get_module(module_key)?;
    match module.parent {
      Some(parent) => {
        let mut path = self.module_path(parent)?;
        path.push(module.name.clone());
        Ok(path)
      }
      None => Ok(vec![module.name.clone()]),
    }
  }

  pub fn connect_audio(&mut self, connection: AudioConnection) -> Result<()> {
    match connection {
      AudioConnection::ModuleOutBindModuleOut(mut src_module_out, mut dst_module_out) => {
        let mut dst_module = self.get_module(dst_module_out.module_key())?;
        let mut dst_port = dst_module.get_audio_output_port(dst_module_out.output_port_key())?;

        let mut src_module = self.get_module(src_module_out.module_key())?;
        let mut src_port = src_module.get_audio_output_port(src_module_out.output_port_key())?;

        if src_module.parent != Some(dst_module_out.module_key()) {
          if dst_module.parent == Some(src_module_out.module_key()) {
            std::mem::swap(&mut src_module_out, &mut dst_module_out);
            std::mem::swap(&mut src_module, &mut dst_module);
            std::mem::swap(&mut src_port, &mut dst_port);
          } else {
            let src_path = port_path(src_module, src_port);
            let dst_path = port_path(dst_module, dst_port);
            Err(Error::BindingOutOfScope(src_path, dst_path))?
          }
        }

        if dst_port.source.is_some() {
          let dst_path = port_path(dst_module, dst_port);
          Err(Error::AudioOutputSourceAlreadyDefined(dst_path))
        } else {
          let dst_module = self.get_module_mut(dst_module_out.module_key())?;
          let dst_port = dst_module.get_audio_output_port_mut(dst_module_out.output_port_key())?;
          dst_port.source = Some(OutputSource::ModuleBinding(src_module_out));
          Ok(())
        }
      }
      AudioConnection::NodeOutBindModuleOut(src_nout, dst_mout) => {
        todo!()
      }
      _ => todo!(),
    }
  }

  pub fn connect_events(&mut self, connection: EventsConnection) -> Result<()> {
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
    P: NodeLike,
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
    P: NodeLike,
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
