use ringbuf::{Consumer, Producer};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use thiserror::Error;

use crate::audio::buffer::AudioBuffer;
use crate::callback::plan::{RenderNode, RenderPlan};
use crate::controller::owned_data::{OwnedData, Ref};
use crate::events::buffer::EventsBuffer;
use crate::key_gen::Key;
use crate::key_store::KeyStore;
use crate::messages::Message;
use crate::processor::ports::audio::AudioPort;
use crate::processor::ports::events::EventsPort;
use crate::processor::ports::{Input, Output};
use crate::processor::BoxedProcessor;
use crate::{EngineConfig, ParamValue, PlanNode, Processor};

pub type ProcessorKey = Key<BoxedProcessor>;
pub type ParamKey = Key<Arc<ParamValue>>;
pub type AudioBufferKey = Key<AudioBuffer>;
pub type EventsBufferKey = Key<EventsBuffer>;

#[derive(Error, Debug, PartialEq)]
pub enum ControllerError {
  // #[error("Processor not found: {0:?}")]
  // ProcessorNotFound(Key<BoxedProcessor>),

  // #[error("Parameters not found: {0:?}")]
  // ParametersNotFound(Key<ProcParams>),

  // #[error("Buffer not found: {0:?}")]
  // BufferNotFound(Key<Buffer>),
  #[error("Failed to send data to the renderer")]
  SendFailure,

  // #[error("Failed to create a Processor for {0} with class {1}")]
  // ProcessorCreationFailed(String, String),

  // #[error("Parameter key not found in the node cache for port {0:?}")]
  // ParamValueKeyNotFound(Key<ParamPort>),
  #[error("Processor with key {0:?} not found")]
  ProcessorNotFound(ProcessorKey),

  #[error("Parameter value with key {0:?} not found")]
  ParamValueNotFound(ParamKey),

  #[error("Audio buffer with key {0:?} not found")]
  AudioBufferNotFound(AudioBufferKey),

  #[error("Event buffer with key {0:?} not found")]
  EventsBufferNotFound(EventsBufferKey),
  // #[error("Parameter slice buffer not found for port {0:?}")]
  // SliceBufferNotFound(Key<ParamPort>),

  // #[error("Audio output buffer not found in the node cache for port {0:?}")]
  // AudioOutBufferNotFound(Key<AudioOutPort>),
}

// TODO figure out how to remove Sync for ControllerError
unsafe impl Sync for ControllerError {}

pub type Result<T> = core::result::Result<T, ControllerError>;

pub struct Controller {
  tx: Producer<Message>,
  rx: Consumer<Message>,

  config: EngineConfig,

  processors: OwnedData<BoxedProcessor>,
  parameters: KeyStore<Arc<ParamValue>>,
  audio_buffers: OwnedData<AudioBuffer>,
  event_buffers: OwnedData<EventsBuffer>,
}

impl Controller {
  pub fn new(tx: Producer<Message>, rx: Consumer<Message>, config: EngineConfig) -> Self {
    Self {
      tx,
      rx,
      config,
      parameters: KeyStore::new(),
      processors: OwnedData::new(),
      audio_buffers: OwnedData::new(),
      event_buffers: OwnedData::new(),
    }
  }

  pub fn add_processor<P>(&mut self, processor: P) -> ProcessorKey
  where
    P: Processor + 'static,
  {
    self.processors.add(Box::new(processor))
  }

  fn get_processor_ref(&self, key: ProcessorKey) -> Result<Ref<BoxedProcessor>> {
    self
      .processors
      .get(key)
      .ok_or(ControllerError::ProcessorNotFound(key))
  }

  pub fn add_parameters(&mut self, initial_values: &[f32]) -> Vec<ParamKey> {
    initial_values
      .iter()
      .cloned()
      .map(|value| self.parameters.add(Arc::new(ParamValue::new(value))))
      .collect()
  }

  fn get_parameter_value(&self, param_key: ParamKey) -> Result<Arc<ParamValue>> {
    self
      .parameters
      .get(param_key)
      .cloned()
      .ok_or(ControllerError::ParamValueNotFound(param_key))
  }

