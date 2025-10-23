mod mqtt;
mod topic;

/// All essential data required for configuring an event broker.
pub mod broker;
/// A variety of notifiers designed for managing interrupt events.
pub mod interrupt;
/// A variety of notifiers designed for managing periodic events.
pub mod periodic;

use core::time::Duration;

use alloc::boxed::Box;

use embassy_executor::{SpawnToken, Spawner};
use embassy_net::Stack;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use embassy_time::Timer;

use esp_hal::gpio::AnyPin;

use log::{error, info};

use tosca::device::DeviceKind;
use tosca::events::{Event, Events, EventsDescription, PeriodicEvent, Topic};

use crate::device::Device;
use crate::error::{Error, ErrorKind};
use crate::state::ValueFromRef;

use broker::BrokerData;
use mqtt::Mqtt;
use topic::TopicBuilder;

use super::events::interrupt::{
    Notifier,
    bool::{BoolFn, monitor_bool_event},
    u8::{U8Fn, monitor_u8_event},
};
use super::events::periodic::{
    PeriodicNotifier,
    bool::{PeriodicBoolFn, monitor_periodic_bool_event},
    u8::{PeriodicU8Fn, monitor_periodic_u8_event},
};

// Internal array capacity.
const CAPACITY: usize = 4;

// Milliseconds to wait for after a task operation.
const WAIT_FOR_MILLISECONDS: u64 = 200;

// All events to write on the network
static EVENTS: Mutex<CriticalSectionRawMutex, Events> = Mutex::new(Events::empty());
// Signal which enables writing over the network
static WRITE_ON_NETWORK: Signal<CriticalSectionRawMutex, u8> = Signal::new();

/// Events configuration.
///
/// It defines all the data necessary to execute the events.
pub struct EventsConfig<S>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    spawner: Spawner,
    stack: Stack<'static>,
    broker: BrokerData,
    topic: Topic,
    device: Device<S>,
}

impl<S> EventsConfig<S>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    /// Creates a new [`EventsConfig`].
    #[inline]
    #[must_use]
    pub fn new(
        spawner: Spawner,
        stack: Stack<'static>,
        broker: BrokerData,
        device: Device<S>,
    ) -> Self {
        Self {
            spawner,
            stack,
            broker,
            topic: match device.description.kind {
                DeviceKind::Light => TopicBuilder::new().prefix("light"),
                _ => TopicBuilder::new().prefix("unknown"),
            }
            .suffix("events")
            .mac(device.wifi_mac)
            .build(),
            device,
        }
    }
}

#[embassy_executor::task]
async fn write_on_network(mut mqtt_client: Mqtt, topic: Topic) {
    loop {
        // The lock will be released at the end of this scope.
        {
            // Wait for until a signal is received.
            let _ = WRITE_ON_NETWORK.wait().await;
        }
        // The lock will be released at the end of this scope after
        // the json data has been obtained.
        let json_data = { serde_json::to_vec(&*EVENTS.lock().await) };

        // Serialize data
        let data = match json_data {
            Ok(data) => data,
            Err(e) => {
                error!("Error retrieving the data: {e}");
                continue;
            }
        };

        // Write the data over the network.
        if let Err(e) = mqtt_client.publish(topic.as_str(), &data).await {
            error!("Error publishing data over the network: {e}");
        }

        // Wait for a bit after writing over the network.
        Timer::after_millis(WAIT_FOR_MILLISECONDS).await;
    }
}

/// An events manager.
///
/// It checks whether events are correct and runs them.
pub struct EventsManager<S>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    config: EventsConfig<S>,
    events: Events,
}

