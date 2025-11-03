use std::collections::{HashMap, HashSet};
use std::net::IpAddr;

use serde::Serialize;

use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::task::JoinHandle;

use tracing::{error, warn};

use tosca::device::{DeviceEnvironment, DeviceKind};
use tosca::events::{BrokerData, EventsDescription};
use tosca::route::RouteConfigs;

use crate::error::Result;
use crate::events::{Events, EventsData, EventsRunner};
use crate::request::{Request, RequestInfo, create_requests};

pub(crate) fn build_device_address(scheme: &str, address: &IpAddr, port: u16) -> String {
    format!("{scheme}://{address}:{port}")
}

/// Device network information.
///
/// All data needed to contact a device in a network.
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct NetworkInformation {
    /// Device complete name.
    pub name: String,
    /// Device addresses.
    pub addresses: HashSet<IpAddr>,
    /// Device port.
    pub port: u16,
    /// Device properties.
    pub properties: HashMap<String, String>,
    /// Device last reachable address.
    pub last_reachable_address: String,
}

impl NetworkInformation {
    /// Creates a [`NetworkInformation`].
    #[must_use]
    pub const fn new(
        name: String,
        addresses: HashSet<IpAddr>,
        port: u16,
        properties: HashMap<String, String>,
        last_reachable_address: String,
    ) -> Self {
        Self {
            name,
            addresses,
            port,
            properties,
            last_reachable_address,
        }
    }
}

/// Device description.
///
/// All properties which describe a device.
#[derive(Debug, PartialEq)]
pub struct Description {
    /// Device kind.
    pub kind: DeviceKind,
    /// Device environment.
    pub environment: DeviceEnvironment,
    /// Device main route.
    pub main_route: String,
}

impl Description {
    /// Creates a [`Description`].
    #[must_use]
    pub const fn new(kind: DeviceKind, environment: DeviceEnvironment, main_route: String) -> Self {
        Self {
            kind,
            environment,
            main_route,
        }
    }
}

/// A compliant device.
#[derive(Debug)]
pub struct Device {
    // Information needed to contact a device in a network.
    network_info: NetworkInformation,
    // All data needed to describe a device.
    description: Description,
    // All device requests.
    requests: HashMap<String, Request>,
    // All device events.
    //
    // If [`None`], the device does not support events.
    pub(crate) events: Option<Events>,
    // The join handle for the event task.
    pub(crate) event_handle: Option<JoinHandle<()>>,
}

impl PartialEq for Device {
    fn eq(&self, other: &Self) -> bool {
        self.network_info == other.network_info
            && self.description == other.description
            && self.requests == other.requests
    }
}

impl Device {
    /// Creates a [`Device`] from [`NetworkInformation`], [`Description`],
    /// and [`RouteConfigs`] data.
    ///
    /// This method might be useful when a device might be created from data
    /// contained in a database.
    #[must_use]
    pub fn new(
        network_info: NetworkInformation,
        description: Description,
        route_configs: RouteConfigs,
    ) -> Self {
        let requests = create_requests(
            route_configs,
            &network_info.last_reachable_address,
            &description.main_route,
            description.environment,
        );

        // TODO: Check if the last reachable address works or it is better to
        // build a new one. Return a Result here, because we have to evaluate
        // data validity.

        Self {
            network_info,
            description,
            requests,
            events: None,
            event_handle: None,
        }
    }

    /// Returns an immutable reference to [`NetworkInformation`].
    #[must_use]
    pub const fn network_info(&self) -> &NetworkInformation {
        &self.network_info
    }

    /// Returns an immutable reference to [`Description`].
    #[must_use]
    pub const fn description(&self) -> &Description {
        &self.description
    }

    /// Returns an immutable reference to [`EventsDescription`].
    ///
    /// If [`None`], the device does not support events.
    #[must_use]
    #[inline]
    pub fn events_metadata(&self) -> Option<&EventsDescription> {
        self.events.as_ref().map(|events| &events.description)
    }

