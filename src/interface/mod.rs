use hal::{
    blocking::{
        spi::Write,
        delay::*
    },
    spi::{Mode, Phase, Polarity},
    digital::*
};

use drawing::color::Color;

pub mod data_interface;

//TODO: test spi mode
/// SPI mode - 
/// For more infos see [Requirements: SPI](index.html#spi)
pub const SPI_MODE: Mode = Mode {
    phase: Phase::CaptureOnFirstTransition,
    polarity: Polarity::IdleLow,
};

use core::marker::Sized;

pub(crate) trait Command {
    fn address(self) -> u8;
}

pub trait WaveshareInterface<SPI, CS, BUSY, DC, RST, D, E>
    where 
        SPI: Write<u8, Error = E>,
        CS: OutputPin,
        BUSY: InputPin,
        DC: OutputPin,
        RST: OutputPin,
        D: DelayUs<u16> + DelayMs<u16>,
{
    /// Get the width of the display
    fn get_width(&self) -> u32;

    /// Get the height of the display
    fn get_height(&self) -> u32;

    /// Creates a new driver from a SPI peripheral, CS Pin, Busy InputPin, DC
    /// 
    /// This already initialises the device. That means [init()](WaveshareInterface::init()) isn't needed directly afterwards
    fn new(
        spi: SPI, 
        cs: CS, 
        busy: BUSY, 
        dc: DC, 
        rst: RST, 
        delay: D
    ) -> Result<Self, E>
        where Self: Sized;

    /// This initialises the EPD and powers it up
    /// 
    /// This function is already called from [new()](WaveshareInterface::new())
    /// 
    /// This function calls [reset()](WaveshareInterface::reset()),
    /// so you don't need to call reset your self when trying to wake your device up
    /// after setting it to sleep.
    fn init(&mut self) -> Result<(), E>;



    fn update_frame(&mut self, buffer: &[u8]) -> Result<(), E>;

    fn update_partial_frame(&mut self, buffer: &[u8], x: u16, y: u16, width: u16, height: u16) -> Result<(), E>;

    /// Displays the frame data from SRAM
    fn display_frame(&mut self) -> Result<(), E>;

    // TODO: add this abstraction function
    // fn update_and_display_frame(&mut self, buffer: &[u8]) -> Result<(), E>;

    /// Clears the frame from the buffer
    /// 
    /// Uses the chosen background color
    fn clear_frame(&mut self) -> Result<(), E>;

    /// Sets the backgroundcolor for various commands like [clear_frame()](WaveshareInterface::clear_frame())
    fn set_background_color(&mut self, color: Color);


    /// Let the device enter deep-sleep mode to save power. 
    /// 
    /// The deep sleep mode returns to standby with a hardware reset. 
    /// But you can also use [reset()](WaveshareInterface::reset()) to awaken.
    /// But as you need to power it up once more anyway you can also just directly use [init()](WaveshareInterface::init()) for resetting
    /// and initialising which already contains the reset
    fn sleep(&mut self) -> Result<(), E>;

    /// Resets the device.
    /// 
    /// Often used to awake the module from deep sleep. See [sleep()](WaveshareInterface::sleep())
    fn reset(&mut self);

    /// Abstraction of setting the delay for simpler calls
    /// 
    /// maximum delay ~65 seconds (u16:max in ms)
    fn delay_ms(&mut self, delay: u16);

    /*
    -display_frame
    -clear_frame
    -set_full_frame
    -set_partial_frame

    //
    -set_quick_lut?
    -set_normal_mode


    fn clear_frame(&mut self, reset_color: Option<Color>) -> Result<(), E>

    fn display_frame_quick(&mut self) -> Result<(), E>

    fn display_frame(&mut self) -> Result<(), E>

    pub fn display_and_transfer_frame(
    &mut self, 
    buffer: &[u8], 
    color: Option<u8>
) -> Result<(), E>

    pub fn set_partial_window(
    &mut self, 
    buffer: &[u8], 
    x: u16, 
    y: u16, 
    w: u16, 
    l: u16, 
    is_dtm1: bool
) -> Result<(), E>

*/

}