impl<S> EventsManager<S>
where
    S: ValueFromRef + Send + Sync + 'static,
{
    /// Configures the [`EventsManager`].
    ///
    /// It allocates the internal structures with a fixed amount of memory.
    #[inline]
    #[must_use]
    pub fn config(config: EventsConfig<S>) -> Self {
        Self {
            config,
            events: Events::with_capacity(CAPACITY),
        }
    }

    /// Monitors a pin running an [`Event<bool>`] notifier.
    ///
    /// If the event is equal to another one, discard that event.
    #[inline]
    #[must_use]
    pub fn bool_event<F, Fut>(
        self,
        name: &'static str,
        description: &'static str,
        func: F,
        pin: AnyPin<'static>,
    ) -> Self
    where
        F: Fn(AnyPin<'static>, Notifier<bool>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let events_ref = self.events.bool_events_as_slice();
        let len = events_ref.len();

        for value in events_ref {
            if value.name == name {
                info!(
                    "The event `{}` is equal to `{}`, discard it.",
                    value.name, name
                );
                return self;
            }
        }

        let event = Event::bool(name).description(description);
        let bool_notifier = Notifier::bool(len);
        // We need to do this because embassy tasks do not support generics.
        let func: BoolFn = Box::new(move |pin, bool_notifier| Box::pin(func(pin, bool_notifier)));
        let task = monitor_bool_event(event, pin, bool_notifier, func);

        self.spawn(name, task, |events| events.add_bool_event(event))
    }

    /// Monitors a pin running a [`PeriodicEvent<bool>`] notifier.
    ///
    /// If the event is equal to another one, discard that event.
    #[inline]
    #[must_use]
    pub fn periodic_bool<F, Fut>(
        self,
        name: &'static str,
        description: &'static str,
        interval: Duration,
        func: F,
        pin: AnyPin<'static>,
    ) -> Self
    where
        F: Fn(AnyPin<'static>, PeriodicNotifier<bool>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let events_ref = self.events.periodic_bool_events_as_slice();
        let len = events_ref.len();
        for value in events_ref {
            if value.event.name == name {
                info!(
                    "The event `{}` is equal to `{}`, discard it.",
                    value.event.name, name
                );
                return self;
            }
        }

        let event = PeriodicEvent::bool(Event::bool(name).description(description), interval);
        let periodic_u8_notifier = PeriodicNotifier::bool(len, interval);
        // We need to do this because embassy tasks do not support generics.
        let func: PeriodicBoolFn =
            Box::new(move |pin, u8_notifier| Box::pin(func(pin, u8_notifier)));
        let task = monitor_periodic_bool_event(event, pin, periodic_u8_notifier, func);

        self.spawn(name, task, |events| events.add_periodic_bool_event(event))
    }

    /// Monitors a pin running an [`Event<u8>`] notifier.
    ///
    /// If the event is equal to another one, discard that event.
    #[inline]
    #[must_use]
    pub fn u8_event<F, Fut>(
        self,
        name: &'static str,
        description: &'static str,
        func: F,
        pin: AnyPin<'static>,
    ) -> Self
    where
        F: Fn(AnyPin<'static>, Notifier<u8>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let events_ref = self.events.u8_events_as_slice();
        let len = events_ref.len();

        for value in events_ref {
            if value.name == name {
                info!(
                    "The event `{}` is equal to `{}`, discard it.",
                    value.name, name
                );
                return self;
            }
        }

        let event = Event::u8(name).description(description);
        let u8_notifier = Notifier::u8(len);
        // We need to do this because embassy tasks do not support generics.
        let func: U8Fn = Box::new(move |pin, u8_notifier| Box::pin(func(pin, u8_notifier)));
        let task = monitor_u8_event(event, pin, u8_notifier, func);

        self.spawn(name, task, |events| events.add_u8_event(event))
    }

    /// Monitors a pin running a [`PeriodicEvent<u8>`] notifier.
    ///
    /// If the event is equal to another one, discard that event.
    #[inline]
    #[must_use]
    pub fn periodic_u8<F, Fut>(
        self,
        name: &'static str,
        description: &'static str,
        interval: Duration,
        func: F,
        pin: AnyPin<'static>,
    ) -> Self
    where
        F: Fn(AnyPin<'static>, PeriodicNotifier<u8>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let events_ref = self.events.periodic_u8_events_as_slice();
        let len = events_ref.len();

        for value in events_ref {
            if value.event.name == name {
                info!(
                    "The event `{}` is equal to `{}`, discard it.",
                    value.event.name, name
                );
                return self;
            }
        }

        let event = PeriodicEvent::u8(Event::u8(name).description(description), interval);
        let periodic_u8_notifier = PeriodicNotifier::u8(len, interval);
        // We need to do this because embassy tasks do not support generics.
        let func: PeriodicU8Fn = Box::new(move |pin, u8_notifier| Box::pin(func(pin, u8_notifier)));
        let task = monitor_periodic_u8_event(event, pin, periodic_u8_notifier, func);

        self.spawn(name, task, |events| events.add_periodic_u8_event(event))
    }

    /// Runs the task which writes the events over the network.
    ///
    /// It returns an [`EventsDescription`] containing eveny event description.
    ///
    /// # Errors
    ///
    /// It fails when:
    ///  - The events manager is empty, hence no events have been inserted.execution and
    ///  - The task which writes over the network cannot interact with its
    ///    scheduler or the network.
    pub async fn run_network_task(self) -> Result<Device<S>, Error> {
        if self.events.is_empty() {
            return Err(Error::new(
                ErrorKind::EmptyEventsManager,
                "No events in the event manager",
            ));
        }

        let mut mqtt_client = Mqtt::new(self.config.stack, self.config.broker).await?;

        // Connect mqtt.
        mqtt_client.connect().await?;

        // Spawn a task to write on network.
        self.config
            .spawner
            .spawn(write_on_network(mqtt_client, self.config.topic.clone()))?;

        Ok(self
            .config
            .device
            .events_description(EventsDescription::new(self.config.topic, self.events)))
    }

    fn spawn<F, T>(mut self, name: &'static str, task: SpawnToken<T>, add_event: F) -> Self
    where
        F: FnOnce(&mut Events),
    {
        if let Err(e) = self.config.spawner.spawn(task) {
            error!("Impossible to spawn the event `{name}`: {e}");
            return self;
        }
        add_event(&mut self.events);
        info!("Spawned the task for event `{name}`");
        self
    }
}
