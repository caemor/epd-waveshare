//! A simple Driver for the Waveshare 4.2" E-Ink Display via SPI
//!
//!
//! Build with the help of documentation/code from [Waveshare](https://www.waveshare.com/wiki/4.2inch_e-Paper_Module),
//! [Ben Krasnows partial Refresh tips](https://benkrasnow.blogspot.de/2017/10/fast-partial-refresh-on-42-e-paper.html) and
//! the driver documents in the `pdfs`-folder as orientation.
//!
//! # Examples
//!
//!```rust, no_run
//!# use embedded_hal_mock::*;
//!# fn main() -> Result<(), MockError> {
//!use embedded_graphics::{
//!    pixelcolor::BinaryColor::On as Black, prelude::*, primitives::Line, style::PrimitiveStyle,
//!};
//!use epd_waveshare::{epd4in2::*, prelude::*};
//!#
//!# let expectations = [];
//!# let mut spi = spi::Mock::new(&expectations);
//!# let expectations = [];
//!# let cs_pin = pin::Mock::new(&expectations);
//!# let busy_in = pin::Mock::new(&expectations);
//!# let dc = pin::Mock::new(&expectations);
//!# let rst = pin::Mock::new(&expectations);
//!# let mut delay = delay::MockNoop::new();
//!
//!// Setup EPD
//!let mut epd = EPD4in2::new(&mut spi, cs_pin, busy_in, dc, rst, &mut delay)?;
//!
//!// Use display graphics from embedded-graphics
//!let mut display = Display4in2::default();
//!
//!// Use embedded graphics for drawing a line
//!let _ = Line::new(Point::new(0, 120), Point::new(0, 295))
//!    .into_styled(PrimitiveStyle::with_stroke(Black, 1))
//!    .draw(&mut display);
//!
//!    // Display updated frame
//!epd.update_frame(&mut spi, &display.buffer())?;
//!epd.display_frame(&mut spi)?;
//!
//!// Set the EPD to sleep
//!epd.sleep(&mut spi)?;
//!# Ok(())
//!# }
//!```
//!
//!
//!
//! BE CAREFUL! The screen can get ghosting/burn-ins through the Partial Fast Update Drawing.

use embedded_hal::{
    blocking::{delay::*, spi::Write},
    digital::v2::*,
};

use crate::interface::DisplayInterface;
use crate::traits::{InternalWiAdditions, QuickRefresh, RefreshLUT, WaveshareDisplay};

//The Lookup Tables for the Display
mod constants;
use crate::epd4in2::constants::*;

/// Width of the display
pub const WIDTH: u32 = 400;
/// Height of the display
pub const HEIGHT: u32 = 300;
/// Default Background Color
pub const DEFAULT_BACKGROUND_COLOR: Color = Color::White;
const IS_BUSY_LOW: bool = true;

use crate::color::Color;

pub(crate) mod command;
use self::command::Command;

#[cfg(feature = "graphics")]
mod graphics;
#[cfg(feature = "graphics")]
pub use self::graphics::Display4in2;

/// EPD4in2 driver
///
pub struct EPD4in2<SPI, CS, BUSY, DC, RST> {
    /// Connection Interface
    interface: DisplayInterface<SPI, CS, BUSY, DC, RST>,
    /// Background Color
    color: Color,
    /// Refresh LUT
    refresh: RefreshLUT,
}