  pub fn set_parameter_value(&mut self, key: ParamKey, value: f32) -> Result<()> {
    let param_value = self.get_parameter_value(key)?;
    param_value.set(value);
    Ok(())
  }

  pub fn add_audio_buffer(&mut self) -> AudioBufferKey {
    self
      .audio_buffers
      .add(AudioBuffer::with_capacity(self.config.audio_buffer_size))
  }

  fn get_audio_buffer_ref(&self, key: AudioBufferKey) -> Result<Ref<AudioBuffer>> {
    self
      .audio_buffers
      .get(key)
      .ok_or(ControllerError::AudioBufferNotFound(key))
  }

  pub fn add_event_buffer(&mut self) -> EventsBufferKey {
    self
      .event_buffers
      .add(EventsBuffer::with_capacity(self.config.event_buffer_size))
  }

  pub fn get_event_buffer_ref(&self, key: EventsBufferKey) -> Result<Ref<EventsBuffer>> {
    self
      .event_buffers
      .get(key)
      .ok_or(ControllerError::EventsBufferNotFound(key))
  }

  pub fn send_render_plan(
    &mut self,
    plan_nodes: Vec<PlanNode>,
    audio_inputs: Vec<AudioBufferKey>,
    audio_outputs: Vec<AudioBufferKey>,
    events_inputs: Vec<EventsBufferKey>,
    events_outputs: Vec<EventsBufferKey>,
  ) -> Result<()> {
    let render_plan = self.build_render_plan(
      plan_nodes,
      audio_inputs,
      audio_outputs,
      events_inputs,
      events_outputs,
    )?;

    self
      .tx
      .push(Message::MoveRenderPlan(Box::new(render_plan)))
      .map_err(|_| ControllerError::SendFailure)
  }

  fn build_render_plan(
    &mut self,
    plan_nodes: Vec<PlanNode>,
    audio_inputs: Vec<AudioBufferKey>,
    audio_outputs: Vec<AudioBufferKey>,
    events_inputs: Vec<EventsBufferKey>,
    events_outputs: Vec<EventsBufferKey>,
  ) -> Result<RenderPlan> {
    let mut nodes = Vec::<RenderNode>::new();
    let mut nodes_by_key = HashMap::<ProcessorKey, usize>::new();
    let mut dependencies = vec![0; plan_nodes.len()];
    let mut triggers = HashMap::<ProcessorKey, HashSet<usize>>::new();

    for (index, node) in plan_nodes.into_iter().enumerate() {
      let processor = self.get_processor_ref(node.processor)?;

      let parameters = self.build_parameters(node.parameters)?;

      let audio_input_ports = self.build_audio_input_ports(node.audio_input_buffers)?;
      let audio_output_ports = self.build_audio_output_ports(node.audio_output_buffers)?;

      let events_input_ports = self.build_events_input_ports(node.events_input_buffers)?;
      let events_output_ports = self.build_events_output_ports(node.events_output_buffers)?;

      let render_node = RenderNode {
        processor,
        parameters,
        audio_input_ports,
        audio_output_ports,
        events_input_ports,
        events_output_ports,
        triggers: Vec::new(),
      };

      dependencies[index] = node.dependencies.len();

      for processor_key in node.dependencies {
        triggers.entry(processor_key).or_default().insert(index);
      }

      nodes.push(render_node);
      nodes_by_key.insert(node.processor, index);
    }

    for (processor_key, index) in nodes_by_key {
      let node = &mut nodes[index];
      node.triggers = triggers
        .entry(processor_key)
        .or_default()
        .iter()
        .cloned()
        .collect();
    }

    let completed = vec![0; nodes.len()];

    let initial_ready = dependencies
      .iter()
      .enumerate()
      .filter_map(|(index, num_deps)| (*num_deps == 0).then(|| index))
      .collect();

    let ready = VecDeque::with_capacity(nodes.len());

    let audio_inputs = self.build_audio_buffers(audio_inputs)?;
    let audio_outputs = self.build_audio_buffers(audio_outputs)?;
    let events_inputs = self.build_events_buffers(events_inputs)?;
    let events_outputs = self.build_events_buffers(events_outputs)?;

    Ok(RenderPlan {
      nodes,
      audio_inputs,
      audio_outputs,
      events_inputs,
      events_outputs,
      dependencies,
      completed,
      initial_ready,
      ready,
    })
  }

