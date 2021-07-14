//! A simple Driver for the Waveshare 2.9" E-Ink Display V2 via SPI
//!
//! Specification: <https://www.waveshare.com/w/upload/7/79/2.9inch-e-paper-v2-specification.pdf>
//!
//! # Example for the 2.9 in E-Ink Display V2
//!
//!```rust, no_run
//!# use embedded_hal_mock::*;
//!# fn main() -> Result<(), MockError> {
//!use embedded_graphics::{
//!    pixelcolor::BinaryColor::On as Black, prelude::*, primitives::{Line, PrimitiveStyle},
//!};
//!use epd_waveshare::{epd2in9_v2::*, prelude::*};
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
//!let mut epd = Epd2in9::new(&mut spi, cs_pin, busy_in, dc, rst, &mut delay)?;
//!
//!// Use display graphics from embedded-graphics
//!let mut display = Display2in9::default();
//!
//!// Use embedded graphics for drawing a line
//!let _ = Line::new(Point::new(0, 120), Point::new(0, 295))
//!    .into_styled(PrimitiveStyle::with_stroke(Black, 1))
//!    .draw(&mut display);
//!
//!// Display updated frame
//!epd.update_frame(&mut spi, &display.buffer(), &mut delay)?;
//!epd.display_frame(&mut spi, &mut delay)?;
//!
//!// Draw something new here
//!
//!// Display new image as a base image for further quick refreshes
//!epd.update_old_frame(&mut spi, &display.buffer(), &mut delay)?;
//!epd.display_frame(&mut spi, &mut delay)?;
//!
//!// Update image here
//!
//!// quick refresh of updated pixels
//!epd.update_new_frame(&mut spi, &display.buffer(), &mut delay)?;
//!epd.display_new_frame(&mut spi, &mut delay)?;
//!
//!// Set the EPD to sleep
//!epd.sleep(&mut spi, &mut delay)?;
//!# Ok(())
//!# }
//!```

/// Width of epd2in9 in pixels
pub const WIDTH: u32 = 128;
/// Height of epd2in9 in pixels
pub const HEIGHT: u32 = 296;
/// Default Background Color (white)
pub const DEFAULT_BACKGROUND_COLOR: Color = Color::White;
const IS_BUSY_LOW: bool = false;

const LUT_PARTIAL_2IN9: [u8; 153] = [
    0x0, 0x40, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x80, 0x80, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x40, 0x40, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x80, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0A, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x22, 0x22, 0x22, 0x22, 0x22,
    0x22, 0x0, 0x0, 0x0,
];

use embedded_hal::{
    blocking::{delay::*, spi::Write},
    digital::v2::*,
};

use crate::type_a::command::Command;

use crate::color::Color;

use crate::traits::*;

use crate::interface::DisplayInterface;
use crate::traits::QuickRefresh;

#[cfg(feature = "graphics")]
mod graphics;
#[cfg(feature = "graphics")]
pub use crate::epd2in9_v2::graphics::Display2in9;

/// Epd2in9 driver
///
pub struct Epd2in9<SPI, CS, BUSY, DC, RST, DELAY> {
    /// SPI
    interface: DisplayInterface<SPI, CS, BUSY, DC, RST, DELAY>,
    /// Color
    background_color: Color,
    /// Refresh LUT
    refresh: RefreshLut,
}

impl<SPI, CS, BUSY, DC, RST, DELAY> Epd2in9<SPI, CS, BUSY, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayMs<u8>,
{
    fn init(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.interface.reset(delay, 2);

        self.wait_until_idle();
        self.interface.cmd(spi, Command::SwReset)?;
        self.wait_until_idle();

        // 3 Databytes:
        // A[7:0]
        // 0.. A[8]
        // 0.. B[2:0]
        // Default Values: A = Height of Screen (0x127), B = 0x00 (GD, SM and TB=0?)
        self.interface
            .cmd_with_data(spi, Command::DriverOutputControl, &[0x27, 0x01, 0x00])?;

        // One Databyte with default value 0x03
        //  -> address: x increment, y increment, address counter is updated in x direction
        self.interface
            .cmd_with_data(spi, Command::DataEntryModeSetting, &[0x03])?;

        self.set_ram_area(spi, 0, 0, WIDTH - 1, HEIGHT - 1)?;

        self.interface
            .cmd_with_data(spi, Command::DisplayUpdateControl1, &[0x00, 0x80])?;

        self.set_ram_counter(spi, 0, 0)?;

        self.wait_until_idle();
        Ok(())
    }
}