impl<SPI, CS, BUSY, DC, RST> InternalWiAdditions<SPI, CS, BUSY, DC, RST>
    for EPD4in2<SPI, CS, BUSY, DC, RST>
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

        // set the power settings
        self.interface.cmd_with_data(
            spi,
            Command::POWER_SETTING,
            &[0x03, 0x00, 0x2b, 0x2b, 0xff],
        )?;

        // start the booster
        self.interface
            .cmd_with_data(spi, Command::BOOSTER_SOFT_START, &[0x17, 0x17, 0x17])?;

        // power on
        self.command(spi, Command::POWER_ON)?;
        delay.delay_ms(5);
        self.wait_until_idle();

        // set the panel settings
        self.cmd_with_data(spi, Command::PANEL_SETTING, &[0x3F])?;

        // Set Frequency, 200 Hz didn't work on my board
        // 150Hz and 171Hz wasn't tested yet
        // TODO: Test these other frequencies
        // 3A 100HZ   29 150Hz 39 200HZ  31 171HZ DEFAULT: 3c 50Hz
        self.cmd_with_data(spi, Command::PLL_CONTROL, &[0x3A])?;

        self.send_resolution(spi)?;

        self.interface
            .cmd_with_data(spi, Command::VCM_DC_SETTING, &[0x12])?;

        //VBDF 17|D7 VBDW 97  VBDB 57  VBDF F7  VBDW 77  VBDB 37  VBDR B7
        self.interface
            .cmd_with_data(spi, Command::VCOM_AND_DATA_INTERVAL_SETTING, &[0x97])?;

        self.set_lut(spi, None)?;

        self.wait_until_idle();
        Ok(())
    }
}

impl<SPI, CS, BUSY, DC, RST> WaveshareDisplay<SPI, CS, BUSY, DC, RST>
    for EPD4in2<SPI, CS, BUSY, DC, RST>
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

        let mut epd = EPD4in2 {
            interface,
            color,
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
        self.wait_until_idle();
        self.interface
            .cmd_with_data(spi, Command::VCOM_AND_DATA_INTERVAL_SETTING, &[0x17])?; //border floating
        self.command(spi, Command::VCM_DC_SETTING)?; // VCOM to 0V
        self.command(spi, Command::PANEL_SETTING)?;

        self.command(spi, Command::POWER_SETTING)?; //VG&VS to 0V fast
        for _ in 0..4 {
            self.send_data(spi, &[0x00])?;
        }

        self.command(spi, Command::POWER_OFF)?;
        self.wait_until_idle();
        self.interface
            .cmd_with_data(spi, Command::DEEP_SLEEP, &[0xA5])?;
        Ok(())
    }

    fn update_frame(&mut self, spi: &mut SPI, buffer: &[u8]) -> Result<(), SPI::Error> {
        self.wait_until_idle();
        let color_value = self.color.get_byte_value();

        self.interface
            .cmd(spi, Command::DATA_START_TRANSMISSION_1)?;
        self.interface
            .data_x_times(spi, color_value, WIDTH / 8 * HEIGHT)?;

        self.interface
            .cmd_with_data(spi, Command::DATA_START_TRANSMISSION_2, buffer)?;
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
        self.wait_until_idle();
        if buffer.len() as u32 != width / 8 * height {
            //TODO: panic!! or sth like that
            //return Err("Wrong buffersize");
        }

        self.command(spi, Command::PARTIAL_IN)?;
        self.command(spi, Command::PARTIAL_WINDOW)?;
        self.send_data(spi, &[(x >> 8) as u8])?;
        let tmp = x & 0xf8;
        self.send_data(spi, &[tmp as u8])?; // x should be the multiple of 8, the last 3 bit will always be ignored
        let tmp = tmp + width - 1;
        self.send_data(spi, &[(tmp >> 8) as u8])?;
        self.send_data(spi, &[(tmp | 0x07) as u8])?;

        self.send_data(spi, &[(y >> 8) as u8])?;
        self.send_data(spi, &[y as u8])?;

        self.send_data(spi, &[((y + height - 1) >> 8) as u8])?;
        self.send_data(spi, &[(y + height - 1) as u8])?;

        self.send_data(spi, &[0x01])?; // Gates scan both inside and outside of the partial window. (default)

        //TODO: handle dtm somehow
        let is_dtm1 = false;
        if is_dtm1 {
            self.command(spi, Command::DATA_START_TRANSMISSION_1)? //TODO: check if data_start transmission 1 also needs "old"/background data here
        } else {
            self.command(spi, Command::DATA_START_TRANSMISSION_2)?
        }

        self.send_data(spi, buffer)?;

        self.command(spi, Command::PARTIAL_OUT)?;
        Ok(())
    }

    fn display_frame(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        self.wait_until_idle();
        self.command(spi, Command::DISPLAY_REFRESH)?;
        Ok(())
    }

    fn update_and_display_frame(&mut self, spi: &mut SPI, buffer: &[u8]) -> Result<(), SPI::Error> {
        self.update_frame(spi, buffer)?;
        self.command(spi, Command::DISPLAY_REFRESH)?;
        Ok(())
    }

    fn clear_frame(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        self.wait_until_idle();
        self.send_resolution(spi)?;

        let color_value = self.color.get_byte_value();

        self.interface
            .cmd(spi, Command::DATA_START_TRANSMISSION_1)?;
        self.interface
            .data_x_times(spi, color_value, WIDTH / 8 * HEIGHT)?;

        self.interface
            .cmd(spi, Command::DATA_START_TRANSMISSION_2)?;
        self.interface
            .data_x_times(spi, color_value, WIDTH / 8 * HEIGHT)?;
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
        refresh_rate: Option<RefreshLUT>,
    ) -> Result<(), SPI::Error> {
        if let Some(refresh_lut) = refresh_rate {
            self.refresh = refresh_lut;
        }
        match self.refresh {
            RefreshLUT::FULL => {
                self.set_lut_helper(spi, &LUT_VCOM0, &LUT_WW, &LUT_BW, &LUT_WB, &LUT_BB)
            }
            RefreshLUT::QUICK => self.set_lut_helper(
                spi,
                &LUT_VCOM0_QUICK,
                &LUT_WW_QUICK,
                &LUT_BW_QUICK,
                &LUT_WB_QUICK,
                &LUT_BB_QUICK,
            ),
        }
    }

    fn is_busy(&self) -> bool {
        self.interface.is_busy(IS_BUSY_LOW)
    }
}

