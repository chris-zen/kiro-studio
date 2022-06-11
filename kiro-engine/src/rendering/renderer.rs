use ringbuf::{Consumer, Producer};

use crate::processor::context::ProcessorContext;
use crate::rendering::buffers::audio::AudioBuffer;
use crate::rendering::buffers::events::EventsBuffer;
use crate::rendering::messages::Message;
use crate::rendering::owned_data::Ref;
use crate::rendering::renderer_plan::RenderPlan;
use crate::EngineConfig;

pub struct Renderer {
  tx: Producer<Message>,
  rx: Consumer<Message>,

  plan: Box<RenderPlan>,
}

unsafe impl Send for Renderer {}

impl Renderer {
  pub fn new(tx: Producer<Message>, rx: Consumer<Message>, _config: EngineConfig) -> Self {
    let plan = Box::new(RenderPlan::default());

    Self { tx, rx, plan }
  }

  pub fn get_audio_inputs(&mut self) -> &[Ref<AudioBuffer>] {
    self.plan.audio_inputs.as_slice()
  }

  pub fn get_audio_outputs(&mut self) -> &[Ref<AudioBuffer>] {
    self.plan.audio_outputs.as_slice()
  }

  pub fn get_events_inputs(&mut self) -> &[Ref<EventsBuffer>] {
    self.plan.events_inputs.as_slice()
  }

  pub fn get_events_outputs(&mut self) -> &[Ref<EventsBuffer>] {
    self.plan.events_outputs.as_slice()
  }

  pub fn render(&mut self, num_samples: usize) {
    self.process_messages();
    self.render_plan(num_samples);
  }

  fn process_messages(&mut self) {
    while let Some(message) = self.rx.pop() {
      match message {
        Message::MoveRenderPlan(plan) => {
          let prev_plan = std::mem::replace(&mut self.plan, plan);
          self.tx.push(Message::MoveRenderPlan(prev_plan)).ok(); // FIXME this will deallocate if failure
        }
      }
    }
  }

  fn render_plan(&mut self, num_samples: usize) {
    self.plan.ready.clear();
    self.plan.ready.extend(self.plan.initial_ready.iter());
    // println!(">>{:?}", self.plan.ready);

    let dependencies = &mut self.plan.dependencies;
    let completed = &mut self.plan.completed;
    completed.fill(0);

    while let Some(node_index) = self.plan.ready.pop_front() {
      // println!(">{} {:?} {:?}", node_index, dependencies, completed);

      if let Some(node) = self.plan.nodes.get_mut(node_index) {
        let mut processor = node.processor.clone();
        node
          .audio_input_ports
          .iter_mut()
          .for_each(|port| port.set_num_samples(num_samples));

        node
          .audio_output_ports
          .iter_mut()
          .for_each(|port| port.set_num_samples(num_samples));

        let mut context = ProcessorContext::new(
          num_samples,
          &node.parameters,
          &node.audio_input_ports,
          &node.audio_output_ports,
          &node.events_input_ports,
          &node.events_output_ports,
        );

        processor.render(&mut context);

        for index in node.triggers.iter().cloned() {
          completed[index] += 1;
          if completed[index] >= dependencies[index] {
            self.plan.ready.push_back(index);
          }
        }
      }
    }
  }
}
