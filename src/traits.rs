use crate::color::Color;
use core::marker::Sized;
use crate::hal::{
    blocking::{delay::*, spi::Write},
    digital::*,
};

/// All commands need to have this trait which gives the address of the command
/// which needs to be send via SPI with activated CommandsPin (Data/Command Pin in CommandMode)
pub(crate) trait Command {
    fn address(self) -> u8;
}

/// Seperates the different LUT for the Display Refresh process
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
    ///  - [new()](WaveshareInterface::new())
    ///  - [`wake_up`]
    ///
    ///
    /// This function calls [reset()](WaveshareInterface::reset()),
    /// so you don't need to call reset your self when trying to wake your device up
    /// after setting it to sleep.
    fn init<DELAY: DelayMs<u8>>(
        &mut self,
        spi: &mut SPI,
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error>;
}

/// All the functions to interact with the EPDs
///
/// This trait includes all public functions to use the EPDS
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
    /// This already initialises the device. That means [init()](WaveshareInterface::init()) isn't needed directly afterwards
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
    /// But you can also use [wake_up()](WaveshareInterface::wake_up()) to awaken.
    /// But as you need to power it up once more anyway you can also just directly use [new()](WaveshareInterface::new()) for resetting
    /// and initialising which already contains the reset
    fn sleep(&mut self, spi: &mut SPI) -> Result<(), SPI::Error>;

    /// Wakes the device up from sleep
    fn wake_up<DELAY: DelayMs<u8>>(
        &mut self,
        spi: &mut SPI,
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error>;

    /// Sets the backgroundcolor for various commands like [clear_frame()](WaveshareInterface::clear_frame())
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
    fn display_frame(&mut self, spi: &mut SPI) -> Result<(), SPI::Error>;

    /// Clears the frame buffer on the EPD with the declared background color
    /// The background color can be changed with [`set_background_color`]
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
}
