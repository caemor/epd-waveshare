//! A simple Driver for the Waveshare 4.2" E-Ink Display via SPI
//!
//! The other Waveshare E-Ink Displays should be added later on
//!
//! Build with the help of documentation/code from [Waveshare](https://www.waveshare.com/wiki/4.2inch_e-Paper_Module),
//! [Ben Krasnows partial Refresh tips](https://benkrasnow.blogspot.de/2017/10/fast-partial-refresh-on-42-e-paper.html) and
//! the driver documents in the `pdfs`-folder as orientation.
//!
//! This driver was built using [`embedded-hal`] traits.
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal/~0.1
//!
//! # Requirements
//!
//! ### SPI
//!
//! - MISO is not connected/available
//! - SPI_MODE_0 is used (CPHL = 0, CPOL = 0)
//! - 8 bits per word, MSB first
//! - Max. Speed tested was 8Mhz but more should be possible
//!
//! ### Other....
//!
//! - Buffersize: Wherever a buffer is used it always needs to be of the size: `width / 8 * length`,
//!   where width and length being either the full e-ink size or the partial update window size
//!
//! # Examples
//!
//! ```ignore
//! let mut epd4in2 = EPD4in2::new(spi, cs, busy, dc, rst, delay).unwrap();
//!
//! let mut buffer =  [0u8, epd4in2.get_width() / 8 * epd4in2.get_height()];
//!
//! // draw something into the buffer
//!
//! epd4in2.display_and_transfer_buffer(buffer, None);
//!
//! // wait and look at the image
//!
//! epd4in2.clear_frame(None);
//!
//! epd4in2.sleep();
//! ```
//!
//!
//!
//! BE CAREFUL! The screen can get ghosting/burn-ins through the Partial Fast Update Drawing.

use hal::{
    blocking::{delay::*, spi::Write},
    digital::*,
};

use traits::{connection_interface::ConnectionInterface, WaveshareInterface, InternalWiAdditions};

//The Lookup Tables for the Display
mod constants;
use self::constants::*;

use color::Color;

pub mod command;
use self::command::Command;

/// EPD4in2 driver
///
pub struct EPD4in2<SPI, CS, BUSY, DC, RST, D> {
    /// Connection Interface
    interface: ConnectionInterface<SPI, CS, BUSY, DC, RST, D>,
    /// Background Color
    color: Color,
}




impl<SPI, CS, BUSY, DC, RST, Delay, ERR>
    InternalWiAdditions<SPI, CS, BUSY, DC, RST, Delay, ERR>
    for EPD4in2<SPI, CS, BUSY, DC, RST, Delay>
where
    SPI: Write<u8, Error = ERR>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    Delay: DelayUs<u16> + DelayMs<u16>,
{
    fn init(&mut self) -> Result<(), ERR> {
        // reset the device
        self.interface.reset();

        // set the power settings
        self.interface.cmd_with_data(Command::POWER_SETTING, &[0x03, 0x00, 0x2b, 0x2b, 0xff])?;

        // start the booster
        self.interface.cmd_with_data(Command::BOOSTER_SOFT_START, &[0x17, 0x17, 0x17])?;        

        // power on
        self.command(Command::POWER_ON)?;
        self.wait_until_idle();

        // set the panel settings
        self.cmd_with_data(Command::PANEL_SETTING, &[0x3F])?;

        // Set Frequency, 200 Hz didn't work on my board
        // 150Hz and 171Hz wasn't tested yet
        // TODO: Test these other frequencies
        // 3A 100HZ   29 150Hz 39 200HZ  31 171HZ DEFAULT: 3c 50Hz
        self.cmd_with_data(Command::PLL_CONTROL, &[0x3A])?;
        
        self.set_lut()?;

        Ok(())
    }
}

impl<SPI, CS, BUSY, DC, RST, Delay, ERR>
    WaveshareInterface<SPI, CS, BUSY, DC, RST, Delay, ERR>
    for EPD4in2<SPI, CS, BUSY, DC, RST, Delay>
