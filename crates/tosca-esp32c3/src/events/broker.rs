use embassy_net::IpAddress;

/// Broker data.
pub enum BrokerData {
    /// Broker `URL` and `port`.
    Url(&'static str, u16),

    /// Broker [`IpAddress`] and `port`.
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
