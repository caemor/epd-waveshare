//! A simple Driver for the Waveshare 5.83" (B) v2 E-Ink Display via SPI
//!
//! # References
//!
//! - [Datasheet](https://www.waveshare.com/5.83inch-e-Paper-B.htm)
//! - [Waveshare C driver](https://github.com/waveshare/e-Paper/blob/master/RaspberryPi_JetsonNano/c/lib/e-Paper/EPD_5in83b_V2.c)
//! - [Waveshare Python driver](https://github.com/waveshare/e-Paper/blob/master/RaspberryPi_JetsonNano/python/lib/waveshare_epd/epd5in83b_V2.py)
use core::fmt::{Debug, Display};
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal_async::{digital::Wait, spi::SpiDevice};

use crate::color::{Color, TriColor};
use crate::error::ErrorKind;
use crate::interface::DisplayInterface;
use crate::traits::{
    ErrorType, InternalWiAdditions, RefreshLut, WaveshareDisplay, WaveshareThreeColorDisplay,
};

pub(crate) mod command;
use self::command::Command;
use crate::buffer_len;

/// Full size buffer for use with the 5in83b v2 EPD
#[cfg(feature = "graphics")]
pub type Display5in83 = crate::graphics::Display<
    WIDTH,
    HEIGHT,
    false,
    { buffer_len(WIDTH as usize, HEIGHT as usize * 2) },
    TriColor,
>;

/// Width of the display
pub const WIDTH: u32 = 648;
/// Height of the display
pub const HEIGHT: u32 = 480;
/// Default Background Color
pub const DEFAULT_BACKGROUND_COLOR: Color = Color::White;
const IS_BUSY_LOW: bool = true;
const NUM_DISPLAY_BITS: u32 = WIDTH / 8 * HEIGHT;
const SINGLE_BYTE_WRITE: bool = true;

/// Epd7in5 driver
///
pub struct Epd5in83<SPI, BUSY, DC, RST> {
    /// Connection Interface
    interface: DisplayInterface<SPI, BUSY, DC, RST, SINGLE_BYTE_WRITE>,
    /// Background Color
    color: Color,
}

impl<SPI, BUSY, DC, RST> ErrorType<SPI, BUSY, DC, RST> for Epd5in83<SPI, BUSY, DC, RST>
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

impl<SPI, BUSY, DC, RST> InternalWiAdditions<SPI, BUSY, DC, RST> for Epd5in83<SPI, BUSY, DC, RST>
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
        // Reset the device
        self.interface.reset(spi, 10_000, 10_000).await?;

        // Start the booster
        self.cmd_with_data(spi, Command::BoosterSoftStart, &[0x17, 0x17, 0x1e, 0x17])
            .await?;

        // Set the power settings: VGH=20V,VGL=-20V,VDH=15V,VDL=-15V
        self.cmd_with_data(spi, Command::PowerSetting, &[0x07, 0x07, 0x3F, 0x3F])
            .await?;

        // Power on
        self.command(spi, Command::PowerOn).await?;
        self.interface.delay(spi, 5000).await?;
        self.wait_until_idle(spi).await?;

        // Set the panel settings: BWROTP
        self.cmd_with_data(spi, Command::PanelSetting, &[0x0F])
            .await?;

        // Set the real resolution
        self.send_resolution(spi).await?;

        // Disable dual SPI
        self.cmd_with_data(spi, Command::DualSPI, &[0x00]).await?;

        // Set Vcom and data interval
        self.cmd_with_data(spi, Command::VcomAndDataIntervalSetting, &[0x11, 0x07])
            .await?;

        // Set S2G and G2S non-overlap periods to 12 (default)
        self.cmd_with_data(spi, Command::TconSetting, &[0x22])
            .await?;

        self.wait_until_idle(spi).await
    }
}

impl<SPI, BUSY, DC, RST> WaveshareThreeColorDisplay<SPI, BUSY, DC, RST>
    for Epd5in83<SPI, BUSY, DC, RST>
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
    async fn update_color_frame(
        &mut self,
        spi: &mut SPI,
        black: &[u8],
        chromatic: &[u8],
    ) -> Result<(), Self::Error> {
        self.update_achromatic_frame(spi, black).await?;
        self.update_chromatic_frame(spi, chromatic).await?;
        Ok(())
    }

    async fn update_achromatic_frame(
        &mut self,
        spi: &mut SPI,
        black: &[u8],
    ) -> Result<(), Self::Error> {
        self.wait_until_idle(spi).await?;
        self.cmd_with_data(spi, Command::DataStartTransmission1, black)
            .await?;
        Ok(())
    }

    async fn update_chromatic_frame(
        &mut self,
        spi: &mut SPI,
        chromatic: &[u8],
    ) -> Result<(), Self::Error> {
        self.wait_until_idle(spi).await?;
        self.cmd_with_data(spi, Command::DataStartTransmission2, chromatic)
            .await?;
        Ok(())
    }
}

