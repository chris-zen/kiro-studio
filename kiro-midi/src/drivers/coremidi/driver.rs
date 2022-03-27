use arc_swap::ArcSwap;
use core_foundation_sys::base::OSStatus;
use coremidi::{
  Client, EventList, InputPortWithContext, Notification, NotifyCallback, Object, ObjectType,
  Protocol, Source,
};
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use thiserror::Error;

use crate::drivers;
use crate::drivers::coremidi::endpoints::Endpoints;
use crate::drivers::coremidi::timestamp::coremidi_timestamp_to_nanos;
use crate::endpoints::{DestinationInfo, EndpointId, SourceId, SourceInfo};
use crate::event::Event;
use crate::filter::Filter;
use crate::input_config::InputConfig;
use crate::input_handler::InputHandler;
use crate::input_info::InputInfo;
use crate::protocol::decoder::DecoderProtocol2;
use crate::source_match::SourceMatches;

type InputName = String;

#[derive(Error, Debug)]
pub enum CoreMidiError {
  #[error("Error creating a new client: {0}")]
  ClientCreate(OSStatus),

  #[error("Error creating an input port: {0}")]
  PortCreate(OSStatus),

  #[error("An input with this name already exists: {0:?}")]
  InputAlreadyExists(InputConfig),

  #[error("Input not found: {0}")]
  InputNotFound(InputName),

  #[error("Source not found: {0}")]
  SourceNotFound(SourceId),

  #[error("Error connecting the source {2:08x} to the input {1}: {0}")]
  ConnectSource(OSStatus, InputName, SourceId),
}

struct Input {
  name: InputName,
  sources: SourceMatches,
  connected: HashSet<SourceId>,
  filters: Arc<ArcSwap<HashMap<SourceId, Filter>>>,
  port: coremidi::InputPortWithContext<SourceId>,
}

pub struct CoreMidiDriver {
  client: Client,
  endpoints: Arc<Mutex<Endpoints>>,
  inputs: Arc<Mutex<HashMap<String, Input>>>,
}

impl drivers::DriverSpec for CoreMidiDriver {
  fn create_input<H>(&mut self, config: InputConfig, handler: H) -> Result<String, drivers::Error>
  where
    H: Into<InputHandler>,
  {
    if self.inputs.lock().contains_key(config.name.as_str()) {
      Err(CoreMidiError::InputAlreadyExists(config).into())
    } else {
      let InputConfig { name, sources } = config;

      let filters = self
        .endpoints
        .lock()
        .connected_sources()
        .into_iter()
        .filter_map(|connected_source| {
          sources
            .match_filter(connected_source.id, connected_source.name.as_str())
            .map(|filter| (connected_source.id, filter))
        })
        .collect::<HashMap<SourceId, Filter>>();

      let filters = Arc::new(ArcSwap::new(Arc::new(filters)));

      let mut port = self.create_input_port(name.clone(), handler.into(), filters.clone())?;

      let endpoints = self.endpoints.lock();

      let mut connected = HashSet::new();

      for source_id in filters.load().keys().cloned() {
        if let Some(source) = endpoints.get_source(source_id) {
          if let Ok(()) = port.connect_source(source, source_id) {
            connected.insert(source_id);
          }
        }
      }

      let input = Input {
        name: name.clone(),
        sources,
        connected,
        filters,
        port,
      };

      self.inputs.lock().insert(name.clone(), input);

      Ok(name)
    }
  }

  fn sources(&self) -> Vec<SourceInfo> {
    let endpoints = self.endpoints.lock();

    let mut source_inputs = HashMap::<SourceId, HashSet<String>>::new();
    for input in self.inputs.lock().values() {
      for source_id in input.connected.iter().cloned() {
        let inputs = source_inputs.entry(source_id).or_default();
        inputs.insert(input.name.clone());
      }
    }

    endpoints
      .connected_sources()
      .into_iter()
      .map(|connected_source| {
        let inputs = source_inputs
          .get(&connected_source.id)
          .map(|inputs| inputs.iter().cloned().collect::<Vec<String>>())
          .unwrap_or_default();
        SourceInfo::new(connected_source.id, connected_source.name.clone(), inputs)
      })
      .collect()
  }