    /// Returns requests information as a vector of [`RequestInfo`].
    #[must_use]
    #[inline]
    pub fn requests_info(&self) -> Vec<RequestInfo<'_>> {
        self.requests
            .iter()
            .map(|(route, sender)| RequestInfo::new(route, sender))
            .collect()
    }

    /// Returns the number of available requests for a device.
    #[must_use]
    #[inline]
    pub fn requests_count(&self) -> usize {
        self.requests.len()
    }

    /// Returns the [`Request`] associated with the given route.
    ///
    /// If [`None`], the given route **does not** exist.
    #[must_use]
    #[inline]
    pub fn request(&self, route: &str) -> Option<&Request> {
        self.requests.get(route)
    }

    // FIXME: Adding id from outside is wrong. Fix this when id will be
    // supported.
    /// Starts the asynchronous event receiver if the [`Device`]
    /// supports events.
    ///
    /// An event receiver task connects to the broker of its associated device
    /// and subscribes to the topic of that device.
    /// When a device sends an event to the broker, the task receives it
    /// from the network, parses it, and forwards it to the [`Receiver`]
    /// returned by this method.
    ///
    /// The `buffer_size` parameter specifies how many messages the buffer
    /// can hold.
    /// When the buffer is full, new send attempts will wait until
    /// a message is consumed from the channel.
    ///
    /// When the [`Receiver`] is dropped, the event receiver task terminates
    /// automatically.
    ///
    /// When [`None`], either the event receiver task has already been started,
    /// or an error occurred while subscribing to the topic.
    #[inline]
    pub async fn start_event_receiver(
        &mut self,
        id: usize,
        buffer_size: usize,
    ) -> Option<Receiver<EventsData>> {
        if self.event_handle.is_some() {
            warn!("Event receiver already started for device with id `{id}`");
            return None;
        }

        let (tx, rx) = mpsc::channel(buffer_size);
        if self.run_event_receiver(id, tx).await.is_err() {
            return None;
        }
        Some(rx)
    }

    pub(crate) async fn run_event_receiver(
        &mut self,
        id: usize,
        sender: Sender<EventsData>,
    ) -> Result<()> {
        if let Some(events) = &self.events {
            let BrokerData { address, port } = events.description.broker_data;
            let topic = events.description.topic.as_str();

            let events_runner =
                EventsRunner::init(id.to_string(), address.to_string(), port, topic)
                    .await
                    .map_err(|e| {
                        error!("Impossible to subscribe to topic {topic} for device {id}: {e}");
                        e
                    })?;

            let cancellation_token = events.cancellation_token.clone();

            let handle = events_runner.run(id, sender, cancellation_token);
            self.event_handle = Some(handle);
        }

        Ok(())
    }

    pub(crate) const fn init(
        network_info: NetworkInformation,
        description: Description,
        requests: HashMap<String, Request>,
        events: Option<Events>,
    ) -> Self {
        Self {
            network_info,
            description,
            requests,
            events,
            event_handle: None,
        }
    }
}

/// A collection of [`Device`]s.
#[derive(Debug, PartialEq)]
pub struct Devices(Vec<Device>);

