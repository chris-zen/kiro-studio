pub type EndpointId = u64;
pub type SourceId = EndpointId;
pub type DestinationId = EndpointId;

#[derive(Debug, Clone)]
pub struct SourceInfo {
  pub id: SourceId,
  pub name: String,
  pub connected_inputs: Vec<String>,
}

impl SourceInfo {
  pub fn new(id: SourceId, name: String, connected_inputs: Vec<String>) -> Self {
    Self {
      id,
      name,
      connected_inputs,
    }
  }
}

#[derive(Debug, Clone)]
pub struct DestinationInfo {
  pub id: DestinationId,
  pub name: String,
}

impl DestinationInfo {
  pub fn new(id: DestinationId, name: String) -> Self {
    Self { id, name }
  }
}
