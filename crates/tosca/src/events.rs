use alloc::string::String;
use alloc::vec::Vec;

use core::net::IpAddr;
use core::time::Duration;

use serde::Serialize;

/// Broker data.
#[derive(Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub struct BrokerData {
    /// Broker address.
    pub address: IpAddr,
    /// Broker port number.
    pub port: u16,
}

impl BrokerData {
    /// Creates a [`BrokerData`] .
    #[must_use]
    pub const fn new(address: IpAddr, port: u16) -> Self {
        Self { address, port }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(not(feature = "deserialize"), derive(Copy))]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
/// An event producing a determined type.
pub struct Event<T: Clone + Copy> {
    /// Event name.
    #[cfg(not(feature = "deserialize"))]
    pub name: &'static str,
    /// Event name.
    #[cfg(feature = "deserialize")]
    pub name: alloc::borrow::Cow<'static, str>,

    /// Event description.
    #[cfg(not(feature = "deserialize"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'static str>,
    /// Event description.
    #[cfg(feature = "deserialize")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<alloc::borrow::Cow<'static, str>>,

    /// Event value.
    pub value: T,
}

impl Event<bool> {
    /// Creates an [`Event<bool>`].
    #[must_use]
    pub const fn bool(name: &'static str) -> Self {
        Self {
            #[cfg(not(feature = "deserialize"))]
            name,
            #[cfg(feature = "deserialize")]
            name: alloc::borrow::Cow::Borrowed(name),
            description: None,
            value: false,
        }
    }
}

impl Event<u8> {
    /// Creates an [`Event<u8>`].
    #[must_use]
    pub const fn u8(name: &'static str) -> Self {
        Self {
            #[cfg(not(feature = "deserialize"))]
            name,
            #[cfg(feature = "deserialize")]
            name: alloc::borrow::Cow::Borrowed(name),
            description: None,
            value: 0,
        }
    }
}

impl<T: Clone + Copy> Event<T> {
    /// Sets the event description.
    #[must_use]
    #[cfg(not(feature = "deserialize"))]
    pub const fn description(mut self, description: &'static str) -> Self {
        self.description = Some(description);
        self
    }

    /// Sets the event description.
    #[must_use]
    #[inline]
    #[cfg(feature = "deserialize")]
    pub fn description(mut self, description: &'static str) -> Self {
        self.description = Some(alloc::borrow::Cow::Borrowed(description));
        self
    }

    /// Removes the event description.
    ///
    /// This method might be useful to reduce the payload sent over the network.
    #[cfg(not(feature = "deserialize"))]
    pub const fn remove_description(&mut self) {
        self.description = None;
    }

    /// Removes the event description.
    ///
    /// This method might be useful to reduce the payload sent over the network.
    #[cfg(feature = "deserialize")]
    #[inline]
    pub fn remove_description(&mut self) {
        self.description = None;
    }

