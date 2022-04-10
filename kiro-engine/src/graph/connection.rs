use crate::graph::module::ModuleKey;
use crate::graph::node::NodeKey;
use crate::graph::port::{
  AudioDescriptor, EventsDescriptor, InputPortKey, OutputPortKey, PortDescriptor,
};

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
pub struct ModuleIn<D>(pub ModuleKey, pub InputPortKey<D>);

impl<D> Copy for ModuleIn<D> where D: PortDescriptor {}

impl<D> ModuleIn<D>
where
  D: PortDescriptor,
{
  pub fn module_key(&self) -> ModuleKey {
    self.0
  }

  pub fn input_port_key(&self) -> InputPortKey<D> {
    self.1
  }

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
pub struct ModuleOut<D>(pub ModuleKey, pub OutputPortKey<D>);

impl<D> Copy for ModuleOut<D> where D: PortDescriptor {}

impl<D> ModuleOut<D>
where
  D: PortDescriptor,
{
  pub fn module_key(&self) -> ModuleKey {
    self.0
  }

  pub fn output_port_key(&self) -> OutputPortKey<D> {
    self.1
  }

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
pub struct NodeIn<D>(pub NodeKey, pub InputPortKey<D>);

impl<D> Copy for NodeIn<D> where D: PortDescriptor {}

impl<D> NodeIn<D>
where
  D: PortDescriptor,
{
  pub fn node_key(&self) -> NodeKey {
    self.0
  }

  pub fn input_port_key(&self) -> InputPortKey<D> {
    self.1
  }

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
pub struct NodeOut<D>(pub NodeKey, pub OutputPortKey<D>);

impl<D> Copy for NodeOut<D> where D: PortDescriptor {}

impl<D> NodeOut<D>
where
  D: PortDescriptor,
{
  pub fn node_key(&self) -> NodeKey {
    self.0
  }

  pub fn output_port_key(&self) -> OutputPortKey<D> {
    self.1
  }

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