impl<SPI, CS, BUSY, DC, RST, DELAY> WaveshareDisplay<SPI, CS, BUSY, DC, RST, DELAY>
    for Epd2in9<SPI, CS, BUSY, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayMs<u8>,
{
    type DisplayColor = Color;
    fn width(&self) -> u32 {
        WIDTH
    }

    fn height(&self) -> u32 {
        HEIGHT
    }

    fn new(
        spi: &mut SPI,
        cs: CS,
        busy: BUSY,
        dc: DC,
        rst: RST,
        delay: &mut DELAY,
    ) -> Result<Self, SPI::Error> {
        let interface = DisplayInterface::new(cs, busy, dc, rst);

        let mut epd = Epd2in9 {
            interface,
            background_color: DEFAULT_BACKGROUND_COLOR,
            refresh: RefreshLut::Full,
        };

        epd.init(spi, delay)?;

        Ok(epd)
    }

    fn sleep(&mut self, spi: &mut SPI, _delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.wait_until_idle();
        // 0x00 for Normal mode (Power on Reset), 0x01 for Deep Sleep Mode
        self.interface
            .cmd_with_data(spi, Command::DeepSleepMode, &[0x01])?;
        Ok(())
    }

    fn wake_up(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.init(spi, delay)?;
        Ok(())
    }

    fn update_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        _delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        self.wait_until_idle();
        self.interface.cmd_with_data(spi, Command::WriteRam, buffer)
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
        //TODO This is copied from epd2in9 but it seems not working. Partial refresh supported by version 2?
        self.wait_until_idle();
        self.set_ram_area(spi, x, y, x + width, y + height)?;
        self.set_ram_counter(spi, x, y)?;

        self.interface
            .cmd_with_data(spi, Command::WriteRam, buffer)?;
        Ok(())
    }

    fn display_frame(&mut self, spi: &mut SPI, _delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.wait_until_idle();
        // Enable clock signal, Enable Analog, Load temperature value, DISPLAY with DISPLAY Mode 1, Disable Analog, Disable OSC
        self.interface
            .cmd_with_data(spi, Command::DisplayUpdateControl2, &[0xF7])?;
        self.interface.cmd(spi, Command::MasterActivation)?;
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

        // clear the ram with the background color
        let color = self.background_color.get_byte_value();

        self.interface.cmd(spi, Command::WriteRam)?;
        self.interface.data_x_times(spi, color, WIDTH / 8 * HEIGHT)
    }

    fn set_background_color(&mut self, background_color: Color) {
        self.background_color = background_color;
    }

    fn background_color(&self) -> &Color {
        &self.background_color
    }

    fn set_lut(
        &mut self,
        _spi: &mut SPI,
        refresh_rate: Option<RefreshLut>,
    ) -> Result<(), SPI::Error> {
        if let Some(refresh_lut) = refresh_rate {
            self.refresh = refresh_lut;
        }
        Ok(())
    }

    fn is_busy(&self) -> bool {
        self.interface.is_busy(IS_BUSY_LOW)
    }
}

impl<SPI, CS, BUSY, DC, RST, DELAY> Epd2in9<SPI, CS, BUSY, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayMs<u8>,
{
    fn wait_until_idle(&mut self) {
        self.interface.wait_until_idle(IS_BUSY_LOW);
    }

    fn use_full_frame(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        // choose full frame/ram
        self.set_ram_area(spi, 0, 0, WIDTH - 1, HEIGHT - 1)?;

        // start from the beginning
        self.set_ram_counter(spi, 0, 0)
    }

