mod config;
mod engine;
mod error;
mod graph;
mod key_gen;
mod key_store;
mod module;
mod node;
mod ports;
pub mod processor;
mod rendering;

pub use crate::config::EngineConfig;
pub use crate::engine::Engine;
pub use crate::error::Error;
pub use crate::graph::param::ParamDescriptor;
pub use crate::graph::port::{AudioDescriptor, EventsDescriptor};
pub use crate::graph::{ModuleDescriptor, NodeDescriptor};
pub use crate::module::Module;
pub use crate::node::ProcessorNode;
pub use crate::ports::{
  AudioNodeIn, AudioNodeOut, EventsNodeIn, EventsNodeOut, ModuleIn, ModuleOut, NodeIn, NodeOut,
};
pub use crate::processor::{context::ProcessorContext, Processor};
pub use crate::rendering::buffers::events::{Event, EventData};
pub use crate::rendering::param_value::ParamValue;

// FIXME make them private
pub use rendering::controller::Controller;
pub use rendering::controller_plan::PlanNode;
pub use rendering::renderer::Renderer;
