//! A simple Driver for the Waveshare 1.54" E-Ink Display via SPI
//!
//! # Example for the 1.54 in E-Ink Display
//!
//! ```ignore
//! use eink_waveshare_rs::{
//!     epd1in54::{EPD1in54, Buffer1in54},
//!     graphics::{Display, DisplayRotation},
//!     prelude::*,
//! };
//!
//! // Setup EPD
//! let mut epd = EPD1in54::new(&mut spi, cs_pin, busy_in, dc, rst, &mut delay)?;
//!
//! // Use display graphics
//! let mut buffer = Buffer1in54::default();
//! let mut display = Display::new(epd.width(), epd.height(), &mut buffer.buffer);
//!
//! // Write some hello world in the screenbuffer
//! display.draw(
//!     Font6x8::render_str("Hello World!")
//!         .with_stroke(Some(Color::Black))
//!         .with_fill(Some(Color::White))
//!         .translate(Coord::new(5, 50))
//!         .into_iter(),
//! );
//!
//! // Display updated frame
//! epd.update_frame(&mut spi, &display.buffer()).unwrap();
//! epd.display_frame(&mut spi).expect("display frame new graphics");
//!
//! // Set the EPD to sleep
//! epd.sleep(&mut spi).expect("sleep");
//! ```

pub const WIDTH: u32 = 200;
pub const HEIGHT: u32 = 200;
//const DPI: u16 = 184;
pub const DEFAULT_BACKGROUND_COLOR: Color = Color::White;

use hal::{
    blocking::{delay::*, spi::Write},
    digital::*,
};

use type_a::{
    command::Command,
    constants::{LUT_FULL_UPDATE, LUT_PARTIAL_UPDATE},
};

use color::Color;

use traits::{RefreshLUT, WaveshareDisplay};

use interface::DisplayInterface;

mod graphics;
pub use epd1in54::graphics::Buffer1in54BlackWhite as Buffer1in54;

/// EPD1in54 driver
///
pub struct EPD1in54<SPI, CS, BUSY, DC, RST> {
    /// SPI
    interface: DisplayInterface<SPI, CS, BUSY, DC, RST>,
    /// Color
    background_color: Color,
    /// Refresh LUT
    refresh: RefreshLUT,
}

impl<SPI, CS, BUSY, DC, RST> EPD1in54<SPI, CS, BUSY, DC, RST>
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

        // 3 Databytes:
        // A[7:0]
        // 0.. A[8]
        // 0.. B[2:0]
        // Default Values: A = Height of Screen (0x127), B = 0x00 (GD, SM and TB=0?)
        self.interface.cmd_with_data(
            spi,
            Command::DRIVER_OUTPUT_CONTROL,
            &[HEIGHT as u8, (HEIGHT >> 8) as u8, 0x00],
        )?;

        // 3 Databytes: (and default values from datasheet and arduino)
        // 1 .. A[6:0]  = 0xCF | 0xD7
        // 1 .. B[6:0]  = 0xCE | 0xD6
        // 1 .. C[6:0]  = 0x8D | 0x9D
        //TODO: test
        self.interface.cmd_with_data(
            spi,
            Command::BOOSTER_SOFT_START_CONTROL,
            &[0xD7, 0xD6, 0x9D],
        )?;

        // One Databyte with value 0xA8 for 7V VCOM
        self.interface
            .cmd_with_data(spi, Command::WRITE_VCOM_REGISTER, &[0xA8])?;

        // One Databyte with default value 0x1A for 4 dummy lines per gate
        self.interface
            .cmd_with_data(spi, Command::SET_DUMMY_LINE_PERIOD, &[0x1A])?;

        // One Databyte with default value 0x08 for 2us per line
        self.interface
            .cmd_with_data(spi, Command::SET_GATE_LINE_WIDTH, &[0x08])?;

        // One Databyte with default value 0x03
        //  -> address: x increment, y increment, address counter is updated in x direction
        self.interface
            .cmd_with_data(spi, Command::DATA_ENTRY_MODE_SETTING, &[0x03])?;

        self.set_lut(spi, None)
    }
}

impl<SPI, CS, BUSY, DC, RST, E> WaveshareDisplay<SPI, CS, BUSY, DC, RST>
    for EPD1in54<SPI, CS, BUSY, DC, RST>
