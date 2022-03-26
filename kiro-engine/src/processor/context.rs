use crate::ParamValue;
use std::sync::Arc;

use crate::processor::ports::audio::AudioPort;
use crate::processor::ports::events::EventsPort;
use crate::processor::ports::{Input, Output};

pub struct ProcessorContext<'a> {
  num_samples: usize,
  parameters: &'a [Arc<ParamValue>],
  audio_inputs: &'a [AudioPort<Input>],
  audio_outputs: &'a [AudioPort<Output>],
  events_inputs: &'a [EventsPort<Input>],
  events_outputs: &'a [EventsPort<Output>],
}

impl<'a> ProcessorContext<'a> {
  pub fn new(
    num_samples: usize,
    parameters: &'a [Arc<ParamValue>],
    audio_inputs: &'a [AudioPort<Input>],
    audio_outputs: &'a [AudioPort<Output>],
    events_inputs: &'a [EventsPort<Input>],
    events_outputs: &'a [EventsPort<Output>],
  ) -> Self {
    Self {
      num_samples,
      parameters,
      audio_inputs,
      audio_outputs,
      events_inputs,
      events_outputs,
    }
  }

  pub fn num_samples(&self) -> usize {
    self.num_samples
  }

  pub fn num_parameters(&self) -> usize {
    self.parameters.len()
  }

  pub fn parameter(&self, index: usize) -> &'a ParamValue {
    &self.parameters[index]
  }

  pub fn num_audio_inputs(&self) -> usize {
    self.audio_inputs.len()
  }

  pub fn audio_input(&self, index: usize) -> &'a AudioPort<Input> {
    &self.audio_inputs[index]
  }

  pub fn num_audio_outputs(&self) -> usize {
    self.audio_outputs.len()
  }

  pub fn audio_output(&self, index: usize) -> &'a AudioPort<Output> {
    &self.audio_outputs[index]
  }

  pub fn num_events_inputs(&self) -> usize {
    self.events_inputs.len()
  }

  pub fn events_input(&self, index: usize) -> &'a EventsPort<Input> {
    &self.events_inputs[index]
  }

  pub fn num_events_outputs(&self) -> usize {
    self.events_outputs.len()
  }

  pub fn events_output(&self, index: usize) -> &'a EventsPort<Output> {
    &self.events_outputs[index]
  }
}
