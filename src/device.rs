use serde::{Deserialize, Serialize};

use crate::economy::Economy;
use crate::energy::Energy;

/// A device kind.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DeviceKind {
    /// Unknown.
    Unknown,
    /// Light.
    Light,
    /// Fridge.
    Fridge,
    /// Camera.
    Camera,
}

impl DeviceKind {
    const fn description(self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::Light => "Light",
            Self::Fridge => "Fridge",
            Self::Camera => "Camera",
        }
    }
}

impl core::fmt::Display for DeviceKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Device information.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Energy information.
    #[serde(skip_serializing_if = "Energy::is_empty")]
    #[serde(default = "Energy::empty")]
    pub energy: Energy,
    /// Economy information.
    #[serde(skip_serializing_if = "Economy::is_empty")]
    #[serde(default = "Economy::empty")]
    pub economy: Economy,
}

impl DeviceInfo {
    /// Creates a [`DeviceInfo`].
    #[must_use]
    pub fn empty() -> Self {
        Self {
            energy: Energy::empty(),
            economy: Economy::empty(),
        }
    }

    /// Adds [`Energy`] data.
    #[must_use]
    pub fn add_energy(mut self, energy: Energy) -> Self {
        self.energy = energy;
        self
    }

    /// Adds [`Economy`] data.
    #[must_use]
    pub fn add_economy(mut self, economy: Economy) -> Self {
        self.economy = economy;
        self
    }
}

#[cfg(feature = "std")]
mod device_data {
    use crate::route::RouteConfigs;

    use super::{Deserialize, DeviceKind, Serialize};

    /// Device data.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct DeviceData {
        /// Device kind.
        pub kind: DeviceKind,
        /// Device main route.
        #[serde(rename = "main route")]
        pub main_route: alloc::borrow::Cow<'static, str>,
        /// All device route configurations.
        pub route_configs: RouteConfigs,
    }
}

#[cfg(not(feature = "std"))]
mod device_data {
    use crate::route::RouteConfigs;

    use super::{DeviceKind, Serialize};

    /// Device data.
    #[derive(Debug, Serialize)]
    pub struct DeviceData {
        /// Device kind.
        pub kind: DeviceKind,
        /// Device main route.
        #[serde(rename = "main route")]
        pub main_route: &'static str,
        /// All device route configurations.
        pub route_configs: RouteConfigs,
    }
}

pub use device_data::DeviceData;

/// A trait to serialize device data.
pub trait DeviceSerializer {
    /// Serializes device data.
    fn serialize_data(&self) -> DeviceData;
}
