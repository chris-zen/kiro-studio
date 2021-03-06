use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use crate::{SampleRate, Signature, Tempo, TicksTime};

pub const MILLIS_PER_SECOND: u64 = 1_000;
pub const NANOS_PER_SECOND: u64 = 1_000_000_000;
// pub const PICOS_PER_SECOND: u64 = 1_000_000_000_000;
// pub const FEMTOS_PER_SECOND: u64 = 1_000_000_000_000_000;
// pub const ATTOS_PER_SECOND: u64 = 1_000_000_000_000_000_000;

pub type UnitType = u64;
pub const UNITS_PER_SECOND: UnitType = NANOS_PER_SECOND as UnitType;
pub const UNITS_PER_NANO: UnitType = UNITS_PER_SECOND / NANOS_PER_SECOND;
pub const UNITS_PER_MILLI: UnitType = UNITS_PER_SECOND / MILLIS_PER_SECOND;

const SECONDS_PER_MINUTE: u64 = 60;
pub const UNITS_PER_MINUTE: u64 = UNITS_PER_SECOND * SECONDS_PER_MINUTE;

///! High resolution time
#[derive(Debug, PartialOrd, PartialEq, Clone, Copy)]
pub struct ClockTime(UnitType);

impl ClockTime {
  pub fn zero() -> ClockTime {
    ClockTime(0)
  }

  pub fn new(units: UnitType) -> ClockTime {
    ClockTime(units)
  }

  pub fn from_nanos(nanos: u64) -> ClockTime {
    ClockTime(nanos * UNITS_PER_NANO)
  }

  pub fn from_millis(nanos: u64) -> ClockTime {
    ClockTime(nanos * UNITS_PER_MILLI)
  }

  pub fn from_seconds(seconds: f64) -> ClockTime {
    ClockTime((seconds * UNITS_PER_SECOND as f64).round() as UnitType)
  }

  pub fn from_samples(samples: u32, sample_rate: SampleRate) -> ClockTime {
    ClockTime(UnitType::from(samples) * UNITS_PER_SECOND / UnitType::from(sample_rate))
  }

  pub fn units(&self) -> UnitType {
    self.0
  }

  pub fn to_seconds(&self) -> f64 {
    self.0 as f64 / UNITS_PER_SECOND as f64
  }

  pub fn to_nanos(&self) -> u64 {
    self.0 as u64 / UNITS_PER_NANO
  }

  pub fn to_ticks(&self, signature: Signature, tempo: Tempo) -> TicksTime {
    let ticks_per_minute = u64::from(TicksTime::per_minute(signature, tempo));
    let ticks = u128::from(self.0) * u128::from(ticks_per_minute) / u128::from(UNITS_PER_MINUTE);
    TicksTime::new(ticks as u64)
  }
}

impl Add for ClockTime {
  type Output = ClockTime;

  fn add(self, rhs: ClockTime) -> ClockTime {
    ClockTime(self.0 + rhs.0)
  }
}

impl AddAssign for ClockTime {
  fn add_assign(&mut self, rhs: ClockTime) {
    *self = *self + rhs;
  }
}

impl Sub for ClockTime {
  type Output = ClockTime;

  fn sub(self, rhs: ClockTime) -> ClockTime {
    ClockTime(self.0 - rhs.0)
  }
}

impl SubAssign for ClockTime {
  fn sub_assign(&mut self, rhs: ClockTime) {
    *self = *self - rhs;
  }
}

impl Mul<u32> for ClockTime {
  type Output = ClockTime;

  fn mul(self, rhs: u32) -> ClockTime {
    ClockTime(self.0 * UnitType::from(rhs))
  }
}

impl MulAssign<u32> for ClockTime {
  fn mul_assign(&mut self, rhs: u32) {
    *self = *self * rhs;
  }
}

impl Mul<ClockTime> for u32 {
  type Output = ClockTime;

  fn mul(self, rhs: ClockTime) -> ClockTime {
    rhs * self
  }
}

impl Div<u32> for ClockTime {
  type Output = ClockTime;

  fn div(self, rhs: u32) -> ClockTime {
    ClockTime(self.0 / UnitType::from(rhs))
  }
}

impl DivAssign<u32> for ClockTime {
  fn div_assign(&mut self, rhs: u32) {
    *self = *self / rhs;
  }
}

#[cfg(test)]
mod tests {
  use super::ClockTime;
  use crate::clock::UNITS_PER_SECOND;

  #[test]
  pub fn clock_time_new() {
    let time = ClockTime::new(15);
    assert_eq!(time.units(), 15);
  }

  #[test]
  pub fn clock_time_zero() {
    let time = ClockTime::zero();
    assert_eq!(time.units(), 0);
  }

  #[test]
  pub fn clock_time_from_nanos() {
    let time = ClockTime::from_nanos(15);
    assert_eq!(time.units(), 15);
  }

  #[test]
  pub fn clock_time_from_millis() {
    let time = ClockTime::from_millis(15);
    assert_eq!(time.units(), 15_000_000);
  }

  #[test]
  pub fn clock_time_from_seconds() {
    let time = ClockTime::from_seconds(0.5);
    assert_eq!(time.units(), UNITS_PER_SECOND / 2);
  }

  #[test]
  pub fn clock_time_add() {
    let time1 = ClockTime::new(15);
    let time2 = ClockTime::new(5);
    assert_eq!(time1 + time2, ClockTime::new(20));
  }

  #[test]
  pub fn clock_time_add_assign() {
    let mut time1 = ClockTime::new(15);
    time1 += ClockTime::new(5);
    assert_eq!(time1, ClockTime::new(20));
  }

  #[test]
  pub fn clock_time_sub() {
    let time1 = ClockTime::new(15);
    let time2 = ClockTime::new(5);
    assert_eq!(time1 - time2, ClockTime::new(10));
  }

  #[test]
  pub fn clock_time_sub_assign() {
    let mut time1 = ClockTime::new(15);
    time1 -= ClockTime::new(5);
    assert_eq!(time1, ClockTime::new(10));
  }

  #[test]
  pub fn clock_time_mul() {
    let time1 = ClockTime::new(15);
    assert_eq!(time1 * 2u32, ClockTime::new(30));
  }

  #[test]
  pub fn clock_time_mul_assign() {
    let mut time1 = ClockTime::new(15);
    time1 *= 2u32;
    assert_eq!(time1, ClockTime::new(30));
  }

  #[test]
  pub fn clock_time_mul_rhs() {
    let time1 = ClockTime::new(15);
    assert_eq!(2u32 * time1, ClockTime::new(30));
  }

  #[test]
  pub fn clock_time_div() {
    let time1 = ClockTime::new(30);
    assert_eq!(time1 / 2u32, ClockTime::new(15));
  }

  #[test]
  pub fn clock_time_div_assign() {
    let mut time1 = ClockTime::new(30);
    time1 /= 2u32;
    assert_eq!(time1, ClockTime::new(15));
  }
}
