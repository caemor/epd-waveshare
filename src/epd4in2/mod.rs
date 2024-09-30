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
//!# use embedded_hal_mock::eh1::*;
//!# fn main() -> Result<(), embedded_hal::spi::ErrorKind> {
//!use embedded_graphics::{
//!    pixelcolor::BinaryColor::On as Black, prelude::*, primitives::{Line, PrimitiveStyle},
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
//!# let mut delay = delay::NoopDelay::new();
//!
//!// Setup EPD
//!let mut epd = Epd4in2::new(&mut spi, busy_in, dc, rst, None)?;
//!
//!// Use display graphics from embedded-graphics
//!let mut display = Display4in2::default();
//!
//!// Use embedded graphics for drawing a line
//!let _ = Line::new(Point::new(0, 120), Point::new(0, 295))
//!    .into_styled(PrimitiveStyle::with_stroke(Color::Black, 1))
//!    .draw(&mut display);
//!
//!    // Display updated frame
//!epd.update_frame(&mut spi, &display.buffer(), &mut delay)?;
//!epd.display_frame(&mut spi, &mut delay)?;
//!
//!// Set the EPD to sleep
//!epd.sleep(&mut spi, &mut delay)?;
//!# Ok(())
//!# }
//!```
//!
//!
//!
//! BE CAREFUL! The screen can get ghosting/burn-ins through the Partial Fast Update Drawing.
use core::fmt::{Debug, Display};
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal_async::{digital::Wait, spi::SpiDevice};

use crate::error::ErrorKind;
use crate::interface::DisplayInterface;
use crate::traits::{ErrorType, InternalWiAdditions, QuickRefresh, RefreshLut, WaveshareDisplay};

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
const SINGLE_BYTE_WRITE: bool = true;

use crate::color::Color;

pub(crate) mod command;
use self::command::Command;
use crate::buffer_len;

/// Full size buffer for use with the 4in2 EPD
#[cfg(feature = "graphics")]
pub type Display4in2 = crate::graphics::Display<
    WIDTH,
    HEIGHT,
    false,
    { buffer_len(WIDTH as usize, HEIGHT as usize) },
    Color,
>;

/// Epd4in2 driver
///
pub struct Epd4in2<SPI, BUSY, DC, RST> {
    /// Connection Interface
    interface: DisplayInterface<SPI, BUSY, DC, RST, SINGLE_BYTE_WRITE>,
    /// Background Color
    color: Color,
    /// Refresh LUT
    refresh: RefreshLut,
}

impl<SPI, BUSY, DC, RST> ErrorType<SPI, BUSY, DC, RST> for Epd4in2<SPI, BUSY, DC, RST>
where
    SPI: SpiDevice,
    SPI::Error: Copy + Debug + Display,
    BUSY: InputPin + Wait,
    BUSY::Error: Copy + Debug + Display,
    DC: OutputPin,
    DC::Error: Copy + Debug + Display,
    RST: OutputPin,
    RST::Error: Copy + Debug + Display,
{
    type Error = ErrorKind<SPI, BUSY, DC, RST>;
}

