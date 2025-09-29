/// All possible error kinds.
#[derive(Copy, Clone)]
pub enum ErrorKind {
    /// Empty events manager.
    EmptyEventsManager,
    /// `DNS` error.
    Dns,
    /// `mDNS` error.
    MDns,
    /// `MQTT` error.
    Mqtt,
    /// Server error.
    Server,
    /// Socket error.
    Socket,
    /// Spawning a task error.
    SpawningTask,
    /// `TCP` error.
    Tcp,
    /// Wi-Fi connection error.
    WiFi,
}

impl ErrorKind {
    const fn description(self) -> &'static str {
        match self {
            Self::EmptyEventsManager => "Empty events manager",
            Self::Dns => "DNS",
            Self::MDns => "mDNS",
            Self::Mqtt => "MQTT",
            Self::Server => "Server",
            Self::Socket => "Socket",
            Self::SpawningTask => "Spawning task",
            Self::Tcp => "TCP",
            Self::WiFi => "Wi-Fi",
        }
    }
}

impl core::fmt::Debug for ErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.description().fmt(f)
    }
}

impl core::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.description().fmt(f)
    }
}

/// A firmware error.
pub struct Error {
    kind: ErrorKind,
    info: &'static str,
}

impl core::fmt::Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.error(f)
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.error(f)
    }
}

impl Error {
    pub(crate) fn new(kind: ErrorKind, info: &'static str) -> Self {
        Self { kind, info }
    }

    fn error(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{} -> {}", self.kind, self.info))
    }
}

impl From<embassy_net::dns::Error> for Error {
    fn from(e: embassy_net::dns::Error) -> Self {
        use embassy_net::dns::Error;
        let err = match e {
            Error::InvalidName => "Invalid Name",
            Error::NameTooLong => "Name too long",
            Error::Failed => "Failed",
        };
        Self::new(ErrorKind::Dns, err)
    }
}

impl<E: edge_nal::io::Error> From<edge_mdns::io::MdnsIoError<E>> for Error {
    fn from(e: edge_mdns::io::MdnsIoError<E>) -> Self {
        use edge_mdns::io::MdnsIoError;
        use edge_mdns::MdnsError;
        use edge_nal::io::ErrorKind;
        let err = match e {
            MdnsIoError::MdnsError(mdns_error) => match mdns_error {
                MdnsError::ShortBuf => "Internal: Short buffer",
                MdnsError::InvalidMessage => "Internal: Invalid message",
            },
            MdnsIoError::NoRecvBufError => "No receiving buffer error",
            MdnsIoError::NoSendBufError => "No sending buffer error",
            MdnsIoError::IoError(io_error) => match io_error.kind() {
                ErrorKind::Other => "I/O: Unspecified error kind",
                ErrorKind::NotFound => "I/O: An entity was not found, often a file",
                ErrorKind::PermissionDenied => "I/O: The operation lacked the necessary privileges to complete",
                ErrorKind::ConnectionRefused => "I/O: The connection was refused by the remote server",
                ErrorKind::ConnectionReset => "I/O: The connection was reset by the remote server",
                ErrorKind::ConnectionAborted => "I/O: The connection was aborted (terminated) by the remote server",
                ErrorKind::NotConnected => "I/O: The network operation failed because it was not connected yet",
                ErrorKind::AddrInUse => "I/O: A socket address could not be bound because the address is already in use elsewhere",
                ErrorKind::AddrNotAvailable => "I/O: A nonexistent interface was requested or the requested address was not local",
                ErrorKind::BrokenPipe => "I/O: The operation failed because a pipe was closed",
                ErrorKind::AlreadyExists => "I/O: An entity already exists, often a file",
                ErrorKind::InvalidInput => "I/O: A parameter was incorrect",
                ErrorKind::InvalidData => "I/O: Data not valid for the operation were encountered",
                ErrorKind::TimedOut => "I/O: The I/O operation’s timeout expired, causing it to be canceled",
                ErrorKind::Interrupted => "I/O: This operation was interrupted",
                ErrorKind::Unsupported => "I/O: This operation is unsupported on this platform",
                ErrorKind::OutOfMemory => "I/O: An operation could not be completed, because it failed to allocate enough memory",
                ErrorKind::WriteZero => "I/O: An attempted write could not write any data",
                _ =>  "I/O: Unknown or still non-existent error",
            }
        };

        Self::new(self::ErrorKind::MDns, err)
    }
}