  fn build_parameters(&mut self, keys: Vec<ParamKey>) -> Result<Vec<Arc<ParamValue>>> {
    keys
      .into_iter()
      .try_fold(Vec::new(), |mut parameters, key| {
        let param_value = self.get_parameter_value(key)?;
        parameters.push(param_value);
        Ok(parameters)
      })
  }

  fn build_audio_input_ports(
    &mut self,
    keys: Vec<Vec<AudioBufferKey>>,
  ) -> Result<Vec<AudioPort<Input>>> {
    keys.into_iter().try_fold(Vec::new(), |mut ports, keys| {
      let buffers = self.build_audio_buffers(keys)?;
      let port = AudioPort::new(buffers);
      ports.push(port);
      Ok(ports)
    })
  }

  fn build_audio_output_ports(
    &mut self,
    keys: Vec<Vec<AudioBufferKey>>,
  ) -> Result<Vec<AudioPort<Output>>> {
    keys.into_iter().try_fold(Vec::new(), |mut ports, keys| {
      // TODO Should we check that the key has not been used for an output buffer already???
      let buffers = self.build_audio_buffers(keys)?;
      let port = AudioPort::new(buffers);
      ports.push(port);
      Ok(ports)
    })
  }

  fn build_audio_buffers(&self, keys: Vec<AudioBufferKey>) -> Result<Vec<Ref<AudioBuffer>>> {
    keys.into_iter().try_fold(Vec::new(), |mut buffers, key| {
      let buffer = self.get_audio_buffer_ref(key)?;
      buffers.push(buffer);
      Ok(buffers)
    })
  }

  fn build_events_input_ports(
    &mut self,
    keys: Vec<EventsBufferKey>,
  ) -> Result<Vec<EventsPort<Input>>> {
    let buffers = self.build_events_buffers(keys)?;
    Ok(buffers.into_iter().map(EventsPort::new).collect())
  }

  fn build_events_output_ports(
    &mut self,
    keys: Vec<EventsBufferKey>,
  ) -> Result<Vec<EventsPort<Output>>> {
    let buffers = self.build_events_buffers(keys)?;
    Ok(buffers.into_iter().map(EventsPort::new).collect())
  }

  fn build_events_buffers(&self, keys: Vec<EventsBufferKey>) -> Result<Vec<Ref<EventsBuffer>>> {
    keys.into_iter().try_fold(Vec::new(), |mut buffers, key| {
      let buffer = self.get_event_buffer_ref(key)?;
      buffers.push(buffer);
      Ok(buffers)
    })
  }