impl<SPI, BUSY, DC, RST> InternalWiAdditions<SPI, BUSY, DC, RST> for Epd4in2<SPI, BUSY, DC, RST>
where
    SPI: SpiDevice,
    SPI::Error: Copy + Debug + Display,
    BUSY: InputPin + Wait,
    BUSY::Error: Copy + Debug + Display,
    DC: OutputPin,
    DC::Error: Copy + Debug + Display,
    RST: OutputPin,
    RST::Error: Copy + Debug + Display,
{
    async fn init(&mut self, spi: &mut SPI) -> Result<(), Self::Error> {
        // reset the device
        self.interface.reset(spi, 10_000, 10_000).await?;

        // set the power settings
        self.interface
            .cmd_with_data(spi, Command::PowerSetting, &[0x03, 0x00, 0x2b, 0x2b, 0xff])
            .await?;

        // start the booster
        self.interface
            .cmd_with_data(spi, Command::BoosterSoftStart, &[0x17, 0x17, 0x17])
            .await?;

        // power on
        self.command(spi, Command::PowerOn).await?;
        self.interface.delay(spi, 5000).await?;
        self.wait_until_idle(spi).await?;

        // set the panel settings
        self.cmd_with_data(spi, Command::PanelSetting, &[0x3F])
            .await?;

        // Set Frequency, 200 Hz didn't work on my board
        // 150Hz and 171Hz wasn't tested yet
        // TODO: Test these other frequencies
        // 3A 100HZ   29 150Hz 39 200HZ  31 171HZ DEFAULT: 3c 50Hz
        self.cmd_with_data(spi, Command::PllControl, &[0x3A])
            .await?;

        self.send_resolution(spi).await?;

        self.interface
            .cmd_with_data(spi, Command::VcmDcSetting, &[0x12])
            .await?;

        //VBDF 17|D7 VBDW 97  VBDB 57  VBDF F7  VBDW 77  VBDB 37  VBDR B7
        self.interface
            .cmd_with_data(spi, Command::VcomAndDataIntervalSetting, &[0x97])
            .await?;

        self.set_lut(spi, None).await?;

        self.wait_until_idle(spi).await?;
        Ok(())
    }
}

