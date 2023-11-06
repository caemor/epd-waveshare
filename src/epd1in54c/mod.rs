//! A simple Driver for the Waveshare 1.54" (C) E-Ink Display via SPI
use core::fmt::{Debug, Display};
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal_async::{digital::Wait, spi::SpiDevice};

use crate::error::ErrorKind;
use crate::interface::DisplayInterface;
use crate::traits::{
    ErrorType, InternalWiAdditions, RefreshLut, WaveshareDisplay, WaveshareThreeColorDisplay,
};

/// Width of epd1in54 in pixels
pub const WIDTH: u32 = 152;
/// Height of epd1in54 in pixels
pub const HEIGHT: u32 = 152;
/// Default Background Color (white)
pub const DEFAULT_BACKGROUND_COLOR: Color = Color::White;
const IS_BUSY_LOW: bool = true;
const NUM_DISPLAY_BITS: u32 = WIDTH / 8 * HEIGHT;
const SINGLE_BYTE_WRITE: bool = true;

use crate::color::Color;

pub(crate) mod command;
use self::command::Command;
use crate::buffer_len;

/// Full size buffer for use with the 1in54c EPD
/// TODO this should be a TriColor, but let's keep it as is at first
#[cfg(feature = "graphics")]
pub type Display1in54c = crate::graphics::Display<
    WIDTH,
    HEIGHT,
    false,
    { buffer_len(WIDTH as usize, HEIGHT as usize) },
    Color,
>;

/// Epd1in54c driver
pub struct Epd1in54c<SPI, BUSY, DC, RST> {
    interface: DisplayInterface<SPI, BUSY, DC, RST, SINGLE_BYTE_WRITE>,
    color: Color,
}

impl<SPI, BUSY, DC, RST> ErrorType<SPI, BUSY, DC, RST> for Epd1in54c<SPI, BUSY, DC, RST>
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

impl<SPI, BUSY, DC, RST> InternalWiAdditions<SPI, BUSY, DC, RST> for Epd1in54c<SPI, BUSY, DC, RST>
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
        // Based on Reference Program Code from:
        // https://www.waveshare.com/w/upload/a/ac/1.54inch_e-Paper_Module_C_Specification.pdf
        // and:
        // https://github.com/waveshare/e-Paper/blob/master/STM32/STM32-F103ZET6/User/e-Paper/EPD_1in54c.c
        self.interface.reset(spi, 10_000, 2_000).await?;

        // start the booster
        self.cmd_with_data(spi, Command::BoosterSoftStart, &[0x17, 0x17, 0x17])
            .await?;

        // power on
        self.command(spi, Command::PowerOn).await?;
        self.interface.delay(spi, 5000).await?;
        self.wait_until_idle(spi).await?;

        // set the panel settings
        self.cmd_with_data(spi, Command::PanelSetting, &[0x0f, 0x0d])
            .await?;

        // set resolution
        self.send_resolution(spi).await?;

        self.cmd_with_data(spi, Command::VcomAndDataIntervalSetting, &[0x77])
            .await?;

        Ok(())
    }
}

impl<SPI, BUSY, DC, RST> WaveshareThreeColorDisplay<SPI, BUSY, DC, RST>
    for Epd1in54c<SPI, BUSY, DC, RST>
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

    async fn update_achromatic_frame(
        &mut self,
        spi: &mut SPI,
        black: &[u8],
    ) -> Result<(), Self::Error> {
        self.wait_until_idle(spi).await?;
        self.cmd_with_data(spi, Command::DataStartTransmission1, black)
            .await
    }

    async fn update_chromatic_frame(
        &mut self,
        spi: &mut SPI,
        chromatic: &[u8],
    ) -> Result<(), Self::Error> {
        self.wait_until_idle(spi).await?;
        self.cmd_with_data(spi, Command::DataStartTransmission2, chromatic)
            .await
    }
}

impl<SPI, BUSY, DC, RST> WaveshareDisplay<SPI, BUSY, DC, RST> for Epd1in54c<SPI, BUSY, DC, RST>
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

        let mut epd = Epd1in54c { interface, color };

        epd.init(spi).await?;

        Ok(epd)
    }

    async fn sleep(&mut self, spi: &mut SPI) -> Result<(), Self::Error> {
        self.wait_until_idle(spi).await?;

        self.command(spi, Command::PowerOff).await?;
        self.wait_until_idle(spi).await?;
        self.cmd_with_data(spi, Command::DeepSleep, &[0xa5]).await
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
        self.update_achromatic_frame(spi, buffer).await?;

        // Clear the chromatic layer
        let color = self.color.get_byte_value();

        self.command(spi, Command::DataStartTransmission2).await?;
        self.interface
            .data_x_times(spi, color, NUM_DISPLAY_BITS)
            .await
    }

    #[allow(unused)]
    async fn update_partial_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), Self::Error> {
        unimplemented!()
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
        let color = DEFAULT_BACKGROUND_COLOR.get_byte_value();

        // Clear the black
        self.command(spi, Command::DataStartTransmission1).await?;
        self.interface
            .data_x_times(spi, color, NUM_DISPLAY_BITS)
            .await?;

        // Clear the chromatic
        self.command(spi, Command::DataStartTransmission2).await?;
        self.interface
            .data_x_times(spi, color, NUM_DISPLAY_BITS)
            .await
    }

    async fn set_lut(
        &mut self,
        _spi: &mut SPI,
        _refresh_rate: Option<RefreshLut>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn wait_until_idle(&mut self, spi: &mut SPI) -> Result<(), Self::Error> {
        self.interface.wait_until_idle(spi, IS_BUSY_LOW).await
    }
}

impl<SPI, BUSY, DC, RST> Epd1in54c<SPI, BUSY, DC, RST>
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
    ) -> Result<(), ErrorKind<SPI, BUSY, DC, RST>> {
        self.interface.cmd(spi, command).await
    }

    async fn send_data(
        &mut self,
        spi: &mut SPI,
        data: &[u8],
    ) -> Result<(), ErrorKind<SPI, BUSY, DC, RST>> {
        self.interface.data(spi, data).await
    }

    async fn cmd_with_data(
        &mut self,
        spi: &mut SPI,
        command: Command,
        data: &[u8],
    ) -> Result<(), ErrorKind<SPI, BUSY, DC, RST>> {
        self.interface.cmd_with_data(spi, command, data).await
    }

    async fn send_resolution(
        &mut self,
        spi: &mut SPI,
    ) -> Result<(), ErrorKind<SPI, BUSY, DC, RST>> {
        let w = self.width();
        let h = self.height();

        self.command(spi, Command::ResolutionSetting).await?;

        // | D7 | D6 | D5 | D4 | D3 | D2 | D1 | D0 |
        // |       HRES[7:3]        |  0 |  0 |  0 |
        self.send_data(spi, &[(w as u8) & 0b1111_1000]).await?;
        // | D7 | D6 | D5 | D4 | D3 | D2 | D1 |      D0 |
        // |  - |  - |  - |  - |  - |  - |  - | VRES[8] |
        self.send_data(spi, &[(w >> 8) as u8]).await?;
        // | D7 | D6 | D5 | D4 | D3 | D2 | D1 |      D0 |
        // |                  VRES[7:0]                 |
        // Specification shows C/D is zero while sending the last byte,
        // but upstream code does not implement it like that. So for now
        // we follow upstream code.
        self.send_data(spi, &[h as u8]).await
    }
}
