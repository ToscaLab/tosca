//! # DS18B20 Driver
//!
//! This crate provides a synchronous, architecture-agnostic driver for the DS18B20 digital temperature sensor.
//! The driver is synchronous due to the device’s strict timing requirements.
//!
//! The DS18B20 communicates over the 1-Wire bus and provides temperature readings with resolutions up to 12 bits.
//! It performs temperature conversions internally and exposes the result via its scratchpad memory, which includes a CRC
//! to ensure data integrity. The driver operates in *single-sensor mode*, using the `Skip ROM` command to address the
//! device directly without specifying its unique 64-bit ROM code — an approach suitable when only one DS18B20 is connected
//! to the bus.
//!
//! For detailed information and specifications, see the [datasheet](https://www.alldatasheet.com/datasheet-pdf/pdf/58557/DALLAS/DS18B20.html).

use core::result::Result;

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};

/// Errors that may occur while interacting with the DS18B20 sensor.
#[derive(Debug)]
pub enum Ds18b20Error<E> {
    /// GPIO pin I/O error.
    Pin(E),
    /// Data integrity error (CRC check failed).
    CrcMismatch,
    /// No presence pulse detected (sensor not found on bus).
    NoPresence,
}

impl<E> From<E> for Ds18b20Error<E> {
    fn from(e: E) -> Self {
        Ds18b20Error::Pin(e)
    }
}

/// DS18B20 driver.
pub struct Ds18b20<P, D>
where
    P: InputPin + OutputPin,
    D: DelayNs,
{
    pin: P,
    delay: D,
}

