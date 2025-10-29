use alloc::boxed::Box;

use embassy_net::{IpAddress, Stack, tcp::TcpSocket};

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;

use embassy_time::{Duration, with_timeout};

use rust_mqtt::client::client::MqttClient;
use rust_mqtt::client::client_config::{ClientConfig, MqttVersion};
use rust_mqtt::packet::v5::{publish_packet::QualityOfService, reason_codes::ReasonCode};
use rust_mqtt::utils::rng_generator::CountingRng;

use log::{info, warn};

use crate::error::{Error, ErrorKind};

// Timeout duration for the socket connection, in seconds.
const SOCKET_TIMEOUT: u64 = 2;
// Maximum packet size, in bytes, sent by a socket.
const MAX_PACKET_SIZE: u32 = 100;
// Size of the socket and MQTT client transmission and reception buffers.
const BUFFER_SIZE: usize = 1024;
// Maximum numbers of properties for MQTT client.
const MAXIMUM_MQTT_PROPERTIES: usize = 5;

pub(crate) struct Mqtt {
    pub(crate) client:
        Mutex<CriticalSectionRawMutex, MqttClient<'static, TcpSocket<'static>, 5, CountingRng>>,
}

impl Mqtt {
    pub(crate) async fn new(
        stack: Stack<'static>,
        remote_endpoint: (IpAddress, u16),
    ) -> Result<Self, Error> {
        let rx_buffer = Box::leak(Box::new([0u8; BUFFER_SIZE]));
        let tx_buffer = Box::leak(Box::new([0u8; BUFFER_SIZE]));
        let recv_buffer = Box::leak(Box::new([0u8; BUFFER_SIZE]));
        let write_buffer = Box::leak(Box::new([0u8; BUFFER_SIZE]));

        let mut socket = TcpSocket::new(stack, &mut rx_buffer[..], &mut tx_buffer[..]);

        info!(
            "Connecting to broker socket with address `{}` on port `{}`...",
            remote_endpoint.0, remote_endpoint.1
        );

        with_timeout(
            Duration::from_secs(SOCKET_TIMEOUT),
            socket.connect(remote_endpoint),
        )
        .await
        .map_err(|_| Error::new(ErrorKind::Timeout, "Broker not available"))??;

        info!("Connected to socket!");

        let mut config = ClientConfig::new(MqttVersion::MQTTv5, CountingRng(20000));
        config.add_max_subscribe_qos(QualityOfService::QoS1);
        config.max_packet_size = MAX_PACKET_SIZE;

        let client = MqttClient::<_, MAXIMUM_MQTT_PROPERTIES, _>::new(
            socket,
            &mut write_buffer[..],
            BUFFER_SIZE,
            &mut recv_buffer[..],
            BUFFER_SIZE,
            config,
        );

        Ok(Self {
            client: Mutex::new(client),
        })
    }

    #[inline]
    pub(crate) async fn connect(&mut self) -> Result<(), Error> {
        self.client
            .lock()
            .await
            .connect_to_broker()
            .await
            .map_err(core::convert::Into::into)
    }

    #[inline]
    pub(crate) async fn publish(&mut self, topic: &str, payload: &[u8]) -> Result<(), Error> {
        let Err(e) = self
            .client
            .lock()
            .await
            .send_message(topic, payload, QualityOfService::QoS1, true)
            .await
        else {
            return Ok(());
        };

        match e {
            ReasonCode::NoMatchingSubscribers
            | ReasonCode::NoSubscriptionExisted
            | ReasonCode::SharedSubscriptionNotSupported => {
                warn!("{}", Error::from(e));
                Ok(())
            }
            _ => Err(e.into()),
        }
    }

    #[inline]
    pub(crate) async fn send_ping(&mut self) -> Result<(), Error> {
        self.client
            .lock()
            .await
            .send_ping()
            .await
            .map_err(core::convert::Into::into)
    }
}
