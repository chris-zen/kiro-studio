use crate::rendering::buffers::audio::AudioBuffer;
use std::collections::VecDeque;
use std::sync::Arc;

use crate::processor::ports::audio::AudioPort;
use crate::processor::ports::events::EventsPort;
use crate::processor::ports::{Input, Output};
use crate::processor::BoxedProcessor;
use crate::rendering::buffers::events::EventsBuffer;
use crate::rendering::owned_data::Ref;
use crate::ParamValue;

#[derive(Debug)]
pub struct RenderNode {
  pub processor: Ref<BoxedProcessor>,
  pub parameters: Vec<Arc<ParamValue>>,
  pub audio_input_ports: Vec<AudioPort<Input>>,
  pub audio_output_ports: Vec<AudioPort<Output>>,
  pub events_input_ports: Vec<EventsPort<Input>>,
  pub events_output_ports: Vec<EventsPort<Output>>,
  pub triggers: Vec<usize>,
}

#[derive(Debug, Default)]
pub struct RenderPlan {
  pub nodes: Vec<RenderNode>,
  pub audio_inputs: Vec<Ref<AudioBuffer>>,
  pub audio_outputs: Vec<Ref<AudioBuffer>>,
  pub events_inputs: Vec<Ref<EventsBuffer>>,
  pub events_outputs: Vec<Ref<EventsBuffer>>,
  pub dependencies: Vec<usize>,
  pub completed: Vec<usize>,
  pub initial_ready: Vec<usize>,
  pub ready: VecDeque<usize>,
}
