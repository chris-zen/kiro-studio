use core::f32::consts as f32_consts;
use core::f64::consts as f64_consts;
use core::fmt::Debug;

use num_traits::ToPrimitive;

use crate::funcs::parabolic_sine::ParabolicSine;

pub trait Float: num_traits::Float + ParabolicSine + Copy + Default + Debug {
  const PI: Self;
  const LN_2: Self;

  fn val<T: ToPrimitive>(v: T) -> Self {
    Self::from(v).unwrap()
  }
}

impl Float for f32 {
  const PI: f32 = f32_consts::PI;
  const LN_2: f32 = f32_consts::LN_2;
}

impl Float for f64 {
  const PI: f64 = f64_consts::PI;
  const LN_2: f64 = f64_consts::LN_2;
}