impl<SPI, CS, BUSY, DC, RST> EPD4in2<SPI, CS, BUSY, DC, RST>
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
        self.send_data(spi, &[(w >> 8) as u8])?;
        self.send_data(spi, &[w as u8])?;
        self.send_data(spi, &[(h >> 8) as u8])?;
        self.send_data(spi, &[h as u8])
    }

    fn set_lut_helper(
        &mut self,
        spi: &mut SPI,
        lut_vcom: &[u8],
        lut_ww: &[u8],
        lut_bw: &[u8],
        lut_wb: &[u8],
        lut_bb: &[u8],
    ) -> Result<(), SPI::Error> {
        self.wait_until_idle();
        // LUT VCOM
        self.cmd_with_data(spi, Command::LUT_FOR_VCOM, lut_vcom)?;

        // LUT WHITE to WHITE
        self.cmd_with_data(spi, Command::LUT_WHITE_TO_WHITE, lut_ww)?;

        // LUT BLACK to WHITE
        self.cmd_with_data(spi, Command::LUT_BLACK_TO_WHITE, lut_bw)?;

        // LUT WHITE to BLACK
        self.cmd_with_data(spi, Command::LUT_WHITE_TO_BLACK, lut_wb)?;

        // LUT BLACK to BLACK
        self.cmd_with_data(spi, Command::LUT_BLACK_TO_BLACK, lut_bb)?;
        Ok(())
    }

    /// Helper function. Sets up the display to send pixel data to a custom
    /// starting point.
    pub fn shift_display(
        &mut self,
        spi: &mut SPI,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), SPI::Error> {
        self.send_data(spi, &[(x >> 8) as u8])?;
        let tmp = x & 0xf8;
        self.send_data(spi, &[tmp as u8])?; // x should be the multiple of 8, the last 3 bit will always be ignored
        let tmp = tmp + width - 1;
        self.send_data(spi, &[(tmp >> 8) as u8])?;
        self.send_data(spi, &[(tmp | 0x07) as u8])?;

        self.send_data(spi, &[(y >> 8) as u8])?;
        self.send_data(spi, &[y as u8])?;

        self.send_data(spi, &[((y + height - 1) >> 8) as u8])?;
        self.send_data(spi, &[(y + height - 1) as u8])?;

        self.send_data(spi, &[0x01])?; // Gates scan both inside and outside of the partial window. (default)

        Ok(())
    }
}