impl<SPI, BUSY, DC, RST> WaveshareDisplay<SPI, BUSY, DC, RST> for Epd4in2<SPI, BUSY, DC, RST>
where
    SPI: SpiDevice,
    SPI::Error: Copy + Debug + Display,
    BUSY: InputPin + Wait,
    BUSY::Error: Copy + Debug + Display,
    DC: OutputPin,
    DC::Error: Copy + Debug + Display,
    RST: OutputPin,
    RST::Error: Copy + Debug + Display,
{
    type DisplayColor = Color;
    async fn new(
        spi: &mut SPI,
        busy: BUSY,
        dc: DC,
        rst: RST,
        delay_us: Option<u32>,
    ) -> Result<Self, Self::Error> {
        let interface = DisplayInterface::new(busy, dc, rst, delay_us);
        let color = DEFAULT_BACKGROUND_COLOR;

        let mut epd = Epd4in2 {
            interface,
            color,
            refresh: RefreshLut::Full,
        };

        epd.init(spi).await?;

        Ok(epd)
    }

    async fn sleep(&mut self, spi: &mut SPI) -> Result<(), Self::Error> {
        self.wait_until_idle(spi).await?;
        self.interface
            .cmd_with_data(spi, Command::VcomAndDataIntervalSetting, &[0x17])
            .await?; //border floating
        self.command(spi, Command::VcmDcSetting).await?; // VCOM to 0V
        self.command(spi, Command::PanelSetting).await?;

        self.command(spi, Command::PowerSetting).await?; //VG&VS to 0V fast
        for _ in 0..4 {
            self.send_data(spi, &[0x00]).await?;
        }

        self.command(spi, Command::PowerOff).await?;
        self.wait_until_idle(spi).await?;
        self.interface
            .cmd_with_data(spi, Command::DeepSleep, &[0xA5])
            .await?;
        Ok(())
    }

    async fn wake_up(&mut self, spi: &mut SPI) -> Result<(), Self::Error> {
        self.init(spi).await
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

    async fn update_frame(&mut self, spi: &mut SPI, buffer: &[u8]) -> Result<(), Self::Error> {
        self.wait_until_idle(spi).await?;
        let color_value = self.color.get_byte_value();

        self.interface
            .cmd(spi, Command::DataStartTransmission1)
            .await?;
        self.interface
            .data_x_times(spi, color_value, WIDTH / 8 * HEIGHT)
            .await?;

        self.interface
            .cmd_with_data(spi, Command::DataStartTransmission2, buffer)
            .await?;
        Ok(())
    }

    async fn update_partial_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), Self::Error> {
        self.wait_until_idle(spi).await?;
        if buffer.len() as u32 != width / 8 * height {
            //TODO: panic!! or sth like that
            //return Err("Wrong buffersize");
        }

        self.command(spi, Command::PartialIn).await?;
        self.command(spi, Command::PartialWindow).await?;
        self.send_data(spi, &[(x >> 8) as u8]).await?;
        let tmp = x & 0xf8;
        self.send_data(spi, &[tmp as u8]).await?; // x should be the multiple of 8, the last 3 bit will always be ignored
        let tmp = tmp + width - 1;
        self.send_data(spi, &[(tmp >> 8) as u8]).await?;
        self.send_data(spi, &[(tmp | 0x07) as u8]).await?;

        self.send_data(spi, &[(y >> 8) as u8]).await?;
        self.send_data(spi, &[y as u8]).await?;

        self.send_data(spi, &[((y + height - 1) >> 8) as u8])
            .await?;
        self.send_data(spi, &[(y + height - 1) as u8]).await?;

        self.send_data(spi, &[0x01]).await?; // Gates scan both inside and outside of the partial window. (default)

        //TODO: handle dtm somehow
        let is_dtm1 = false;
        if is_dtm1 {
            self.command(spi, Command::DataStartTransmission1).await?; //TODO: check if data_start transmission 1 also needs "old"/background data here
        } else {
            self.command(spi, Command::DataStartTransmission2).await?;
        }

        self.send_data(spi, buffer).await?;

        self.command(spi, Command::PartialOut).await?;
        Ok(())
    }

    async fn display_frame(&mut self, spi: &mut SPI) -> Result<(), Self::Error> {
        self.wait_until_idle(spi).await?;
        self.command(spi, Command::DisplayRefresh).await?;
        Ok(())
    }

    async fn update_and_display_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
    ) -> Result<(), Self::Error> {
        self.update_frame(spi, buffer).await?;
        self.command(spi, Command::DisplayRefresh).await?;
        Ok(())
    }

    async fn clear_frame(&mut self, spi: &mut SPI) -> Result<(), Self::Error> {
        self.wait_until_idle(spi).await?;
        self.send_resolution(spi).await?;

        let color_value = self.color.get_byte_value();

        self.interface
            .cmd(spi, Command::DataStartTransmission1)
            .await?;
        self.interface
            .data_x_times(spi, color_value, WIDTH / 8 * HEIGHT)
            .await?;

        self.interface
            .cmd(spi, Command::DataStartTransmission2)
            .await?;
        self.interface
            .data_x_times(spi, color_value, WIDTH / 8 * HEIGHT)
            .await?;
        Ok(())
    }

    async fn set_lut(
        &mut self,
        spi: &mut SPI,
        refresh_rate: Option<RefreshLut>,
    ) -> Result<(), Self::Error> {
        if let Some(refresh_lut) = refresh_rate {
            self.refresh = refresh_lut;
        }
        match self.refresh {
            RefreshLut::Full => {
                self.set_lut_helper(spi, &LUT_VCOM0, &LUT_WW, &LUT_BW, &LUT_WB, &LUT_BB)
                    .await
            }
            RefreshLut::Quick => {
                self.set_lut_helper(
                    spi,
                    &LUT_VCOM0_QUICK,
                    &LUT_WW_QUICK,
                    &LUT_BW_QUICK,
                    &LUT_WB_QUICK,
                    &LUT_BB_QUICK,
                )
                .await
            }
        }
    }

    async fn wait_until_idle(&mut self, spi: &mut SPI) -> Result<(), Self::Error> {
        self.interface.wait_until_idle(spi, IS_BUSY_LOW).await
    }
}

