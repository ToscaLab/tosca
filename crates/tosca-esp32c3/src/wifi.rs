use embassy_executor::Spawner;

use esp_hal::peripherals::WIFI;

use esp_radio::Controller;
use esp_radio::wifi::{
    ClientConfig, Config, Interfaces, ModeConfig, WifiController, WifiEvent, WifiStaState,
    sta_state,
};

use log::{error, info};

use crate::error::{Error, ErrorKind, Result};
use crate::mk_static;

pub(crate) const WIFI_RECONNECT_DELAY: u64 = 2;

/// The `Wi-Fi` controller.
pub struct Wifi {
    _esp_wifi_controller: &'static Controller<'static>,
    controller: WifiController<'static>,
    interfaces: Interfaces<'static>,
    spawner: Spawner,
}

impl Wifi {
    /// Configures the [`Wifi`] controller with the given parameters.
    ///
    /// # Errors
    ///
    /// Unable to initialize the `Wi-Fi` controller and retrieve the
    /// corresponding network interfaces.
    pub fn configure(peripherals_wifi: WIFI<'static>, spawner: Spawner) -> Result<Self> {
        let esp_wifi_controller = &*mk_static!(Controller<'static>, esp_radio::init()?);
        let (controller, interfaces) =
            esp_radio::wifi::new(esp_wifi_controller, peripherals_wifi, Config::default())?;

        Ok(Self {
            _esp_wifi_controller: esp_wifi_controller,
            controller,
            interfaces,
            spawner,
        })
    }

    /// Connects a device to a `Wi-Fi` access point.
    ///
    /// # Errors
    ///
    /// - Missing `Wi-Fi` SSID
    /// - Missing `Wi-Fi` password
    /// - Failure to set up the `Wi-Fi` configuration
    /// - Failure to spawn the task to connect the device to the access point
    ///   via `Wi-Fi`.
    pub async fn connect(mut self, ssid: &str, password: &str) -> Result<Interfaces<'static>> {
        if ssid.is_empty() {
            return Err(Error::new(ErrorKind::WiFi, "Missing Wi-Fi SSID"));
        }

        if password.is_empty() {
            return Err(Error::new(ErrorKind::WiFi, "Missing Wi-Fi password"));
        }

        let client_config = ModeConfig::Client(
            ClientConfig::default()
                .with_ssid(ssid.into())
                .with_password(password.into()),
        );

        self.controller.set_config(&client_config)?;

        self.spawner.spawn(connect(self.controller))?;

        // Wait until Wi-Fi is connected.
        while sta_state() != WifiStaState::Connected {
            embassy_time::Timer::after_millis(100).await;
        }

        Ok(self.interfaces)
    }
}

#[embassy_executor::task]
async fn connect(mut wifi_controller: WifiController<'static>) {
    info!("Wi-Fi connection task started");
    loop {
        if sta_state() == WifiStaState::Connected {
            wifi_controller
                .wait_for_event(WifiEvent::StaDisconnected)
                .await;
            embassy_time::Timer::after_secs(WIFI_RECONNECT_DELAY).await;
        }

        if !matches!(wifi_controller.is_started(), Ok(true)) {
            info!("Starting Wi-Fi...");
            wifi_controller
                .start_async()
                .await
                .map_err(Error::from)
                .expect("Impossible to start Wi-Fi");
            info!("Wi-Fi started");
        }

        info!("Attempting to connect...");
        if let Err(e) = wifi_controller.connect_async().await {
            error!("Wi-Fi connect failed: {e:?}");
            embassy_time::Timer::after_secs(WIFI_RECONNECT_DELAY).await;
        } else {
            info!("Wi-Fi connected!");
        }
    }
}
