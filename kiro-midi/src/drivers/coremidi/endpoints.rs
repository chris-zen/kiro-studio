use std::collections::hash_map;
use std::collections::HashMap;

use crate::endpoints::{DestinationId, SourceId};

pub struct ConnectedSource {
  pub id: SourceId,
  pub name: String,
  pub source: coremidi::Source,
}

pub struct ConnectedDestination {
  pub id: SourceId,
  pub name: String,
  pub destination: coremidi::Destination,
}

pub struct DisconnectedSource {
  pub id: SourceId,
  pub name: String,
}

pub struct DisconnectedDestination {
  pub id: SourceId,
  pub name: String,
}

pub struct Endpoints {
  connected_sources: HashMap<SourceId, ConnectedSource>,
  connected_destinations: HashMap<DestinationId, ConnectedDestination>,
  disconnected_sources: HashMap<SourceId, DisconnectedSource>,
  disconnected_destinations: HashMap<DestinationId, DisconnectedDestination>,
}

impl Endpoints {
  pub fn new() -> Self {
    Self {
      connected_sources: HashMap::new(),
      connected_destinations: HashMap::new(),
      disconnected_sources: HashMap::new(),
      disconnected_destinations: HashMap::new(),
    }
  }

  pub fn connected_sources(&self) -> Vec<&ConnectedSource> {
    let mut sources = self
      .connected_sources
      .values()
      .collect::<Vec<&ConnectedSource>>();
    sources.sort_unstable_by(|source1, source2| source1.name.cmp(&source2.name));
    sources
  }

  pub fn connected_destinations(&self) -> Vec<&ConnectedDestination> {
    let mut destinations = self
      .connected_destinations
      .values()
      .collect::<Vec<&ConnectedDestination>>();
    destinations
      .sort_unstable_by(|destination1, destination2| destination1.name.cmp(&destination2.name));
    destinations
  }

  pub fn add_source(&mut self, id: SourceId, name: String, source: coremidi::Source) {
    if let hash_map::Entry::Vacant(connected_source) = self.connected_sources.entry(id) {
      self.disconnected_sources.remove(&id);
      connected_source.insert(ConnectedSource { id, name, source });
    }
  }

  pub fn remove_source(&mut self, source: coremidi::Source) -> Option<ConnectedSource> {
    let maybe_connected_source = self
      .connected_sources
      .iter()
      .find_map(|(id, connected_source)| (connected_source.source == source).then(|| *id))
      .and_then(|id| self.connected_sources.remove(&id));

    maybe_connected_source.map(|connected_source| {
      self.disconnected_sources.insert(
        connected_source.id,
        DisconnectedSource {
          id: connected_source.id,
          name: connected_source.name.clone(),
        },
      );

      connected_source
    })
  }

  pub fn get_source(&self, source_id: SourceId) -> Option<&coremidi::Source> {
    self
      .connected_sources
      .get(&source_id)
      .map(|connected_source| &connected_source.source)
  }

  pub fn add_destination(
    &mut self,
    id: DestinationId,
    name: String,
    destination: coremidi::Destination,
  ) {
    if let hash_map::Entry::Vacant(connected_destination) = self.connected_destinations.entry(id) {
      self.disconnected_destinations.remove(&id);
      connected_destination.insert(ConnectedDestination {
        id,
        name,
        destination,
      });
    }
  }

  pub fn remove_destination(&mut self, destination: coremidi::Destination) {
    let maybe_connected_destination = self
      .connected_destinations
      .iter()
      .find_map(|(id, connected_destination)| {
        (connected_destination.destination == destination).then(|| *id)
      })
      .and_then(|id| self.connected_destinations.remove(&id));

    if let Some(connected_destination) = maybe_connected_destination {
      self.disconnected_destinations.insert(
        connected_destination.id,
        DisconnectedDestination {
          id: connected_destination.id,
          name: connected_destination.name,
        },
      );
    }
  }
}