impl<SPI, BUSY, DC, RST> Epd4in2<SPI, BUSY, DC, RST>
where
    SPI: SpiDevice,
    SPI::Error: Copy + Debug + Display,
    BUSY: InputPin + Wait,
    BUSY::Error: Copy + Debug + Display,
    DC: OutputPin,
    DC::Error: Copy + Debug + Display,
    RST: OutputPin,
    RST::Error: Copy + Debug + Display,
{
    async fn command(
        &mut self,
        spi: &mut SPI,
        command: Command,
    ) -> Result<(), <Self as ErrorType<SPI, BUSY, DC, RST>>::Error> {
        self.interface.cmd(spi, command).await
    }

    async fn send_data(
        &mut self,
        spi: &mut SPI,
        data: &[u8],
    ) -> Result<(), <Self as ErrorType<SPI, BUSY, DC, RST>>::Error> {
        self.interface.data(spi, data).await
    }

    async fn cmd_with_data(
        &mut self,
        spi: &mut SPI,
        command: Command,
        data: &[u8],
    ) -> Result<(), <Self as ErrorType<SPI, BUSY, DC, RST>>::Error> {
        self.interface.cmd_with_data(spi, command, data).await
    }

    async fn send_resolution(
        &mut self,
        spi: &mut SPI,
    ) -> Result<(), <Self as ErrorType<SPI, BUSY, DC, RST>>::Error> {
        let w = self.width();
        let h = self.height();

        self.command(spi, Command::ResolutionSetting).await?;
        self.send_data(spi, &[(w >> 8) as u8]).await?;
        self.send_data(spi, &[w as u8]).await?;
        self.send_data(spi, &[(h >> 8) as u8]).await?;
        self.send_data(spi, &[h as u8]).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn set_lut_helper(
        &mut self,
        spi: &mut SPI,
        lut_vcom: &[u8],
        lut_ww: &[u8],
        lut_bw: &[u8],
        lut_wb: &[u8],
        lut_bb: &[u8],
    ) -> Result<(), <Self as ErrorType<SPI, BUSY, DC, RST>>::Error> {
        self.wait_until_idle(spi).await?;
        // LUT VCOM
        self.cmd_with_data(spi, Command::LutForVcom, lut_vcom)
            .await?;

        // LUT WHITE to WHITE
        self.cmd_with_data(spi, Command::LutWhiteToWhite, lut_ww)
            .await?;

        // LUT BLACK to WHITE
        self.cmd_with_data(spi, Command::LutBlackToWhite, lut_bw)
            .await?;

        // LUT WHITE to BLACK
        self.cmd_with_data(spi, Command::LutWhiteToBlack, lut_wb)
            .await?;

        // LUT BLACK to BLACK
        self.cmd_with_data(spi, Command::LutBlackToBlack, lut_bb)
            .await?;
        Ok(())
    }

    /// Helper function. Sets up the display to send pixel data to a custom
    /// starting point.
    pub async fn shift_display(
        &mut self,
        spi: &mut SPI,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), <Self as ErrorType<SPI, BUSY, DC, RST>>::Error> {
        self.send_data(spi, &[(x >> 8) as u8]).await?;
        let tmp = x & 0xf8;
        self.send_data(spi, &[tmp as u8]).await?; // x should be the multiple of 8, the last 3 bit will always be ignored
        let tmp = tmp + width - 1;
        self.send_data(spi, &[(tmp >> 8) as u8]).await?;
        self.send_data(spi, &[(tmp | 0x07) as u8]).await?;

        self.send_data(spi, &[(y >> 8) as u8]).await?;
        self.send_data(spi, &[y as u8]).await?;

        self.send_data(spi, &[((y + height - 1) >> 8) as u8])
            .await?;
        self.send_data(spi, &[(y + height - 1) as u8]).await?;

        self.send_data(spi, &[0x01]).await?; // Gates scan both inside and outside of the partial window. (default)

        Ok(())
    }
}

impl<SPI, BUSY, DC, RST> QuickRefresh<SPI, BUSY, DC, RST> for Epd4in2<SPI, BUSY, DC, RST>
where
    SPI: SpiDevice,
    SPI::Error: Copy + Debug + Display,
    BUSY: InputPin + Wait,
    BUSY::Error: Copy + Debug + Display,
    DC: OutputPin,
    DC::Error: Copy + Debug + Display,
    RST: OutputPin,
    RST::Error: Copy + Debug + Display,
{
    /// To be followed immediately after by `update_old_frame`.
    async fn update_old_frame(&mut self, spi: &mut SPI, buffer: &[u8]) -> Result<(), Self::Error> {
        self.wait_until_idle(spi).await?;

        self.interface
            .cmd(spi, Command::DataStartTransmission1)
            .await?;

        self.interface.data(spi, buffer).await?;

        Ok(())
    }

    /// To be used immediately after `update_old_frame`.
    async fn update_new_frame(&mut self, spi: &mut SPI, buffer: &[u8]) -> Result<(), Self::Error> {
        self.wait_until_idle(spi).await?;
        // self.send_resolution(spi).await?;

        self.interface
            .cmd(spi, Command::DataStartTransmission2)
            .await?;

        self.interface.data(spi, buffer).await?;

        Ok(())
    }

    /// This is a wrapper around `display_frame` for using this device as a true
    /// `QuickRefresh` device.
    async fn display_new_frame(&mut self, spi: &mut SPI) -> Result<(), Self::Error> {
        self.display_frame(spi).await
    }

    /// This is wrapper around `update_new_frame` and `display_frame` for using
    /// this device as a true `QuickRefresh` device.
    ///
    /// To be used immediately after `update_old_frame`.
    async fn update_and_display_new_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
    ) -> Result<(), Self::Error> {
        self.update_new_frame(spi, buffer).await?;
        self.display_frame(spi).await
    }

    async fn update_partial_old_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), Self::Error> {
        self.wait_until_idle(spi).await?;

        if buffer.len() as u32 != width / 8 * height {
            //TODO: panic!! or sth like that
            //return Err("Wrong buffersize");
        }

        self.interface.cmd(spi, Command::PartialIn).await?;
        self.interface.cmd(spi, Command::PartialWindow).await?;

        self.shift_display(spi, x, y, width, height).await?;

        self.interface
            .cmd(spi, Command::DataStartTransmission1)
            .await?;

        self.interface.data(spi, buffer).await?;

        Ok(())
    }

    /// Always call `update_partial_old_frame` before this, with buffer-updating code
    /// between the calls.
    async fn update_partial_new_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), Self::Error> {
        self.wait_until_idle(spi).await?;
        if buffer.len() as u32 != width / 8 * height {
            //TODO: panic!! or sth like that
            //return Err("Wrong buffersize");
        }

        self.shift_display(spi, x, y, width, height).await?;

        self.interface
            .cmd(spi, Command::DataStartTransmission2)
            .await?;

        self.interface.data(spi, buffer).await?;

        self.interface.cmd(spi, Command::PartialOut).await?;
        Ok(())
    }

    async fn clear_partial_frame(
        &mut self,
        spi: &mut SPI,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), Self::Error> {
        self.wait_until_idle(spi).await?;
        self.send_resolution(spi).await?;

        let color_value = self.color.get_byte_value();

        self.interface.cmd(spi, Command::PartialIn).await?;
        self.interface.cmd(spi, Command::PartialWindow).await?;

        self.shift_display(spi, x, y, width, height).await?;

        self.interface
            .cmd(spi, Command::DataStartTransmission1)
            .await?;
        self.interface
            .data_x_times(spi, color_value, width / 8 * height)
            .await?;

        self.interface
            .cmd(spi, Command::DataStartTransmission2)
            .await?;
        self.interface
            .data_x_times(spi, color_value, width / 8 * height)
            .await?;

        self.interface.cmd(spi, Command::PartialOut).await?;
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
