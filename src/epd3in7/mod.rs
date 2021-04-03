//! A simple Driver for the Waveshare 3.7" E-Ink Display via SPI
//!
//!
//! Build with the help of documentation/code from [Waveshare](https://www.waveshare.com/wiki/3.7inch_e-Paper_HAT),
use embedded_hal::{
    blocking::{delay::*, spi::Write},
    digital::v2::{InputPin, OutputPin},
};

pub(crate) mod command;
mod constants;
#[cfg(feature = "graphics")]
mod graphics;

use self::command::Command;
use self::constants::{LUT_CLEAR, LUT_FULL_UPDATE};
#[cfg(feature = "graphics")]
pub use self::graphics::Display3in7;
use crate::buffer_len;
use crate::color::Color;
use crate::interface::DisplayInterface;
use crate::traits::{InternalWiAdditions, RefreshLUT, WaveshareDisplay};

/// Width of the display.
pub const WIDTH: u32 = 280;

/// Height of the display
pub const HEIGHT: u32 = 480;

/// Default Background Color
pub const DEFAULT_BACKGROUND_COLOR: Color = Color::White;

const IS_BUSY_LOW: bool = true;

/// EPD3in7 driver
pub struct EPD3in7<SPI, CS, BUSY, DC, RST> {
    /// Connection Interface
    interface: DisplayInterface<SPI, CS, BUSY, DC, RST>,
    /// Background Color
    background_color: Color,
    /// Refresh LUT
    refresh: RefreshLUT,
}

impl<SPI, CS, BUSY, DC, RST> InternalWiAdditions<SPI, CS, BUSY, DC, RST>
    for EPD3in7<SPI, CS, BUSY, DC, RST>
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
        self.interface.reset(delay, 5);

        self.interface.cmd(spi, Command::SW_RESET)?;
        delay.delay_ms(200u8);
        delay.delay_ms(100u8);

        self.interface
            .cmd_with_data(spi, Command::AUTO_WRITE_RED_RAM_REGULAR_PATTERN, &[0xF7])?;
        self.interface.wait_until_idle(IS_BUSY_LOW);
        self.interface
            .cmd_with_data(spi, Command::AUTO_WRITE_BW_RAM_REGULAR_PATTERN, &[0xF7])?;
        self.interface.wait_until_idle(IS_BUSY_LOW);

        self.interface
            .cmd_with_data(spi, Command::GATE_SETTING, &[0xDF, 0x01, 0x00])?;
        self.interface
            .cmd_with_data(spi, Command::GATE_VOLTAGE, &[0x00])?;
        self.interface
            .cmd_with_data(spi, Command::GATE_VOLTAGE_SOURCE, &[0x41, 0xA8, 0x32])?;

        self.interface
            .cmd_with_data(spi, Command::DATA_ENTRY_SEQUENCE, &[0x03])?;

        self.interface
            .cmd_with_data(spi, Command::BORDER_WAVEFORM_CONTROL, &[0x03])?;

        self.interface.cmd_with_data(
            spi,
            Command::BOOSTER_SOFT_START_CONTROL,
            &[0xAE, 0xC7, 0xC3, 0xC0, 0xC0],
        )?;

        self.interface
            .cmd_with_data(spi, Command::TEMPERATURE_SENSOR_SELECTION, &[0x80])?;

        self.interface
            .cmd_with_data(spi, Command::WRITE_VCOM_REGISTER, &[0x44])?;

        self.interface.cmd_with_data(
            spi,
            Command::DISPLAY_OPTION,
            &[0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x4F, 0xFF, 0xFF, 0xFF, 0xFF],
        )?;

        self.interface.cmd_with_data(
            spi,
            Command::SET_RAM_X_ADDRESS_START_END_POSITION,
            &[0x00, 0x00, 0x17, 0x01],
        )?;
        self.interface.cmd_with_data(
            spi,
            Command::SET_RAM_Y_ADDRESS_START_END_POSITION,
            &[0x00, 0x00, 0xDF, 0x01],
        )?;

        self.interface
            .cmd_with_data(spi, Command::DIPSLAY_UPDATE_SEQUENCE_SETTING, &[0xCF])?;

        Ok(())
    }
}

impl<SPI, CS, BUSY, DC, RST> WaveshareDisplay<SPI, CS, BUSY, DC, RST>
    for EPD3in7<SPI, CS, BUSY, DC, RST>
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
        let mut epd = EPD3in7 {
            interface: DisplayInterface::new(cs, busy, dc, rst),
            background_color: DEFAULT_BACKGROUND_COLOR,
            refresh: RefreshLUT::FULL,
        };

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
        self.interface.cmd_with_data(spi, Command::SLEEP, &[0xF7])?;
        self.interface.cmd(spi, Command::POWER_OFF)?;
        self.interface.cmd_with_data(spi, Command::SLEEP_2, &[0xA5])?;
        Ok(())
    }

    fn set_background_color(&mut self, color: Self::DisplayColor) {
        self.background_color = color;
    }

    fn background_color(&self) -> &Self::DisplayColor {
        &self.background_color
    }

    fn width(&self) -> u32 {
        WIDTH
    }

    fn height(&self) -> u32 {
        HEIGHT
    }

    fn update_frame(&mut self, spi: &mut SPI, buffer: &[u8]) -> Result<(), SPI::Error> {
        assert!(buffer.len() == buffer_len(WIDTH as usize, HEIGHT as usize));
        self.interface
            .cmd_with_data(spi, Command::SET_RAM_X_ADDRESS_COUNTER, &[0x00, 0x00])?;
        self.interface
            .cmd_with_data(spi, Command::SET_RAM_Y_ADDRESS_COUNTER, &[0x00, 0x00])?;

        self.interface
            .cmd_with_data(spi, Command::WRITE_RAM, buffer)?;

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
        todo!()
    }

    fn display_frame(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        self.interface.cmd(spi, Command::DISPLAY_UPDATE_SEQUENCE)?;
        self.interface.wait_until_idle(IS_BUSY_LOW);
        Ok(())
    }

    fn update_and_display_frame(&mut self, spi: &mut SPI, buffer: &[u8]) -> Result<(), SPI::Error> {
        self.update_frame(spi, buffer)?;
        self.display_frame(spi)?;
        Ok(())
    }

    fn clear_frame(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        let color = self.background_color.get_byte_value();
        self.interface
            .cmd_with_data(spi, Command::SET_RAM_X_ADDRESS_COUNTER, &[0x00, 0x00])?;
        self.interface
            .cmd_with_data(spi, Command::SET_RAM_Y_ADDRESS_COUNTER, &[0x00, 0x00])?;

        self.interface.cmd(spi, Command::WRITE_RAM)?;
        self.interface.cmd_with_data(
            spi,
            Command::WRITE_RAM,
            &[color; buffer_len(WIDTH as usize, HEIGHT as usize)],
        )?;
        self.interface
            .cmd_with_data(spi, Command::WRITE_LUT_REGISTER, &LUT_CLEAR)?;

        Ok(())
    }

    fn set_lut(
        &mut self,
        spi: &mut SPI,
        refresh_rate: Option<RefreshLUT>,
    ) -> Result<(), SPI::Error> {
        let buffer = match refresh_rate {
            Some(RefreshLUT::FULL) | None => &LUT_FULL_UPDATE,
            Some(RefreshLUT::QUICK) => &LUT_FULL_UPDATE,
        };

        self.interface
            .cmd_with_data(spi, Command::WRITE_LUT_REGISTER, buffer)?;
        Ok(())
    }

    fn is_busy(&self) -> bool {
        self.interface.is_busy(IS_BUSY_LOW)
    }
}
