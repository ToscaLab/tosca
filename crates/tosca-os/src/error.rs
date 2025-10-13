use std::borrow::Cow;

use tosca::device::DeviceKind;

/// All possible error kinds.
#[derive(Debug, Copy, Clone)]
pub enum ErrorKind {
    /// Service error.
    Service,
    /// Not found address.
    NotFoundAddress,
    /// Serialize/Deserialize error.
    Serialization,
    /// A device error.
    Device,
    /// External error.
    ///
    /// An error caused by an external dependency.
    External,
}

impl ErrorKind {
    pub(crate) const fn description(self) -> &'static str {
        match self {
            Self::Service => "Service",
            Self::NotFoundAddress => "Not Found Address",
            Self::Serialization => "Serialization",
            Self::Device => "Device",
            Self::External => "External",
        }
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// A firmware error.
pub struct Error {
    kind: ErrorKind,
    description: Cow<'static, str>,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f)
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f)
    }
}

impl Error {
    /// Creates an [`Error`] from an [`ErrorKind`] and a description.
    pub fn new(kind: ErrorKind, description: impl Into<Cow<'static, str>>) -> Self {
        Self {
            kind,
            description: description.into(),
        }
    }

    /// Creates an [`Error`] for [`ErrorKind::External`] with a specific
    /// description.
    pub fn external(description: impl Into<Cow<'static, str>>) -> Self {
        Self::new(ErrorKind::External, description)
    }

    pub(crate) fn device(
        device_type: DeviceKind,
        description: impl Into<Cow<'static, str>>,
    ) -> Self {
        let description = description.into();
        Self::new(
            ErrorKind::Device,
            format!("{description} [{device_type} Device]"),
        )
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.kind)?;
        write!(f, "Cause: {}", self.description)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::new(ErrorKind::Serialization, e.to_string())
    }
}

/// A specialized [`Result`] type for [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use tosca::device::DeviceKind;

    use super::Error;

    #[test]
    fn device_error() {
        let error = Error::device(DeviceKind::Light, "This hazard is not correct");
        assert_eq!(
            error.to_string(),
            r"Device
Cause: This hazard is not correct [Light Device]"
        );
    }
}
