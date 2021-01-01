//! A simple Driver for the Waveshare 2.7" B Tri-Color E-Ink Display via SPI
//!
//! [Documentation](https://www.waveshare.com/wiki/2.7inch_e-Paper_HAT_(B))

use embedded_hal::{
    blocking::{delay::*, spi::Write},
    digital::v2::*,
};

use crate::interface::DisplayInterface;
use crate::traits::{
    InternalWiAdditions, RefreshLUT, WaveshareDisplay, WaveshareThreeColorDisplay,
};

// The Lookup Tables for the Display
mod constants;
use crate::epd2in7b::constants::*;

/// Width of the display
pub const WIDTH: u32 = 176;
/// Height of the display
pub const HEIGHT: u32 = 264;
/// Default Background Color
pub const DEFAULT_BACKGROUND_COLOR: Color = Color::White;
const IS_BUSY_LOW: bool = true;

use crate::color::Color;

pub(crate) mod command;
use self::command::Command;

#[cfg(feature = "graphics")]
mod graphics;
#[cfg(feature = "graphics")]
pub use self::graphics::Display2in7b;

/// EPD2in7b driver
pub struct EPD2in7b<SPI, CS, BUSY, DC, RST> {
    /// Connection Interface
    interface: DisplayInterface<SPI, CS, BUSY, DC, RST>,
    /// Background Color
    color: Color,
}

impl<SPI, CS, BUSY, DC, RST> InternalWiAdditions<SPI, CS, BUSY, DC, RST>
    for EPD2in7b<SPI, CS, BUSY, DC, RST>
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
        // reset the device
        self.interface.reset(delay, 10);

        // power on
        self.command(spi, Command::POWER_ON)?;
        delay.delay_ms(5);
        self.wait_until_idle();

        // set panel settings, 0xbf is bw, 0xaf is multi-color
        self.interface
            .cmd_with_data(spi, Command::PANEL_SETTING, &[0xaf])?;

        // pll control
        self.interface
            .cmd_with_data(spi, Command::PLL_CONTROL, &[0x3a])?;

        // set the power settings
        self.interface.cmd_with_data(
            spi,
            Command::POWER_SETTING,
            &[0x03, 0x00, 0x2b, 0x2b, 0x09],
        )?;

        // start the booster
        self.interface
            .cmd_with_data(spi, Command::BOOSTER_SOFT_START, &[0x07, 0x07, 0x17])?;

        // power optimization
        self.interface
            .cmd_with_data(spi, Command::POWER_OPTIMIZATION, &[0x60, 0xa5])?;
        self.interface
            .cmd_with_data(spi, Command::POWER_OPTIMIZATION, &[0x89, 0xa5])?;
        self.interface
            .cmd_with_data(spi, Command::POWER_OPTIMIZATION, &[0x90, 0x00])?;
        self.interface
            .cmd_with_data(spi, Command::POWER_OPTIMIZATION, &[0x93, 0x2a])?;
        self.interface
            .cmd_with_data(spi, Command::POWER_OPTIMIZATION, &[0x73, 0x41])?;

        self.interface
            .cmd_with_data(spi, Command::VCM_DC_SETTING, &[0x12])?;

        self.interface
            .cmd_with_data(spi, Command::VCOM_AND_DATA_INTERVAL_SETTING, &[0x87])?;

        self.set_lut(spi, None)?;

        self.interface
            .cmd_with_data(spi, Command::PARTIAL_DISPLAY_REFRESH, &[0x00])?;

        self.wait_until_idle();
        Ok(())
    }
}