where
    SPI: Write<u8, Error = ERR>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    Delay: DelayUs<u16> + DelayMs<u16>,
{
    /// Creates a new driver from a SPI peripheral, CS Pin, Busy InputPin, DC
    ///
    /// This already initialises the device. That means [init()](init()) isn't needed directly afterwards
    ///
    /// # Example
    ///
    /// ```ignore
    /// //buffer = some image data;
    ///
    /// let mut epd4in2 = EPD4in2::new(spi, cs, busy, dc, rst, delay);
    ///
    /// epd4in2.display_and_transfer_frame(buffer, None);
    ///
    /// epd4in2.sleep();
    /// ```
    fn new(spi: SPI, cs: CS, busy: BUSY, dc: DC, rst: RST, delay: Delay) -> Result<Self, ERR> {
        let interface = ConnectionInterface::new(spi, cs, busy, dc, rst, delay);
        let color = DEFAULT_BACKGROUND_COLOR;

        let mut epd = EPD4in2 {
            interface,
            color,
        };

        epd.init()?;

        Ok(epd)
    }

    fn wake_up(&mut self) -> Result<(), ERR> {
        self.init()
    }

    //TODO: is such a long delay really needed inbetween?
    fn sleep(&mut self) -> Result<(), ERR> {
        self.interface.cmd_with_data(Command::VCOM_AND_DATA_INTERVAL_SETTING, &[0x17])?; //border floating
        self.command(Command::VCM_DC_SETTING)?; // VCOM to 0V
        self.command(Command::PANEL_SETTING)?;
        self.delay_ms(100);

        self.command(Command::POWER_SETTING)?; //VG&VS to 0V fast
        for _ in 0..4 {
            self.send_data(&[0x00])?;
        }
        self.delay_ms(100);

        self.command(Command::POWER_OFF)?;
        self.wait_until_idle();
        self.interface.cmd_with_data(Command::DEEP_SLEEP, &[0xA5])
    }

    fn update_frame(&mut self, buffer: &[u8]) -> Result<(), ERR> {
        let color_value = self.color.get_byte_value();

        self.send_resolution()?;

        self.interface.cmd_with_data(Command::VCM_DC_SETTING, &[0x12])?;

        //TODO: this was a send_command instead of a send_data. check if it's alright and doing what it should do (setting the default values)
        //self.send_command_u8(0x97)?; //VBDF 17|D7 VBDW 97  VBDB 57  VBDF F7  VBDW 77  VBDB 37  VBDR B7
        self.interface.cmd_with_data(Command::VCOM_AND_DATA_INTERVAL_SETTING, &[0x97])?;


        self.command(Command::DATA_START_TRANSMISSION_1)?;
        self.interface.data_x_times(color_value, buffer.len() as u16)?;

        self.delay_ms(2);

        self.interface.cmd_with_data(Command::DATA_START_TRANSMISSION_2, buffer)
    }

    fn update_partial_frame(
        &mut self,
        buffer: &[u8],
        x: u16,
        y: u16,
        width: u16,
        height: u16,
    ) -> Result<(), ERR> {
        if buffer.len() as u16 != width / 8 * height {
            //TODO: panic!! or sth like that
            //return Err("Wrong buffersize");
        }

        self.command(Command::PARTIAL_IN)?;
        self.command(Command::PARTIAL_WINDOW)?;
        self.send_data(&[(x >> 8) as u8])?;
        let tmp = x & 0xf8;
        self.send_data(&[tmp as u8])?; // x should be the multiple of 8, the last 3 bit will always be ignored
        let tmp = tmp + width - 1;
        self.send_data(&[(tmp >> 8) as u8])?;
        self.send_data(&[(tmp | 0x07) as u8])?;

        self.send_data(&[(y >> 8) as u8])?;
        self.send_data(&[y as u8])?;

        self.send_data(&[((y + height - 1) >> 8) as u8])?;
        self.send_data(&[(y + height - 1) as u8])?;

        self.send_data(&[0x01])?; // Gates scan both inside and outside of the partial window. (default)

        //TODO: handle dtm somehow
        let is_dtm1 = false;
        if is_dtm1 {
            self.command(Command::DATA_START_TRANSMISSION_1)?
        } else {
            self.command(Command::DATA_START_TRANSMISSION_2)?
        }

        self.send_data(buffer)?;

        self.command(Command::PARTIAL_OUT)
    }



    fn display_frame(&mut self) -> Result<(), ERR> {
        self.command(Command::DISPLAY_REFRESH)?;

        self.wait_until_idle();
        Ok(())
    }

    fn clear_frame(&mut self) -> Result<(), ERR> {
        self.send_resolution()?;

        let size = WIDTH / 8 * HEIGHT;
        let color_value = self.color.get_byte_value();

        self.command(Command::DATA_START_TRANSMISSION_1)?;
        self.interface.data_x_times(color_value, size)?;

        self.delay_ms(2);

        self.command(Command::DATA_START_TRANSMISSION_2)?;
        self.interface.data_x_times(color_value, size)
    }

    /// Sets the backgroundcolor for various commands like [WaveshareInterface::clear_frame()](clear_frame())
    fn set_background_color(&mut self, color: Color) {
        self.color = color;
    }

    fn background_color(&self) -> &Color {
        &self.color
    }

    fn width(&self) -> u16 {
        WIDTH
    }

    fn height(&self) -> u16 {
        HEIGHT
    }


    fn delay_ms(&mut self, delay: u16) {
        self.interface.delay_ms(delay)
    }
}

