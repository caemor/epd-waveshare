//! A simple Driver for the Waveshare 2.13" (B/C) E-Ink Display via SPI
//! More information on this display can be found at the [Waveshare Wiki](https://www.waveshare.com/wiki/2.13inch_e-Paper_HAT_(B))
//! This driver was build and tested for 212x104, 2.13inch E-Ink display HAT for Raspberry Pi, three-color, SPI interface
//!
//! # Example for the 2.13" E-Ink Display
//!
//!```rust, no_run
//!# use embedded_hal_mock::eh1::*;
//!# fn main() -> Result<(), embedded_hal::spi::ErrorKind> {
//!use embedded_graphics::{prelude::*, primitives::{Line, PrimitiveStyle, PrimitiveStyleBuilder}};
//!use epd_waveshare::{epd2in13bV4::*, prelude::*};
//!#
//!# let expectations = [];
//!# let mut spi = spi::Mock::new(&expectations);
//!# let expectations = [];
//!# let cs_pin = pin::Mock::new(&expectations);
//!# let busy_in = pin::Mock::new(&expectations);
//!# let dc = pin::Mock::new(&expectations);
//!# let rst = pin::Mock::new(&expectations);
//!# let mut delay = delay::NoopDelay::new();
//!
//!// Setup EPD
//!let mut epd = Epd2in13bV4::new(&mut spi, busy_in, dc, rst, &mut delay, None)?;
//!
//!// Use display graphics from embedded-graphics
//!// This display is for the black/white/chromatic pixels
//!let mut tricolor_display = Display2in13bV4::default();
//!
//!// Use embedded graphics for drawing a black line
//!let _ = Line::new(Point::new(0, 120), Point::new(0, 200))
//!    .into_styled(PrimitiveStyle::with_stroke(TriColor::Black, 1))
//!    .draw(&mut tricolor_display);
//!
//!// We use `chromatic` but it will be shown as red/yellow
//!let _ = Line::new(Point::new(15, 120), Point::new(15, 200))
//!    .into_styled(PrimitiveStyle::with_stroke(TriColor::Chromatic, 1))
//!    .draw(&mut tricolor_display);
//!
//!// Display updated frame
//!epd.update_color_frame(
//!    &mut spi,
//!    &mut delay,
//!    &tricolor_display.bw_buffer(),
//!    &tricolor_display.chromatic_buffer()
//!)?;
//!epd.display_frame(&mut spi, &mut delay)?;
//!
//!// Set the EPD to sleep
//!epd.sleep(&mut spi, &mut delay)?;
//!# Ok(())
//!# }
//!```
use embedded_hal::{delay::*, digital::*, spi::SpiDevice};

use crate::interface::DisplayInterface;
use crate::traits::{
    InternalWiAdditions, RefreshLut, WaveshareDisplay, WaveshareThreeColorDisplay,
};

/// Width of epd2in13bV4 in pixels
pub const WIDTH: u32 = 122;
/// Height of epd2in13bV4 in pixels
pub const HEIGHT: u32 = 250;
/// Default background color (white) of epd2in13bV4 display
pub const DEFAULT_BACKGROUND_COLOR: TriColor = TriColor::White;

/// Number of bits for b/w buffer and same for chromatic buffer
const NUM_DISPLAY_BYTES: u32 = (WIDTH + 7) / 8 * HEIGHT;

const IS_BUSY_LOW: bool = false;
const SINGLE_BYTE_WRITE: bool = true;

use crate::color::TriColor;

pub(crate) mod command;
use self::command::Command;
use crate::buffer_len;

/// Full size buffer for use with the 2.13" b V4 EPD
#[cfg(feature = "graphics")]
pub type Display2in13bV4 = crate::graphics::Display<
    WIDTH,
    HEIGHT,
    false,
    { buffer_len(WIDTH as usize, HEIGHT as usize * 2) },
    TriColor,
>;

/// Epd2in13bV4 driver
pub struct Epd2in13bV4<SPI, BUSY, DC, RST, DELAY> {
    interface: DisplayInterface<SPI, BUSY, DC, RST, DELAY, SINGLE_BYTE_WRITE>,
    color: TriColor,
}

impl<SPI, BUSY, DC, RST, DELAY> InternalWiAdditions<SPI, BUSY, DC, RST, DELAY>
    for Epd2in13bV4<SPI, BUSY, DC, RST, DELAY>