impl<SPI, CS, BUSY, DC, RST> WaveshareDisplay<SPI, CS, BUSY, DC, RST>
    for EPD2in7b<SPI, CS, BUSY, DC, RST>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    type DisplayColor = Color;
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

        let mut epd = EPD2in7b { interface, color };

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
        self.wait_until_idle();
        self.interface
            .cmd_with_data(spi, Command::VCOM_AND_DATA_INTERVAL_SETTING, &[0xf7])?;

        self.command(spi, Command::POWER_OFF)?;
        self.wait_until_idle();
        self.interface
            .cmd_with_data(spi, Command::DEEP_SLEEP, &[0xA5])?;
        Ok(())
    }

    fn update_frame(&mut self, spi: &mut SPI, buffer: &[u8]) -> Result<(), SPI::Error> {
        self.interface
            .cmd(spi, Command::DATA_START_TRANSMISSION_1)?;
        self.send_buffer_helper(spi, buffer)?;

        // Clear chromatic layer since we won't be using it here
        self.interface
            .cmd(spi, Command::DATA_START_TRANSMISSION_2)?;
        self.interface
            .data_x_times(spi, !self.color.get_byte_value(), WIDTH * HEIGHT / 8)?;

        self.interface.cmd(spi, Command::DATA_STOP)?;
        Ok(())
    }

    fn update_partial_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), SPI::Error> {
        self.interface
            .cmd(spi, Command::PARTIAL_DATA_START_TRANSMISSION_1)?;

        self.send_data(spi, &[(x >> 8) as u8])?;
        self.send_data(spi, &[(x & 0xf8) as u8])?;
        self.send_data(spi, &[(y >> 8) as u8])?;
        self.send_data(spi, &[(y & 0xff) as u8])?;
        self.send_data(spi, &[(width >> 8) as u8])?;
        self.send_data(spi, &[(width & 0xf8) as u8])?;
        self.send_data(spi, &[(height >> 8) as u8])?;
        self.send_data(spi, &[(height & 0xff) as u8])?;
        self.wait_until_idle();

        self.send_buffer_helper(spi, buffer)?;

        self.interface.cmd(spi, Command::DATA_STOP)
    }

    fn display_frame(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        self.command(spi, Command::DISPLAY_REFRESH)?;
        self.wait_until_idle();
        Ok(())
    }

    fn update_and_display_frame(&mut self, spi: &mut SPI, buffer: &[u8]) -> Result<(), SPI::Error> {
        self.update_frame(spi, buffer)?;
        self.command(spi, Command::DISPLAY_REFRESH)?;
        Ok(())
    }

    fn clear_frame(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        self.wait_until_idle();

        let color_value = self.color.get_byte_value();
        self.interface
            .cmd(spi, Command::DATA_START_TRANSMISSION_1)?;
        self.interface
            .data_x_times(spi, color_value, WIDTH * HEIGHT / 8)?;

        self.interface.cmd(spi, Command::DATA_STOP)?;

        self.interface
            .cmd(spi, Command::DATA_START_TRANSMISSION_2)?;
        self.interface
            .data_x_times(spi, color_value, WIDTH * HEIGHT / 8)?;
        self.interface.cmd(spi, Command::DATA_STOP)?;
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
        spi: &mut SPI,
        _refresh_rate: Option<RefreshLUT>,
    ) -> Result<(), SPI::Error> {
        self.wait_until_idle();
        self.cmd_with_data(spi, Command::LUT_FOR_VCOM, &LUT_VCOM_DC)?;
        self.cmd_with_data(spi, Command::LUT_WHITE_TO_WHITE, &LUT_WW)?;
        self.cmd_with_data(spi, Command::LUT_BLACK_TO_WHITE, &LUT_BW)?;
        self.cmd_with_data(spi, Command::LUT_WHITE_TO_BLACK, &LUT_WB)?;
        self.cmd_with_data(spi, Command::LUT_BLACK_TO_BLACK, &LUT_BB)?;
        Ok(())
    }

    fn is_busy(&self) -> bool {
        self.interface.is_busy(IS_BUSY_LOW)
    }
}

impl<SPI, CS, BUSY, DC, RST> WaveshareThreeColorDisplay<SPI, CS, BUSY, DC, RST>
    for EPD2in7b<SPI, CS, BUSY, DC, RST>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    fn update_color_frame(
        &mut self,
        spi: &mut SPI,
        black: &[u8],
        chromatic: &[u8],
    ) -> Result<(), SPI::Error> {
        self.update_achromatic_frame(spi, black)?;
        self.update_chromatic_frame(spi, chromatic)
    }

    /// Update only the black/white data of the display.
    ///
    /// Finish by calling `update_chromatic_frame`.
    fn update_achromatic_frame(
        &mut self,
        spi: &mut SPI,
        achromatic: &[u8],
    ) -> Result<(), SPI::Error> {
        self.interface
            .cmd(spi, Command::DATA_START_TRANSMISSION_1)?;

        self.send_buffer_helper(spi, achromatic)?;

        self.interface.cmd(spi, Command::DATA_STOP)
    }

    /// Update only chromatic data of the display.
    ///
    /// This data takes precedence over the black/white data.
    fn update_chromatic_frame(
        &mut self,
        spi: &mut SPI,
        chromatic: &[u8],
    ) -> Result<(), SPI::Error> {
        self.interface
            .cmd(spi, Command::DATA_START_TRANSMISSION_2)?;

        self.send_buffer_helper(spi, chromatic)?;

        self.interface.cmd(spi, Command::DATA_STOP)?;
        self.wait_until_idle();

        Ok(())
    }
}

