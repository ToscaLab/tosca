use std::time::Duration;

use tosca::events::{Events as ToscaEvents, EventsDescription};

use rumqttc::v5::{
    AsyncClient, Event, EventLoop, MqttOptions, mqttbytes::QoS, mqttbytes::v5::Packet,
};

use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

use tokio_util::sync::CancellationToken;

use tracing::{error, warn};

use crate::error::Result;

// The capacity of the bounded asynchronous channel.
const ASYNC_CHANNEL_CAPACITY: usize = 10;

// Keep alive time to send `pingreq` to broker when the connection is idle.
const KEEP_ALIVE_TIME: Duration = Duration::from_secs(5);

/// Events data transmitted by an asynchronous event receiver task.
#[derive(Debug)]
pub struct EventsData {
    /// Device identifier.
    pub device_id: usize,
    /// Device events.
    pub events: ToscaEvents,
}

impl EventsData {
    pub(crate) const fn new(device_id: usize, events: ToscaEvents) -> Self {
        Self { device_id, events }
    }
}

#[derive(Debug)]
pub(crate) struct Events {
    // Events description.
    pub(crate) description: EventsDescription,
    // The token used to cancel the event task.
    pub(crate) cancellation_token: CancellationToken,
}

impl Events {
    pub(crate) fn new(description: EventsDescription) -> Self {
        Self {
            description,
            cancellation_token: CancellationToken::new(),
        }
    }
}

#[inline]
async fn run_event_subscriber(
    client: AsyncClient,
    mut eventloop: EventLoop,
    id: usize,
    sender: Sender<EventsData>,
    cancellation_token: CancellationToken,
) {
    loop {
        tokio::select! {
            // Use the cancellation token to stop the loop
            () = cancellation_token.cancelled() => { break; }
            // Poll the `MQTT` event coming from the network
            event = eventloop.poll() => {
                let event =  match &event {
                    Ok(event) => {
                        event
                    }
                    Err(e) => {
                        error!("Error in receiving the event, discard it: {e}");
                        continue;
                    }
                };

                let packet = match event {
                    Event::Incoming(packet) => packet,
                    Event::Outgoing(outgoing) => {
                        warn!("Outgoing packet, discard it: {:?}", outgoing);
                        continue;
                    }
                };

                let Packet::Publish(packet) = packet else {
                    warn!("Packet ignored: {:?}", packet);
                    continue;
                };

                let tosca_events: ToscaEvents = match serde_json::from_slice(&packet.payload) {
                    Ok(tosca_events) => tosca_events,
                    Err(e) => {
                        error!("Error converting packet bytes into events: {e}");
                        continue;
                    }
                };

                if let Err(e) = sender.send(EventsData::new(id, tosca_events)).await{
                    error!("Error sending events to the receiver, stopping the event receiver: {e}");
                    break;
                }

            }
        }
    }
    drop(sender);
    drop(eventloop);
    drop(client);
}

pub(crate) struct EventsRunner {
    client: AsyncClient,
    eventloop: EventLoop,
}

impl EventsRunner {
    #[inline]
    pub(crate) async fn init(
        client_identifier: String,
        broker_address: String,
        broker_port: u16,
        topic: &str,
    ) -> Result<Self> {
        let mut mqttoptions = MqttOptions::new(client_identifier, broker_address, broker_port);
        mqttoptions.set_keep_alive(KEEP_ALIVE_TIME);

        let (client, eventloop) = AsyncClient::new(mqttoptions, ASYNC_CHANNEL_CAPACITY);
        client.subscribe(topic, QoS::AtMostOnce).await?;

        Ok(Self { client, eventloop })
    }

    #[inline]
    pub(crate) fn run(
        self,
        id: usize,
        sender: Sender<EventsData>,
        cancellation_token: CancellationToken,
    ) -> JoinHandle<()> {
        tokio::spawn(run_event_subscriber(
            self.client,
            self.eventloop,
            id,
            sender,
            cancellation_token,
        ))
    }
}
