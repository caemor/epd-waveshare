use crate::color::Color;
use core::marker::Sized;
use embedded_hal::{
    blocking::{delay::*, spi::Write},
    digital::v2::*,
};

/// All commands need to have this trait which gives the address of the command
/// which needs to be send via SPI with activated CommandsPin (Data/Command Pin in CommandMode)
pub(crate) trait Command {
    fn address(self) -> u8;
}

/// Seperates the different LUT for the Display Refresh process
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum RefreshLUT {
    /// The "normal" full Lookuptable for the Refresh-Sequence
    FULL,
    /// The quick LUT where not the full refresh sequence is followed.
    /// This might lead to some
    QUICK,
}

impl Default for RefreshLUT {
    fn default() -> Self {
        RefreshLUT::FULL
    }
}

pub(crate) trait InternalWiAdditions<SPI, CS, BUSY, DC, RST>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    /// This initialises the EPD and powers it up
    ///
    /// This function is already called from
    ///  - [new()](WaveshareDisplay::new())
    ///  - [`wake_up`]
    ///
    ///
    /// This function calls [reset](WaveshareDisplay::reset),
    /// so you don't need to call reset your self when trying to wake your device up
    /// after setting it to sleep.
    fn init<DELAY: DelayMs<u8>>(
        &mut self,
        spi: &mut SPI,
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error>;
}

/// Functions to interact with three color panels
pub trait WaveshareThreeColorDisplay<SPI, CS, BUSY, DC, RST>:
    WaveshareDisplay<SPI, CS, BUSY, DC, RST>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    /// Transmit data to the SRAM of the EPD
    ///
    /// Updates both the black and the secondary color layers
    fn update_color_frame(
        &mut self,
        spi: &mut SPI,
        black: &[u8],
        chromatic: &[u8],
    ) -> Result<(), SPI::Error>;

    /// Update only the black/white data of the display.
    ///
    /// This must be finished by calling `update_chromatic_frame`.
    fn update_achromatic_frame(&mut self, spi: &mut SPI, black: &[u8]) -> Result<(), SPI::Error>;

    /// Update only the chromatic data of the display.
    ///
    /// This should be preceded by a call to `update_achromatic_frame`.
    /// This data takes precedence over the black/white data.
    fn update_chromatic_frame(&mut self, spi: &mut SPI, chromatic: &[u8])
        -> Result<(), SPI::Error>;
}

/// All the functions to interact with the EPDs
///
/// This trait includes all public functions to use the EPDs
///
/// # Example
///
///```rust, no_run
///# use embedded_hal_mock::*;
///# fn main() -> Result<(), MockError> {
///use embedded_graphics::{
///    pixelcolor::BinaryColor::On as Black, prelude::*, primitives::Line, style::PrimitiveStyle,
///};
///use epd_waveshare::{epd4in2::*, prelude::*};
///#
///# let expectations = [];
///# let mut spi = spi::Mock::new(&expectations);
///# let expectations = [];
///# let cs_pin = pin::Mock::new(&expectations);
///# let busy_in = pin::Mock::new(&expectations);
///# let dc = pin::Mock::new(&expectations);
///# let rst = pin::Mock::new(&expectations);
///# let mut delay = delay::MockNoop::new();
///
///// Setup EPD
///let mut epd = EPD4in2::new(&mut spi, cs_pin, busy_in, dc, rst, &mut delay)?;
///
///// Use display graphics from embedded-graphics
///let mut display = Display4in2::default();
///
///// Use embedded graphics for drawing a line
///let _ = Line::new(Point::new(0, 120), Point::new(0, 295))
///    .into_styled(PrimitiveStyle::with_stroke(Black, 1))
///    .draw(&mut display);
///
///    // Display updated frame
///epd.update_frame(&mut spi, &display.buffer())?;
///epd.display_frame(&mut spi)?;
///
///// Set the EPD to sleep
///epd.sleep(&mut spi)?;
///# Ok(())
///# }
///```
pub trait WaveshareDisplay<SPI, CS, BUSY, DC, RST>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    /// Creates a new driver from a SPI peripheral, CS Pin, Busy InputPin, DC
    ///
    /// This already initialises the device.
    fn new<DELAY: DelayMs<u8>>(
        spi: &mut SPI,
        cs: CS,
        busy: BUSY,
        dc: DC,
        rst: RST,
        delay: &mut DELAY,
    ) -> Result<Self, SPI::Error>
    where
        Self: Sized;

    /// Let the device enter deep-sleep mode to save power.
    ///
    /// The deep sleep mode returns to standby with a hardware reset.
    fn sleep(&mut self, spi: &mut SPI) -> Result<(), SPI::Error>;

    /// Wakes the device up from sleep
    ///
    /// Also reintialises the device if necessary.
    fn wake_up<DELAY: DelayMs<u8>>(
        &mut self,
        spi: &mut SPI,
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error>;

    /// Sets the backgroundcolor for various commands like [clear_frame](WaveshareDisplay::clear_frame)
    fn set_background_color(&mut self, color: Color);

    /// Get current background color
    fn background_color(&self) -> &Color;

    /// Get the width of the display
    fn width(&self) -> u32;

    /// Get the height of the display
    fn height(&self) -> u32;

    /// Transmit a full frame to the SRAM of the EPD
    fn update_frame(&mut self, spi: &mut SPI, buffer: &[u8]) -> Result<(), SPI::Error>;

    /// Transmits partial data to the SRAM of the EPD
    ///
    /// (x,y) is the top left corner
    ///
    /// BUFFER needs to be of size: width / 8 * height !
    fn update_partial_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), SPI::Error>;

    /// Displays the frame data from SRAM
    ///
    /// This function waits until the device isn`t busy anymore
    fn display_frame(&mut self, spi: &mut SPI) -> Result<(), SPI::Error>;

    /// Provide a combined update&display and save some time (skipping a busy check in between)
    fn update_and_display_frame(&mut self, spi: &mut SPI, buffer: &[u8]) -> Result<(), SPI::Error>;

    /// Clears the frame buffer on the EPD with the declared background color
    ///
    /// The background color can be changed with [`WaveshareDisplay::set_background_color`]
    fn clear_frame(&mut self, spi: &mut SPI) -> Result<(), SPI::Error>;

    /// Trait for using various Waveforms from different LUTs
    /// E.g. for partial refreshes
    ///
    /// A full refresh is needed after a certain amount of quick refreshes!
    ///
    /// WARNING: Quick Refresh might lead to ghosting-effects/problems with your display. Especially for the 4.2in Display!
    ///
    /// If None is used the old value will be loaded on the LUTs once more
    fn set_lut(
        &mut self,
        spi: &mut SPI,
        refresh_rate: Option<RefreshLUT>,
    ) -> Result<(), SPI::Error>;

    /// Checks if the display is busy transmitting data
    ///
    /// This is normally handled by the more complicated commands themselves,
    /// but in the case you send data and commands directly you might need to check
    /// if the device is still busy
    fn is_busy(&self) -> bool;
}
