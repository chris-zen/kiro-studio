use thiserror::Error;

use crate::graph::module::ModuleKey;
use crate::graph::node::NodeKey;
use crate::graph::port::{
  AudioInputPortKey, AudioOutputPortKey, EventsInputPortKey, EventsOutputPortKey,
};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
  #[error("Module not found: {0}")]
  ModuleNotFound(ModuleKey),

  #[error("Node not found: {0}")]
  NodeNotFound(NodeKey),

  #[error("Port not found: {0}")]
  PortNotFound(String),

  #[error("Dynamic ports not available")]
  DynamicPortsNotAvailable,

  #[error("Binding between {0} and {1} is out of scope. They require a parent-child relationship")]
  BindingOutOfScope(String, String),

  #[error("Connection between {0} and {1} is out of scope. They require a sibling relationship")]
  ConnectionOutOfScope(String, String),

  #[error("{0} input source for {1} is already defined")]
  InputSourceAlreadyDefined(String, String),

  #[error("{0} output source for {1} is already defined")]
  OutputSourceAlreadyDefined(String, String),

  #[error("Input port not found for module '{0}': {1}")]
  InputPortNotFound(String, String),

  #[error("Output port not found for module '{0}': {1}")]
  OutputPortNotFound(String, String),
}