where
    SPI: Write<u8, Error = E>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    fn width(&self) -> u32 {
        WIDTH
    }

    fn height(&self) -> u32 {
        HEIGHT
    }

    fn new<DELAY: DelayMs<u8>>(
        spi: &mut SPI,
        cs: CS,
        busy: BUSY,
        dc: DC,
        rst: RST,
        delay: &mut DELAY,
    ) -> Result<Self, SPI::Error> {
        let interface = DisplayInterface::new(cs, busy, dc, rst);

        let mut epd = EPD1in54 {
            interface,
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
        // 0x00 for Normal mode (Power on Reset), 0x01 for Deep Sleep Mode
        //TODO: is 0x00 needed here or would 0x01 be even more efficient?
        self.interface
            .cmd_with_data(spi, Command::DEEP_SLEEP_MODE, &[0x00])?;

        self.wait_until_idle();
        Ok(())
    }

    fn update_frame(&mut self, spi: &mut SPI, buffer: &[u8]) -> Result<(), SPI::Error> {
        self.use_full_frame(spi)?;
        self.interface
            .cmd_with_data(spi, Command::WRITE_RAM, buffer)
    }

    //TODO: update description: last 3 bits will be ignored for width and x_pos
    fn update_partial_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), SPI::Error> {
        self.set_ram_area(spi, x, y, x + width, y + height)?;
        self.set_ram_counter(spi, x, y)?;

        self.interface
            .cmd_with_data(spi, Command::WRITE_RAM, buffer)
    }

    fn display_frame(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        // enable clock signal, enable cp, display pattern -> 0xC4 (tested with the arduino version)
        //TODO: test control_1 or control_2 with default value 0xFF (from the datasheet)
        self.interface
            .cmd_with_data(spi, Command::DISPLAY_UPDATE_CONTROL_2, &[0xC4])?;

        self.interface.cmd(spi, Command::MASTER_ACTIVATION)?;
        // MASTER Activation should not be interupted to avoid currption of panel images
        // therefore a terminate command is send
        self.interface.cmd(spi, Command::NOP)
    }

    fn clear_frame(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        self.use_full_frame(spi)?;

        // clear the ram with the background color
        let color = self.background_color.get_byte_value();

        self.interface.cmd(spi, Command::WRITE_RAM)?;
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
        spi: &mut SPI,
        refresh_rate: Option<RefreshLUT>,
    ) -> Result<(), SPI::Error> {
        if let Some(refresh_lut) = refresh_rate {
            self.refresh = refresh_lut;
        }
        match self.refresh {
            RefreshLUT::FULL => self.set_lut_helper(spi, &LUT_FULL_UPDATE),
            RefreshLUT::QUICK => self.set_lut_helper(spi, &LUT_PARTIAL_UPDATE),
        }
    }
}

impl<SPI, CS, BUSY, DC, RST> EPD1in54<SPI, CS, BUSY, DC, RST>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    fn wait_until_idle(&mut self) {
        self.interface.wait_until_idle(false);
    }

    pub(crate) fn use_full_frame(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        // choose full frame/ram
        self.set_ram_area(spi, 0, 0, WIDTH - 1, HEIGHT - 1)?;

        // start from the beginning
        self.set_ram_counter(spi, 0, 0)
    }

    pub(crate) fn set_ram_area(
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
            Command::SET_RAM_X_ADDRESS_START_END_POSITION,
            &[(start_x >> 3) as u8, (end_x >> 3) as u8],
        )?;

        // 2 Databytes: A[7:0] & 0..A[8] for each - start and end
        self.interface.cmd_with_data(
            spi,
            Command::SET_RAM_Y_ADDRESS_START_END_POSITION,
            &[
                start_y as u8,
                (start_y >> 8) as u8,
                end_y as u8,
                (end_y >> 8) as u8,
            ],
        )
    }

    pub(crate) fn set_ram_counter(
        &mut self,
        spi: &mut SPI,
        x: u32,
        y: u32,
    ) -> Result<(), SPI::Error> {
        // x is positioned in bytes, so the last 3 bits which show the position inside a byte in the ram
        // aren't relevant
        self.interface
            .cmd_with_data(spi, Command::SET_RAM_X_ADDRESS_COUNTER, &[(x >> 3) as u8])?;

        // 2 Databytes: A[7:0] & 0..A[8]
        self.interface.cmd_with_data(
            spi,
            Command::SET_RAM_Y_ADDRESS_COUNTER,
            &[y as u8, (y >> 8) as u8],
        )?;

        self.wait_until_idle();
        Ok(())
    }

    fn set_lut_helper(&mut self, spi: &mut SPI, buffer: &[u8]) -> Result<(), SPI::Error> {
        assert!(buffer.len() == 30);
        self.interface
            .cmd_with_data(spi, Command::WRITE_LUT_REGISTER, buffer)
    }
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