impl<SPI, CS, BUSY, DC, RST> QuickRefresh<SPI, CS, BUSY, DC, RST>
    for EPD4in2<SPI, CS, BUSY, DC, RST>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    /// To be followed immediately after by `update_old_frame`.
    fn update_old_frame(&mut self, spi: &mut SPI, buffer: &[u8]) -> Result<(), SPI::Error> {
        self.wait_until_idle();

        self.interface
            .cmd(spi, Command::DATA_START_TRANSMISSION_1)?;

        self.interface.data(spi, buffer)?;

        Ok(())
    }

    /// To be used immediately after `update_old_frame`.
    fn update_new_frame(&mut self, spi: &mut SPI, buffer: &[u8]) -> Result<(), SPI::Error> {
        self.wait_until_idle();
        // self.send_resolution(spi)?;

        self.interface
            .cmd(spi, Command::DATA_START_TRANSMISSION_2)?;

        self.interface.data(spi, buffer)?;

        Ok(())
    }

    fn update_partial_old_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), SPI::Error> {
        self.wait_until_idle();

        if buffer.len() as u32 != width / 8 * height {
            //TODO: panic!! or sth like that
            //return Err("Wrong buffersize");
        }

        self.interface.cmd(spi, Command::PARTIAL_IN)?;
        self.interface.cmd(spi, Command::PARTIAL_WINDOW)?;

        self.shift_display(spi, x, y, width, height)?;

        self.interface
            .cmd(spi, Command::DATA_START_TRANSMISSION_1)?;

        self.interface.data(spi, buffer)?;

        Ok(())
    }

    /// Always call `update_partial_old_frame` before this, with buffer-updating code
    /// between the calls.
    fn update_partial_new_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), SPI::Error> {
        self.wait_until_idle();
        if buffer.len() as u32 != width / 8 * height {
            //TODO: panic!! or sth like that
            //return Err("Wrong buffersize");
        }

        self.shift_display(spi, x, y, width, height)?;

        self.interface
            .cmd(spi, Command::DATA_START_TRANSMISSION_2)?;

        self.interface.data(spi, buffer)?;

        self.interface.cmd(spi, Command::PARTIAL_OUT)?;
        Ok(())
    }

    fn clear_partial_frame(
        &mut self,
        spi: &mut SPI,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), SPI::Error> {
        self.wait_until_idle();
        self.send_resolution(spi)?;

        let color_value = self.color.get_byte_value();

        self.interface.cmd(spi, Command::PARTIAL_IN)?;
        self.interface.cmd(spi, Command::PARTIAL_WINDOW)?;

        self.shift_display(spi, x, y, width, height)?;

        self.interface
            .cmd(spi, Command::DATA_START_TRANSMISSION_1)?;
        self.interface
            .data_x_times(spi, color_value, width / 8 * height)?;

        self.interface
            .cmd(spi, Command::DATA_START_TRANSMISSION_2)?;
        self.interface
            .data_x_times(spi, color_value, width / 8 * height)?;

        self.interface.cmd(spi, Command::PARTIAL_OUT)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epd_size() {
        assert_eq!(WIDTH, 400);
        assert_eq!(HEIGHT, 300);
        assert_eq!(DEFAULT_BACKGROUND_COLOR, Color::White);
    }
}