impl From<rust_mqtt::packet::v5::reason_codes::ReasonCode> for Error {
    fn from(e: rust_mqtt::packet::v5::reason_codes::ReasonCode) -> Self {
        use rust_mqtt::packet::v5::reason_codes::ReasonCode;
        let err = match e {
            ReasonCode::Success => "Success",
            ReasonCode::GrantedQoS1 => "Granted Qo S1",
            ReasonCode::GrantedQoS2 => "Granted Qo S2",
            ReasonCode::DisconnectWithWillMessage => "Disconnect with will message",
            ReasonCode::NoMatchingSubscribers => "No matching subscribers",
            ReasonCode::NoSubscriptionExisted => "No subscription existed",
            ReasonCode::ContinueAuth => "Continue authentication",
            ReasonCode::ReAuthenticate => "Reauthenticate",
            ReasonCode::UnspecifiedError => "Unspecified error",
            ReasonCode::MalformedPacket => "Malformed packet",
            ReasonCode::ProtocolError => "Protocol error",
            ReasonCode::ImplementationSpecificError => "Implementation specific error",
            ReasonCode::UnsupportedProtocolVersion => "Unsupported protocol version",
            ReasonCode::ClientIdNotValid => "Client ID not valid",
            ReasonCode::BadUserNameOrPassword => "Bad username or password",
            ReasonCode::NotAuthorized => "Not authorized",
            ReasonCode::ServerUnavailable => "Server unavailable",
            ReasonCode::ServerBusy => "Server busy",
            ReasonCode::Banned => "Banned",
            ReasonCode::ServerShuttingDown => "Server shutting down",
            ReasonCode::BadAuthMethod => "Bad authentication method",
            ReasonCode::KeepAliveTimeout => "Keep alive timeout",
            ReasonCode::SessionTakeOver => "Sessions take over",
            ReasonCode::TopicFilterInvalid => "Topic filter invalid",
            ReasonCode::TopicNameInvalid => "Topic name invalid",
            ReasonCode::PacketIdentifierInUse => "Packet identifier in use",
            ReasonCode::PacketIdentifierNotFound => "Packet identifier not found",
            ReasonCode::ReceiveMaximumExceeded => "Receive maximum exceeded",
            ReasonCode::TopicAliasInvalid => "Topic alias invalid",
            ReasonCode::PacketTooLarge => "Packet too large",
            ReasonCode::MessageRateTooHigh => "Message rate too high",
            ReasonCode::QuotaExceeded => "Quota exceeded",
            ReasonCode::AdministrativeAction => "Administrative action",
            ReasonCode::PayloadFormatInvalid => "Payload format invalid",
            ReasonCode::RetainNotSupported => "Retain not supported",
            ReasonCode::QoSNotSupported => "QoS not supported",
            ReasonCode::UseAnotherServer => "Use another server",
            ReasonCode::ServerMoved => "Server moved",
            ReasonCode::SharedSubscriptionNotSupported => "Shared subscription not supported",
            ReasonCode::ConnectionRateExceeded => "Connection rate exceeded",
            ReasonCode::MaximumConnectTime => "Maximum connect time",
            ReasonCode::SubscriptionIdentifiersNotSupported => {
                "Subscription identifiers not supported"
            }
            ReasonCode::WildcardSubscriptionNotSupported => "Wildcard subscription not supported",
            ReasonCode::TimerNotSupported => "Timer not supported",
            ReasonCode::BuffError => "Buffering error",
            ReasonCode::NetworkError => "Network error",
        };

        Self::new(ErrorKind::Mqtt, err)
    }
}

