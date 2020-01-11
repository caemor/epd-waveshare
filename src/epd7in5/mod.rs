//! A simple Driver for the Waveshare 7.5" E-Ink Display via SPI
//!
//! # References
//!
//! - [Datasheet](https://www.waveshare.com/wiki/7.5inch_e-Paper_HAT)
//! - [Waveshare C driver](https://github.com/waveshare/e-Paper/blob/702def06bcb75983c98b0f9d25d43c552c248eb0/RaspberryPi%26JetsonNano/c/lib/e-Paper/EPD_7in5.c)
//! - [Waveshare Python driver](https://github.com/waveshare/e-Paper/blob/702def06bcb75983c98b0f9d25d43c552c248eb0/RaspberryPi%26JetsonNano/python/lib/waveshare_epd/epd7in5.py)

use embedded_hal::{
    blocking::{delay::*, spi::Write},
    digital::v2::{InputPin, OutputPin},
};

use crate::color::Color;
use crate::interface::DisplayInterface;
use crate::traits::{InternalWiAdditions, RefreshLUT, WaveshareDisplay};

pub(crate) mod command;
use self::command::Command;

#[cfg(feature = "graphics")]
mod graphics;
#[cfg(feature = "graphics")]
pub use self::graphics::Display7in5;

pub const WIDTH: u32 = 640;
pub const HEIGHT: u32 = 384;
pub const DEFAULT_BACKGROUND_COLOR: Color = Color::White;
const IS_BUSY_LOW: bool = true;

/// EPD7in5 driver
///
pub struct EPD7in5<SPI, CS, BUSY, DC, RST> {
    /// Connection Interface
    interface: DisplayInterface<SPI, CS, BUSY, DC, RST>,
    /// Background Color
    color: Color,
}

impl<SPI, CS, BUSY, DC, RST> InternalWiAdditions<SPI, CS, BUSY, DC, RST>
    for EPD7in5<SPI, CS, BUSY, DC, RST>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    fn init<DELAY: DelayMs<u8>>(
        &mut self,
        spi: &mut SPI,
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        // Reset the device
        self.interface.reset(delay);

        // Set the power settings
        self.cmd_with_data(spi, Command::POWER_SETTING, &[0x37, 0x00])?;

        // Set the panel settings:
        // - 600 x 448
        // - Using LUT from external flash
        self.cmd_with_data(spi, Command::PANEL_SETTING, &[0xCF, 0x08])?;

        // Start the booster
        self.cmd_with_data(spi, Command::BOOSTER_SOFT_START, &[0xC7, 0xCC, 0x28])?;

        // Power on
        self.command(spi, Command::POWER_ON)?;
        delay.delay_ms(5);
        self.wait_until_idle();

        // Set the clock frequency to 50Hz (default)
        self.cmd_with_data(spi, Command::PLL_CONTROL, &[0x3C])?;

        // Select internal temperature sensor (default)
        self.cmd_with_data(spi, Command::TEMPERATURE_CALIBRATION, &[0x00])?;

        // Set Vcom and data interval to 10 (default), border output to white
        self.cmd_with_data(spi, Command::VCOM_AND_DATA_INTERVAL_SETTING, &[0x77])?;

        // Set S2G and G2S non-overlap periods to 12 (default)
        self.cmd_with_data(spi, Command::TCON_SETTING, &[0x22])?;

        // Set the real resolution
        self.send_resolution(spi)?;

        // Set VCOM_DC to -1.5V
        self.cmd_with_data(spi, Command::VCM_DC_SETTING, &[0x1E])?;

        // This is in all the Waveshare controllers for EPD7in5
        self.cmd_with_data(spi, Command::FLASH_MODE, &[0x03])?;

        self.wait_until_idle();
        Ok(())
    }
}