impl<SPI, BUSY, DC, RST> WaveshareDisplay<SPI, BUSY, DC, RST> for Epd5in83<SPI, BUSY, DC, RST>
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

        let mut epd = Epd5in83 { interface, color };

        epd.init(spi).await?;

        Ok(epd)
    }

    async fn sleep(&mut self, spi: &mut SPI) -> Result<(), Self::Error> {
        self.wait_until_idle(spi).await?;
        self.command(spi, Command::PowerOff).await?;
        self.wait_until_idle(spi).await?;
        self.cmd_with_data(spi, Command::DeepSleep, &[0xA5]).await
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
        self.update_achromatic_frame(spi, buffer).await?;
        let color = self.color.get_byte_value();
        self.command(spi, Command::DataStartTransmission2).await?;
        self.interface
            .data_x_times(spi, color, NUM_DISPLAY_BITS)
            .await
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
            //TODO panic or error
        }

        let hrst_upper = (x / 8) as u8 >> 6;
        let hrst_lower = ((x / 8) << 3) as u8;
        let hred_upper = ((x + width) / 8) as u8 >> 6;
        let hred_lower = (((x + width) / 8) << 3) as u8 & 0b111;
        let vrst_upper = (y >> 8) as u8;
        let vrst_lower = y as u8;
        let vred_upper = ((y + height) >> 8) as u8;
        let vred_lower = (y + height) as u8;
        let pt_scan = 0x01; // Gates scan both inside and outside of the partial window. (default)

        self.command(spi, Command::PartialIn).await?;
        self.command(spi, Command::PartialWindow).await?;
        self.send_data(
            spi,
            &[
                hrst_upper, hrst_lower, hred_upper, hred_lower, vrst_upper, vrst_lower, vred_upper,
                vred_lower, pt_scan,
            ],
        )
        .await?;
        self.command(spi, Command::DataStartTransmission1).await?;
        self.send_data(spi, buffer).await?;

        let color = TriColor::Black.get_byte_value(); //We need it black, so red channel will be rendered transparent
        self.command(spi, Command::DataStartTransmission2).await?;
        self.interface
            .data_x_times(spi, color, width * height / 8)
            .await?;

        self.command(spi, Command::DisplayRefresh).await?;
        self.wait_until_idle(spi).await?;

        self.command(spi, Command::PartialOut).await
    }

    async fn display_frame(&mut self, spi: &mut SPI) -> Result<(), Self::Error> {
        self.command(spi, Command::DisplayRefresh).await?;
        self.wait_until_idle(spi).await
    }

    async fn update_and_display_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
    ) -> Result<(), Self::Error> {
        self.update_frame(spi, buffer).await?;
        self.display_frame(spi).await
    }

    async fn clear_frame(&mut self, spi: &mut SPI) -> Result<(), Self::Error> {
        self.wait_until_idle(spi).await?;

        // The Waveshare controllers all implement clear using 0x33
        self.command(spi, Command::DataStartTransmission1).await?;
        self.interface
            .data_x_times(spi, 0xFF, NUM_DISPLAY_BITS)
            .await?;

        self.command(spi, Command::DataStartTransmission2).await?;
        self.interface
            .data_x_times(spi, 0x00, NUM_DISPLAY_BITS)
            .await
    }

    async fn set_lut(
        &mut self,
        _spi: &mut SPI,
        _refresh_rate: Option<RefreshLut>,
    ) -> Result<(), Self::Error> {
        unimplemented!();
    }

    async fn wait_until_idle(&mut self, spi: &mut SPI) -> Result<(), Self::Error> {
        self.interface.wait_until_idle(spi, IS_BUSY_LOW).await
    }
}

impl<SPI, BUSY, DC, RST> Epd5in83<SPI, BUSY, DC, RST>
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

        self.command(spi, Command::TconResolution).await?;
        self.send_data(spi, &[(w >> 8) as u8]).await?;
        self.send_data(spi, &[w as u8]).await?;
        self.send_data(spi, &[(h >> 8) as u8]).await?;
        self.send_data(spi, &[h as u8]).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epd_size() {
        assert_eq!(WIDTH, 648);
        assert_eq!(HEIGHT, 480);
        assert_eq!(DEFAULT_BACKGROUND_COLOR, Color::White);
    }
}