impl<P, D> Ds18b20<P, D>
where
    P: InputPin + OutputPin,
    D: DelayNs,
{
    // 1-Wire protocol timing constants.
    const RESET_LOW_US: u32 = 480;
    const PRESENCE_WAIT_US: u32 = 70;
    const PRESENCE_RELEASE_US: u32 = 410;

    const WRITE_1_LOW_US: u32 = 6;
    const WRITE_1_HIGH_US: u32 = 64;
    const WRITE_0_LOW_US: u32 = 60;
    const WRITE_0_HIGH_US: u32 = 10;

    const READ_INIT_LOW_US: u32 = 6;
    const READ_SAMPLE_US: u32 = 9;
    const READ_RECOVERY_US: u32 = 55;

    const CONVERSION_WAIT_MS: u32 = 750; // Max conversion time at 12-bit resolution.

    // DS18B20 ROM and function commands.
    const CMD_SKIP_ROM: u8 = 0xCC;
    const CMD_CONVERT_T: u8 = 0x44;
    const CMD_READ_SCRATCHPAD: u8 = 0xBE;

    // Temperature resolution of the DS18B20 sensor.
    // Each bit in the 12-bit temperature reading corresponds to 0.0625 °C.
    const TEMPERATURE_RESOLUTION_C_PER_LSB: f32 = 0.0625;

    /// Creates a new [`Ds18b20`] driver with the given pin and delay provider.
    #[must_use]
    pub fn new(pin: P, delay: D) -> Self {
        Self { pin, delay }
    }

    /// Performs a bus reset and checks for the presence pulse from the sensor.
    ///
    /// Returns `Ok(true)` if a sensor is detected, `Ok(false)` otherwise.
    pub fn reset(&mut self) -> Result<bool, Ds18b20Error<P::Error>> {
        self.pin.set_low()?;
        self.delay.delay_us(Self::RESET_LOW_US);

        self.pin.set_high()?;
        self.delay.delay_us(Self::PRESENCE_WAIT_US);

        // Sensor should pull the line low to indicate presence.
        let present = self.pin.is_low()?;
        self.delay.delay_us(Self::PRESENCE_RELEASE_US);

        Ok(present)
    }

    /// Performs a full temperature measurement sequence:
    /// 1. Initiates a temperature conversion.
    /// 2. Waits for conversion to complete.
    /// 3. Reads and CRC-verifies the scratchpad data.
    /// 4. Returns the measured temperature in °C.
    #[must_use]
    pub fn read_temperature(&mut self) -> Result<f32, Ds18b20Error<P::Error>> {
        // 1. Reset and check presence.
        if !self.reset()? {
            return Err(Ds18b20Error::NoPresence);
        }

        // 2. Start temperature conversion.
        self.write_byte(Self::CMD_SKIP_ROM)?;
        self.write_byte(Self::CMD_CONVERT_T)?;

        // 3. Wait for conversion completion (poll line or timeout).
        for _ in 0..Self::CONVERSION_WAIT_MS {
            if self.pin.is_high()? {
                break;
            }
            self.delay.delay_ms(1);
        }

        // 4. Reset again to read scratchpad.
        if !self.reset()? {
            return Err(Ds18b20Error::NoPresence);
        }

        self.write_byte(Self::CMD_SKIP_ROM)?;
        self.write_byte(Self::CMD_READ_SCRATCHPAD)?;

        let data = self.read_scratchpad()?;

        // 5. Validate CRC.
        let crc_calc = Self::crc8(&data[0..8]);
        if crc_calc != data[8] {
            return Err(Ds18b20Error::CrcMismatch);
        }

        // 6. Convert raw temperature to °C.
        let raw_temp = ((data[1] as i16) << 8) | (data[0] as i16);
        let temp = raw_temp as f32 * Self::TEMPERATURE_RESOLUTION_C_PER_LSB;

        Ok(temp)
    }

    fn write_bit(&mut self, bit: bool) -> Result<(), Ds18b20Error<P::Error>> {
        // Write a single bit to the 1-Wire bus.
        if bit {
            // Logic 1: short low pulse.
            self.pin.set_low()?;
            self.delay.delay_us(Self::WRITE_1_LOW_US);
            self.pin.set_high()?;
            self.delay.delay_us(Self::WRITE_1_HIGH_US);
        } else {
            // Logic 0: long low pulse.
            self.pin.set_low()?;
            self.delay.delay_us(Self::WRITE_0_LOW_US);
            self.pin.set_high()?;
            self.delay.delay_us(Self::WRITE_0_HIGH_US);
        }

        Ok(())
    }

    fn read_bit(&mut self) -> Result<bool, Ds18b20Error<P::Error>> {
        self.pin.set_low()?;
        self.delay.delay_us(Self::READ_INIT_LOW_US);
        self.pin.set_high()?;
        self.delay.delay_us(Self::READ_SAMPLE_US);

        // Read a single bit from the 1-Wire bus.
        let bit = self.pin.is_high()?;
        self.delay.delay_us(Self::READ_RECOVERY_US);

        Ok(bit)
    }

    fn write_byte(&mut self, byte: u8) -> Result<(), Ds18b20Error<P::Error>> {
        // Write a full byte to the 1-Wire bus (LSB first).
        for i in 0..8 {
            self.write_bit((byte >> i) & 1 != 0)?;
        }

        Ok(())
    }

    fn read_byte(&mut self) -> Result<u8, Ds18b20Error<P::Error>> {
        let mut byte = 0;

        // Read a full byte from the 1-Wire bus (LSB first).
        for i in 0..8 {
            if self.read_bit()? {
                byte |= 1 << i;
            }
        }

        Ok(byte)
    }

    fn read_scratchpad(&mut self) -> Result<[u8; 9], Ds18b20Error<P::Error>> {
        let mut data = [0u8; 9];

        // Read the 9-byte scratchpad from the DS18B20.
        for b in &mut data {
            *b = self.read_byte()?;
        }

        Ok(data)
    }

    fn crc8(data: &[u8]) -> u8 {
        let mut crc: u8 = 0;

        // Compute the Dallas/Maxim CRC8 checksum (polynomial 0x31).
        for &byte in data {
            let mut b = byte;
            for _ in 0..8 {
                let mix = (crc ^ b) & 0x01;
                crc >>= 1;
                if mix != 0 {
                    crc ^= 0x8C;
                }
                b >>= 1;
            }
        }

        crc
    }
}
