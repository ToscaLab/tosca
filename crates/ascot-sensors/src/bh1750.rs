//! # BH1750 Driver
//!
//! This crate provides an asynchronous, architecture-agnostic driver for the BH1750 ambient light sensor,
//! allowing reading of light intensity in lux over the I²C protocol.
//!
//! The driver implements all instructions of the sensor's instruction set architecture.
//!
//! For detailed information and specifications, see the [datasheet](https://www.alldatasheet.com/datasheet-pdf/pdf/338083/ROHM/BH1750FVI.html).

use embedded_hal_async::i2c::I2c;
use embedded_hal_async::delay::DelayNs;

/// Errors that may occur while interacting with the BH1750 sensor.
#[derive(Debug, Copy, Clone)]
pub enum Bh1750Error<E> {
    /// I²C bus error.
    I2c(E),
    /// Continous measurement not started.
    ///
    /// Occurs when attempting to read a continuous measurement before it has been started.
    ContinuousMeasurementNotStarted,
}

impl<E> From<E> for Bh1750Error<E> {
    fn from(e: E) -> Self {
        Bh1750Error::I2c(e)
    }
}

/// I²C address of the BH1750 sensor.
///
/// The sensor supports two possible addresses depending on how the ADD pin is connected.
#[derive(Debug, Clone, Copy)]
pub enum Address {
    /// Low: `0x23` when ADD is connected to GND or floating.
    Low = 0x23,
    /// High: `0x23` when ADD is connected to VCC.
    High = 0x5C,
}

/// Measurement resolution modes for the BH1750 sensor.
#[derive(Debug, Clone, Copy)]
pub enum Resolution {
    /// High resolution mode: 1 lx per count.
    ///
    /// Measurement time is 120 ms, assuming default `MTreg` value.
    High,
    /// High resolution mode 2: 0.5 lx per count.
    ///
    /// Measurement time is 120 ms, assuming default `MTreg` value.
    High2,
    /// Low resolution mode: 4 lx per count.
    ///
    /// Measurement time is 16 ms, assuming default `MTreg` value.
    Low,
}

impl Resolution {
    #[inline]
    const fn continuous_measurement_opcode(self) -> u8 {
        match self {
            Self::High => 0x10,
            Self::High2 => 0x11,
            Self::Low => 0x13,
        }
    }

    #[inline]
    const fn one_time_measurement_opcode(self) -> u8 {
        match self {
            Self::High => 0x20,
            Self::High2 => 0x21,
            Self::Low => 0x23,
        }
    }

    #[inline]
    const fn default_measurement_time_ms(self) -> u32 {
        // Returns the default lux per count for this resolution mode,
        // assuming default `MTreg` value.
        match self {
            Self::High | Self::High2 => 120,
            Self::Low => 16,
        }
    }

    #[inline]
    const fn default_resolution_lx_count(self) -> f32 {
        // Returns the default measurement time for this resolution mode,
        // assuming default `MTreg` value.
        match self {
            Self::High => 1.0,
            Self::High2 => 0.5,
            Self::Low => 4.0,
        }
    }
}

/// BH1750 driver.
pub struct Bh1750<I2C, D>
where
    D: DelayNs,
{
    i2c: I2C,
    delay: D,
    address: Address,
    mtreg: u8,
    continuous_resolution: Option<Resolution>,
}

