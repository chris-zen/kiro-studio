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

  #[error("Dynamic ports not available")]
  DynamicPortsNotAvailable,

  #[error("Audio input source for {0} is already defined")]
  AudioInputSourceAlreadyDefined(String),

  #[error("Audio output source for {0} is already defined")]
  AudioOutputSourceAlreadyDefined(String),

  #[error("Audio input port not found for module '{0}': {1}")]
  AudioInputPortNotFound(String, AudioInputPortKey),

  #[error("Audio output port not found for module '{0}': {1}")]
  AudioOutputPortNotFound(String, AudioOutputPortKey),

  #[error("Events input source for {0} is already defined")]
  EventsInputSourceAlreadyDefined(String),

  #[error("Events output source for {0} is already defined")]
  EventsOutputSourceAlreadyDefined(String),

  #[error("Events input port not found for module '{0}': {1}")]
  EventsInputPortNotFound(String, EventsInputPortKey),

  #[error("Events output port not found for module '{0}': {1}")]
  EventsOutputPortNotFound(String, EventsOutputPortKey),

  #[error("Binding between {0} and {1} is out of scope. They require a parent-child relationship")]
  BindingOutOfScope(String, String),

  #[error("Connection between {0} and {1} is out of scope. They require a sibling relationship")]
  ConnectionOutOfScope(String, String),
}