where
    SPI: SpiDevice,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayNs,
{
    fn init(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        // Values taken from datasheet and sample code

        // reset the device
        self.interface.reset(delay, 20_000, 2_000);
        self.wait_until_idle(spi, delay)?;

        // reset s/w settings
        self.interface.cmd(spi, Command::SwReset)?;
        self.wait_until_idle(spi, delay)?;

        // ???
        self.cmd_with_data(spi, Command::DriverOutputControl, &[0xF9, 0x00, 0x00])?;

        // Enter data entry mode
        self.cmd_with_data(spi, Command::DataEntryModeSetting, &[0x03])?;

        // Set the screen resolution
        self.send_resolution(spi)?;

        // Initialize the cursor
        self.set_cursor(spi, 0, 0)?;

        // Select border waveform
        self.cmd_with_data(spi, Command::SelectBorderWaveform, &[0x05])?;

        // Display update control
        self.cmd_with_data(spi, Command::DisplayUpdateControl, &[0x80, 0x80])?;
        self.wait_until_idle(spi, delay)?;

        Ok(())
    }
}

impl<SPI, BUSY, DC, RST, DELAY> WaveshareThreeColorDisplay<SPI, BUSY, DC, RST, DELAY>
    for Epd2in13bV4<SPI, BUSY, DC, RST, DELAY>
where
    SPI: SpiDevice,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayNs,
{
    fn update_color_frame(
        &mut self,
        spi: &mut SPI,
        delay: &mut DELAY,
        black: &[u8],
        chromatic: &[u8],
    ) -> Result<(), SPI::Error> {
        self.update_achromatic_frame(spi, delay, black)?;
        self.update_chromatic_frame(spi, delay, chromatic)?;
        Ok(())
    }

    /// Update only the black/white data of the display.
    ///
    /// Finish by calling `update_chromatic_frame`.
    fn update_achromatic_frame(
        &mut self,
        spi: &mut SPI,
        _delay: &mut DELAY,
        black: &[u8],
    ) -> Result<(), SPI::Error> {
        self.set_cursor(spi, 0, 0)?;
        self.interface.cmd(spi, Command::WriteRamBlackWhite)?;
        self.interface.data(spi, black)?;

        Ok(())
    }

    /// Update only chromatic data of the display.
    ///
    /// This data takes precedence over the black/white data.
    fn update_chromatic_frame(
        &mut self,
        spi: &mut SPI,
        _delay: &mut DELAY,
        chromatic: &[u8],
    ) -> Result<(), SPI::Error> {
        self.set_cursor(spi, 0, 0)?;
        self.interface.cmd(spi, Command::WriteRamRed)?;
        // A TriColor display stores colored pixels as 1s in it's bitfield,
        // but this screen considers 1s to be white, while 0s are
        // considered  red. Therfore we have to flip the bitfiled before we
        // send it to the device
        let mut inverted_chromatic = [0u8; { NUM_DISPLAY_BYTES as usize}];
        for (i, &byte) in chromatic.iter().enumerate() {
            inverted_chromatic[i] = !byte;
        }
        self.interface.data(spi, &inverted_chromatic[..chromatic.len()])?;

        Ok(())
    }
}

impl<SPI, BUSY, DC, RST, DELAY> WaveshareDisplay<SPI, BUSY, DC, RST, DELAY>
    for Epd2in13bV4<SPI, BUSY, DC, RST, DELAY>