  fn destinations(&self) -> Vec<DestinationInfo> {
    self
      .endpoints
      .lock()
      .connected_destinations()
      .into_iter()
      .map(|connected_destination| {
        DestinationInfo::new(connected_destination.id, connected_destination.name.clone())
      })
      .collect()
  }

  fn inputs(&self) -> Vec<InputInfo> {
    self
      .inputs
      .lock()
      .values()
      .map(|input| InputInfo {
        name: input.name.clone(),
        sources: input.sources.clone(),
        connected_sources: input.connected.iter().cloned().collect(),
      })
      .collect()
  }

  fn get_input_config(&self, name: &str) -> Option<InputConfig> {
    self.inputs.lock().get(name).map(|input| InputConfig {
      name: input.name.clone(),
      sources: input.sources.clone(),
    })
  }

  fn set_input_sources(&self, name: &str, sources: SourceMatches) -> Result<(), drivers::Error> {
    let endpoints = self.endpoints.lock();

    let mut inputs = self.inputs.lock();

    let input = inputs
      .get_mut(name)
      .ok_or_else(|| CoreMidiError::InputNotFound(name.to_string()))?;

    let connected_sources = endpoints
      .connected_sources()
      .into_iter()
      .filter_map(|connected_source| {
        sources
          .match_filter(connected_source.id, connected_source.name.as_str())
          .map(|filter| (connected_source.id, filter, &connected_source.source))
      })
      .collect::<Vec<(SourceId, Filter, &Source)>>();

    let mut filters = HashMap::<SourceId, Filter>::with_capacity(connected_sources.len());
    let mut disconnected = input.connected.clone();

    for (source_id, filter, source) in connected_sources {
      filters.insert(source_id, filter);
      if !input.connected.contains(&source_id) {
        if let Ok(()) = input.port.connect_source(source, source_id) {
          input.connected.insert(source_id);
        }
      } else {
        disconnected.remove(&source_id);
      }
    }

    for source_id in disconnected {
      if let Some(source) = endpoints.get_source(source_id) {
        input.port.disconnect_source(source).ok();
      }
    }

    input.sources = sources;
    input.filters.swap(Arc::new(filters));

    Ok(())
  }
}

impl CoreMidiDriver {
  pub fn new(name: &str) -> Result<Self, drivers::Error> {
    let endpoints = Arc::new(Mutex::new(Endpoints::new()));
    let inputs = Arc::new(Mutex::new(HashMap::new()));
    let callback = Self::notifications_callback(endpoints.clone(), inputs.clone());
    let client =
      Client::new_with_notifications(name, callback).map_err(CoreMidiError::ClientCreate)?;
    Self::initialize_endpoints(endpoints.clone());

    Ok(Self {
      client,
      endpoints,
      inputs,
    })
  }

  fn create_input_port(
    &self,
    name: String,
    mut handler: InputHandler,
    filters: Arc<ArcSwap<HashMap<SourceId, Filter>>>,
  ) -> Result<InputPortWithContext<SourceId>, CoreMidiError> {
    let default_filter = Filter::new();
    let mut decoder = DecoderProtocol2::default();
    self
      .client
      .input_port_with_protocol(
        name.clone().as_str(),
        Protocol::Midi20,
        move |events, source_id: &mut SourceId| {
          Self::handle_input(
            name.as_str(),
            &filters,
            &default_filter,
            &mut decoder,
            &mut handler,
            events,
            *source_id,
          );
        },
      )
      .map_err(CoreMidiError::PortCreate)
  }

  fn handle_input(
    _name: &str,
    filters: &ArcSwap<HashMap<SourceId, Filter>>,
    default_filter: &Filter,
    decoder: &mut DecoderProtocol2,
    handler: &mut InputHandler,
    events: &EventList,
    source_id: SourceId,
  ) {
    let filters = filters.load();
    let filter = filters.get(&source_id).unwrap_or(default_filter);
    // println!("filter: {:#?}", filter);
    // println!("\n==> [{}:{:08x}:{}] {:?}", name, source_id, source_id, events);

    for event in events.iter() {
      decoder.reset();
      let timestamp = coremidi_timestamp_to_nanos(event.timestamp());
      for word in event.data() {
        if let Ok(Some(message)) = decoder.next(*word, filter) {
          let event = Event {
            timestamp,
            endpoint: source_id,
            message,
          };
          handler.call(event);
        }
      }
    }
  }

