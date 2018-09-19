use core::marker::Sized;
use hal::{
    blocking::{delay::*, spi::Write},
    digital::*,
};

use color::Color;

/// Interface for the physical connection between display and the controlling device
pub(crate) mod connection_interface;

/// All commands need to have this trait which gives the address of the command
/// which needs to be send via SPI with activated CommandsPin (Data/Command Pin in CommandMode)
pub(crate) trait Command {
    fn address(self) -> u8;
}

//TODO: add LUT trait with set_fast_lut and set_manual_lut and set_normal_lut or sth like that?
// for partial updates
trait LUTSupport<ERR> {
    fn set_lut(&mut self) -> Result<(), ERR>;
    fn set_lut_quick(&mut self) -> Result<(), ERR>;
    fn set_lut_manual(&mut self, data: &[u8]) -> Result<(), ERR>;
}

pub(crate) trait InternalWiAdditions<SPI, CS, BUSY, DC, RST, Delay, ERR>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    Delay: DelayUs<u16> + DelayMs<u16>,
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
    fn init(&mut self) -> Result<(), ERR>;
}


pub trait WaveshareInterface<SPI, CS, BUSY, DC, RST, Delay, ERR>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    Delay: DelayUs<u16> + DelayMs<u16>,
{
    

    /// Creates a new driver from a SPI peripheral, CS Pin, Busy InputPin, DC
    ///
    /// This already initialises the device. That means [init()](WaveshareInterface::init()) isn't needed directly afterwards
    fn new(
        spi: SPI, cs: CS, busy: BUSY, dc: DC, rst: RST, delay: Delay,
    ) -> Result<Self, ERR>
    where
        Self: Sized;

    // TODO: add this abstraction function
    /// Loads a full image on the EPD and displays it
    fn update_and_display_frame(&mut self, buffer: &[u8]) -> Result<(), ERR> {
        self.update_frame(buffer)?;
        self.display_frame()
    }

    /// Loads a partial image on the EPD and displays it
    fn update_and_display_partial_frame(
        &mut self,
        buffer: &[u8],
        x: u16,
        y: u16,
        width: u16,
        height: u16,
    ) -> Result<(), ERR> {
        self.update_partial_frame(buffer, x, y, width, height)?;
        self.display_frame()
    }   

    /// Let the device enter deep-sleep mode to save power.
    ///
    /// The deep sleep mode returns to standby with a hardware reset.
    /// But you can also use [reset()](WaveshareInterface::reset()) to awaken.
    /// But as you need to power it up once more anyway you can also just directly use [init()](WaveshareInterface::init()) for resetting
    /// and initialising which already contains the reset
    fn sleep(&mut self) -> Result<(), ERR>;

    fn wake_up(&mut self) -> Result<(), ERR>;   
    

    /// Sets the backgroundcolor for various commands like [clear_frame()](WaveshareInterface::clear_frame())
    fn set_background_color(&mut self, color: Color);

    /// Get current background color
    fn background_color(&self) -> &Color;

    /// Get the width of the display
    fn get_width(&self) -> u16;

    /// Get the height of the display
    fn get_height(&self) -> u16;

    /// Abstraction of setting the delay for simpler calls
    ///
    /// maximum delay ~65 seconds (u16:max in ms)
    fn delay_ms(&mut self, delay: u16);

    // void DisplayFrame(const unsigned char* frame_buffer);
    /// Transmit a full frame to the SRAM of the DPD
    ///
    fn update_frame(&mut self, buffer: &[u8]) -> Result<(), ERR>;

    /// Transmits partial data to the SRAM of the EPD
    ///
    /// BUFFER needs to be of size: w / 8 * h !
    fn update_partial_frame(
        &mut self,
        buffer: &[u8],
        x: u16,
        y: u16,
        width: u16,
        height: u16,
    ) -> Result<(), ERR>;

    /// Displays the frame data from SRAM
    fn display_frame(&mut self) -> Result<(), ERR>;

    /// Clears the frame from the buffer with the declared background color
    /// The background color can be changed with [`set_background_color`]
    ///
    /// Uses the chosen background color
    fn clear_frame(&mut self) -> Result<(), ERR>;
}
