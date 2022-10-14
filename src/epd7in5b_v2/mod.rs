//! A simple Driver for the Waveshare 7.5" (B) E-Ink Display (V2) via SPI
//!
//! # References
//!
//! - [Datasheet](https://www.waveshare.com/wiki/7.5inch_e-Paper_HAT)
//! - [Waveshare C driver](https://github.com/waveshare/e-Paper/blob/702def0/RaspberryPi%26JetsonNano/c/lib/e-Paper/EPD_7in5_V2.c)
//! - [Waveshare Python driver](https://github.com/waveshare/e-Paper/blob/702def0/RaspberryPi%26JetsonNano/python/lib/waveshare_epd/epd7in5_V2.py)
//!
//! Important note for V2:
//! Revision V2 has been released on 2019.11, the resolution is upgraded to 800×480, from 640×384 of V1.
//! The hardware and interface of V2 are compatible with V1, however, the related software should be updated.

use embedded_hal::{
    blocking::{delay::*, spi::Write},
    digital::v2::{InputPin, OutputPin},
};

use crate::color::TriColor;
use crate::interface::DisplayInterface;
use crate::traits::{InternalWiAdditions, RefreshLut, WaveshareDisplay};



pub(crate) mod command;
use self::command::Command;

#[cfg(feature = "graphics")]
mod graphics;
#[cfg(feature = "graphics")]
pub use self::graphics::Display7in5;

/// Width of the display
pub const WIDTH: u32 = 800;
/// Height of the display
pub const HEIGHT: u32 = 480;
/// Default Background Color
pub const DEFAULT_BACKGROUND_COLOR: TriColor = TriColor::White;

const NUM_DISPLAY_BYTES: usize = WIDTH as usize * HEIGHT as usize / 8;
const IS_BUSY_LOW: bool = true;

/// Epd7in5 (V2) driver
///
pub struct Epd7in5<SPI, CS, BUSY, DC, RST, DELAY> {
    /// Connection Interface
    interface: DisplayInterface<SPI, CS, BUSY, DC, RST, DELAY>,
    /// Background Color
    color: TriColor,
}

impl<SPI, CS, BUSY, DC, RST, DELAY> InternalWiAdditions<SPI, CS, BUSY, DC, RST, DELAY>
    for Epd7in5<SPI, CS, BUSY, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayMs<u8>,
{
    fn init(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        // Reset the device
        // C driver does 200/2 original rust driver does 10/2
        self.interface.reset(delay, 200, 2);

        // V2 procedure as described here:
        // https://github.com/waveshare/e-Paper/blob/master/RaspberryPi%26JetsonNano/python/lib/waveshare_epd/epd7in5bc_V2.py
        // and as per specs:
        // https://www.waveshare.com/w/upload/6/60/7.5inch_e-Paper_V2_Specification.pdf

        self.cmd_with_data(spi, Command::PowerSetting, &[0x07, 0x07, 0x3F, 0x3F])?;
        self.command(spi, Command::PowerOn)?;
        // C driver adds a static 100ms delay here
        self.wait_until_idle(spi, delay)?;
        // Done, but this is also the default
        self.cmd_with_data(spi, Command::PanelSetting, &[0x0F])?; // 0x1F = B/W mode ? doesnt seem to work
        // Not done in C driver, this is the default
        //self.cmd_with_data(spi, Command::PllControl, &[0x06])?;
        self.cmd_with_data(spi, Command::TconResolution, &[0x03, 0x20, 0x01, 0xE0])?;
        // Documentation removed in v3 but done in v2 and works in v3
        self.cmd_with_data(spi, Command::DualSpi, &[0x00])?;
        //                    0x10 in BW mode  (Work ?) V
        //                    0x12 in BW mode to disable new/old thing
        //                    0x01 -> Black border
        //                    0x11 -> White norder
        //                    0x21 -> Red border
        //                    0x31 -> don't touch border
        //                    the second nibble can change polarity (may be easier for default
        //                    display initialization)                   V
        self.cmd_with_data(spi, Command::VcomAndDataIntervalSetting, &[0x11, 0x07])?;
        // This is the default
        self.cmd_with_data(spi, Command::TconSetting, &[0x22])?;
        self.cmd_with_data(spi, Command::SpiFlashControl, &[0x00, 0x00, 0x00, 0x00])?;
        // Not in C driver
        self.wait_until_idle(spi, delay)?;
        Ok(())
    }
}

