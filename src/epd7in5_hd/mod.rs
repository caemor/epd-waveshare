//! A simple Driver for the Waveshare 7.5" E-Ink Display (HD) via SPI
//!
//! Color values for this driver are inverted compared to the [EPD 7in5 V2 driver](crate::epd7in5_v2)
//! *EPD 7in5 HD:* White = 1/0xFF, Black = 0/0x00
//! *EPD 7in5 V2:* White = 0/0x00, Black = 1/0xFF
//!
//! # References
//!
//! - [Datasheet](https://www.waveshare.com/w/upload/2/27/7inch_HD_e-Paper_Specification.pdf)
//! - [Waveshare Python driver](https://github.com/waveshare/e-Paper/blob/master/RaspberryPi_JetsonNano/python/lib/waveshare_epd/epd7in5_HD.py)
//!
use core::fmt::{Debug, Display};
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal_async::{digital::Wait, spi::SpiDevice};

use crate::color::Color;
use crate::error::ErrorKind;
use crate::interface::DisplayInterface;
use crate::traits::{ErrorType, InternalWiAdditions, RefreshLut, WaveshareDisplay};

pub(crate) mod command;
use self::command::Command;
use crate::buffer_len;

/// Full size buffer for use with the 7in5 HD EPD
#[cfg(feature = "graphics")]
pub type Display7in5 = crate::graphics::Display<
    WIDTH,
    HEIGHT,
    false,
    { buffer_len(WIDTH as usize, HEIGHT as usize) },
    Color,
>;

/// Width of the display
pub const WIDTH: u32 = 880;
/// Height of the display
pub const HEIGHT: u32 = 528;
/// Default Background Color
pub const DEFAULT_BACKGROUND_COLOR: Color = Color::White; // Inverted for HD as compared to 7in5 v2 (HD: 0xFF = White)
const IS_BUSY_LOW: bool = false;
const SINGLE_BYTE_WRITE: bool = false;

/// EPD7in5 (HD) driver
///
pub struct Epd7in5<SPI, BUSY, DC, RST> {
    /// Connection Interface
    interface: DisplayInterface<SPI, BUSY, DC, RST, SINGLE_BYTE_WRITE>,
    /// Background Color
    color: Color,
}

impl<SPI, BUSY, DC, RST> ErrorType<SPI, BUSY, DC, RST> for Epd7in5<SPI, BUSY, DC, RST>
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

impl<SPI, BUSY, DC, RST> InternalWiAdditions<SPI, BUSY, DC, RST> for Epd7in5<SPI, BUSY, DC, RST>
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
        self.interface.reset(spi, 10_000, 2_000).await?;

        // HD procedure as described here:
        // https://github.com/waveshare/e-Paper/blob/master/RaspberryPi_JetsonNano/python/lib/waveshare_epd/epd7in5_HD.py
        // and as per specs:
        // https://www.waveshare.com/w/upload/2/27/7inch_HD_e-Paper_Specification.pdf

        self.wait_until_idle(spi).await?;
        self.command(spi, Command::SwReset).await?;
        self.wait_until_idle(spi).await?;

        self.cmd_with_data(spi, Command::AutoWriteRed, &[0xF7])
            .await?;
        self.wait_until_idle(spi).await?;
        self.cmd_with_data(spi, Command::AutoWriteBw, &[0xF7])
            .await?;
        self.wait_until_idle(spi).await?;

        self.cmd_with_data(spi, Command::SoftStart, &[0xAE, 0xC7, 0xC3, 0xC0, 0x40])
            .await?;

        self.cmd_with_data(spi, Command::DriverOutputControl, &[0xAF, 0x02, 0x01])
            .await?;

        self.cmd_with_data(spi, Command::DataEntry, &[0x01]).await?;

        self.cmd_with_data(spi, Command::SetRamXStartEnd, &[0x00, 0x00, 0x6F, 0x03])
            .await?;
        self.cmd_with_data(spi, Command::SetRamYStartEnd, &[0xAF, 0x02, 0x00, 0x00])
            .await?;

        self.cmd_with_data(spi, Command::VbdControl, &[0x05])
            .await?;

        self.cmd_with_data(spi, Command::TemperatureSensorControl, &[0x80])
            .await?;

        self.cmd_with_data(spi, Command::DisplayUpdateControl2, &[0xB1])
            .await?;

        self.command(spi, Command::MasterActivation).await?;
        self.wait_until_idle(spi).await?;

        self.cmd_with_data(spi, Command::SetRamXAc, &[0x00, 0x00])
            .await?;
        self.cmd_with_data(spi, Command::SetRamYAc, &[0x00, 0x00])
            .await
    }
}

impl<SPI, BUSY, DC, RST> WaveshareDisplay<SPI, BUSY, DC, RST> for Epd7in5<SPI, BUSY, DC, RST>
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

        let mut epd = Epd7in5 { interface, color };

        epd.init(spi).await?;

        Ok(epd)
    }

    async fn wake_up(&mut self, spi: &mut SPI) -> Result<(), Self::Error> {
        self.init(spi).await
    }

    async fn sleep(&mut self, spi: &mut SPI) -> Result<(), Self::Error> {
        self.wait_until_idle(spi).await?;
        self.cmd_with_data(spi, Command::DeepSleep, &[0x01]).await
    }

    async fn update_frame(&mut self, spi: &mut SPI, buffer: &[u8]) -> Result<(), Self::Error> {
        self.wait_until_idle(spi).await?;
        self.cmd_with_data(spi, Command::SetRamYAc, &[0x00, 0x00])
            .await?;
        self.cmd_with_data(spi, Command::WriteRamBw, buffer).await?;
        self.cmd_with_data(spi, Command::DisplayUpdateControl2, &[0xF7])
            .await
    }

    async fn update_partial_frame(
        &mut self,
        _spi: &mut SPI,
        _buffer: &[u8],
        _x: u32,
        _y: u32,
        _width: u32,
        _height: u32,
    ) -> Result<(), Self::Error> {
        unimplemented!();
    }

    async fn display_frame(&mut self, spi: &mut SPI) -> Result<(), Self::Error> {
        self.command(spi, Command::MasterActivation).await?;
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
        let pixel_count = WIDTH / 8 * HEIGHT;
        let background_color_byte = self.color.get_byte_value();

        self.wait_until_idle(spi).await?;
        self.cmd_with_data(spi, Command::SetRamYAc, &[0x00, 0x00])
            .await?;

        for cmd in &[Command::WriteRamBw, Command::WriteRamRed] {
            self.command(spi, *cmd).await?;
            self.interface
                .data_x_times(spi, background_color_byte, pixel_count)
                .await?;
        }

        self.cmd_with_data(spi, Command::DisplayUpdateControl2, &[0xF7])
            .await?;
        self.command(spi, Command::MasterActivation).await?;
        self.wait_until_idle(spi).await
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

impl<SPI, BUSY, DC, RST> Epd7in5<SPI, BUSY, DC, RST>
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

    async fn cmd_with_data(
        &mut self,
        spi: &mut SPI,
        command: Command,
        data: &[u8],
    ) -> Result<(), <Self as ErrorType<SPI, BUSY, DC, RST>>::Error> {
        self.interface.cmd_with_data(spi, command, data).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epd_size() {
        assert_eq!(WIDTH, 880);
        assert_eq!(HEIGHT, 528);
        assert_eq!(DEFAULT_BACKGROUND_COLOR, Color::White);
    }
}
