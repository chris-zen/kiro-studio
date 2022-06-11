use kiro_dsp::oscillators::osc_waveform::OscWaveform;
use kiro_dsp::oscillators::pitched_oscillator::PitchedOscillator;
use kiro_dsp::smoother::{LinearSteps, LinearStepsSmoother};
use kiro_dsp::waveforms::saw_blep::{self, SawBlep};
use kiro_dsp::waveforms::sine_parabolic::SineParabolic;
use kiro_dsp::waveforms::triangle_dpw2x::TriangleDpw2x;
use kiro_engine::processor::ProcessorContext;
use kiro_engine::{
  AudioDescriptor, AudioNodeOut, Engine, EventData, EventsDescriptor, Module, ModuleDescriptor,
  NodeDescriptor, ParamDescriptor, Processor, ProcessorNode,
};
use kiro_midi::{
  self as midi,
  messages::{
    channel_voice::{ChannelVoice, ChannelVoiceMessage},
    MessageType,
  },
};
use kiro_time::SampleRate;

use crate::graph::Error;

pub struct VoiceNode {
  node: ProcessorNode,
  audio_out: AudioNodeOut,
}

impl VoiceNode {
  pub fn try_new(engine: &mut Engine, name: &str, sample_rate: SampleRate) -> Result<Self, Error> {
    let node = engine.create_processor(name, VoiceProcessor::new(sample_rate as f32))?;
    let events_in = node.events_input(VoiceProcessor::EVENTS_IN_NAME)?;
    let audio_out = node.audio_output(VoiceProcessor::AUDIO_OUT_NAME)?;
    Ok(Self { node, audio_out })
  }
}

pub struct VoiceProcessor {
  waveforms: [OscWaveform<f32>; 3],
  waveform_index: usize,
  osc: PitchedOscillator<f32>,
  shape: LinearStepsSmoother<f32>,
  semitones: LinearStepsSmoother<f32>,
  cents: LinearStepsSmoother<f32>,
  pitch_bend: LinearStepsSmoother<f32>,
  amplitude: LinearStepsSmoother<f32>,
}

impl VoiceProcessor {
  pub const NUM_SHAPES: usize = 3;

  pub const AUDIO_OUT_NAME: &'static str = "audio-out";
  pub const AUDIO_OUT_INDEX: usize = 0;

  pub const EVENTS_IN_NAME: &'static str = "events-in";
  pub const EVENTS_IN_INDEX: usize = 0;

  pub const SHAPE_INDEX: usize = 0;
  pub const SEMITONES_INDEX: usize = 1;
  pub const CENTS_INDEX: usize = 2;
  pub const PITCH_BEND_INDEX: usize = 3;
  pub const AMPLITUDE_INDEX: usize = 4;

  pub fn new(sample_rate: f32) -> Self {
    let waveforms: [OscWaveform<f32>; Self::NUM_SHAPES] = [
      OscWaveform::SineParabolic(SineParabolic),
      OscWaveform::TriangleDpw2x(TriangleDpw2x::default()),
      OscWaveform::SawBlep(
        SawBlep::default()
          .with_mode(saw_blep::Mode::Bipolar)
          .with_correction(saw_blep::Correction::EightPointBlepWithInterpolation),
      ),
    ];
    let params = Self::static_descriptor().parameters;
    let osc = PitchedOscillator::new(sample_rate, waveforms[0].clone(), 80.0);
    let smoothing_strategy = LinearSteps::from_time(sample_rate, 0.0005);
    Self {
      waveforms,
      waveform_index: 0,
      osc,
      shape: LinearStepsSmoother::new(
        params[Self::SHAPE_INDEX].initial,
        smoothing_strategy.clone(),
      ),
      semitones: LinearStepsSmoother::new(
        params[Self::SEMITONES_INDEX].initial,
        smoothing_strategy.clone(),
      ),
      cents: LinearStepsSmoother::new(
        params[Self::CENTS_INDEX].initial,
        smoothing_strategy.clone(),
      ),
      pitch_bend: LinearStepsSmoother::new(
        params[Self::PITCH_BEND_INDEX].initial,
        smoothing_strategy.clone(),
      ),
      amplitude: LinearStepsSmoother::new(
        params[Self::AMPLITUDE_INDEX].initial,
        smoothing_strategy.clone(),
      ),
    }
  }
}

impl Processor for VoiceProcessor {
  fn static_descriptor() -> NodeDescriptor
  where
    Self: Sized,
  {
    NodeDescriptor::new()
      .with_audio_ports(|ports| {
        ports.static_outputs(vec![AudioDescriptor::new(Self::AUDIO_OUT_NAME, 1)])
      })
      .with_events_ports(|ports| {
        ports.static_inputs(vec![EventsDescriptor::new(Self::EVENTS_IN_NAME)])
      })
      .with_parameters(vec![
        ParamDescriptor::new("shape")
          .initial(2.0)
          .max(Self::NUM_SHAPES as f32),
        ParamDescriptor::new("semitones")
          .min(-12.0 * 4.0)
          .max(12.0 * 4.0),
        ParamDescriptor::new("cents").min(-100.0).max(100.0),
        ParamDescriptor::new("pitch-bend").min(-1.0).max(1.0),
        ParamDescriptor::new("amplitude").initial(1.0).max(1.0),
      ])
  }

  fn render(&mut self, context: &mut ProcessorContext) {
    let shape = context.parameter(Self::SHAPE_INDEX).get();
    let waveform_index = shape.round().max(0.0) as usize;
    if waveform_index != self.waveform_index && waveform_index < self.waveforms.len() {
      self.waveform_index = waveform_index;
      let waveform = &self.waveforms[waveform_index];
      self.osc.set_waveform(waveform.clone())
    }

    self
      .semitones
      .set_target(context.parameter(Self::SEMITONES_INDEX).get());
    self
      .cents
      .set_target(context.parameter(Self::CENTS_INDEX).get());
    self
      .pitch_bend
      .set_target(context.parameter(Self::PITCH_BEND_INDEX).get());
    self
      .amplitude
      .set_target(context.parameter(Self::AMPLITUDE_INDEX).get());

    let events = context.events_input(Self::EVENTS_IN_INDEX);
    for event in events.iter() {
      match event.data {
        EventData::Midi(midi::messages::Message {
          group: _,
          mtype:
            MessageType::ChannelVoice(ChannelVoice {
              channel: _,
              message,
            }),
        }) => match message {
          ChannelVoiceMessage::NoteOn { note, velocity, .. } => {
            self
              .osc
              .set_pitch_frequency(midi::note_freq::KEY_FREQ[note as usize]);
            self.osc.set_amplitude(velocity as f32 / u16::MAX as f32);
          }
          ChannelVoiceMessage::NoteOff { .. } => {
            self.osc.set_amplitude(0.0);
          }
          _ => {}
        },
        _ => {}
      }
    }

    let mut output = context.audio_output(Self::AUDIO_OUT_INDEX).channel_mut(0);
    for sample in output.as_mut_slice().iter_mut() {
      self.semitones.next_value_with(|semitones| {
        self.osc.set_semitones(semitones);
      });

      self.cents.next_value_with(|cents| {
        self.osc.set_cents(cents);
      });

      self.pitch_bend.next_value_with(|pitch_bend| {
        self.osc.set_pitch_bend(pitch_bend);
      });

      self.amplitude.next_value_with(|amplitude| {
        self.osc.set_amplitude(amplitude);
      });

      *sample = self.osc.generate();
    }
  }
}
