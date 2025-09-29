use core::net::Ipv4Addr;

use alloc::boxed::Box;

use esp_hal::rng::Rng;

use esp_wifi::wifi::WifiDevice;

use embassy_executor::Spawner;
use embassy_net::{Config, DhcpConfig, Runner, Stack, StackResources};
use embassy_time::Timer;

use log::info;

use crate::error::Result;

const MILLISECONDS_TO_WAIT: u64 = 100;

/// Retrieves the [`Ipv4Addr`] from the network stack.
#[inline]
pub async fn get_ip(stack: Stack<'static>) -> Ipv4Addr {
    info!("Waiting till the link is up...");
    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after_millis(MILLISECONDS_TO_WAIT).await;
    }

    info!("Waiting to get IP address...");
    loop {
        if let Some(config) = stack.config_v4() {
            info!("Got IP: {}", config.address);
            return config.address.address();
        }
        Timer::after_millis(MILLISECONDS_TO_WAIT).await;
    }
}

#[embassy_executor::task]
async fn task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await;
}

/// The network stack builder.
pub struct NetworkStack;

impl NetworkStack {
    /// Builds a [`NetworkStack`] .
    ///
    /// # Errors
    ///
    /// Failure to spawn the network stack task.
    pub fn build<const SOCKET_STACK_SIZE: usize>(
        mut rng: Rng,
        wifi_interface: WifiDevice<'static>,
        spawner: Spawner,
    ) -> Result<Stack<'static>> {
        let config = Config::dhcpv4(DhcpConfig::default());
        let seed = u64::from(rng.random()) << 32 | u64::from(rng.random());

        // FIXME: We need to use `Box::leak` and then `Box::new` because
        // `make_static` does not accept **ANY** kind of generic, not even const
        // generics.
        let resources = Box::leak(Box::new(StackResources::<SOCKET_STACK_SIZE>::new()));

        let (stack, runner) = embassy_net::new(wifi_interface, config, resources, seed);

        spawner.spawn(task(runner))?;

        Ok(stack)
    }
}