impl<SPI, CS, BUSY, DC, RST, DELAY> WaveshareDisplay<SPI, CS, BUSY, DC, RST, DELAY>
    for Epd7in5<SPI, CS, BUSY, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayMs<u8>,
{
    type DisplayColor = TriColor;
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

        let mut epd = Epd7in5 { interface, color };

        epd.init(spi, delay)?;

        Ok(epd)
    }

    fn wake_up(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.init(spi, delay)
    }

    fn sleep(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.wait_until_idle(spi, delay)?;
        self.command(spi, Command::PowerOff)?;
        self.wait_until_idle(spi, delay)?;
        self.cmd_with_data(spi, Command::DeepSleep, &[0xA5])?;
        Ok(())
    }

    fn update_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        self.wait_until_idle(spi, delay)?;
        // (B) version sends one buffer for black and one for red
        self.cmd_with_data(spi, Command::DataStartTransmission1, &buffer[.. NUM_DISPLAY_BYTES])?;
        self.cmd_with_data(spi, Command::DataStartTransmission2, &buffer[NUM_DISPLAY_BYTES..])?;
        Ok(())
    }

    fn update_partial_frame(
        &mut self,
        _spi: &mut SPI,
        _buffer: &[u8],
        _x: u32,
        _y: u32,
        _width: u32,
        _height: u32,
    ) -> Result<(), SPI::Error> {
        unimplemented!()
    }

    fn display_frame(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.wait_until_idle(spi, delay)?;
        self.command(spi, Command::DisplayRefresh)?;
        Ok(())
    }

    fn update_and_display_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        self.update_frame(spi, buffer, delay)?;
        self.command(spi, Command::DisplayRefresh)?;
        Ok(())
    }

    fn clear_frame(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.wait_until_idle(spi, delay)?;
        self.send_resolution(spi)?;

        self.command(spi, Command::DataStartTransmission1)?;
        self.interface.data_x_times(spi, 0xFF, WIDTH * HEIGHT / 8)?;

        self.command(spi, Command::DataStartTransmission2)?;
        self.interface.data_x_times(spi, 0x00, WIDTH * HEIGHT / 8)?;

        self.command(spi, Command::DisplayRefresh)?;

        Ok(())
    }

    fn set_background_color(&mut self, color: Self::DisplayColor) {
        self.color = color;
    }

    fn background_color(&self) -> &Self::DisplayColor {
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
        _spi: &mut SPI,
        _refresh_rate: Option<RefreshLut>,
    ) -> Result<(), SPI::Error> {
        unimplemented!();
    }

    fn is_busy(&self) -> bool {
        self.interface.is_busy(IS_BUSY_LOW)
    }
}

impl<SPI, CS, BUSY, DC, RST, DELAY> Epd7in5<SPI, CS, BUSY, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayMs<u8>,
{
    /// temporary replacement for missing delay in the trait to call wait_until_idle
    pub fn update_partial_frame2(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        self.wait_until_idle(spi, delay)?;
        if buffer.len() as u32 != width / 8 * height {
            //TODO panic or error
        }

        let hrst_upper = (x / 8) as u8 >> 5;
        let hrst_lower = ((x / 8) << 3) as u8;
        let hred_upper = ((x + width) / 8 - 1) as u8 >> 5;
        let hred_lower = (((x + width) / 8 - 1) << 3) as u8 | 0b111;
        let vrst_upper = (y >> 8) as u8;
        let vrst_lower = y as u8;
        let vred_upper = ((y + height - 1) >> 8) as u8;
        let vred_lower = (y + height - 1) as u8;
        let pt_scan = 0x01; // Gates scan both inside and outside of the partial window. (default)

        self.command(spi, Command::PartialIn)?;
        self.cmd_with_data(spi, Command::PartialWindow,
            &[
                hrst_upper, hrst_lower, hred_upper, hred_lower, vrst_upper, vrst_lower, vred_upper,
                vred_lower, pt_scan,
            ],
        )?;
        let half = buffer.len() / 2;
        self.cmd_with_data(spi, Command::DataStartTransmission1, &buffer[..half])?;
        self.cmd_with_data(spi, Command::DataStartTransmission2, &buffer[half..])?;

        self.command(spi, Command::DisplayRefresh)?;
        self.wait_until_idle(spi, delay)?;

        self.command(spi, Command::PartialOut)?;
        Ok(())
    }

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

    /// wait
    pub fn wait_until_idle(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        // C driver first sends the command
        self.interface.cmd(spi, Command::GetStatus)?;
        while self.interface.is_busy(IS_BUSY_LOW) {
            self.interface.cmd(spi, Command::GetStatus)?;
            // C driver doesn't wait here but we have to
            // because be want to be able to give back control if we are in a RT thread
            delay.delay_ms(10);
        }
        // C driver add 200ms here
        Ok(())
    }

    fn send_resolution(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        let w = self.width();
        let h = self.height();

        self.command(spi, Command::TconResolution)?;
        self.send_data(spi, &[(w >> 8) as u8])?;
        self.send_data(spi, &[w as u8])?;
        self.send_data(spi, &[(h >> 8) as u8])?;
        self.send_data(spi, &[h as u8])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epd_size() {
        assert_eq!(WIDTH, 800);
        assert_eq!(HEIGHT, 480);
        assert_eq!(DEFAULT_BACKGROUND_COLOR, TriColor::White);
    }
}
