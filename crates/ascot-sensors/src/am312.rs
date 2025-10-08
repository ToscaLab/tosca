//! # AM312 Driver
//!
//! This crate provides an asynchronous, architecture-agnostic driver for the AM312 PIR motion sensor,
//! which signals motion via a digital pin:
//! - **High** when movement is detected.
//! - **Low** when no movement is detected.
//!
//! After power-on, the AM312 requires a calibration period of typically 10 to 60 seconds before motion readings are reliable.
//! Therefore, make sure to wait for this period before calling any motion detection methods.
//!
//! For detailed information and specifications, see the [datasheet](https://www.alldatasheet.com/datasheet-pdf/pdf/1179499/ETC2/AM312.html).

use core::result::Result;

use embedded_hal::digital::InputPin;

use embedded_hal_async::digital::Wait;
use embedded_hal_async::delay::DelayNs;

/// AM312 driver.
pub struct Am312<P, D>
where
        P: InputPin + Wait,
        D: DelayNs,
    {
        pin: P,
        delay: D
    }

impl<P, D> Am312<P, D>
where
    P: InputPin + Wait,
    D: DelayNs,
{
    const DEBOUNCE_MS: u32 = 50;

    /// Creates a new [`Am312`] driver with the given input pin.
    #[must_use]
    #[inline]
    pub async fn new(pin: P, delay: D) -> Self {
        Self {
            pin,
            delay
        }
    }

    /// Waits until motion is detected.
    pub async fn wait_for_motion_start(&mut self) -> Result<(), P::Error> {
        loop {
            self.pin.wait_for_rising_edge().await?;

            // Debounce.
            self.delay.delay_ms(Self::DEBOUNCE_MS).await;

            if self.pin.is_high()? {
                return Ok(());
            }
        }
    }

    /// Waits until motion ends.
    pub async fn wait_for_motion_end(&mut self) -> Result<(), P::Error> {
        loop {
            self.pin.wait_for_falling_edge().await?;

            // Debounce.
            self.delay.delay_ms(Self::DEBOUNCE_MS).await;

            if self.pin.is_low()? {
                return Ok(());
            }
        }
    }

    /// Returns whether motion is currently detected.
    #[must_use]
    #[inline]
    pub fn is_motion_detected(&mut self) -> Result<bool, P::Error> {
        self.pin.is_high()
    }
}
