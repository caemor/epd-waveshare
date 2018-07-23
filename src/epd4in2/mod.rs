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
    spi::{Mode, Phase, Polarity},
};

use interface::{connection_interface::ConnectionInterface, WaveshareInterface};

//The Lookup Tables for the Display
mod constants;
use self::constants::*;

use drawing::color::Color;

pub mod command;
pub use self::command::Command;

//TODO: test spi mode
/// SPI mode -
/// For more infos see [Requirements: SPI](index.html#spi)
pub const SPI_MODE: Mode = Mode {
    phase: Phase::CaptureOnFirstTransition,
    polarity: Polarity::IdleLow,
};

/// EPD4in2 driver
///
pub struct EPD4in2<SPI, CS, BUSY, DC, RST, D> {
    /// Connection Interface
    interface: ConnectionInterface<SPI, CS, BUSY, DC, RST, D>,
    /// Width
    width: u16,
    /// Height
    height: u16,
    /// Background Color
    color: Color,
}

impl<SPI, CS, BUSY, DC, RST, D, E> WaveshareInterface<SPI, CS, BUSY, DC, RST, D, E>
    for EPD4in2<SPI, CS, BUSY, DC, RST, D>
where
    SPI: Write<u8, Error = E>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    D: DelayUs<u16> + DelayMs<u16>,
{
    fn get_width(&self) -> u16 {
        self.width
    }

    fn get_height(&self) -> u16 {
        self.height
    }

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
    fn new(spi: SPI, cs: CS, busy: BUSY, dc: DC, rst: RST, delay: D) -> Result<Self, E> {
        let width = WIDTH as u16;
        let height = HEIGHT as u16;

        let interface = ConnectionInterface::new(spi, cs, busy, dc, rst, delay);
        let color = Color::White;
        let mut epd = EPD4in2 {
            interface,
            width,
            height,
            color,
        };

        epd.init()?;

        Ok(epd)
    }

    fn init(&mut self) -> Result<(), E> {
        // reset the device
        self.reset();

        // set the power settings
        self.send_command(Command::POWER_SETTING)?;
        self.send_data(0x03)?; //VDS_EN, VDG_EN
        self.send_data(0x00)?; //VCOM_HV, VGHL_LV[1], VGHL_LV[0]
        self.send_data(0x2b)?; //VDH
        self.send_data(0x2b)?; //VDL
        self.send_data(0xff)?; //VDHR

        // start the booster
        self.send_command(Command::BOOSTER_SOFT_START)?;
        for _ in 0..3 {
            self.send_data(0x17)?; //07 0f 17 1f 27 2F 37 2f
        }

        // power on
        self.send_command(Command::POWER_ON)?;
        self.wait_until_idle();

        // set the panel settings
        self.send_command(Command::PANEL_SETTING)?;
        // 0x0F Red Mode, LUT from OTP
        // 0x1F B/W Mode, LUT from OTP
        // 0x2F Red Mode, LUT set by registers
        // 0x3F B/W Mode, LUT set by registers
        self.send_data(0x3F)?;

        // the values used by waveshare before for the panel settings
        // instead of our one liner:
        // SendData(0xbf);    // KW-BF   KWR-AF  BWROTP 0f
        // SendData(0x0b);

        // Set Frequency, 200 Hz didn't work on my board
        // 150Hz and 171Hz wasn't tested yet
        // TODO: Test these other frequencies
        // 3A 100HZ   29 150Hz 39 200HZ  31 171HZ DEFAULT: 3c 50Hz
        self.send_command(Command::PLL_CONTROL)?;
        self.send_data(0x3A)?;

        self.set_lut()?;

        Ok(())
    }

    fn sleep(&mut self) -> Result<(), E> {
        self.send_command(Command::VCOM_AND_DATA_INTERVAL_SETTING)?;
        self.send_data(0x17)?; //border floating
        self.send_command(Command::VCM_DC_SETTING)?; // VCOM to 0V
        self.send_command(Command::PANEL_SETTING)?;
        self.delay_ms(100);

        self.send_command(Command::POWER_SETTING)?; //VG&VS to 0V fast
        for _ in 0..4 {
            self.send_data(0x00)?;
        }
        self.delay_ms(100);

        self.send_command(Command::POWER_OFF)?;
        self.wait_until_idle();
        self.send_command(Command::DEEP_SLEEP)?;
        self.send_data(0xA5)?;

        Ok(())
    }

    fn reset(&mut self) {
        self.interface.reset()
    }

    fn delay_ms(&mut self, delay: u16) {
        self.interface.delay_ms(delay)
    }

    fn update_frame(&mut self, buffer: &[u8]) -> Result<(), E> {
        let color_value = self.color.get_byte_value();

        self.send_resolution()?;

        self.send_command(Command::VCM_DC_SETTING)?;
        self.send_data(0x12)?;

        self.send_command(Command::VCOM_AND_DATA_INTERVAL_SETTING)?;
        //TODO: this was a send_command instead of a send_data. check if it's alright and doing what it should do (setting the default values)
        //self.send_command_u8(0x97)?; //VBDF 17|D7 VBDW 97  VBDB 57  VBDF F7  VBDW 77  VBDB 37  VBDR B7
        self.send_data(0x97)?;

        self.send_command(Command::DATA_START_TRANSMISSION_1)?;
        for _ in 0..(buffer.len()) {
            self.send_data(color_value)?;
        }
        self.delay_ms(2);

        self.send_command(Command::DATA_START_TRANSMISSION_2)?;
        //self.send_multiple_data(buffer)?;
        
        for &elem in buffer.iter() {
            self.send_data(elem)?;
        }

        Ok(())
    }

    fn update_partial_frame(
        &mut self,
        buffer: &[u8],
        x: u16,
        y: u16,
        width: u16,
        height: u16,
    ) -> Result<(), E> {
        if buffer.len() as u16 != width / 8 * height {
            //TODO: panic!! or sth like that
            //return Err("Wrong buffersize");
        }

        self.send_command(Command::PARTIAL_IN)?;
        self.send_command(Command::PARTIAL_WINDOW)?;
        self.send_data((x >> 8) as u8)?;
        let tmp = x & 0xf8;
        self.send_data(tmp as u8)?; // x should be the multiple of 8, the last 3 bit will always be ignored
        let tmp = tmp + width - 1;
        self.send_data((tmp >> 8) as u8)?;
        self.send_data((tmp | 0x07) as u8)?;

        self.send_data((y >> 8) as u8)?;
        self.send_data(y as u8)?;

        self.send_data(((y + height - 1) >> 8) as u8)?;
        self.send_data((y + height - 1) as u8)?;

        self.send_data(0x01)?; // Gates scan both inside and outside of the partial window. (default)

        //TODO: handle dtm somehow
        let is_dtm1 = false;
        if is_dtm1 {
            self.send_command(Command::DATA_START_TRANSMISSION_1)?
        } else {
            self.send_command(Command::DATA_START_TRANSMISSION_2)?
        }

        self.send_multiple_data(buffer)?;

        self.send_command(Command::PARTIAL_OUT)
    }

    fn update_and_display_frame(&mut self, buffer: &[u8]) -> Result<(), E>{
        self.update_frame(buffer)?;
        self.display_frame()
    }


    fn display_frame(&mut self) -> Result<(), E> {
        self.send_command(Command::DISPLAY_REFRESH)?;

        self.wait_until_idle();
        Ok(())
    }

    // TODO: add this abstraction function
    // fn update_and_display_frame(&mut self, buffer: &[u8]) -> Result<(), E>;

    fn clear_frame(&mut self) -> Result<(), E> {
        self.send_resolution()?;

        let size = self.width / 8 * self.height;
        let color_value = self.color.get_byte_value();

        self.send_command(Command::DATA_START_TRANSMISSION_1)?;
        self.delay_ms(2);
        for _ in 0..size {
            self.send_data(color_value)?;
        }

        self.delay_ms(2);

        self.send_command(Command::DATA_START_TRANSMISSION_2)?;
        self.delay_ms(2);
        for _ in 0..size {
            self.send_data(color_value)?;
        }
        Ok(())
    }

    /// Sets the backgroundcolor for various commands like [WaveshareInterface::clear_frame()](clear_frame())
    fn set_background_color(&mut self, color: Color) {
        self.color = color;
    }
}