impl<SPI, CS, BUSY, DC, RST> EPD2in7b<SPI, CS, BUSY, DC, RST>
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

    fn send_buffer_helper(&mut self, spi: &mut SPI, buffer: &[u8]) -> Result<(), SPI::Error> {
        // Based on the waveshare implementation, all data for color values is flipped. This helper
        // method makes that transmission easier
        for b in buffer.iter() {
            self.send_data(spi, &[!b])?;
        }
        Ok(())
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

    /// Refresh display for partial frame
    pub fn display_partial_frame(
        &mut self,
        spi: &mut SPI,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), SPI::Error> {
        self.command(spi, Command::PARTIAL_DISPLAY_REFRESH)?;
        self.send_data(spi, &[(x >> 8) as u8])?;
        self.send_data(spi, &[(x & 0xf8) as u8])?;
        self.send_data(spi, &[(y >> 8) as u8])?;
        self.send_data(spi, &[(y & 0xff) as u8])?;
        self.send_data(spi, &[(width >> 8) as u8])?;
        self.send_data(spi, &[(width & 0xf8) as u8])?;
        self.send_data(spi, &[(height >> 8) as u8])?;
        self.send_data(spi, &[(height & 0xff) as u8])?;
        self.wait_until_idle();
        Ok(())
    }

    /// Update black/achromatic frame
    pub fn update_partial_achromatic_frame(
        &mut self,
        spi: &mut SPI,
        achromatic: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), SPI::Error> {
        self.interface
            .cmd(spi, Command::PARTIAL_DATA_START_TRANSMISSION_1)?;
        self.send_data(spi, &[(x >> 8) as u8])?;
        self.send_data(spi, &[(x & 0xf8) as u8])?;
        self.send_data(spi, &[(y >> 8) as u8])?;
        self.send_data(spi, &[(y & 0xff) as u8])?;
        self.send_data(spi, &[(width >> 8) as u8])?;
        self.send_data(spi, &[(width & 0xf8) as u8])?;
        self.send_data(spi, &[(height >> 8) as u8])?;
        self.send_data(spi, &[(height & 0xff) as u8])?;
        self.wait_until_idle();

        for b in achromatic.iter() {
            // Flipping based on waveshare implementation
            self.send_data(spi, &[!b])?;
        }

        Ok(())
    }

    /// Update partial chromatic/red frame
    pub fn update_partial_chromatic_frame(
        &mut self,
        spi: &mut SPI,
        chromatic: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), SPI::Error> {
        self.interface
            .cmd(spi, Command::PARTIAL_DATA_START_TRANSMISSION_2)?;
        self.send_data(spi, &[(x >> 8) as u8])?;
        self.send_data(spi, &[(x & 0xf8) as u8])?;
        self.send_data(spi, &[(y >> 8) as u8])?;
        self.send_data(spi, &[(y & 0xff) as u8])?;
        self.send_data(spi, &[(width >> 8) as u8])?;
        self.send_data(spi, &[(width & 0xf8) as u8])?;
        self.send_data(spi, &[(height >> 8) as u8])?;
        self.send_data(spi, &[(height & 0xff) as u8])?;
        self.wait_until_idle();

        for b in chromatic.iter() {
            // Flipping based on waveshare implementation
            self.send_data(spi, &[!b])?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epd_size() {
        assert_eq!(WIDTH, 176);
        assert_eq!(HEIGHT, 264);
        assert_eq!(DEFAULT_BACKGROUND_COLOR, Color::White);
    }
}
