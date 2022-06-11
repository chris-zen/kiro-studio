use crate::float::Float;
use num_traits::ToPrimitive;

pub type LinearStepsSmoother<F> = Smoother<F, LinearSteps<F>>;
pub type ExponentialStepsSmoother<F> = Smoother<F, ExponentialStepsSmoothing<F>>;
pub type Ln2Smoother<F> = Smoother<F, Ln2Smothing<F>>;

pub struct Smoother<F, S> {
  value: F,
  target: F,
  strategy: S,
}

impl<F, S: SmoothingStrategy<F>> Smoother<F, S>
where
  F: Float,
  S: SmoothingStrategy<F>,
{
  pub fn new(value: F, strategy: S) -> Self {
    Self {
      value,
      target: value,
      strategy,
    }
  }

  pub fn reset(&mut self, value: F) {
    self.value = value;
    self.target = value;
    self.strategy.reset();
  }

  pub fn set_target(&mut self, target: F) {
    self.target = target;
    self.strategy.target_updated(self.value, self.target);
  }

  pub fn next_value(&mut self) -> F {
    self.value = self.strategy.next_value(self.value, self.target);
    self.value
  }

  pub fn next_value_opt(&mut self) -> Option<F> {
    self.value = self.strategy.next_value(self.value, self.target);
    (self.value != self.target).then(|| self.value)
  }

  pub fn next_value_with<U>(&mut self, mut update: U)
  where
    U: FnMut(F),
  {
    if let Some(value) = self.next_value_opt() {
      update(value);
    }
  }
}

pub trait SmoothingStrategy<F> {
  fn reset(&mut self);
  fn target_updated(&mut self, value: F, target: F);
  fn next_value(&mut self, value: F, target: F) -> F;
}

pub struct NoSmoothing;

impl<F> SmoothingStrategy<F> for NoSmoothing {
  fn reset(&mut self) {}
  fn target_updated(&mut self, _value: F, _target: F) {}
  fn next_value(&mut self, _value: F, target: F) -> F {
    target
  }
}

#[derive(Clone)]
pub struct LinearSteps<F> {
  num_steps: usize,
  current_step: usize,
  value_delta: F,
}

impl<F> LinearSteps<F>
where
  F: Float + ToPrimitive,
{
  pub fn new(num_steps: usize) -> Self {
    Self {
      num_steps,
      current_step: num_steps,
      value_delta: F::zero(),
    }
  }

  pub fn from_time(sample_rate: F, time: F) -> Self {
    Self::new(F::floor(sample_rate * time).to_usize().unwrap_or(0))
  }
}

impl<F> SmoothingStrategy<F> for LinearSteps<F>
where
  F: Float,
{
  fn reset(&mut self) {
    self.current_step = self.num_steps;
  }

  fn target_updated(&mut self, value: F, target: F) {
    self.current_step = 0;
    let num_steps = F::from(self.num_steps).unwrap_or(F::one());
    self.value_delta = (target - value) / num_steps;
  }

  fn next_value(&mut self, value: F, target: F) -> F {
    if self.current_step < self.num_steps {
      self.current_step += 1;
      value + self.value_delta
    } else {
      target
    }
  }
}

#[derive(Clone)]
pub struct ExponentialStepsSmoothing<F> {
  num_steps: usize,
  current_step: usize,
  value_delta: F,
}

impl<F> ExponentialStepsSmoothing<F>
where
  F: Float + ToPrimitive,
{
  pub fn new(num_steps: usize) -> Self {
    Self {
      num_steps,
      current_step: num_steps,
      value_delta: F::zero(),
    }
  }

  pub fn from_time(sample_rate: F, time: F) -> Self {
    Self::new(F::floor(sample_rate * time).to_usize().unwrap_or(0))
  }
}

impl<F> SmoothingStrategy<F> for ExponentialStepsSmoothing<F>
where
  F: Float,
{
  fn reset(&mut self) {
    self.current_step = self.num_steps;
  }

  fn target_updated(&mut self, value: F, target: F) {
    self.current_step = 0;
    let num_steps = F::from(self.num_steps).unwrap_or(F::one());
    self.value_delta = ((target.abs().ln() - value.abs().ln()) / num_steps).exp()
  }

  fn next_value(&mut self, value: F, target: F) -> F {
    if self.current_step < self.num_steps {
      self.current_step += 1;
      value * self.value_delta
    } else {
      target
    }
  }
}

#[derive(Clone)]
pub struct Ln2Smothing<F> {
  target: F,
  value: F,
  factor: F,
}

impl<F> Ln2Smothing<F>
where
  F: Float,
{
  pub fn new(sample_rate: F, time: F) -> Self {
    Self {
      target: F::zero(),
      value: F::zero(),
      factor: F::val(F::LN_2) / F::min(sample_rate * time, F::one()),
    }
  }
}

impl<F> SmoothingStrategy<F> for Ln2Smothing<F>
where
  F: Float,
{
  fn reset(&mut self) {
    self.value = self.target;
  }

  fn target_updated(&mut self, value: F, target: F) {
    self.target = target;
    self.value = value;
  }

  fn next_value(&mut self, _value: F, _target: F) -> F {
    if self.value != self.target {
      self.value = self.value + (self.target - self.value) * self.factor;
      self.value
    } else {
      self.target
    }
  }
}