impl<I2C, E, D> Bh1750<I2C, D>
where
    I2C: I2c<u8, Error = E>,
    D: DelayNs,
{
    // Instruction set architecture opcodes.
    const POWER_DOWN: u8 = 0x00;
    const POWER_ON: u8 = 0x01;
    const RESET: u8 = 0x07;

    // MTreg configuration opcodes.
    // The 8-bit MTreg value is split into a high and a low instruction byte.
    const MTREG_HIGH: u8 = 0x40;
    const MTREG_LOW: u8 = 0x60;

    const MTREG_MIN: u8 = 31;       // Minimum allowed MTreg value.
    const MTREG_MAX: u8 = 254;      // Maximum allowed MTreg value.
    const DEFAULT_MTREG: u8 = 69;   // Default per datasheet.

    /// Creates a new [`Bh1750`] driver with the given I²C bus, delay provider, and address.
    ///
    /// The `MTreg` is initialized to its default value.
    #[must_use]
    pub fn new(i2c: I2C, delay: D, address: Address) -> Self {
        Self {
            i2c,
            delay,
            address,
            mtreg: Self::DEFAULT_MTREG,
            continuous_resolution: None,
        }
    }

    /// Puts the sensor into the `Power On` state.
    #[must_use]
    pub async fn power_on(&mut self) -> Result<(), Bh1750Error<E>> {
        self.send_instruction(Self::POWER_ON).await
    }

    /// Puts the sensor into the `Power Down` state.
    #[must_use]
    pub async fn power_down(&mut self) -> Result<(), Bh1750Error<E>> {
        self.send_instruction(Self::POWER_DOWN).await
    }

    /// Resets the sensor data register.
    ///
    /// Must be called only when the sensor is in the `Power On` state.
    #[must_use]
    pub async fn reset(&mut self) -> Result<(), Bh1750Error<E>> {
        self.send_instruction(Self::RESET).await
    }

    /// Sets the measurement time register (`MTreg`) to adjust sensitivity.
    ///
    /// The value is automatically clamped between [`MTREG_MIN`] and [`MTREG_MAX`].
    #[must_use]
    pub async fn set_mtreg(&mut self, mtreg: u8) -> Result<(), Bh1750Error<E>> {
        let mt = mtreg.clamp(Self::MTREG_MIN, Self::MTREG_MAX);

        // Split the 8-bit MTreg value into two parts and send as separate opcodes.
        let high = Self::MTREG_HIGH | (mt >> 5);
        let low = Self::MTREG_LOW | (mt & 0x1F);

        self.send_instruction(high).await?;
        self.send_instruction(low).await?;

        self.mtreg = mt;

        Ok(())
    }

    /// Performs a one-time measurement and returns the light level in lux.
    ///
    /// According to the datasheet, the sensor automatically returns to the `Power Down` state after a one-time measurement.
    #[must_use]
    pub async fn one_time_measurement(&mut self, res: Resolution) -> Result<f32, Bh1750Error<E>> {
        self.start_one_time_measurement(res).await?;
        self.delay.delay_ms(self.measurement_time_ms(res)).await;
        let raw = self.read_raw().await?;

        Ok(self.raw_to_lux(raw, res))
    }

    /// Starts a continuous measurement at the given resolution.
    ///
    /// The chosen resolution is stored internally, allowing later calls to [`Self::read_continuous_measurement`].
    #[must_use]
    pub async fn start_continuous_measurement(&mut self, res: Resolution) -> Result<(), Bh1750Error<E>> {
        self.send_instruction(res.continuous_measurement_opcode()).await?;
        self.continuous_resolution = Some(res);

        Ok(())
    }

    /// Reads the latest value from a continuous measurement in lux.
    ///
    /// Returns an error [`Bh1750Error::ContinuousMeasurementNotStarted`] if the measurement was not started.
    #[must_use]
    pub async fn read_continuous_measurement(&mut self) -> Result<f32, Bh1750Error<E>> {
        let res = self
            .continuous_resolution
            .ok_or(Bh1750Error::ContinuousMeasurementNotStarted)?;

        // Wait for the effective measurement duration.
        self.delay.delay_ms(self.measurement_time_ms(res)).await;

        let raw = self.read_raw().await?;

        Ok(self.raw_to_lux(raw, res))
    }

    async fn start_one_time_measurement(&mut self, res: Resolution) -> Result<(), Bh1750Error<E>> {
        self.send_instruction(res.one_time_measurement_opcode()).await
    }

    async fn read_raw(&mut self) -> Result<u16, E> {
        let mut buf = [0u8; 2];
        self.i2c.read(self.address as u8, &mut buf).await?;

        Ok(u16::from_be_bytes(buf))
    }

    fn raw_to_lux(&self, raw: u16, res: Resolution) -> f32 {
        // Convert the raw 16-bit reading to lux.
        //
        // Formula from BH1750 datasheet:
        //   lux = (raw_value / 1.2) * (resolution_factor) * (MTreg / 69)
        // where:
        //   - 1.2 is a scaling constant defined by the sensor manufacturer,
        //   - resolution_factor = 1.0, 0.5, or 4.0 depending on mode,
        //   - current MTreg value.
        raw as f32
            * res.default_resolution_lx_count()
            * (self.mtreg as f32 / Self::DEFAULT_MTREG as f32)
            / 1.2
    }

    #[inline]
    fn measurement_time_ms(&self, res: Resolution) -> u32 {
        // Adjust measurement time according to the current MTreg value.
        // The measurement time scales linearly with MTreg:
        //   t = default_time * (MTreg / 69)
        res.default_measurement_time_ms() * self.mtreg as u32 / Self::DEFAULT_MTREG as u32
    }

    #[inline]
    async fn send_instruction(&mut self, instr: u8) -> Result<(), Bh1750Error<E>> {
        self.i2c.write(self.address as u8, &[instr]).await?;

        Ok(())
    }
}
