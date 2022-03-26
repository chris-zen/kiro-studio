use crate::controller::controller::{AudioBufferKey, EventsBufferKey, ParamKey, ProcessorKey};

#[derive(Debug)]
pub struct PlanNode {
  pub(crate) processor: ProcessorKey,
  pub(crate) parameters: Vec<ParamKey>,
  pub(crate) audio_input_buffers: Vec<Vec<AudioBufferKey>>,
  pub(crate) audio_output_buffers: Vec<Vec<AudioBufferKey>>,
  pub(crate) events_input_buffers: Vec<EventsBufferKey>,
  pub(crate) events_output_buffers: Vec<EventsBufferKey>,
  pub(crate) dependencies: Vec<ProcessorKey>,
}

impl PlanNode {
  pub fn new(processor: ProcessorKey) -> Self {
    Self {
      processor,
      parameters: Vec::new(),
      audio_input_buffers: Vec::new(),
      audio_output_buffers: Vec::new(),
      events_input_buffers: Vec::new(),
      events_output_buffers: Vec::new(),
      dependencies: Vec::new(),
    }
  }

  pub fn with_parameters(mut self, parameter_keys: Vec<ParamKey>) -> Self {
    self.parameters.extend(parameter_keys);
    self
  }

  pub fn with_parameter(mut self, parameter_key: ParamKey) -> Self {
    self.parameters.push(parameter_key);
    self
  }

  pub fn with_audio_input_port(mut self, audio_buffer_keys: Vec<AudioBufferKey>) -> Self {
    self.audio_input_buffers.push(audio_buffer_keys);
    self
  }

  pub fn with_audio_output_port(mut self, audio_buffer_keys: Vec<AudioBufferKey>) -> Self {
    self.audio_output_buffers.push(audio_buffer_keys);
    self
  }

  pub fn with_event_inputs(mut self, event_buffer_keys: Vec<EventsBufferKey>) -> Self {
    self.events_input_buffers.extend(event_buffer_keys);
    self
  }

  pub fn with_event_input(mut self, event_buffer_key: EventsBufferKey) -> Self {
    self.events_input_buffers.push(event_buffer_key);
    self
  }

  pub fn with_event_outputs(mut self, event_buffer_keys: Vec<EventsBufferKey>) -> Self {
    self.events_output_buffers.extend(event_buffer_keys);
    self
  }

  pub fn with_event_output(mut self, event_buffer_key: EventsBufferKey) -> Self {
    self.events_output_buffers.push(event_buffer_key);
    self
  }

  pub fn with_dependencies(mut self, processor_keys: Vec<ProcessorKey>) -> Self {
    self.dependencies.extend(processor_keys);
    self
  }

  pub fn with_dependency(mut self, processor_key: ProcessorKey) -> Self {
    self.dependencies.push(processor_key);
    self
  }
}