  pub fn process_messages(&mut self) {
    self.rx.pop_each(
      move |message| {
        match message {
          Message::MoveRenderPlan(plan) => {
            drop(plan);
          }
        }
        true
      },
      None,
    );
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::processor::ProcessorContext;
  use crate::Processor;
  use kiro_audio_graph::{AudioDescriptor, MidiDescriptor, NodeDescriptor, ParamDescriptor};
  use ringbuf::RingBuffer;

  struct TestProcessor(NodeDescriptor);

  impl Processor for TestProcessor {
    fn render(&mut self, _context: &mut ProcessorContext) {
      unimplemented!()
    }
  }

  struct TestProcessorFactory;

  impl ProcessorFactory for TestProcessorFactory {
    fn supported_classes(&self) -> Vec<String> {
      vec!["source-class".to_string(), "sink-class".to_string()]
    }

    fn create(&self, node: &Node) -> Option<Box<dyn Processor>> {
      Some(Box::new(TestProcessor(node.descriptor().clone())))
    }
  }

  fn create_graph() -> anyhow::Result<(Graph, NodeRef, NodeRef, NodeRef)> {
    let mut g = Graph::new();

    let source_desc = NodeDescriptor::new("source-class")
      .static_audio_outputs(vec![AudioDescriptor::new("OUT", 1)])
      .static_midi_outputs(vec![MidiDescriptor::new("OUT")]);
    let sink_desc = NodeDescriptor::new("sink-class")
      .static_audio_inputs(vec![
        AudioDescriptor::new("IN1", 1),
        AudioDescriptor::new("IN2", 1),
      ])
      .static_audio_outputs(vec![AudioDescriptor::new("OUT", 1)])
      .static_parameters(vec![
        ParamDescriptor::new("P1"),
        ParamDescriptor::new("P2"),
        ParamDescriptor::new("P3"),
      ])
      .static_midi_inputs(vec![MidiDescriptor::new("IN")]);

    let n1 = g.add_node("N1", source_desc.clone())?;
    let n2 = g.add_node("N2", source_desc.clone())?;
    let n3 = g.add_node("N3", sink_desc.clone())?;

    g.connect_audio(n1, g.audio_input(n3, "IN1")?)?;
    g.connect_audio(n2, g.audio_input(n3, "IN2")?)?;
    g.connect(n2, g.param(n3, "P1")?)?;

    let n3_out = g.audio_output(n3, "OUT")?;
    g.bind_output(n3_out, "OUT")?;

    Ok((g, n1, n2, n3))
  }

  fn create_controller_without_processor_factory() -> anyhow::Result<Controller> {
    let ring_buffer = RingBuffer::new(1);
    let (tx, rx) = ring_buffer.split();
    let config = EngineConfig::default();
    Ok(Controller::new(tx, rx, config))
  }

  fn create_controller() -> anyhow::Result<Controller> {
    let mut controller = create_controller_without_processor_factory()?;
    controller.register_processor_factory(TestProcessorFactory);
    Ok(controller)
  }

  #[test]
  fn update_graph_processor_factory_not_found() -> anyhow::Result<()> {
    let (g, _, _, _) = create_graph()?;
    let mut ct = create_controller_without_processor_factory()?;

    let result = ct.update_graph(&g);
    match result {
      Err(ControllerError::ProcessorFactoryNotFound(node, class)) => {
        assert!(node.contains("Node[N1]") || node.contains("Node[N2]"));
        assert_eq!(class, "source-class");
      }
      _ => assert!(false, "unexpected result"),
    }

    Ok(())
  }

  #[test]
  fn update_graph_success() -> anyhow::Result<()> {
    let (g, n1, n2, n3) = create_graph()?;
    let mut ct = create_controller()?;

    ct.update_graph(&g)?;

    assert_eq!(ct.parameters.len(), 3);
    assert_eq!(ct.processors.len(), 3);
    assert_eq!(ct.audio_buffers.len(), 6); // empty + 3 output buffers + 2 param slice buffers

    let nc1 = ct.nodes.get(&n1).unwrap();
    assert_eq!(nc1.parameter_value_keys.len(), 0);
    assert_eq!(
      nc1.audio_output_buffers.values().cloned().flatten().count(),
      1
    );
    assert_eq!(nc1.allocated_buffers.len(), 0);
    assert_eq!(nc1.render_ops.len(), 1);
    assert!(match nc1.render_ops.get(0).unwrap() {
      RenderOp::RenderProcessor { .. } => true,
      _ => false,
    });

    let nc2 = ct.nodes.get(&n2).unwrap();
    assert_eq!(nc2.parameter_value_keys.len(), 0);
    assert_eq!(
      nc2.audio_output_buffers.values().cloned().flatten().count(),
      1
    );
    assert_eq!(nc2.allocated_buffers.len(), 0);
    assert_eq!(nc2.render_ops.len(), 1);
    assert!(match nc2.render_ops.get(0).unwrap() {
      RenderOp::RenderProcessor { .. } => true,
      _ => false,
    });

    let nc3 = ct.nodes.get(&n3).unwrap();
    assert_eq!(nc3.parameter_value_keys.len(), 3);
    assert_eq!(
      nc3.audio_output_buffers.values().cloned().flatten().count(),
      1
    );
    assert_eq!(nc3.allocated_buffers.len(), 3);
    assert_eq!(nc3.render_ops.len(), 1);
    assert!(match nc3.render_ops.get(0).unwrap() {
      RenderOp::RenderProcessor { .. } => true,
      _ => false,
    });

    Ok(())
  }
}