impl From<edge_http::io::Error<edge_nal_embassy::TcpError>> for Error {
    fn from(e: edge_http::io::Error<edge_nal_embassy::TcpError>) -> Self {
        use edge_http::io::Error;
        use edge_http::ws::UpgradeError;
        use edge_http::HeadersMismatchError;
        match e {
            Error::InvalidHeaders => Self::new(ErrorKind::Server, "Invalid headers"),
            Error::InvalidBody => Self::new(ErrorKind::Server, "Invalid body"),
            Error::TooManyHeaders => Self::new(ErrorKind::Server, "Too many headers"),
            Error::TooLongHeaders => Self::new(ErrorKind::Server, "Too long headers"),
            Error::TooLongBody => Self::new(ErrorKind::Server, "Too long body"),
            Error::IncompleteHeaders => Self::new(ErrorKind::Server, "Incomplete headers"),
            Error::IncompleteBody => Self::new(ErrorKind::Server, "Incomplete body"),
            Error::InvalidState => Self::new(ErrorKind::Server, "Invalid state"),
            Error::ConnectionClosed => Self::new(ErrorKind::Server, "Connection closed"),
            Error::HeadersMismatchError(e) => match e {
                HeadersMismatchError::ResponseConnectionTypeMismatchError =>
                Self::new(ErrorKind::Server, "Connection type mismatch: Keep-Alive connection type in the response, while the request contained a Close connection type"),
                HeadersMismatchError::BodyTypeError(e) => Self::new(ErrorKind::Server, e),
            }
            Error::WsUpgradeError(e) => match e {
                 UpgradeError::NoVersion => Self::new(ErrorKind::Server, "No `Sec-WebSocket-Version` header"),
                 UpgradeError::NoSecKey => Self::new(ErrorKind::Server, "No `Sec-WebSocket-Key` header"),
                 UpgradeError::UnsupportedVersion => Self::new(ErrorKind::Server, "Unsupported `Sec-WebSocket-Version`"),
            }
            Error::Io(e) => Self::from(e),
        }
    }
}

impl From<embassy_net::tcp::ConnectError> for Error {
    fn from(e: embassy_net::tcp::ConnectError) -> Self {
        use embassy_net::tcp::ConnectError;
        let err = match e {
            ConnectError::InvalidState => "The socket is already connected or listening",
            ConnectError::ConnectionReset => {
                "The remote host rejected the connection with a RST packet"
            }
            ConnectError::TimedOut => "Connect timed out",
            ConnectError::NoRoute => "No route to host",
        };
        Self::new(ErrorKind::Socket, err)
    }
}

impl From<embassy_executor::SpawnError> for Error {
    fn from(e: embassy_executor::SpawnError) -> Self {
        let err = match e {
            embassy_executor::SpawnError::Busy => "Busy",
        };
        Self::new(ErrorKind::SpawningTask, err)
    }
}

impl From<edge_nal_embassy::TcpError> for Error {
    fn from(e: edge_nal_embassy::TcpError) -> Self {
        use edge_nal_embassy::TcpError;
        use embassy_net::tcp::AcceptError;
        use embassy_net::tcp::Error;
        match e {
            TcpError::General(e) => match e {
                Error::ConnectionReset => Self::new(ErrorKind::Tcp, "The connection was reset"),
            },
            TcpError::Connect(e) => Self::from(e),
            TcpError::Accept(e) => match e {
                AcceptError::InvalidState => Self::new(
                    ErrorKind::Tcp,
                    "The socket is already connected or listening.",
                ),
                AcceptError::InvalidPort => Self::new(ErrorKind::Tcp, "Invalid listen port"),
                AcceptError::ConnectionReset => Self::new(
                    ErrorKind::Tcp,
                    "The remote host rejected the connection with a RST packet.",
                ),
            },
            TcpError::NoBuffers => Self::new(ErrorKind::Tcp, "No buffers available"),
            TcpError::UnsupportedProto => Self::new(ErrorKind::Tcp, "Unsupported protocol"),
        }
    }
}

