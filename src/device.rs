use serde::{Deserialize, Serialize};

use crate::economy::Economy;
use crate::energy::Energy;

/// A device kind.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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

/// Device environment.
///
/// Some information about the device environment on which a firmware runs on.
/// It might be an operating system or the name of the underlying hardware
/// architecture.
///
/// This enumerator allows to discriminate the different implementations among
/// the supported architectures on a controller side.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DeviceEnvironment {
    /// Operating system.
    Os,
    /// Esp32.
    Esp32,
}

#[cfg(feature = "alloc")]
mod device_data {
    use crate::route::RouteConfigs;

    use super::{Deserialize, DeviceEnvironment, DeviceKind, Serialize};

    /// Device data.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct DeviceData {
        /// Device kind.
        pub kind: DeviceKind,
        /// Device environment.
        pub environment: DeviceEnvironment,
        /// Device main route.
        #[serde(rename = "main route")]
        pub main_route: alloc::borrow::Cow<'static, str>,
        /// All device route configurations.
        pub route_configs: RouteConfigs,
    }

    impl DeviceData {
        /// Creates a [`DeviceData`].
        pub fn new(
            kind: DeviceKind,
            environment: DeviceEnvironment,
            main_route: impl Into<alloc::borrow::Cow<'static, str>>,
            route_configs: RouteConfigs,
        ) -> Self {
            Self {
                kind,
                environment,
                main_route: main_route.into(),
                route_configs,
            }
        }
    }
}

#[cfg(not(feature = "alloc"))]
mod device_data {
    use crate::route::RouteConfigs;

    use super::{DeviceEnvironment, DeviceKind, Serialize};

    /// Device data.
    #[derive(Debug, Serialize)]
    pub struct DeviceData {
        /// Device kind.
        pub kind: DeviceKind,
        /// Device environment.
        pub environment: DeviceEnvironment,
        /// Device main route.
        #[serde(rename = "main route")]
        pub main_route: &'static str,
        /// All device route configurations.
        pub route_configs: RouteConfigs,
    }

    impl DeviceData {
        /// Creates a [`DeviceData`].
        pub fn new(
            kind: DeviceKind,
            environment: DeviceEnvironment,
            main_route: &'static str,
            route_configs: RouteConfigs,
        ) -> Self {
            Self {
                kind,
                environment,
                main_route,
                route_configs,
            }
        }
    }
}

pub use device_data::DeviceData;
