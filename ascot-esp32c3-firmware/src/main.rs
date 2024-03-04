//! HTTP Server with JSON POST handler
//!
//! Go to 192.168.71.1 to test

use anyhow::bail;

use core::convert::TryInto;

use std::thread::sleep;
use std::time::Duration;

use embedded_svc::{
    http::{Headers, Method},
    io::{Read, Write},
};

use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::peripheral,
    http::server::EspHttpServer,
    nvs::EspDefaultNvsPartition,
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};

use log::info;

use serde::Deserialize;

static INDEX_HTML: &str = include_str!("../http_server_page.html");

// Max payload length
const MAX_LEN: usize = 128;

// Need lots of stack to parse JSON
const STACK_SIZE: usize = 10240;

// Wi-Fi channel, between 1 and 11
//const CHANNEL: u8 = 11;

#[toml_cfg::toml_config]
pub struct WifiConfig {
    #[default("")]
    ssid: &'static str,
    #[default("")]
    password: &'static str,
}

#[derive(Deserialize)]
struct FormData<'a> {
    first_name: &'a str,
    age: u32,
    birthplace: &'a str,
}

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    // Retrieves Wi-Fi SSID and password.
    let wifi_config = WIFI_CONFIG;

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    // Connects to Wi-Fi
    let _wifi = connect_wifi(wifi_config, peripherals.modem, sys_loop, nvs)?;

    let server_configuration = esp_idf_svc::http::server::Configuration {
        stack_size: STACK_SIZE,
        ..Default::default()
    };

    let mut server = EspHttpServer::new(&server_configuration)?;

    server.fn_handler("/", Method::Get, |req| {
        req.into_ok_response()?
            .write_all(INDEX_HTML.as_bytes())
            .map(|_| ())
    })?;

    server.fn_handler::<anyhow::Error, _>("/post", Method::Post, |mut req| {
        let len = req.content_len().unwrap_or(0) as usize;

        if len > MAX_LEN {
            req.into_status_response(413)?
                .write_all("Request too big".as_bytes())?;
            return Ok(());
        }

        let mut buf = vec![0; len];
        req.read_exact(&mut buf)?;
        let mut resp = req.into_ok_response()?;

        if let Ok(form) = serde_json::from_slice::<FormData>(&buf) {
            write!(
                resp,
                "Hello, {}-year-old {} from {}!",
                form.age, form.first_name, form.birthplace
            )?;
        } else {
            resp.write_all("JSON error".as_bytes())?;
        }

        Ok(())
    })?;

    loop {
        sleep(Duration::from_millis(1000));
    }
}

fn connect_wifi(
    wifi_config: WifiConfig,
    modem: impl peripheral::Peripheral<P = esp_idf_svc::hal::modem::Modem> + 'static,
    sys_loop: EspSystemEventLoop,
    nvs: EspDefaultNvsPartition,
) -> anyhow::Result<EspWifi<'static>> {
    if wifi_config.ssid.is_empty() {
        bail!("Missing Wi-Fi SSID")
    }

    let auth_method = if wifi_config.password.is_empty() {
        info!("Wifi password is empty");
        AuthMethod::None
    } else {
        AuthMethod::WPA2Personal
    };

    let mut esp_wifi = EspWifi::new(modem, sys_loop.clone(), Some(nvs))?;
    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sys_loop)?;

    // Set default Wi-Fi configuration.
    wifi.set_configuration(&Configuration::Client(ClientConfiguration::default()))?;

    // Start Wi-Fi
    wifi.start()?;

    // Scan the network looking for all possible access points information.
    let ap_infos = wifi.scan()?;

    // Iter on all access points looking for the one associated with
    // the input SSID.
    let input_ap = ap_infos.into_iter().find(|a| a.ssid == wifi_config.ssid);

    let channel = if let Some(input_ap) = input_ap {
        info!(
            "Found input SSID {} on channel {}",
            wifi_config.ssid, input_ap.channel
        );
        Some(input_ap.channel)
    } else {
        info!(
            "Input access point {} not found during the scanning process, an unknown channel will be used.",
            wifi_config.ssid
        );
        None
    };

    // Configure Wi-FI
    let wifi_configuration = Configuration::Client(ClientConfiguration {
        ssid: wifi_config.ssid.try_into().unwrap(),
        password: wifi_config.password.try_into().unwrap(),
        auth_method,
        channel,
        ..Default::default()
    });

    // Sets the Wi-Fi configuration in order to connect to the access point.
    wifi.set_configuration(&wifi_configuration)?;

    // Connects to Wi-Fi
    wifi.connect()?;

    // DHCP lease phase
    wifi.wait_netif_up()?;

    // Gets the firmware IP information
    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;

    info!("Wifi DHCP info: {:?}", ip_info);

    Ok(esp_wifi)
}
