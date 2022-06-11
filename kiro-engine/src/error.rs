use thiserror::Error;

use crate::graph;
use crate::rendering::controller;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
  #[error("Graph: {0}")]
  Graph(#[from] graph::Error),

  #[error("Controller: {0}")]
  Controller(#[from] controller::Error),
}
