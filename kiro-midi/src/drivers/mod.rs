#[cfg(target_os = "macos")]
mod coremidi;

#[cfg(target_os = "macos")]
pub use crate::drivers::coremidi::Driver;