impl<SPI, CS, BUSY, DC, RST, D, ERR> EPD4in2<SPI, CS, BUSY, DC, RST, D>
where
    SPI: Write<u8, Error = ERR>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    D: DelayUs<u16> + DelayMs<u16>,
{
    fn command(&mut self, command: Command) -> Result<(), ERR> {
        self.interface.cmd(command)
    }

    fn send_data(&mut self, data: &[u8]) -> Result<(), ERR> {
        self.interface.data(data)
    }

    fn cmd_with_data(&mut self, command: Command, data: &[u8]) -> Result<(), ERR> {
        self.interface.cmd_with_data(command, data)
    }

    fn wait_until_idle(&mut self) {
        self.interface.wait_until_idle(true)
    }

    fn send_resolution(&mut self) -> Result<(), ERR> {
        let w = self.width();
        let h = self.height();

        self.command(Command::RESOLUTION_SETTING)?;
        self.send_data(&[(w >> 8) as u8])?;
        self.send_data(&[w as u8])?;
        self.send_data(&[(h >> 8) as u8])?;
        self.send_data(&[h as u8])
    }

    /// Fill the look-up table for the EPD
    //TODO: make public?
    fn set_lut(&mut self) -> Result<(), ERR> {
        self.set_lut_helper(&LUT_VCOM0, &LUT_WW, &LUT_BW, &LUT_WB, &LUT_BB)
    }

    /// Fill the look-up table for a quick display (partial refresh)
    ///
    /// Is automatically done by [EPD4in2::display_frame_quick()](EPD4in2::display_frame_quick())
    /// //TODO: make public?
    #[cfg(feature = "epd4in2_fast_update")]
    fn set_lut_quick(&mut self) -> Result<(), ERR> {
        self.set_lut_helper(
            &LUT_VCOM0_QUICK,
            &LUT_WW_QUICK,
            &LUT_BW_QUICK,
            &LUT_WB_QUICK,
            &LUT_BB_QUICK,
        )
    }

    fn set_lut_helper(
        &mut self,
        lut_vcom: &[u8],
        lut_ww: &[u8],
        lut_bw: &[u8],
        lut_wb: &[u8],
        lut_bb: &[u8],
    ) -> Result<(), ERR> {
        // LUT VCOM
        self.command(Command::LUT_FOR_VCOM)?;
        self.send_data(lut_vcom)?;

        // LUT WHITE to WHITE
        self.command(Command::LUT_WHITE_TO_WHITE)?;
        self.send_data(lut_ww)?;

        // LUT BLACK to WHITE
        self.command(Command::LUT_BLACK_TO_WHITE)?;
        self.send_data(lut_bw)?;

        // LUT WHITE to BLACK
        self.command(Command::LUT_WHITE_TO_BLACK)?;
        self.send_data(lut_wb)?;

        // LUT BLACK to BLACK
        self.command(Command::LUT_BLACK_TO_BLACK)?;
        self.send_data(lut_bb)?;

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