  fn notifications_callback(
    endpoints: Arc<Mutex<Endpoints>>,
    mut inputs: Arc<Mutex<HashMap<InputName, Input>>>,
  ) -> NotifyCallback {
    NotifyCallback::by_ownership(move |notification: Notification| match notification {
      Notification::ObjectAdded(info) => match info.child_type {
        ObjectType::Source => Self::handle_source_connected(&endpoints, &mut inputs, info.child),
        ObjectType::Destination => Self::handle_destination_connected(&endpoints, info.child),
        _ => {}
      },
      Notification::ObjectRemoved(info) => match info.child_type {
        ObjectType::Source => Self::handle_source_disconnected(&endpoints, &mut inputs, info.child),
        ObjectType::Destination => Self::handle_destination_disconnected(&endpoints, info.child),
        _ => {}
      },
      _ => {}
    })
  }

  fn handle_source_connected(
    endpoints: &Arc<Mutex<Endpoints>>,
    inputs: &mut Arc<Mutex<HashMap<InputName, Input>>>,
    object: Object,
  ) {
    if let Some((source_id, name)) = Self::object_info(&object) {
      let mut endpoints = endpoints.lock();
      endpoints.add_source(source_id, name.clone(), object.into());
      if let Some(source) = endpoints.get_source(source_id) {
        Self::connect_source(&mut inputs.lock(), source_id, name, source);
      }
    }
  }

  fn connect_source(
    inputs: &mut HashMap<InputName, Input>,
    source_id: SourceId,
    source_name: String,
    source: &Source,
  ) {
    for input in inputs.values_mut() {
      if !input.connected.contains(&source_id) {
        if let Some(filter) = input.sources.match_filter(source_id, source_name.as_str()) {
          let mut filters = input.filters.load().as_ref().clone();
          filters.insert(source_id, filter);
          input.filters.swap(Arc::new(filters));
          input.port.connect_source(source, source_id).ok();
          input.connected.insert(source_id);
        }
      }
    }
  }

  fn handle_source_disconnected(
    endpoints: &Arc<Mutex<Endpoints>>,
    inputs: &mut Arc<Mutex<HashMap<InputName, Input>>>,
    object: Object,
  ) {
    if let Some(connected_source) = endpoints.lock().remove_source(object.into()) {
      Self::disconnect_source(
        &mut inputs.lock(),
        connected_source.id,
        connected_source.name,
        connected_source.source,
      );
    }
  }

  fn disconnect_source(
    inputs: &mut HashMap<InputName, Input>,
    source_id: SourceId,
    source_name: String,
    source: Source,
  ) {
    for input in inputs.values_mut() {
      if input
        .sources
        .match_index(source_id, source_name.as_str())
        .is_some()
      {
        let mut filters = input.filters.load().as_ref().clone();
        filters.remove(&source_id);
        input.filters.swap(Arc::new(filters));
        input.port.disconnect_source(&source).ok();
        input.connected.remove(&source_id);
      }
    }
  }

  fn handle_destination_connected(endpoints: &Arc<Mutex<Endpoints>>, object: Object) {
    if let Some((id, name)) = Self::object_info(&object) {
      endpoints.lock().add_destination(id, name, object.into());
    }
  }

  fn handle_destination_disconnected(endpoints: &Arc<Mutex<Endpoints>>, object: Object) {
    endpoints.lock().remove_destination(object.into());
  }

  fn object_info(object: &coremidi::Object) -> Option<(EndpointId, String)> {
    let maybe_id = object.unique_id().map(|id| id as u64);
    let maybe_name = object.display_name();
    maybe_id.zip(maybe_name)
  }

  fn initialize_endpoints(endpoints: Arc<Mutex<Endpoints>>) {
    let mut endpoints = endpoints.lock();
    for source in coremidi::Sources {
      if let Some((id, name)) = Self::object_info(&source) {
        endpoints.add_source(id, name, source);
      }
    }
    for destination in coremidi::Destinations {
      if let Some((id, name)) = Self::object_info(&destination) {
        endpoints.add_destination(id, name, destination);
      }
    }
  }
}
