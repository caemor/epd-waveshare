//! A simple Driver for the Waveshare 1.54" (C) E-Ink Display via SPI

use embedded_hal::{
    blocking::{delay::*, spi::Write},
    digital::v2::*,
};

use crate::interface::DisplayInterface;
use crate::traits::{
    InternalWiAdditions, RefreshLut, WaveshareDisplay, WaveshareThreeColorDisplay,
};

/// Width of epd1in54 in pixels
pub const WIDTH: u32 = 152;
/// Height of epd1in54 in pixels
pub const HEIGHT: u32 = 152;
/// Default Background Color (white)
pub const DEFAULT_BACKGROUND_COLOR: Color = Color::White;
const IS_BUSY_LOW: bool = true;
const NUM_DISPLAY_BITS: u32 = WIDTH * HEIGHT / 8;

use crate::color::Color;

pub(crate) mod command;
use self::command::Command;

#[cfg(feature = "graphics")]
mod graphics;

#[cfg(feature = "graphics")]
pub use self::graphics::Display1in54c;

/// Epd1in54c driver
pub struct Epd1in54c<SPI, CS, BUSY, DC, RST, DELAY> {
    interface: DisplayInterface<SPI, CS, BUSY, DC, RST, DELAY>,
    color: Color,
}

impl<SPI, CS, BUSY, DC, RST, DELAY> InternalWiAdditions<SPI, CS, BUSY, DC, RST, DELAY>
    for Epd1in54c<SPI, CS, BUSY, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayMs<u8>,
{
    fn init(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        // Based on Reference Program Code from:
        // https://www.waveshare.com/w/upload/a/ac/1.54inch_e-Paper_Module_C_Specification.pdf
        // and:
        // https://github.com/waveshare/e-Paper/blob/master/STM32/STM32-F103ZET6/User/e-Paper/EPD_1in54c.c
        self.interface.reset(delay, 2);

        // start the booster
        self.cmd_with_data(spi, Command::BoosterSoftStart, &[0x17, 0x17, 0x17])?;

        // power on
        self.command(spi, Command::PowerOn)?;
        delay.delay_ms(5);
        self.wait_until_idle();

        // set the panel settings
        self.cmd_with_data(spi, Command::PanelSetting, &[0x0f, 0x0d])?;

        // set resolution
        self.send_resolution(spi)?;

        self.cmd_with_data(spi, Command::VcomAndDataIntervalSetting, &[0x77])?;

        Ok(())
    }
}

impl<SPI, CS, BUSY, DC, RST, DELAY> WaveshareThreeColorDisplay<SPI, CS, BUSY, DC, RST, DELAY>
    for Epd1in54c<SPI, CS, BUSY, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayMs<u8>,
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

    fn update_achromatic_frame(&mut self, spi: &mut SPI, black: &[u8]) -> Result<(), SPI::Error> {
        self.wait_until_idle();
        self.cmd_with_data(spi, Command::DataStartTransmission1, black)?;

        Ok(())
    }

    fn update_chromatic_frame(
        &mut self,
        spi: &mut SPI,
        chromatic: &[u8],
    ) -> Result<(), SPI::Error> {
        self.wait_until_idle();
        self.cmd_with_data(spi, Command::DataStartTransmission2, chromatic)?;

        Ok(())
    }
}

impl<SPI, CS, BUSY, DC, RST, DELAY> WaveshareDisplay<SPI, CS, BUSY, DC, RST, DELAY>
    for Epd1in54c<SPI, CS, BUSY, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayMs<u8>,
{
    type DisplayColor = Color;
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

        let mut epd = Epd1in54c { interface, color };

        epd.init(spi, delay)?;

        Ok(epd)
    }

    fn sleep(&mut self, spi: &mut SPI, _delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.wait_until_idle();

        self.command(spi, Command::PowerOff)?;
        self.wait_until_idle();
        self.cmd_with_data(spi, Command::DeepSleep, &[0xa5])?;

        Ok(())
    }

    fn wake_up(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
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

    fn update_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        _delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        self.update_achromatic_frame(spi, buffer)?;

        // Clear the chromatic layer
        let color = self.color.get_byte_value();

        self.command(spi, Command::DataStartTransmission2)?;
        self.interface.data_x_times(spi, color, NUM_DISPLAY_BITS)?;

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
        unimplemented!()
    }

    fn display_frame(&mut self, spi: &mut SPI, _delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.command(spi, Command::DisplayRefresh)?;
        self.wait_until_idle();

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

    fn clear_frame(&mut self, spi: &mut SPI, _delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.wait_until_idle();
        let color = DEFAULT_BACKGROUND_COLOR.get_byte_value();

        // Clear the black
        self.command(spi, Command::DataStartTransmission1)?;
        self.interface.data_x_times(spi, color, NUM_DISPLAY_BITS)?;

        // Clear the chromatic
        self.command(spi, Command::DataStartTransmission2)?;
        self.interface.data_x_times(spi, color, NUM_DISPLAY_BITS)?;

        Ok(())
    }

    fn set_lut(
        &mut self,
        _spi: &mut SPI,
        _refresh_rate: Option<RefreshLut>,
    ) -> Result<(), SPI::Error> {
        Ok(())
    }

    fn is_busy(&self) -> bool {
        self.interface.is_busy(IS_BUSY_LOW)
    }
}

impl<SPI, CS, BUSY, DC, RST, DELAY> Epd1in54c<SPI, CS, BUSY, DC, RST, DELAY>
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

    fn wait_until_idle(&mut self) {
        let _ = self.interface.wait_until_idle(IS_BUSY_LOW);
    }

    fn send_resolution(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        let w = self.width();
        let h = self.height();

        self.command(spi, Command::ResolutionSetting)?;

        // | D7 | D6 | D5 | D4 | D3 | D2 | D1 | D0 |
        // |       HRES[7:3]        |  0 |  0 |  0 |
        self.send_data(spi, &[(w as u8) & 0b1111_1000])?;
        // | D7 | D6 | D5 | D4 | D3 | D2 | D1 |      D0 |
        // |  - |  - |  - |  - |  - |  - |  - | VRES[8] |
        self.send_data(spi, &[(w >> 8) as u8])?;
        // | D7 | D6 | D5 | D4 | D3 | D2 | D1 |      D0 |
        // |                  VRES[7:0]                 |
        // Specification shows C/D is zero while sending the last byte,
        // but upstream code does not implement it like that. So for now
        // we follow upstream code.
        self.send_data(spi, &[h as u8])
    }
}
