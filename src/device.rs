use alloc::borrow::Cow;

use heapless::Vec;

use serde::{Deserialize, Serialize};

use crate::route::RouteConfig;
use crate::MAXIMUM_ELEMENTS;

/// Device data.
#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceData<'a> {
    /// Device kind.
    pub kind: DeviceKind,
    /// Main device route.
    #[serde(rename = "main route")]
    pub main_route: Cow<'a, str>,
    /// All routes configurations for a device.
    pub routes_configs: Vec<RouteConfig<'a>, MAXIMUM_ELEMENTS>,
}

/// A [`Device`] kind.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DeviceKind {
    /// Light.
    Light,
    /// Fridge.
    Fridge,
}

/// A trait to serialize device data.
pub trait DeviceSerializer<'a> {
    /// Serializes device data.
    fn serialize_data(&self) -> DeviceData<'a>;
}
