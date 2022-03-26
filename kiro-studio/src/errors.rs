use thiserror::Error;

#[derive(Debug, Clone, Copy, Error)]
pub enum Error {}

pub type Result<T> = core::result::Result<T, Error>;
