//! A simple Driver for the Waveshare 1.54" (B) E-Ink Display via SPI

use embedded_hal::{
    blocking::{delay::*, spi::Write},
    digital::*,
};

use crate::interface::DisplayInterface;
use crate::traits::{InternalWiAdditions, RefreshLUT, WaveshareDisplay, WaveshareThreeColorDisplay};

//The Lookup Tables for the Display
mod constants;
use crate::epd1in54b::constants::*;

pub const WIDTH: u32 = 200;
pub const HEIGHT: u32 = 200;
pub const DEFAULT_BACKGROUND_COLOR: Color = Color::White;
const IS_BUSY_LOW: bool = true;

use crate::color::Color;

pub(crate) mod command;
use self::command::Command;

#[cfg(feature = "graphics")]
mod graphics;
#[cfg(feature = "graphics")]
pub use self::graphics::Display1in54b;

/// EPD1in54b driver
pub struct EPD1in54b<SPI, CS, BUSY, DC, RST> {
    interface: DisplayInterface<SPI, CS, BUSY, DC, RST>,
    color: Color,
}

impl<SPI, CS, BUSY, DC, RST> InternalWiAdditions<SPI, CS, BUSY, DC, RST>
    for EPD1in54b<SPI, CS, BUSY, DC, RST>
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
        self.interface.reset(delay);

        // set the power settings
        self.interface
            .cmd_with_data(spi, Command::POWER_SETTING, &[0x07, 0x00, 0x08, 0x00])?;

        // start the booster
        self.interface
            .cmd_with_data(spi, Command::BOOSTER_SOFT_START, &[0x07, 0x07, 0x07])?;

        // power on
        self.command(spi, Command::POWER_ON)?;
        delay.delay_ms(5);
        self.wait_until_idle();

        // set the panel settings
        self.cmd_with_data(spi, Command::PANEL_SETTING, &[0xCF])?;

        self.cmd_with_data(spi, Command::VCOM_AND_DATA_INTERVAL_SETTING, &[0x37])?;

        // PLL
        self.cmd_with_data(spi, Command::PLL_CONTROL, &[0x39])?;

        // set resolution
        self.send_resolution(spi)?;

        self.cmd_with_data(spi, Command::VCM_DC_SETTING, &[0x0E])?;

        self.set_lut(spi, None)?;

        self.wait_until_idle();

        Ok(())
    }
}

impl<SPI, CS, BUSY, DC, RST> WaveshareThreeColorDisplay<SPI, CS, BUSY, DC, RST>
    for EPD1in54b<SPI, CS, BUSY, DC, RST>
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
        red: &[u8],
    ) -> Result<(), SPI::Error> {
        self.send_resolution(spi)?;

        self.interface
            .cmd(spi, Command::DATA_START_TRANSMISSION_1)?;

        for b in black {
            let expanded = expand_bits(*b);
            self.interface.data(spi, &expanded)?;
        }

        self.interface
            .cmd(spi, Command::DATA_START_TRANSMISSION_2)?;
        self.interface.data(spi, red)?;

        self.wait_until_idle();
        Ok(())
    }
}