impl Default for Devices {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for Devices {
    type Item = Device;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Devices {
    type Item = &'a Device;
    type IntoIter = std::slice::Iter<'a, Device>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut Devices {
    type Item = &'a mut Device;
    type IntoIter = std::slice::IterMut<'a, Device>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl Devices {
    /// Creates a [`Device`]s collection.
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Creates [`Devices`] from a vector of [`Device`]s.
    #[must_use]
    pub const fn from_devices(devices: Vec<Device>) -> Self {
        Self(devices)
    }

    /// Adds a [`Device`].
    #[inline]
    pub fn add(&mut self, device: Device) {
        self.0.push(device);
    }

    /// Checks whether the collection is empty.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of [`Device`] contained in a collection.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Gets a [`Device`] reference identified by the given index.
    #[must_use]
    #[inline]
    pub fn get(&self, index: usize) -> Option<&Device> {
        self.0.get(index)
    }

    /// Returns an iterator over [`Device`]s.
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, Device> {
        <&Self as IntoIterator>::into_iter(self)
    }

    #[inline]
    pub(crate) fn iter_mut(&mut self) -> std::slice::IterMut<'_, Device> {
        <&mut Self as IntoIterator>::into_iter(self)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::collections::{HashMap, HashSet};

    use tosca::device::{DeviceEnvironment, DeviceKind};
    use tosca::hazards::{Hazard, Hazards};
    use tosca::parameters::Parameters;
    use tosca::route::{Route, RouteConfigs};

    use super::{Description, Device, Devices, NetworkInformation, build_device_address};

    fn create_network_info(address: &str, port: u16) -> NetworkInformation {
        let ip_address = address.parse().unwrap();

        let complete_address = build_device_address("http", &ip_address, port);

        let mut addresses = HashSet::new();
        addresses.insert(ip_address);
        addresses.insert("172.0.0.1".parse().unwrap());

        let mut properties = HashMap::new();
        properties.insert("scheme".into(), "http".into());

        NetworkInformation::new(
            "device-name1._tosca._tcp.local.".into(),
            addresses,
            port,
            properties,
            complete_address,
        )
    }

    fn create_description(device_kind: DeviceKind, main_route: &str) -> Description {
        Description::new(device_kind, DeviceEnvironment::Os, main_route.into())
    }

    pub(crate) fn create_light() -> Device {
        let network_info = create_network_info("192.168.1.174", 5000);
        let description = create_description(DeviceKind::Light, "light/");

        let light_on_route = Route::put("On", "/on")
            .description("Turn light on.")
            .with_hazard(Hazard::ElectricEnergyConsumption);

        let light_off_route = Route::put("Off", "/off")
            .description("Turn light off.")
            .with_hazard(Hazard::LogEnergyConsumption);

        let toggle_route = Route::get("Toggle", "/toggle")
            .description("Toggle a light.")
            .with_hazards(
                Hazards::new()
                    .insert(Hazard::FireHazard)
                    .insert(Hazard::ElectricEnergyConsumption),
            )
            .with_parameters(Parameters::new().rangeu64("brightness", (0, 20, 1)));

        let route_configs = RouteConfigs::new()
            .insert(light_on_route.serialize_data())
            .insert(light_off_route.serialize_data())
            .insert(toggle_route.serialize_data());

        Device::new(network_info, description, route_configs)
    }

    pub(crate) fn create_unknown() -> Device {
        let network_info = create_network_info("192.168.1.176", 5500);
        let description = create_description(DeviceKind::Unknown, "ip-camera/");

        let camera_stream_route = Route::get("Stream", "/stream")
            .description("View camera stream.")
            .with_hazards(
                Hazards::new()
                    .insert(Hazard::ElectricEnergyConsumption)
                    .insert(Hazard::VideoDisplay)
                    .insert(Hazard::VideoRecordAndStore),
            );

        let screenshot_route = Route::get("Take screenshot", "/take-screenshot")
            .description("Take a screenshot.")
            .with_hazards(
                Hazards::new()
                    .insert(Hazard::ElectricEnergyConsumption)
                    .insert(Hazard::TakeDeviceScreenshots)
                    .insert(Hazard::TakePictures),
            );

        let route_configs = RouteConfigs::new()
            .insert(camera_stream_route.serialize_data())
            .insert(screenshot_route.serialize_data());

        Device::new(network_info, description, route_configs)
    }

    #[test]
    fn check_devices() {
        let devices_vector = vec![create_light(), create_unknown()];

        let devices_from_vector = Devices::from_devices(devices_vector);

        let mut devices = Devices::new();

        // A device is empty when being created.
        assert!(devices.is_empty());

        devices.add(create_light());
        devices.add(create_unknown());

        // Compare devices created with two different methods.
        assert_eq!(devices_from_vector, devices);

        // A device must not be empty.
        assert!(!devices.is_empty());

        // Check number of elements in devices.
        assert_eq!(devices.len(), 2);

        // Get a non-existent device.
        assert_eq!(devices.get(1000), None);

        // Get a reference to a device. The order is important.
        assert_eq!(devices.get(1), Some(&create_unknown()));
    }
}
