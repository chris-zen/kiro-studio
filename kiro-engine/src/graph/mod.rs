mod inner;
mod module;
mod node;
mod param;
mod port;

use std::cell::RefCell;
use std::rc::Rc;
use thiserror::Error;

use crate::graph::inner::InnerGraph;
use crate::graph::port::HasPorts;
use crate::graph::port::PortDescriptor;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {}

pub struct Graph(Rc<RefCell<InnerGraph>>);