impl<SPI, CS, BUSY, DC, RST> WaveshareDisplay<SPI, CS, BUSY, DC, RST>
    for EPD1in54b<SPI, CS, BUSY, DC, RST>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
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

        let mut epd = EPD1in54b { interface, color };

        epd.init(spi, delay)?;

        Ok(epd)
    }

    fn sleep(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        self.interface
            .cmd_with_data(spi, Command::VCOM_AND_DATA_INTERVAL_SETTING, &[0x17])?; //border floating

        self.interface
            .cmd_with_data(spi, Command::VCM_DC_SETTING, &[0x00])?; // VCOM to 0V

        self.interface
            .cmd_with_data(spi, Command::POWER_SETTING, &[0x02, 0x00, 0x00, 0x00])?; //VG&VS to 0V fast

        self.wait_until_idle();

        //NOTE: The example code has a 1s delay here

        self.command(spi, Command::POWER_OFF)?;

        Ok(())
    }

    fn wake_up<DELAY: DelayMs<u8>>(
        &mut self,
        spi: &mut SPI,
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        self.init(spi, delay)
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

    fn update_frame(&mut self, spi: &mut SPI, buffer: &[u8]) -> Result<(), SPI::Error> {
        self.send_resolution(spi)?;

        self.interface
            .cmd(spi, Command::DATA_START_TRANSMISSION_1)?;

        for b in buffer {
            // Two bits per pixel
            let expanded = expand_bits(*b);
            self.interface.data(spi, &expanded)?;
        }

        //NOTE: Example code has a delay here

        // Clear the read layer
        let color = self.color.get_byte_value();
        let nbits = WIDTH * (HEIGHT / 8);

        self.interface
            .cmd(spi, Command::DATA_START_TRANSMISSION_2)?;
        self.interface.data_x_times(spi, color, nbits)?;

        //NOTE: Example code has a delay here

        self.wait_until_idle();
        Ok(())
    }

    #[allow(unused)]
    fn update_partial_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), SPI::Error> {
        Ok(())
    }

    fn display_frame(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        self.command(spi, Command::DISPLAY_REFRESH)?;

        self.wait_until_idle();
        Ok(())
    }

    fn clear_frame(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        self.send_resolution(spi)?;

        let color = DEFAULT_BACKGROUND_COLOR.get_byte_value();

        // Clear the black
        self.interface
            .cmd(spi, Command::DATA_START_TRANSMISSION_1)?;

        // Uses 2 bits per pixel
        self.interface
            .data_x_times(spi, color, 2 * (WIDTH * HEIGHT / 8))?;

        // Clear the red
        self.interface
            .cmd(spi, Command::DATA_START_TRANSMISSION_2)?;
        self.interface
            .data_x_times(spi, color, WIDTH * HEIGHT / 8)?;

        self.wait_until_idle();
        Ok(())
    }

    fn set_lut(
        &mut self,
        spi: &mut SPI,
        _refresh_rate: Option<RefreshLUT>,
    ) -> Result<(), SPI::Error> {
        self.interface
            .cmd_with_data(spi, Command::LUT_FOR_VCOM, LUT_VCOM0)?;
        self.interface
            .cmd_with_data(spi, Command::LUT_WHITE_TO_WHITE, LUT_WHITE_TO_WHITE)?;
        self.interface
            .cmd_with_data(spi, Command::LUT_BLACK_TO_WHITE, LUT_BLACK_TO_WHITE)?;
        self.interface.cmd_with_data(spi, Command::LUT_G0, LUT_G1)?;
        self.interface.cmd_with_data(spi, Command::LUT_G1, LUT_G2)?;
        self.interface
            .cmd_with_data(spi, Command::LUT_RED_VCOM, LUT_RED_VCOM)?;
        self.interface
            .cmd_with_data(spi, Command::LUT_RED0, LUT_RED0)?;
        self.interface
            .cmd_with_data(spi, Command::LUT_RED1, LUT_RED1)?;

        Ok(())
    }

    fn is_busy(&self) -> bool {
        self.interface.is_busy(IS_BUSY_LOW)
    }
}

impl<SPI, CS, BUSY, DC, RST> EPD1in54b<SPI, CS, BUSY, DC, RST>
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

        self.command(spi, Command::RESOLUTION_SETTING)?;

        self.send_data(spi, &[w as u8])?;
        self.send_data(spi, &[(h >> 8) as u8])?;
        self.send_data(spi, &[h as u8])
    }
}

fn expand_bits(bits: u8) -> [u8; 2] {
    let mut x = bits as u16;

    x = (x | (x << 4)) & 0x0F0F;
    x = (x | (x << 2)) & 0x3333;
    x = (x | (x << 1)) & 0x5555;
    x = x | (x << 1);

    [(x >> 8) as u8, (x & 0xFF) as u8]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epd_size() {
        assert_eq!(WIDTH, 200);
        assert_eq!(HEIGHT, 200);
        assert_eq!(DEFAULT_BACKGROUND_COLOR, Color::White);
    }
}
