//! A simple Driver for the Waveshare 2.7" B Tri-Color E-Ink Display via SPI
//!
//! [Documentation](https://www.waveshare.com/wiki/2.7inch_e-Paper_HAT_(B))
use core::fmt::{Debug, Display};
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal_async::{digital::Wait, spi::SpiDevice};

use crate::error::ErrorKind;
use crate::interface::DisplayInterface;
use crate::traits::{
    ErrorType, InternalWiAdditions, RefreshLut, WaveshareDisplay, WaveshareThreeColorDisplay,
};

// The Lookup Tables for the Display
mod constants;
use crate::epd2in7b::constants::*;

/// Width of the display
pub const WIDTH: u32 = 176;
/// Height of the display
pub const HEIGHT: u32 = 264;
/// Default Background Color
pub const DEFAULT_BACKGROUND_COLOR: Color = Color::White;
const IS_BUSY_LOW: bool = true;
const SINGLE_BYTE_WRITE: bool = true;

use crate::color::Color;

pub(crate) mod command;
use self::command::Command;
use crate::buffer_len;

/// Full size buffer for use with the 2in7B EPD
/// TODO this should be a TriColor, but let's keep it as is at first
#[cfg(feature = "graphics")]
pub type Display2in7b = crate::graphics::Display<
    WIDTH,
    HEIGHT,
    false,
    { buffer_len(WIDTH as usize, HEIGHT as usize) },
    Color,
>;

/// Epd2in7b driver
pub struct Epd2in7b<SPI, BUSY, DC, RST> {
    /// Connection Interface
    interface: DisplayInterface<SPI, BUSY, DC, RST, SINGLE_BYTE_WRITE>,
    /// Background Color
    color: Color,
}

impl<SPI, BUSY, DC, RST> ErrorType<SPI, BUSY, DC, RST> for Epd2in7b<SPI, BUSY, DC, RST>
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

impl<SPI, BUSY, DC, RST> InternalWiAdditions<SPI, BUSY, DC, RST> for Epd2in7b<SPI, BUSY, DC, RST>
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
        self.interface.reset(spi, 10_000, 2_000).await?;

        // power on
        self.command(spi, Command::PowerOn).await?;
        self.interface.delay(spi, 5000).await?;
        self.wait_until_idle(spi).await?;

        // set panel settings, 0xbf is bw, 0xaf is multi-color
        self.interface
            .cmd_with_data(spi, Command::PanelSetting, &[0xaf])
            .await?;

        // pll control
        self.interface
            .cmd_with_data(spi, Command::PllControl, &[0x3a])
            .await?;

        // set the power settings
        self.interface
            .cmd_with_data(spi, Command::PowerSetting, &[0x03, 0x00, 0x2b, 0x2b, 0x09])
            .await?;

        // start the booster
        self.interface
            .cmd_with_data(spi, Command::BoosterSoftStart, &[0x07, 0x07, 0x17])
            .await?;

        // power optimization
        self.interface
            .cmd_with_data(spi, Command::PowerOptimization, &[0x60, 0xa5])
            .await?;
        self.interface
            .cmd_with_data(spi, Command::PowerOptimization, &[0x89, 0xa5])
            .await?;
        self.interface
            .cmd_with_data(spi, Command::PowerOptimization, &[0x90, 0x00])
            .await?;
        self.interface
            .cmd_with_data(spi, Command::PowerOptimization, &[0x93, 0x2a])
            .await?;
        self.interface
            .cmd_with_data(spi, Command::PowerOptimization, &[0x73, 0x41])
            .await?;

        self.interface
            .cmd_with_data(spi, Command::VcmDcSetting, &[0x12])
            .await?;

        self.interface
            .cmd_with_data(spi, Command::VcomAndDataIntervalSetting, &[0x87])
            .await?;

        self.set_lut(spi, None).await?;

        self.interface
            .cmd_with_data(spi, Command::PartialDisplayRefresh, &[0x00])
            .await?;

        self.wait_until_idle(spi).await?;
        Ok(())
    }
}