where
    SPI: SpiDevice,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayNs,
{
    type DisplayColor = TriColor;
    fn new(
        spi: &mut SPI,
        busy: BUSY,
        dc: DC,
        rst: RST,
        delay: &mut DELAY,
        delay_us: Option<u32>,
    ) -> Result<Self, SPI::Error> {
        let interface = DisplayInterface::new(busy, dc, rst, delay_us);
        let color = DEFAULT_BACKGROUND_COLOR;

        let mut epd = Epd2in13bV4 { interface, color };

        epd.init(spi, delay)?;

        Ok(epd)
    }

    fn sleep(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.interface.cmd_with_data(spi, Command::DeepSleepMode, &[0x01])?;
        delay.delay_us(200_000);
        Ok(())
    }

    fn wake_up(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.init(spi, delay)
    }

    fn set_background_color(&mut self, color: TriColor) {
        self.color = color;
    }

    fn background_color(&self) -> &TriColor {
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
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        self.set_cursor(spi, 0, 0)?;
        self.update_achromatic_frame(spi, delay, buffer)?;

        // Clear the chromatic layer
        self.interface.cmd(spi, Command::WriteRamRed)?;
        self.interface.data_x_times(spi, 0xFF, NUM_DISPLAY_BYTES)?;

        Ok(())
    }

    #[allow(unused)]
    fn update_partial_frame(
        &mut self,
        spi: &mut SPI,
        delay: &mut DELAY,
        buffer: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), SPI::Error> {
        Ok(())
    }

    fn display_frame(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.command(spi, Command::MasterActivation)?;
        self.wait_until_idle(spi, delay)?;
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
        self.send_resolution(spi)?;

        match DEFAULT_BACKGROUND_COLOR {
            TriColor::Chromatic => {
                // Clear the black
                self.interface.cmd(spi, Command::WriteRamBlackWhite)?;
                self.interface.data_x_times(spi, TriColor::White.get_byte_value(), NUM_DISPLAY_BYTES)?;
                // Clear the chromatic
                self.interface.cmd(spi, Command::WriteRamRed)?;
                self.interface.data_x_times(spi, 0x00, NUM_DISPLAY_BYTES)?;
            }
            _ => {
                // Clear the black
                self.interface.cmd(spi, Command::WriteRamBlackWhite)?;
                self.interface.data_x_times(spi, DEFAULT_BACKGROUND_COLOR.get_byte_value(), NUM_DISPLAY_BYTES)?;
                // Clear the chromatic
                self.interface.cmd(spi, Command::WriteRamRed)?;
                self.interface.data_x_times(spi, 0xFF, NUM_DISPLAY_BYTES)?;
            }
        }

        Ok(())
    }

    fn set_lut(
        &mut self,
        _spi: &mut SPI,
        _delay: &mut DELAY,
        _refresh_rate: Option<RefreshLut>,
    ) -> Result<(), SPI::Error> {
        Ok(())
    }

    fn wait_until_idle(&mut self, _spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.interface.wait_until_idle(delay, IS_BUSY_LOW);
        Ok(())
    }
}

impl<SPI, BUSY, DC, RST, DELAY> Epd2in13bV4<SPI, BUSY, DC, RST, DELAY>
where
    SPI: SpiDevice,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayNs,
{
    fn command(&mut self, spi: &mut SPI, command: Command) -> Result<(), SPI::Error> {
        self.interface.cmd(spi, command)
    }

    #[allow(unused)]
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

    fn send_resolution(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        let w = self.width();
        let h = self.height();

        self.set_window(spi, 0, 0, w, h)
    }

    fn set_window(&mut self, spi: &mut SPI, x: u32, y: u32, width: u32, height: u32) -> Result<(), SPI::Error> {
        let xstart = x;
        let xend = xstart + width;
        let ystart = y;
        let yend = ystart + height;

        self.cmd_with_data(
            spi,
            Command::SetRamXAddressStartEndPosition, 
            &[
                ((xstart>>3) & 0xFF).try_into().unwrap(),
                ((xend>>3) & 0xFF).try_into().unwrap()
            ]
        )?;
        self.cmd_with_data(
            spi,
            Command::SetRamYAddressStartEndPosition,
            &[
                (ystart & 0xFF).try_into().unwrap(),
                ((ystart >> 8) & 0xFF).try_into().unwrap(),
                (yend & 0xFF).try_into().unwrap(),
                ((yend >> 8) & 0xFF).try_into().unwrap()
            ]
        )?;

        Ok(())
    }

    fn set_cursor(&mut self, spi: &mut SPI, x: u32, y: u32) -> Result<(), SPI::Error> {
        self.cmd_with_data(
            spi,
            Command::SetRamXAddressCounter, 
            &[
                (x & 0xFF).try_into().unwrap()
            ]
        )?;
        self.cmd_with_data(
            spi,
            Command::SetRamYAddressCounter, 
            &[
                (y & 0xFF).try_into().unwrap(), 
                ((y >> 8) & 0xFF).try_into().unwrap()
            ]
        )?;
        Ok(())
    }

    /// Set the outer border of the display to the chosen color.
    pub fn set_border_color(&mut self, _spi: &mut SPI, _color: TriColor) -> Result<(), SPI::Error> {
        Ok(())
    }
}
