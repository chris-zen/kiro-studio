use crate::key_store::HasId;

#[derive(Debug, Clone, PartialEq)]
pub struct ParamDescriptor {
  pub id: String,
  pub initial: f32,
  pub min: f32,
  pub max: f32,
  pub center: f32,
}

impl ParamDescriptor {
  pub fn new<S: Into<String>>(id: S) -> Self {
    Self {
      id: id.into(),
      initial: 0.0,
      min: 0.0,
      max: 1.0,
      center: 0.0,
    }
  }

  pub fn initial(mut self, initial: f32) -> Self {
    self.initial = initial;
    self
  }

  pub fn min(mut self, min: f32) -> Self {
    self.min = min;
    self
  }

  pub fn max(mut self, max: f32) -> Self {
    self.max = max;
    self
  }

  pub fn center(mut self, center: f32) -> Self {
    self.center = center;
    self
  }
}

impl HasId for ParamDescriptor {
  fn id(&self) -> &str {
    self.id.as_str()
  }
}