impl<SPI, BUSY, DC, RST> WaveshareDisplay<SPI, BUSY, DC, RST> for Epd2in7b<SPI, BUSY, DC, RST>
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

        let mut epd = Epd2in7b { interface, color };

        epd.init(spi).await?;

        Ok(epd)
    }

    async fn wake_up(&mut self, spi: &mut SPI) -> Result<(), Self::Error> {
        self.init(spi).await
    }

    async fn sleep(&mut self, spi: &mut SPI) -> Result<(), Self::Error> {
        self.wait_until_idle(spi).await?;
        self.interface
            .cmd_with_data(spi, Command::VcomAndDataIntervalSetting, &[0xf7])
            .await?;

        self.command(spi, Command::PowerOff).await?;
        self.wait_until_idle(spi).await?;
        self.interface
            .cmd_with_data(spi, Command::DeepSleep, &[0xA5])
            .await?;
        Ok(())
    }

    async fn update_frame(&mut self, spi: &mut SPI, buffer: &[u8]) -> Result<(), Self::Error> {
        self.interface
            .cmd(spi, Command::DataStartTransmission1)
            .await?;
        self.send_buffer_helper(spi, buffer).await?;

        // Clear chromatic layer since we won't be using it here
        self.interface
            .data_x_times(spi, !self.color.get_byte_value(), WIDTH / 8 * HEIGHT)
            .await?;

        self.interface.cmd(spi, Command::DataStop).await?;
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
        self.interface
            .cmd(spi, Command::PartialDataStartTransmission1)
            .await?;

        self.send_data(spi, &[(x >> 8) as u8]).await?;
        self.send_data(spi, &[(x & 0xf8) as u8]).await?;
        self.send_data(spi, &[(y >> 8) as u8]).await?;
        self.send_data(spi, &[(y & 0xff) as u8]).await?;
        self.send_data(spi, &[(width >> 8) as u8]).await?;
        self.send_data(spi, &[(width & 0xf8) as u8]).await?;
        self.send_data(spi, &[(height >> 8) as u8]).await?;
        self.send_data(spi, &[(height & 0xff) as u8]).await?;
        self.wait_until_idle(spi).await?;

        self.send_buffer_helper(spi, buffer).await?;

        self.interface.cmd(spi, Command::DataStop).await
    }

    async fn display_frame(&mut self, spi: &mut SPI) -> Result<(), Self::Error> {
        self.command(spi, Command::DisplayRefresh).await?;
        self.wait_until_idle(spi).await?;
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

        let color_value = self.color.get_byte_value();
        self.interface
            .data_x_times(spi, color_value, WIDTH / 8 * HEIGHT)
            .await?;

        self.interface.cmd(spi, Command::DataStop).await?;

        self.interface
            .cmd(spi, Command::DataStartTransmission2)
            .await?;
        self.interface
            .data_x_times(spi, color_value, WIDTH / 8 * HEIGHT)
            .await?;
        self.interface.cmd(spi, Command::DataStop).await?;
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

    async fn set_lut(
        &mut self,
        spi: &mut SPI,
        _refresh_rate: Option<RefreshLut>,
    ) -> Result<(), Self::Error> {
        self.wait_until_idle(spi).await?;
        self.cmd_with_data(spi, Command::LutForVcom, &LUT_VCOM_DC)
            .await?;
        self.cmd_with_data(spi, Command::LutWhiteToWhite, &LUT_WW)
            .await?;
        self.cmd_with_data(spi, Command::LutBlackToWhite, &LUT_BW)
            .await?;
        self.cmd_with_data(spi, Command::LutWhiteToBlack, &LUT_WB)
            .await?;
        self.cmd_with_data(spi, Command::LutBlackToBlack, &LUT_BB)
            .await?;
        Ok(())
    }

    async fn wait_until_idle(&mut self, spi: &mut SPI) -> Result<(), Self::Error> {
        self.interface.wait_until_idle(spi, IS_BUSY_LOW).await
    }
}

impl<SPI, BUSY, DC, RST> WaveshareThreeColorDisplay<SPI, BUSY, DC, RST>
    for Epd2in7b<SPI, BUSY, DC, RST>
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
        self.update_chromatic_frame(spi, chromatic).await
    }

    /// Update only the black/white data of the display.
    ///
    /// Finish by calling `update_chromatic_frame`.
    async fn update_achromatic_frame(
        &mut self,
        spi: &mut SPI,
        achromatic: &[u8],
    ) -> Result<(), Self::Error> {
        self.interface
            .cmd(spi, Command::DataStartTransmission1)
            .await?;

        self.send_buffer_helper(spi, achromatic).await?;

        self.interface.cmd(spi, Command::DataStop).await
    }

    /// Update only chromatic data of the display.
    ///
    /// This data takes precedence over the black/white data.
    async fn update_chromatic_frame(
        &mut self,
        spi: &mut SPI,
        chromatic: &[u8],
    ) -> Result<(), Self::Error> {
        self.interface
            .cmd(spi, Command::DataStartTransmission2)
            .await?;

        self.send_buffer_helper(spi, chromatic).await?;

        self.interface.cmd(spi, Command::DataStop).await?;
        self.wait_until_idle(spi).await?;

        Ok(())
    }
}

