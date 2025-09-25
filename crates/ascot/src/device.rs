use serde::{Deserialize, Serialize};

use crate::economy::Economy;
use crate::energy::Energy;
use crate::route::RouteConfigs;

/// A device kind.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DeviceKind {
    /// Unknown.
    Unknown,
    /// Light.
    Light,
    /// Camera.
    Camera,
}

impl DeviceKind {
    const fn description(self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::Light => "Light",
            Self::Camera => "Camera",
        }
    }
}

impl core::fmt::Display for DeviceKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.description())
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

/// Device data.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct DeviceData {
    /// Device kind.
    pub kind: DeviceKind,
    /// Device environment.
    pub environment: DeviceEnvironment,
    /// Device description.
    pub description: Option<alloc::borrow::Cow<'static, str>>,
    /// Device main route.
    #[serde(rename = "main route")]
    pub main_route: alloc::borrow::Cow<'static, str>,
    /// All device route configurations.
    pub route_configs: RouteConfigs,
    /// Wi-Fi MAC address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wifi_mac: Option<[u8; 6]>,
    /// Ethernet MAC address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ethernet_mac: Option<[u8; 6]>,
    /// Number of mandatory routes.
    pub mandatory_routes: u8,
}

impl DeviceData {
    /// Creates a [`DeviceData`].
    #[must_use]
    pub fn new(
        kind: DeviceKind,
        environment: DeviceEnvironment,
        main_route: impl Into<alloc::borrow::Cow<'static, str>>,
        route_configs: RouteConfigs,
        wifi_mac: Option<[u8; 6]>,
        ethernet_mac: Option<[u8; 6]>,
        mandatory_routes: u8,
    ) -> Self {
        Self {
            kind,
            environment,
            description: None,
            main_route: main_route.into(),
            route_configs,
            wifi_mac,
            ethernet_mac,
            mandatory_routes,
        }
    }

    /// Sets the device description.
    #[must_use]
    pub fn description(mut self, description: impl Into<alloc::borrow::Cow<'static, str>>) -> Self {
        self.description = Some(description.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::collections::OutputSet;
    use crate::route::{Route, RouteConfigs};

    use crate::economy::{Cost, CostTimespan, Economy, Roi};
    use crate::energy::{
        CarbonFootprint, Energy, EnergyClass, EnergyEfficiency, WaterUseEfficiency,
    };
    use crate::{deserialize, serialize};

    use super::{DeviceData, DeviceEnvironment, DeviceInfo, DeviceKind};

    fn energy() -> Energy {
        let energy_efficiencies = OutputSet::init(EnergyEfficiency::new(-50, EnergyClass::A))
            .insert(EnergyEfficiency::new(50, EnergyClass::B));

        let carbon_footprints = OutputSet::init(CarbonFootprint::new(-50, EnergyClass::A))
            .insert(CarbonFootprint::new(50, EnergyClass::B));

        let water_use_efficiency = WaterUseEfficiency::init_with_gpp(2.5)
            .penman_monteith_equation(3.2)
            .wer(1.1);

        Energy::init_with_energy_efficiencies(energy_efficiencies)
            .carbon_footprints(carbon_footprints)
            .water_use_efficiency(water_use_efficiency)
    }

    fn economy() -> Economy {
        let costs = OutputSet::init(Cost::new(100, CostTimespan::Week))
            .insert(Cost::new(1000, CostTimespan::Month));

        let roi =
            OutputSet::init(Roi::new(10, EnergyClass::A)).insert(Roi::new(20, EnergyClass::B));

        Economy::init_with_costs(costs).roi(roi)
    }

    fn routes() -> RouteConfigs {
        OutputSet::init(Route::put("On", "/on").serialize_data())
            .insert(Route::put("Off", "/off").serialize_data())
    }

    #[test]
    fn test_device_kind() {
        for device_kind in &[DeviceKind::Unknown, DeviceKind::Light, DeviceKind::Camera] {
            assert_eq!(
                deserialize::<DeviceKind>(serialize(device_kind)),
                *device_kind
            );
        }
    }

    #[test]
    fn test_device_environment() {
        for device_environment in &[DeviceEnvironment::Os, DeviceEnvironment::Esp32] {
            assert_eq!(
                deserialize::<DeviceEnvironment>(serialize(device_environment)),
                *device_environment
            );
        }
    }

    #[test]
    fn test_device_info() {
        let mut device_info = DeviceInfo::empty();

        device_info = device_info.add_energy(energy()).add_economy(economy());

        assert_eq!(
            deserialize::<DeviceInfo>(serialize(&device_info)),
            device_info
        );
    }

    #[test]
    fn test_device_data() {
        let device_data = DeviceData::new(
            DeviceKind::Light,
            DeviceEnvironment::Os,
            "/light",
            routes(),
            None,
            None,
            2,
        )
        .description("A light device.");

        assert_eq!(
            deserialize::<DeviceData>(serialize(&device_data)),
            device_data
        );
    }
}
