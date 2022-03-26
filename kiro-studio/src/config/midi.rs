#[derive(Debug, Clone, Default)]
pub struct MidiConfig {
  endpoints: Vec<EndpointConfig>,
}

#[derive(Debug, Clone, Default)]
pub struct EndpointConfig {}