impl<SPI, CS, BUSY, DC, RST> WaveshareDisplay<SPI, CS, BUSY, DC, RST>
    for EPD7in5<SPI, CS, BUSY, DC, RST>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    /// Creates a new driver from a SPI peripheral, CS Pin, Busy InputPin, DC
    ///
    /// This already initialises the device. That means [init()] isn't needed
    /// directly afterwards.
    ///
    /// [init()]: InternalWiAdditions::init
    ///
    /// # Example
    ///
    /// ```ignore
    /// //buffer = some image data;
    ///
    /// let mut epd7in5 = EPD7in5::new(spi, cs, busy, dc, rst, delay);
    ///
    /// epd7in5.display_and_transfer_frame(buffer, None);
    ///
    /// epd7in5.sleep();
    /// ```
    fn new<DELAY: DelayMs<u8>>(
        spi: &mut SPI,
        cs: CS,
        busy: BUSY,
        dc: DC,
        rst: RST,
        delay: &mut DELAY,
    ) -> Result<Self, SPI::Error> {
        let interface = DisplayInterface::new(cs, busy, dc, rst);
        let color = DEFAULT_BACKGROUND_COLOR;

        let mut epd = EPD7in5 { interface, color };

        epd.init(spi, delay)?;

        Ok(epd)
    }

    fn wake_up<DELAY: DelayMs<u8>>(
        &mut self,
        spi: &mut SPI,
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        self.init(spi, delay)
    }

    fn sleep(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        self.command(spi, Command::POWER_OFF)?;
        self.wait_until_idle();
        self.cmd_with_data(spi, Command::DEEP_SLEEP, &[0xA5])?;

        self.wait_until_idle();
        Ok(())
    }

    fn update_frame(&mut self, spi: &mut SPI, buffer: &[u8]) -> Result<(), SPI::Error> {
        self.command(spi, Command::DATA_START_TRANSMISSION_1)?;
        for byte in buffer {
            let mut temp = *byte;
            for _ in 0..4 {
                let mut data = if temp & 0x80 == 0 { 0x00 } else { 0x03 };
                data <<= 4;
                temp <<= 1;
                data |= if temp & 0x80 == 0 { 0x00 } else { 0x03 };
                temp <<= 1;
                self.send_data(spi, &[data])?;
            }
        }

        self.wait_until_idle();
        Ok(())
    }

    fn update_partial_frame(
        &mut self,
        _spi: &mut SPI,
        _buffer: &[u8],
        _x: u32,
        _y: u32,
        _width: u32,
        _height: u32,
    ) -> Result<(), SPI::Error> {
        unimplemented!();
    }

    fn display_frame(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        self.command(spi, Command::DISPLAY_REFRESH)?;

        self.wait_until_idle();
        Ok(())
    }

    fn clear_frame(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        self.send_resolution(spi)?;

        // The Waveshare controllers all implement clear using 0x33
        self.command(spi, Command::DATA_START_TRANSMISSION_1)?;
        self.interface
            .data_x_times(spi, 0x33, WIDTH / 8 * HEIGHT * 4)?;

        self.wait_until_idle();
        Ok(())
    }

    fn set_background_color(&mut self, color: Color) {
        self.color = color;
    }

    fn background_color(&self) -> &Color {
        &self.color
    }

    fn width(&self) -> u32 {
        WIDTH
    }

    fn height(&self) -> u32 {
        HEIGHT
    }

    fn set_lut(
        &mut self,
        _spi: &mut SPI,
        _refresh_rate: Option<RefreshLUT>,
    ) -> Result<(), SPI::Error> {
        unimplemented!();
    }

    fn is_busy(&self) -> bool {
        self.interface.is_busy(IS_BUSY_LOW)
    }
}

impl<SPI, CS, BUSY, DC, RST> EPD7in5<SPI, CS, BUSY, DC, RST>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    fn command(&mut self, spi: &mut SPI, command: Command) -> Result<(), SPI::Error> {
        self.interface.cmd(spi, command)
    }

    fn send_data(&mut self, spi: &mut SPI, data: &[u8]) -> Result<(), SPI::Error> {
        self.interface.data(spi, data)
    }

    fn cmd_with_data(
        &mut self,
        spi: &mut SPI,
        command: Command,
        data: &[u8],
    ) -> Result<(), SPI::Error> {
        self.interface.cmd_with_data(spi, command, data)
    }

    fn wait_until_idle(&mut self) {
        self.interface.wait_until_idle(IS_BUSY_LOW)
    }

    fn send_resolution(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        let w = self.width();
        let h = self.height();

        self.command(spi, Command::TCON_RESOLUTION)?;
        self.send_data(spi, &[(w >> 8) as u8])?;
        self.send_data(spi, &[w as u8])?;
        self.send_data(spi, &[(h >> 8) as u8])?;
        self.send_data(spi, &[h as u8])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epd_size() {
        assert_eq!(WIDTH, 640);
        assert_eq!(HEIGHT, 384);
        assert_eq!(DEFAULT_BACKGROUND_COLOR, Color::White);
    }
}
