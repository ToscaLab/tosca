//! # DHT22 Driver
//!
//! This crate provides an architecture-agnostic driver for the DHT22 temperature and humidity sensor.
//! The driver is primarily synchronous to meet the strict timing requirements of the custom sensor's single-wire protocol.
//! Only the initial start signal uses a brief asynchronous wait to initiate communication without blocking the executor,
//! while all subsequent timing-critical operations are handled using precise blocking delays to ensure correct measurements.
//!
//! The DHT22 provides measurements of:
//! - **Humidity** in relative humidity percentage (%RH)
//! - **Temperature** in degrees Celsius (°C)
//!
//! For detailed information and specifications, see the [datasheet](https://www.alldatasheet.com/datasheet-pdf/pdf/1132459/ETC2/DHT22.html)
//! and a description of the proprietary [communication protocol](https://www.ocfreaks.com/basics-interfacing-dht11-dht22-humidity-temperature-sensor-mcu/).

use core::result::Result::{self, Ok, Err};

use embedded_hal::digital::{InputPin, OutputPin, PinState};
use embedded_hal::delay::DelayNs as SyncDelay;

use embedded_hal_async::delay::DelayNs as AsyncDelay;

/// Represents a single humidity and temperature measurement.
#[derive(Debug, Clone, Copy)]
pub struct Measurement {
    /// Relative humidity in RH%.
    pub humidity: f32,
    /// Temperature in °C.
    pub temperature: f32,
}

/// Errors that may occur while interacting with the DHT22 sensor.
#[derive(Debug)]
pub enum Dht22Error<E> {
    /// GPIO pin errors.
    Pin(E),
    /// Data checksum mismatch.
    ChecksumMismatch,
    /// Timeout waiting for sensor response.
    Timeout,
}

impl<E> From<E> for Dht22Error<E> {
    fn from(e: E) -> Self {
        Dht22Error::Pin(e)
    }
}

/// DHT22 driver.
pub struct Dht22<P, D>
where
    P: InputPin + OutputPin,
    D: SyncDelay + AsyncDelay,
{
    pin: P,
    delay: D,
}

impl<P, D> Dht22<P, D>
where
    P: InputPin + OutputPin,
    D: SyncDelay + AsyncDelay,
{
    // Protocol-specific timing constants.
    const START_SIGNAL_LOW_MS: u32 = 18;    // MCU pulls line low for at least 18 ms to initiate communication.
    const START_SIGNAL_HIGH_US: u32 = 40;   // Then releases the line (high) for ~20–40 µs.
    const BIT_SAMPLE_DELAY_US: u32 = 35;    // Time after which to sample the data bit.
    const POLL_DELAY_US: u32 = 1;           // Delay between pin state polls when waiting for edges.
    const MAX_ATTEMPTS: usize = 100;        // Maximum polling iterations before timeout.

    /// Creates a new [`Dht22`] driver with the given pin and delay provider.
    #[must_use]
    pub fn new(pin: P, delay: D) -> Self {
        Self { pin, delay }
    }

    /// Reads a single humidity and temperature measurement.
    #[must_use]
    pub fn read(&mut self) -> Result<Measurement, Dht22Error<P::Error>> {
        // Initiate communication by sending the start signal to the sensor.
        self.send_start_signal()?;

        // Wait for the sensor’s response (low → high handshake).
        self.wait_for_sensor_response()?;

        // Read 5 bytes: humidity high + low, temperature high + low, and checksum.
        let (hh, hl, th, tl, checksum) = self.read_raw_data()?;

        // Validate that the transmitted checksum matches the calculated one.
        Self::validate_checksum(hh, hl, th, tl, checksum)?;

        Ok(Measurement {
            humidity: Self::decode_humidity(hh, hl),
            temperature: Self::decode_temperature(th, tl),
        })
    }

    fn send_start_signal(&mut self) -> Result<(), Dht22Error<P::Error>> {
        // Pull the line low for at least 18 ms to signal the sensor.
        self.pin.set_low()?;
        SyncDelay::delay_ms(&mut self.delay, Self::START_SIGNAL_LOW_MS);

        // Release the line high briefly before the sensor takes control of it.
        self.pin.set_high()?;
        SyncDelay::delay_us(&mut self.delay, Self::START_SIGNAL_HIGH_US);

        Ok(())
    }

    fn wait_for_sensor_response(&mut self) -> Result<(), Dht22Error<P::Error>> {
        // The sensor pulls the line low and then high to acknowledge.
        self.wait_until_state(PinState::Low)?;
        self.wait_until_state(PinState::High)?;

        Ok(())
    }

    fn read_raw_data(&mut self) -> Result<(u8, u8, u8, u8, u8), Dht22Error<P::Error>> {
        // Sequentially read 5 bytes from the sensor.
        Ok((
            self.read_byte()?,
            self.read_byte()?,
            self.read_byte()?,
            self.read_byte()?,
            self.read_byte()?,
        ))
    }

    #[inline]
    fn validate_checksum(hh: u8, hl: u8, th: u8, tl: u8, checksum: u8) -> Result<(), Dht22Error<P::Error>> {
        // The checksum is the low 8 bits of the sum of the first four bytes.
        let sum = hh.wrapping_add(hl).wrapping_add(th).wrapping_add(tl);

        if sum != checksum {
            Err(Dht22Error::ChecksumMismatch)
        } else {
            Ok(())
        }
    }

    #[inline]
    fn decode_humidity(high: u8, low: u8) -> f32 {
        // Combine two bytes into a 16-bit integer and divide by 10 (sensor sends humidity * 10).
        ((high as u16) << 8 | low as u16) as f32 / 10.0
    }

    #[inline]
    fn decode_temperature(high: u8, low: u8) -> f32 {
        // The 16-bit temperature value has its sign bit at bit 15 (high byte’s MSB).
        let mut t = (((high & 0x7F) as u16) << 8 | low as u16) as f32 / 10.0;

        // If the sign bit is set, temperature is negative.
        if high & 0x80 != 0 {
            t = -t;
        }

        t
    }

    fn wait_until_state(&mut self, state: PinState) -> Result<(), Dht22Error<P::Error>> {
        // Poll the pin until it matches the desired state or timeout occurs.
        for _ in 0..Self::MAX_ATTEMPTS {
            let reached = match state {
                PinState::High => self.pin.is_high()?,
                PinState::Low => self.pin.is_low()?,
            };
            if reached {
                return Ok(());
            }
            SyncDelay::delay_us(&mut self.delay, Self::POLL_DELAY_US);
        }

        Err(Dht22Error::Timeout)
    }

    fn read_byte(&mut self) -> Result<u8, Dht22Error<P::Error>> {
        let mut byte = 0;

        // Each bit transmission consists of a low pulse followed by a high pulse.
        // The duration of the high pulse determines whether the bit is 0 or 1.
        for i in 0..8 {
            self.wait_until_state(PinState::Low)?;  // Wait for the start of bit transmission.
            self.wait_until_state(PinState::High)?; // Wait for the high phase.

            // Sample after ~30 µs to determine bit value.
            SyncDelay::delay_us(&mut self.delay, Self::BIT_SAMPLE_DELAY_US);

            // If the line is still high, the bit is 1; otherwise, it's 0.
            if self.pin.is_high()? {
                byte |= 1 << (7 - i); // Bits are transmitted MSB first.
            }
        }

        Ok(byte)
    }
}