impl<SPI, BUSY, DC, RST> Epd2in7b<SPI, BUSY, DC, RST>
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

    async fn send_buffer_helper(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
    ) -> Result<(), <Self as ErrorType<SPI, BUSY, DC, RST>>::Error> {
        // Based on the waveshare implementation, all data for color values is flipped. This helper
        // method makes that transmission easier
        for b in buffer.iter() {
            self.send_data(spi, &[!b]).await?;
        }
        Ok(())
    }

    async fn cmd_with_data(
        &mut self,
        spi: &mut SPI,
        command: Command,
        data: &[u8],
    ) -> Result<(), <Self as ErrorType<SPI, BUSY, DC, RST>>::Error> {
        self.interface.cmd_with_data(spi, command, data).await

    }

    /// Refresh display for partial frame
    pub async fn display_partial_frame(
        &mut self,
        spi: &mut SPI,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), <Self as ErrorType<SPI, BUSY, DC, RST>>::Error> {
        self.command(spi, Command::PartialDisplayRefresh).await?;
        self.send_data(spi, &[(x >> 8) as u8]).await?;
        self.send_data(spi, &[(x & 0xf8) as u8]).await?;
        self.send_data(spi, &[(y >> 8) as u8]).await?;
        self.send_data(spi, &[(y & 0xff) as u8]).await?;
        self.send_data(spi, &[(width >> 8) as u8]).await?;
        self.send_data(spi, &[(width & 0xf8) as u8]).await?;
        self.send_data(spi, &[(height >> 8) as u8]).await?;
        self.send_data(spi, &[(height & 0xff) as u8]).await?;
        self.wait_until_idle(spi).await?;
        Ok(())
    }

    /// Update black/achromatic frame
    #[allow(clippy::too_many_arguments)]
    pub async fn update_partial_achromatic_frame(
        &mut self,
        spi: &mut SPI,
        achromatic: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), <Self as ErrorType<SPI, BUSY, DC, RST>>::Error> {
        self.interface
            .cmd(spi, Command::PartialDataStartTransmission1)
            .await?;
        self.send_data(spi, &[(x >> 8) as u8]).await?;
        self.send_data(spi, &[(x & 0xf8) as u8]).await?;
        self.send_data(spi, &[(y >> 8) as u8]).await?;
        self.send_data(spi, &[(y & 0xff) as u8]).await?;
        self.send_data(spi, &[(width >> 8) as u8]).await?;
        self.send_data(spi, &[(width & 0xf8) as u8]).await?;
        self.send_data(spi, &[(height >> 8) as u8]).await?;
        self.send_data(spi, &[(height & 0xff) as u8]).await?;
        self.wait_until_idle(spi).await?;

        for b in achromatic.iter() {
            // Flipping based on waveshare implementation
            self.send_data(spi, &[!b]).await?;
        }

        Ok(())
    }

    /// Update partial chromatic/red frame
    #[allow(clippy::too_many_arguments)]
    pub async fn update_partial_chromatic_frame(
        &mut self,
        spi: &mut SPI,
        chromatic: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), <Self as ErrorType<SPI, BUSY, DC, RST>>::Error> {
        self.interface
            .cmd(spi, Command::PartialDataStartTransmission2)
            .await?;
        self.send_data(spi, &[(x >> 8) as u8]).await?;
        self.send_data(spi, &[(x & 0xf8) as u8]).await?;
        self.send_data(spi, &[(y >> 8) as u8]).await?;
        self.send_data(spi, &[(y & 0xff) as u8]).await?;
        self.send_data(spi, &[(width >> 8) as u8]).await?;
        self.send_data(spi, &[(width & 0xf8) as u8]).await?;
        self.send_data(spi, &[(height >> 8) as u8]).await?;
        self.send_data(spi, &[(height & 0xff) as u8]).await?;
        self.wait_until_idle(spi).await?;

        for b in chromatic.iter() {
            // Flipping based on waveshare implementation
            self.send_data(spi, &[!b]).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epd_size() {
        assert_eq!(WIDTH, 176);
        assert_eq!(HEIGHT, 264);
        assert_eq!(DEFAULT_BACKGROUND_COLOR, Color::White);
    }
}