    // Updates the event value.
    pub(crate) const fn update_value(&mut self, value: T) {
        self.value = value;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(not(feature = "deserialize"), derive(Copy))]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
/// A periodic [`Event`].
///
/// An event is periodic when it is checked every certain time interval.
pub struct PeriodicEvent<T: Clone + Copy> {
    /// The underlying [`Event`].
    pub event: Event<T>,
    /// Time interval to verify whether the event occurred.
    pub interval: Duration,
}

impl PeriodicEvent<bool> {
    /// Creates a [`PeriodicEvent<bool>`].
    #[must_use]
    pub const fn bool(event: Event<bool>, interval: Duration) -> Self {
        Self { event, interval }
    }
}

impl PeriodicEvent<u8> {
    /// Creates a [`PeriodicEvent<u8>`].
    #[must_use]
    pub const fn u8(event: Event<u8>, interval: Duration) -> Self {
        Self { event, interval }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
/// The topic on which the events will be published over the network.
///
/// This information uniquely identifies a set of events, so an external device
/// can retrieve all events data in a database using this value as identifier.
pub struct Topic(String);

impl Topic {
    /// Creates an empty [`Topic`].
    #[must_use]
    pub const fn empty() -> Self {
        Self(String::new())
    }

    /// Creates a [`Topic`].
    #[must_use]
    pub const fn new(value: String) -> Self {
        Self(value)
    }

    /// Returns the [`Topic`] as a [`&str`].
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[allow(clippy::struct_field_names)]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
/// All considered events of a system.
pub struct Events {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    bool_events: Vec<Event<bool>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    u8_events: Vec<Event<u8>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    periodic_bool_events: Vec<PeriodicEvent<bool>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    periodic_u8_events: Vec<PeriodicEvent<u8>>,
}

impl Events {
    /// Creates an empty [`Events`].
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            bool_events: Vec::new(),
            u8_events: Vec::new(),
            periodic_bool_events: Vec::new(),
            periodic_u8_events: Vec::new(),
        }
    }

    /// Creates an [`Events`] with the same memory capacity for each of its
    /// internal event sequences.
    #[inline]
    #[must_use]
    pub fn with_capacity(size: usize) -> Self {
        Self {
            bool_events: Vec::with_capacity(size),
            u8_events: Vec::with_capacity(size),
            periodic_bool_events: Vec::with_capacity(size),
            periodic_u8_events: Vec::with_capacity(size),
        }
    }

    /// Adds a sequence of [`Event<bool>`].
    #[inline]
    #[must_use]
    pub fn bool_events(mut self, bool_events: Vec<Event<bool>>) -> Self {
        self.bool_events = bool_events;
        self
    }

    /// Adds a sequence of [`Event<u8>`].
    #[inline]
    #[must_use]
    pub fn u8_events(mut self, u8_events: Vec<Event<u8>>) -> Self {
        self.u8_events = u8_events;
        self
    }

    /// Adds a sequence of [`PeriodicEvent<bool>`].
    #[inline]
    #[must_use]
    pub fn periodic_bool_events(mut self, periodic_bool_events: Vec<PeriodicEvent<bool>>) -> Self {
        self.periodic_bool_events = periodic_bool_events;
        self
    }

    /// Adds a sequence of [`PeriodicEvent<u8>`].
    #[inline]
    #[must_use]
    pub fn periodic_u8_events(mut self, periodic_u8_events: Vec<PeriodicEvent<u8>>) -> Self {
        self.periodic_u8_events = periodic_u8_events;
        self
    }

    /// Adds a single [`Event<bool>`].
    #[inline]
    pub fn add_bool_event(&mut self, bool_event: Event<bool>) {
        self.bool_events.push(bool_event);
    }

    /// Adds a single [`Event<u8>`].
    #[inline]
    pub fn add_u8_event(&mut self, u8_event: Event<u8>) {
        self.u8_events.push(u8_event);
    }

    /// Adds a single [`PeriodicEvent<bool>`].
    #[inline]
    pub fn add_periodic_bool_event(&mut self, periodic_bool_event: PeriodicEvent<bool>) {
        self.periodic_bool_events.push(periodic_bool_event);
    }

    /// Adds a single [`PeriodicEvent<u8>`].
    #[inline]
    pub fn add_periodic_u8_event(&mut self, periodic_u8_event: PeriodicEvent<u8>) {
        self.periodic_u8_events.push(periodic_u8_event);
    }

    /// Updates the [`Event<bool>`] value located at the given index.
    #[inline]
    pub fn update_bool_value(&mut self, index: usize, value: bool) {
        self.bool_events[index].update_value(value);
    }

    /// Updates the [`Event<u8>`] value located at the given index.
    #[inline]
    pub fn update_u8_value(&mut self, index: usize, value: u8) {
        self.u8_events[index].update_value(value);
    }

    /// Updates the [`PeriodicEvent<bool>`] value located at the given index.
    #[inline]
    pub fn update_periodic_bool_value(&mut self, index: usize, value: bool) {
        self.periodic_bool_events[index].event.update_value(value);
    }

    /// Updates the [`PeriodicEvent<u8>`] value located at the given index.
    #[inline]
    pub fn update_periodic_u8_value(&mut self, index: usize, value: u8) {
        self.periodic_u8_events[index].event.update_value(value);
    }

    /// Returns an immutable slice to a sequence of [`Event<bool>`].
    #[inline]
    #[must_use]
    pub fn bool_events_as_slice(&self) -> &[Event<bool>] {
        self.bool_events.as_slice()
    }

    /// Returns an immutable slice to a sequence of [`Event<u8>`].
    #[inline]
    #[must_use]
    pub fn u8_events_as_slice(&self) -> &[Event<u8>] {
        self.u8_events.as_slice()
    }

    /// Returns an immutable slice to a sequence of [`PeriodicEvent<bool>`].
    #[inline]
    #[must_use]
    pub fn periodic_bool_events_as_slice(&self) -> &[PeriodicEvent<bool>] {
        self.periodic_bool_events.as_slice()
    }

    /// Returns an immutable slice to a sequence of [`PeriodicEvent<u8>`].
    #[inline]
    #[must_use]
    pub fn periodic_u8_events_as_slice(&self) -> &[PeriodicEvent<u8>] {
        self.periodic_u8_events.as_slice()
    }

    /// Checks whether [`Events`] is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.bool_events.is_empty()
            && self.u8_events.is_empty()
            && self.periodic_bool_events.is_empty()
            && self.periodic_u8_events.is_empty()
    }
}

#[derive(Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
/// All events to be published over the network, including their associated
/// topic and broker data.
pub struct EventsDescription {
    broker_data: BrokerData,
    topic: Topic,
    events: Events,
}

impl EventsDescription {
    /// Creates an [`EventsDescription`].
    #[must_use]
    pub const fn new(broker_data: BrokerData, topic: Topic, events: Events) -> Self {
        Self {
            broker_data,
            topic,
            events,
        }
    }
}
