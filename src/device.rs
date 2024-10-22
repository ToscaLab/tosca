use serde::{Deserialize, Serialize};

use crate::route::RouteConfigs;

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

/// Device data.
#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceData<'a> {
    /// Device kind.
    pub kind: DeviceKind,
    /// Device main route.
    #[serde(rename = "main route")]
    pub main_route: &'a str,
    #[serde(borrow)]
    /// All device route configurations.
    pub route_configs: RouteConfigs<'a>,
}

/// A trait to serialize device data.
pub trait DeviceSerializer {
    /// Serializes device data.
    fn serialize_data(&self) -> DeviceData;
}