impl From<esp_wifi::InitializationError> for Error {
    fn from(e: esp_wifi::InitializationError) -> Self {
        use esp_wifi::InitializationError;
        match e {
            InitializationError::General(_) => Self::new(ErrorKind::WiFi, "General error"),
            InitializationError::WifiError(e) => Self::from(e),
            InitializationError::WrongClockConfig => Self::new(
                ErrorKind::WiFi,
                "The current CPU clock frequency is too low",
            ),
            InitializationError::InterruptsDisabled => Self::new(
                ErrorKind::WiFi,
                "Tried to initialize while interrupts are disabled. This is not supported",
            ),
            _ => Self::new(ErrorKind::WiFi, "Unknown or still non-existent error"),
        }
    }
}

impl From<esp_wifi::wifi::WifiError> for Error {
    fn from(e: esp_wifi::wifi::WifiError) -> Self {
        use esp_wifi::wifi::InternalWifiError;
        use esp_wifi::wifi::WifiError;
        let err = match e {
            WifiError::NotInitialized => "Not initialized module",
            WifiError::InternalError(internal_wifi_error) => match internal_wifi_error {
                InternalWifiError::EspErrNoMem => "Internal: Out of memory",
                InternalWifiError::EspErrInvalidArg => "Internal: Invalid argument",
                InternalWifiError::EspErrWifiNotInit => "Internal: Wi-Fi driver was not installed",
                InternalWifiError::EspErrWifiNotStarted => "Internal: Wi-Fi driver was not started",
                InternalWifiError::EspErrWifiNotStopped => "Internal: Wi-Fi driver was not stopped",
                InternalWifiError::EspErrWifiIf => "Internal: Wi-Fi interface error",
                InternalWifiError::EspErrWifiMode => "Internal: Wi-Fi mode error",
                InternalWifiError::EspErrWifiState => "Internal: Wi-Fi internal state error",
                InternalWifiError::EspErrWifiConn => {
                    "Internal: Wi-Fi internal control block of station or soft-AP error"
                }
                InternalWifiError::EspErrWifiNvs => "Internal: Wi-Fi internal NVS module error",
                InternalWifiError::EspErrWifiMac => "Internal: MAC address is invalid",
                InternalWifiError::EspErrWifiSsid => "Internal: SSID is invalid",
                InternalWifiError::EspErrWifiPassword => "Internal: Password is invalid",
                InternalWifiError::EspErrWifiTimeout => "Internal: Timeout error",
                InternalWifiError::EspErrWifiWakeFail => {
                    "Internal: WiFi is in sleep state(RF closed) and wakeup fail"
                }
                InternalWifiError::EspErrWifiWouldBlock => "Internal: The caller would block",
                InternalWifiError::EspErrWifiNotConnect => {
                    "Internal: Station still in disconnect status"
                }
                InternalWifiError::EspErrWifiPost => {
                    "Internal: Failed to post the event to WiFi task"
                }
                InternalWifiError::EspErrWifiInitState => {
                    "Internal: Invalid WiFi state when init/deinit is called"
                }
                InternalWifiError::EspErrWifiStopState => {
                    "Internal: Returned when WiFi is stopping"
                }
                InternalWifiError::EspErrWifiNotAssoc => {
                    "Internal: The WiFi connection is not associated"
                }
                InternalWifiError::EspErrWifiTxDisallow => "Internal: The WiFi TX is disallowed",
            },
            WifiError::Disconnected => {
                "Device disconnected from the network or failed to connect to it"
            }
            WifiError::UnknownWifiMode => "Unknown Wi-Fi mode (not Sta/Ap/ApSta)",
            WifiError::Unsupported => "Unsupported operation or mode ",
            WifiError::InvalidArguments => "Invalid Arguments",
            _ => "Unknown or still non-existent error",
        };
        Self::new(ErrorKind::WiFi, err)
    }
}

/// A specialized [`Result`] type for [`Error`].
pub type Result<T> = core::result::Result<T, Error>;
