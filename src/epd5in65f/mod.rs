//! A simple Driver for the Waveshare 6.65 inch (F) E-Ink Display via SPI
//!
//! # References
//!
//! - [Datasheet](https://www.waveshare.com/wiki/5.65inch_e-Paper_Module_(F))
//! - [Waveshare C driver](https://github.com/waveshare/e-Paper/blob/master/RaspberryPi%26JetsonNano/c/lib/e-Paper/EPD_5in65f.c)
//! - [Waveshare Python driver](https://github.com/waveshare/e-Paper/blob/master/RaspberryPi%26JetsonNano/python/lib/waveshare_epd/epd5in65f.py)

use embedded_hal::{
    blocking::{delay::*, spi::Write},
    digital::v2::{InputPin, OutputPin},
};

use crate::color::OctColor;
use crate::interface::DisplayInterface;
use crate::traits::{InternalWiAdditions, RefreshLut, WaveshareDisplay};

pub(crate) mod command;
use self::command::Command;

#[cfg(feature = "graphics")]
mod graphics;
#[cfg(feature = "graphics")]
pub use self::graphics::Display5in65f;

/// Width of the display
pub const WIDTH: u32 = 600;
/// Height of the display
pub const HEIGHT: u32 = 448;
/// Default Background Color
pub const DEFAULT_BACKGROUND_COLOR: OctColor = OctColor::White;
const IS_BUSY_LOW: bool = true;

/// Epd5in65f driver
///
pub struct Epd5in65f<SPI, CS, BUSY, DC, RST, DELAY> {
    /// Connection Interface
    interface: DisplayInterface<SPI, CS, BUSY, DC, RST, DELAY>,
    /// Background Color
    color: OctColor,
}

impl<SPI, CS, BUSY, DC, RST, DELAY> InternalWiAdditions<SPI, CS, BUSY, DC, RST, DELAY>
    for Epd5in65f<SPI, CS, BUSY, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayMs<u8>,
{
    fn init(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        // Reset the device
        self.interface.reset(delay, 2);

        self.cmd_with_data(spi, Command::PanelSetting, &[0xEF, 0x08])?;
        self.cmd_with_data(spi, Command::PowerSetting, &[0x37, 0x00, 0x23, 0x23])?;
        self.cmd_with_data(spi, Command::PowerOffSequenceSetting, &[0x00])?;
        self.cmd_with_data(spi, Command::BoosterSoftStart, &[0xC7, 0xC7, 0x1D])?;
        self.cmd_with_data(spi, Command::PllControl, &[0x3C])?;
        self.cmd_with_data(spi, Command::TemperatureSensor, &[0x00])?;
        self.update_vcom(spi)?;
        self.cmd_with_data(spi, Command::TconSetting, &[0x22])?;
        self.send_resolution(spi)?;

        self.cmd_with_data(spi, Command::FlashMode, &[0xAA])?;

        delay.delay_ms(100);

        self.update_vcom(spi)?;
        Ok(())
    }
}

impl<SPI, CS, BUSY, DC, RST, DELAY> WaveshareDisplay<SPI, CS, BUSY, DC, RST, DELAY>
    for Epd5in65f<SPI, CS, BUSY, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayMs<u8>,
{
    type DisplayColor = OctColor;
    fn new(
        spi: &mut SPI,
        cs: CS,
        busy: BUSY,
        dc: DC,
        rst: RST,
        delay: &mut DELAY,
    ) -> Result<Self, SPI::Error> {
        let interface = DisplayInterface::new(cs, busy, dc, rst);
        let color = DEFAULT_BACKGROUND_COLOR;

        let mut epd = Epd5in65f { interface, color };

        epd.init(spi, delay)?;

        Ok(epd)
    }

    fn wake_up(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.init(spi, delay)
    }

    fn sleep(&mut self, spi: &mut SPI, _delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.cmd_with_data(spi, Command::DeepSleep, &[0xA5])?;
        Ok(())
    }

    fn update_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        _delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        self.wait_busy_high();
        self.update_vcom(spi)?;
        self.send_resolution(spi)?;
        self.cmd_with_data(spi, Command::DataStartTransmission1, buffer)?;
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

    fn display_frame(&mut self, spi: &mut SPI, _delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.wait_busy_high();
        self.command(spi, Command::PowerOn)?;
        self.wait_busy_high();
        self.command(spi, Command::DisplayRefresh)?;
        self.wait_busy_high();
        self.command(spi, Command::PowerOff)?;
        self.wait_busy_low();
        Ok(())
    }

    fn update_and_display_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        self.update_frame(spi, buffer, delay)?;
        self.display_frame(spi, delay)?;
        Ok(())
    }

    fn clear_frame(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        let bg = OctColor::colors_byte(self.color, self.color);
        self.wait_busy_high();
        self.update_vcom(spi)?;
        self.send_resolution(spi)?;
        self.command(spi, Command::DataStartTransmission1)?;
        self.interface.data_x_times(spi, bg, WIDTH * HEIGHT / 2)?;
        self.display_frame(spi, delay)?;
        Ok(())
    }

    fn set_background_color(&mut self, color: OctColor) {
        self.color = color;
    }

    fn background_color(&self) -> &OctColor {
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
        _refresh_rate: Option<RefreshLut>,
    ) -> Result<(), SPI::Error> {
        unimplemented!();
    }

    fn is_busy(&self) -> bool {
        self.interface.is_busy(IS_BUSY_LOW)
    }
}

impl<SPI, CS, BUSY, DC, RST, DELAY> Epd5in65f<SPI, CS, BUSY, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayMs<u8>,
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

    fn wait_busy_high(&mut self) {
        let _ = self.interface.wait_until_idle(true);
    }
    fn wait_busy_low(&mut self) {
        let _ = self.interface.wait_until_idle(false);
    }
    fn send_resolution(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        let w = self.width();
        let h = self.height();

        self.command(spi, Command::TconResolution)?;
        self.send_data(spi, &[(w >> 8) as u8])?;
        self.send_data(spi, &[w as u8])?;
        self.send_data(spi, &[(h >> 8) as u8])?;
        self.send_data(spi, &[h as u8])
    }

    fn update_vcom(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        let bg_color = (self.color.get_nibble() & 0b111) << 5;
        self.cmd_with_data(spi, Command::VcomAndDataIntervalSetting, &[0x17 | bg_color])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epd_size() {
        assert_eq!(WIDTH, 600);
        assert_eq!(HEIGHT, 448);
        assert_eq!(DEFAULT_BACKGROUND_COLOR, OctColor::White);
    }
}