impl<SPI, CS, BUSY, DC, RST, D, E> EPD4in2<SPI, CS, BUSY, DC, RST, D>
where
    SPI: Write<u8, Error = E>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    D: DelayUs<u16> + DelayMs<u16>,
{
    fn send_command(&mut self, command: Command) -> Result<(), E> {
        self.interface.send_command(command)
    }

    fn send_data(&mut self, val: u8) -> Result<(), E> {
        self.interface.send_data(val)
    }

    fn send_multiple_data(&mut self, data: &[u8]) -> Result<(), E> {
        self.interface.send_multiple_data(data)
    }

    fn wait_until_idle(&mut self) {
        self.interface.wait_until_idle(true)
    }

    fn send_resolution(&mut self) -> Result<(), E> {
        let w = self.get_width();
        let h = self.get_height();

        self.send_command(Command::RESOLUTION_SETTING)?;
        self.send_data((w >> 8) as u8)?;
        self.send_data(w as u8)?;
        self.send_data((h >> 8) as u8)?;
        self.send_data(h as u8)
    }

    /// Fill the look-up table for the EPD
    //TODO: make public?
    fn set_lut(&mut self) -> Result<(), E> {
        self.set_lut_helper(&LUT_VCOM0, &LUT_WW, &LUT_BW, &LUT_WB, &LUT_BB)
    }

    /// Fill the look-up table for a quick display (partial refresh)
    /// 
    /// Is automatically done by [EPD4in2::display_frame_quick()](EPD4in2::display_frame_quick()) 
    /// //TODO: make public?
    #[cfg(feature = "epd4in2_fast_update")]
    fn set_lut_quick(&mut self) -> Result<(), E> {
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
    ) -> Result<(), E> {
        // LUT VCOM
        self.send_command(Command::LUT_FOR_VCOM)?;
        self.send_multiple_data(lut_vcom)?;

        // LUT WHITE to WHITE
        self.send_command(Command::LUT_WHITE_TO_WHITE)?;
        self.send_multiple_data(lut_ww)?;

        // LUT BLACK to WHITE
        self.send_command(Command::LUT_BLACK_TO_WHITE)?;
        self.send_multiple_data(lut_bw)?;

        // LUT WHITE to BLACK
        self.send_command(Command::LUT_WHITE_TO_BLACK)?;
        self.send_multiple_data(lut_wb)?;

        // LUT BLACK to BLACK
        self.send_command(Command::LUT_BLACK_TO_BLACK)?;
        self.send_multiple_data(lut_bb)?;

        Ok(())
    }
}
