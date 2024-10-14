use core::net::{Ipv4Addr, Ipv6Addr};

use edge_mdns::buf::VecBufAccess;
use edge_mdns::domain::base::Ttl;
use edge_mdns::host::{Service, ServiceAnswers};
use edge_mdns::io::{self, DEFAULT_SOCKET};
use edge_mdns::{host::Host, HostAnswersMdnsHandler};

use edge_nal::UdpSplit;
use edge_nal_std::Stack;

use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::signal::Signal;

use esp_idf_svc::hal::task::block_on;

use log::info;

use rand::{thread_rng, RngCore};

use crate::error::Result;

// Service hostname
const SERVICE_HOSTNAME: &str = "ascot";

// Service name
const SERVICE_NAME: &str = "ascot";

pub(crate) mod internal_service {
    use super::Result;
    pub trait Service {
        fn run(self) -> Result<()>;
    }
}

/// A mDNS-SD service.
pub struct MdnsSdService {
    http_address: Ipv4Addr,
    service_hostname: &'static str,
    service_name: &'static str,
    stack: Stack,
    send_buf: VecBufAccess<NoopRawMutex, 1500>,
    recv_buf: VecBufAccess<NoopRawMutex, 1500>,
}

impl internal_service::Service for MdnsSdService {
    fn run(self) -> Result<()> {
        block_on(self.mdns_sd_service())
    }
}

impl MdnsSdService {
    /// Creates a new [`MdnsSdService`] instance.
    pub fn new(http_address: Ipv4Addr) -> Self {
        // Create stack
        let stack = Stack::new();

        // Create sender and receiver buffers for mdns-sd protocol.
        let (recv_buf, send_buf) = (VecBufAccess::new(), VecBufAccess::new());

        Self {
            http_address,
            service_hostname: SERVICE_HOSTNAME,
            service_name: SERVICE_NAME,
            stack,
            send_buf,
            recv_buf,
        }
    }

    /// Sets a service hostname.
    pub fn service_hostname(mut self, service_hostname: &'static str) -> Self {
        self.service_hostname = service_hostname;
        self
    }

    /// Sets a service name.
    pub fn service_name(mut self, service_name: &'static str) -> Self {
        self.service_name = service_name;
        self
    }

    async fn mdns_sd_service(self) -> Result<()> {
        // No ipv6 up and running.
        // To have it running, we need to get at least a link-local ipv6 addr
        // first, using an `esp-idf-sys` API call once the wifi is up and running:
        // `esp_idf_svc::sys::esp_netif_create_ip6_linklocal`.
        // Moreover, we can't just pass "0" for the interface.
        // We need to pass `wifi.sta_netif().index()`
        // Sometimes, "0" does work on PCs, but not with ESP-IDF.
        // This API is very picky about having a correct ipv6-capable
        // interface rather than just "all" (= 0).
        let mut socket = io::bind(
            &self.stack,
            DEFAULT_SOCKET,
            Some(Ipv4Addr::UNSPECIFIED),
            None,
        )
        .await?;

        let (recv, send) = socket.split();

        info!(
            "About to run an mDNS responder reachable from a PC. \
             It will be addressable using {}.local, \
             so try to `ping {}.local`.",
            self.service_hostname, self.service_hostname
        );

        let host = Host {
            hostname: self.service_hostname,
            ipv4: self.http_address,
            ipv6: Ipv6Addr::UNSPECIFIED,
            ttl: Ttl::from_secs(60),
        };

        info!(
            "About to run an mDNS service with name `{}` of type `HTTPS` \
             on port `443`.",
            self.service_name
        );

        let service = Service {
            name: self.service_name,
            priority: 1,
            weight: 5,
            service: "_https",
            protocol: "_tcp",
            port: 443,
            service_subtypes: &[],
            txt_kvs: &[],
        };

        // A way to notify the mDNS responder that the data in `Host` had changed
        // Not necessary for this example, because the data is hard-coded
        let signal = Signal::new();

        let mdns = io::Mdns::<NoopRawMutex, _, _, _, _>::new(
            Some(Ipv4Addr::UNSPECIFIED),
            // No ipv6 up and running.
            None,
            recv,
            send,
            self.recv_buf,
            self.send_buf,
            |buf| thread_rng().fill_bytes(buf),
            &signal,
        );

        mdns.run(HostAnswersMdnsHandler::new(ServiceAnswers::new(
            &host, &service,
        )))
        .await
        .map_err(|e| e.into())
    }
}
