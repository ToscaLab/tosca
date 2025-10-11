use alloc::string::String;
use alloc::vec::Vec;

use log::info;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Device {
    kind: String,
}

#[derive(Deserialize, Debug)]
struct MandatoryRoute {
    path: String,
    hazards: Vec<String>,
    parameters: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct Mandatory {
    on: MandatoryRoute,
}

#[derive(Deserialize, Debug)]
struct Parameter {
    kind: String,
    default: f64,
    min: f64,
    max: f64,
}

#[derive(Deserialize, Debug)]
struct Parameters {
    brightness: Parameter,
}

#[derive(Deserialize, Debug)]
struct LocalizedText {
    #[serde(rename = "device.name")]
    device_name: String,
    #[serde(rename = "device.description")]
    device_description: String,
    #[serde(rename = "on.name")]
    on_name: String,
    #[serde(rename = "on.description")]
    on_description: String,
    #[serde(rename = "brightness.name")]
    brightness_name: String,
}

/// A device configuration.
#[derive(Deserialize, Debug)]
pub struct Config {
    device: Device,
    mandatory: Mandatory,
    parameters: Parameters,
    en: LocalizedText,
    it: LocalizedText,
}

/// Parse the configuration file.
pub fn parse_config(yaml_str: &str) {}