    fn set_ram_area(
        &mut self,
        spi: &mut SPI,
        start_x: u32,
        start_y: u32,
        end_x: u32,
        end_y: u32,
    ) -> Result<(), SPI::Error> {
        assert!(start_x < end_x);
        assert!(start_y < end_y);

        // x is positioned in bytes, so the last 3 bits which show the position inside a byte in the ram
        // aren't relevant
        self.interface.cmd_with_data(
            spi,
            Command::SetRamXAddressStartEndPosition,
            &[(start_x >> 3) as u8, (end_x >> 3) as u8],
        )?;

        // 2 Databytes: A[7:0] & 0..A[8] for each - start and end
        self.interface.cmd_with_data(
            spi,
            Command::SetRamYAddressStartEndPosition,
            &[
                start_y as u8,
                (start_y >> 8) as u8,
                end_y as u8,
                (end_y >> 8) as u8,
            ],
        )
    }

    fn set_ram_counter(&mut self, spi: &mut SPI, x: u32, y: u32) -> Result<(), SPI::Error> {
        self.wait_until_idle();
        // x is positioned in bytes, so the last 3 bits which show the position inside a byte in the ram
        // aren't relevant
        self.interface
            .cmd_with_data(spi, Command::SetRamXAddressCounter, &[x as u8])?;

        // 2 Databytes: A[7:0] & 0..A[8]
        self.interface.cmd_with_data(
            spi,
            Command::SetRamYAddressCounter,
            &[y as u8, (y >> 8) as u8],
        )?;
        Ok(())
    }

    /// Set your own LUT, this function is also used internally for set_lut
    fn set_lut_helper(&mut self, spi: &mut SPI, buffer: &[u8]) -> Result<(), SPI::Error> {
        self.wait_until_idle();
        self.interface
            .cmd_with_data(spi, Command::WriteLutRegister, buffer)?;
        self.wait_until_idle();
        Ok(())
    }
}

impl<SPI, CS, BUSY, DC, RST, DELAY> QuickRefresh<SPI, CS, BUSY, DC, RST, DELAY>
    for Epd2in9<SPI, CS, BUSY, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayMs<u8>,
{
    /// To be followed immediately by `update_new_frame`.
    fn update_old_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        _delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        self.wait_until_idle();
        self.interface
            .cmd_with_data(spi, Command::WriteRam, buffer)?;
        self.interface
            .cmd_with_data(spi, Command::WriteRam2, buffer)
    }

    /// To be used immediately after `update_old_frame`.
    fn update_new_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        self.wait_until_idle();
        self.interface.reset(delay, 2);

        self.set_lut_helper(spi, &LUT_PARTIAL_2IN9)?;
        self.interface.cmd_with_data(
            spi,
            Command::WriteOtpSelection,
            &[0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00],
        )?;
        self.interface
            .cmd_with_data(spi, Command::BorderWaveformControl, &[0x80])?;
        self.interface
            .cmd_with_data(spi, Command::DisplayUpdateControl2, &[0xC0])?;
        self.interface.cmd(spi, Command::MasterActivation)?;

        self.wait_until_idle();

        self.use_full_frame(spi)?;

        self.interface
            .cmd_with_data(spi, Command::WriteRam, buffer)?;
        Ok(())
    }

    /// For a quick refresh of the new updated frame. To be used immediately after `update_new_frame`
    fn display_new_frame(&mut self, spi: &mut SPI, _delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.wait_until_idle();
        self.interface
            .cmd_with_data(spi, Command::DisplayUpdateControl2, &[0x0F])?;
        self.interface.cmd(spi, Command::MasterActivation)?;
        self.wait_until_idle();
        Ok(())
    }

    /// Updates and displays the new frame.
    fn update_and_display_new_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        self.update_new_frame(spi, buffer, delay)?;
        self.display_new_frame(spi, delay)?;
        Ok(())
    }

    /// Partial quick refresh not supported yet
    #[allow(unused)]
    fn update_partial_old_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), SPI::Error> {
        //TODO supported by display?
        unimplemented!()
    }

    /// Partial quick refresh not supported yet
    #[allow(unused)]
    fn update_partial_new_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), SPI::Error> {
        //TODO supported by display?
        unimplemented!()
    }

    /// Partial quick refresh not supported yet
    #[allow(unused)]
    fn clear_partial_frame(
        &mut self,
        spi: &mut SPI,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), SPI::Error> {
        //TODO supported by display?
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epd_size() {
        assert_eq!(WIDTH, 128);
        assert_eq!(HEIGHT, 296);
        assert_eq!(DEFAULT_BACKGROUND_COLOR, Color::White);
    }
}
