mod connection;
mod error;
mod inner;
mod module;
mod node;
mod param;
mod port;

use std::cell::RefCell;
use std::rc::Rc;
use thiserror::Error;

use crate::graph::inner::InnerGraph;
use crate::graph::port::NodeLike;
use crate::graph::port::PortDescriptor;

pub struct Graph(Rc<RefCell<InnerGraph>>);
