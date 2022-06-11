use std::collections::{HashMap, HashSet};

use crate::graph::connection::{NodeAudioOut, NodeEventsOut, NodeOut};
use crate::graph::port::{
  AudioInputSource, AudioOutputSource, EventsInputSource, EventsOutputSource, PortDescriptor,
  PortType,
};
use crate::graph::NodeKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Source {
  AudioInput(AudioInputSource),
  AudioOutput(AudioOutputSource),
  EventsInput(EventsInputSource),
  EventsOutput(EventsOutputSource),
}

impl From<AudioInputSource> for Source {
  fn from(source: AudioInputSource) -> Self {
    Source::AudioInput(source)
  }
}

impl From<AudioOutputSource> for Source {
  fn from(source: AudioOutputSource) -> Self {
    Source::AudioOutput(source)
  }
}

impl From<EventsInputSource> for Source {
  fn from(source: EventsInputSource) -> Self {
    Source::EventsInput(source)
  }
}

impl From<EventsOutputSource> for Source {
  fn from(source: EventsOutputSource) -> Self {
    Source::EventsOutput(source)
  }
}

#[derive(Debug, Clone)]
pub enum Output {
  Audio(NodeAudioOut),
  Events(NodeEventsOut),
}

impl Output {
  pub fn node_key(&self) -> NodeKey {
    match self {
      Self::Audio(node_out) => node_out.node_key(),
      Self::Events(node_out) => node_out.node_key(),
    }
  }
}

impl From<NodeAudioOut> for Output {
  fn from(node_out: NodeAudioOut) -> Self {
    Output::Audio(node_out)
  }
}

impl From<NodeEventsOut> for Output {
  fn from(node_out: NodeEventsOut) -> Self {
    Output::Events(node_out)
  }
}

#[derive(Default)]
pub struct Topology {
  pub nodes: Vec<NodeKey>,
  pub source_nodes: HashMap<NodeKey, HashSet<NodeKey>>,
  pub destination_nodes: HashMap<NodeKey, HashSet<NodeKey>>,
  pub source_ports: HashMap<Source, Output>,
}
