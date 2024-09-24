use alloc::borrow::Cow;

/// All possible error kinds.
#[derive(Copy, Clone)]
pub enum ErrorKind {
    /// Wi-Fi connection error.
    WiFi,
    /// Esp32-C3 internal error.
    Esp32C3,
    /// Esp32-C3 input/output error.
    Esp32C3IO,
    /// Service error.
    Service,
    /// Serialize/Deserialize error.
    Serialization,
    /// Light error.
    Light,
}

impl ErrorKind {
    pub(crate) const fn description(self) -> &'static str {
        match self {
            ErrorKind::WiFi => "Wi-Fi",
            ErrorKind::Esp32C3 => "Esp32-C3 internal error",
            ErrorKind::Esp32C3IO => "Esp32-C3 input/output",
            ErrorKind::Service => "Service",
            ErrorKind::Serialization => "Serialization",
            ErrorKind::Light => "Light device",
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

/// Library error.
pub struct Error {
    kind: ErrorKind,
    info: Cow<'static, str>,
}

impl core::fmt::Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.error().fmt(f)
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.error().fmt(f)
    }
}

impl Error {
    pub(crate) fn new(kind: ErrorKind, info: impl Into<Cow<'static, str>>) -> Self {
        Self {
            kind,
            info: info.into(),
        }
    }

    pub(crate) fn error(&self) -> String {
        format!("{}: {}", self.kind, self.info)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::new(ErrorKind::Serialization, e.to_string())
    }
}

impl From<esp_idf_svc::hal::sys::EspError> for Error {
    fn from(e: esp_idf_svc::hal::sys::EspError) -> Self {
        Self::new(ErrorKind::Esp32C3, e.to_string())
    }
}

impl From<esp_idf_svc::io::EspIOError> for Error {
    fn from(e: esp_idf_svc::io::EspIOError) -> Self {
        Self::new(ErrorKind::Esp32C3IO, e.to_string())
    }
}

impl<E: core::fmt::Debug> From<edge_mdns::io::MdnsIoError<E>> for Error {
    fn from(e: edge_mdns::io::MdnsIoError<E>) -> Self {
        Self::new(ErrorKind::Service, format!("{:?}", e))
    }
}

/// A specialized [`Result`] type for [`Error`].
pub type Result<T> = core::result::Result<T, Error>;
