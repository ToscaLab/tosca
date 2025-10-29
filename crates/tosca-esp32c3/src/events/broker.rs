use embassy_net::IpAddress;

/// A network packet broker data.
pub enum BrokerData {
    /// URL and port.
    Url(&'static str, u16),

    /// [`IpAddress`] and port.
    Ip(IpAddress, u16),
}

impl BrokerData {
    /// Creates a [`BrokerData`] from `URL` and `port`.
    #[must_use]
    pub const fn url(url: &'static str, port: u16) -> Self {
        Self::Url(url, port)
    }

    /// Creates a [`BrokerData`] from [`IpAddress`] and `port`.
    #[must_use]
    pub const fn ip(ip: IpAddress, port: u16) -> Self {
        Self::Ip(ip, port)
    }
}
