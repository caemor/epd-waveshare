//! A simple Driver for the Waveshare 2.9" (B) E-Ink Display via SPI

use embedded_hal::{
    blocking::{delay::*, spi::Write},
    digital::v2::*,
};

use crate::interface::DisplayInterface;
use crate::traits::{
    InternalWiAdditions, RefreshLUT, WaveshareDisplay, WaveshareThreeColorDisplay,
};

/// Width of epd2in9b in pixels
pub const WIDTH: u32 = 128;
/// Height of epd2in9b in pixels
pub const HEIGHT: u32 = 296;
/// Default background color (white) of epd2in9b display
pub const DEFAULT_BACKGROUND_COLOR: Color = Color::White;

const NUM_DISPLAY_BITS: u32 = WIDTH * HEIGHT / 8;

const IS_BUSY_LOW: bool = true;
const VCOM_DATA_INTERVAL: u8 = 0x07;
const WHITE_BORDER: u8 = 0x70;
const BLACK_BORDER: u8 = 0x30;
const RED_BORDER: u8 = 0xb0;
const FLOATING_BORDER: u8 = 0xF0;

use crate::color::{Color, TriColor};

pub(crate) mod command;
use self::command::Command;

#[cfg(feature = "graphics")]
mod graphics;
#[cfg(feature = "graphics")]
pub use self::graphics::Display2in9b;

/// EPD2in9b driver
pub struct EPD2in9b<SPI, CS, BUSY, DC, RST> {
    interface: DisplayInterface<SPI, CS, BUSY, DC, RST>,
    color: Color,
}

impl<SPI, CS, BUSY, DC, RST> InternalWiAdditions<SPI, CS, BUSY, DC, RST>
    for EPD2in9b<SPI, CS, BUSY, DC, RST>
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
        // Values taken from datasheet and sample code

        self.interface.reset(delay);

        // start the booster
        self.interface
            .cmd_with_data(spi, Command::BOOSTER_SOFT_START, &[0x17, 0x17, 0x17])?;

        // power on
        self.command(spi, Command::POWER_ON)?;
        delay.delay_ms(5);
        self.wait_until_idle();

        // set the panel settings
        self.cmd_with_data(spi, Command::PANEL_SETTING, &[0x8F])?;

        self.cmd_with_data(
            spi,
            Command::VCOM_AND_DATA_INTERVAL_SETTING,
            &[WHITE_BORDER | VCOM_DATA_INTERVAL],
        )?;

        // set resolution
        self.send_resolution(spi)?;

        self.cmd_with_data(spi, Command::VCM_DC_SETTING, &[0x0A])?;

        self.wait_until_idle();

        Ok(())
    }
}

impl<SPI, CS, BUSY, DC, RST> WaveshareThreeColorDisplay<SPI, CS, BUSY, DC, RST>
    for EPD2in9b<SPI, CS, BUSY, DC, RST>
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
        self.update_mono_frame(spi, black)?;
        self.update_red_frame(spi, red)
    }
}

impl<SPI, CS, BUSY, DC, RST> WaveshareDisplay<SPI, CS, BUSY, DC, RST>
    for EPD2in9b<SPI, CS, BUSY, DC, RST>
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

        let mut epd = EPD2in9b { interface, color };

        epd.init(spi, delay)?;

        Ok(epd)
    }

    fn sleep(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        // Section 8.2 from datasheet
        self.interface.cmd_with_data(
            spi,
            Command::VCOM_AND_DATA_INTERVAL_SETTING,
            &[FLOATING_BORDER | VCOM_DATA_INTERVAL],
        )?;

        self.command(spi, Command::POWER_OFF)?;
        // The example STM code from Github has a wait after POWER_OFF
        self.wait_until_idle();

        self.cmd_with_data(spi, Command::DEEP_SLEEP, &[0xA5])?;

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
        self.interface
            .cmd(spi, Command::DATA_START_TRANSMISSION_1)?;

        self.interface.data(spi, &buffer)?;

        // Clear the red layer
        let color = self.color.get_byte_value();

        self.interface
            .cmd(spi, Command::DATA_START_TRANSMISSION_2)?;
        self.interface.data_x_times(spi, color, NUM_DISPLAY_BITS)?;

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

    fn update_and_display_frame(&mut self, spi: &mut SPI, buffer: &[u8]) -> Result<(), SPI::Error> {
        self.update_frame(spi, buffer)?;
        self.display_frame(spi)?;
        Ok(())
    }

    fn clear_frame(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        self.send_resolution(spi)?;

        let color = DEFAULT_BACKGROUND_COLOR.get_byte_value();

        // Clear the black
        self.interface
            .cmd(spi, Command::DATA_START_TRANSMISSION_1)?;

        self.interface.data_x_times(spi, color, NUM_DISPLAY_BITS)?;

        // Clear the red
        self.interface
            .cmd(spi, Command::DATA_START_TRANSMISSION_2)?;
        self.interface.data_x_times(spi, color, NUM_DISPLAY_BITS)?;

        self.wait_until_idle();
        Ok(())
    }

    fn set_lut(
        &mut self,
        _spi: &mut SPI,
        _refresh_rate: Option<RefreshLUT>,
    ) -> Result<(), SPI::Error> {
        Ok(())
    }

    fn is_busy(&self) -> bool {
        self.interface.is_busy(IS_BUSY_LOW)
    }
}

impl<SPI, CS, BUSY, DC, RST> EPD2in9b<SPI, CS, BUSY, DC, RST>
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

    /// Set the outer border of the display to the chosen color.
    pub fn set_border_color(&mut self, spi: &mut SPI, color: TriColor) -> Result<(), SPI::Error> {
        let border = match color {
            TriColor::Black => BLACK_BORDER,
            TriColor::White => WHITE_BORDER,
            TriColor::Red => RED_BORDER,
        };
        self.cmd_with_data(
            spi,
            Command::VCOM_AND_DATA_INTERVAL_SETTING,
            &[border | VCOM_DATA_INTERVAL],
        )
    }

    /// Update only the black/white data of the display.
    ///
    /// Finish by calling `update_red_frame`.
    pub fn update_mono_frame(&mut self, spi: &mut SPI, black: &[u8]) -> Result<(), SPI::Error> {
        self.interface
            .cmd(spi, Command::DATA_START_TRANSMISSION_1)?;
        self.interface.data(spi, black)?;
        Ok(())
    }

    /// Update only red data of the display.
    ///
    /// This data takes precedence over the black/white data.
    pub fn update_red_frame(&mut self, spi: &mut SPI, red: &[u8]) -> Result<(), SPI::Error> {
        self.interface
            .cmd(spi, Command::DATA_START_TRANSMISSION_2)?;
        self.interface.data(spi, red)?;

        self.wait_until_idle();
        Ok(())
    }
}
