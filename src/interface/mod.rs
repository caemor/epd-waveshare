use hal::{
    blocking::{
        spi::Write,
        delay::*
    },
    digital::*
};
use core::marker::Sized;

use drawing::color::Color;

/// Interface for the physical connection between display and the controlling device
pub mod connection_interface;
use self::connection_interface::ConnectionInterface;


/// All commands need to have this trait which gives the address of the command
/// which needs to be send via SPI with activated CommandsPin (Data/Command Pin in CommandMode)
pub(crate) trait Command {
    fn address(self) -> u8;
}


pub trait Displays {
    fn width(self) -> u8;
    fn height(self) -> u8;
}




//TODO: add LUT trait with set_fast_lut and set_manual_lut and set_normal_lut or sth like that?
// for partial updates
trait LUTSupport<Error> {
    fn set_lut(&mut self) -> Result<(), Error>;
    fn set_lut_quick(&mut self) -> Result<(), Error>;
    fn set_lut_manual(&mut self, data: &[u8]) -> Result<(), Error>;
}


pub trait WaveshareInterface<SPI, CS, BUSY, DataCommand, RST, Delay, Error>
    where 
        SPI: Write<u8>,
        CS: OutputPin,
        BUSY: InputPin,
        DataCommand: OutputPin,
        RST: OutputPin,
        Delay: DelayUs<u16> + DelayMs<u16>,
{
    /// Get the width of the display
    fn get_width(&self) -> u16;

    /// Get the height of the display
    fn get_height(&self) -> u16;

    /// Creates a new driver from a SPI peripheral, CS Pin, Busy InputPin, DC
    /// 
    /// This already initialises the device. That means [init()](WaveshareInterface::init()) isn't needed directly afterwards
    fn new(
        interface: ConnectionInterface<SPI, CS, BUSY, DataCommand, RST, Delay>
    ) -> Result<Self, Error>
        where Self: Sized;

    /// This initialises the EPD and powers it up
    /// 
    /// This function is already called from [new()](WaveshareInterface::new())
    /// 
    /// This function calls [reset()](WaveshareInterface::reset()),
    /// so you don't need to call reset your self when trying to wake your device up
    /// after setting it to sleep.
    fn init(&mut self) -> Result<(), Error>;


    // void DisplayFrame(const unsigned char* frame_buffer);
    /// Transmit a full frame to the SRAM of the DPD
    /// 
    fn update_frame(&mut self, buffer: &[u8]) -> Result<(), Error>;

    //TODO: is dtm always used?
    /// Transmit partial data to the SRAM of the EPD,
    /// the final parameter dtm chooses between the 2
    /// internal buffers 
    /// 
    /// Normally it should be dtm2, so use false
    /// 
    /// BUFFER needs to be of size: w / 8 * l !
    fn update_partial_frame(&mut self, buffer: &[u8], x: u16, y: u16, width: u16, height: u16) -> Result<(), Error>;

    /// Displays the frame data from SRAM
    fn display_frame(&mut self) -> Result<(), Error>;

    // TODO: add this abstraction function
    fn update_and_display_frame(&mut self, buffer: &[u8]) -> Result<(), Error>;

    /// Clears the frame from the buffer
    /// 
    /// Uses the chosen background color
    fn clear_frame(&mut self) -> Result<(), Error>;

    /// Sets the backgroundcolor for various commands like [clear_frame()](WaveshareInterface::clear_frame())
    fn set_background_color(&mut self, color: Color);


    /// Let the device enter deep-sleep mode to save power. 
    /// 
    /// The deep sleep mode returns to standby with a hardware reset. 
    /// But you can also use [reset()](WaveshareInterface::reset()) to awaken.
    /// But as you need to power it up once more anyway you can also just directly use [init()](WaveshareInterface::init()) for resetting
    /// and initialising which already contains the reset
    fn sleep(&mut self) -> Result<(), Error>;

    /// Resets the device.
    /// 
    /// Often used to awake the module from deep sleep. See [sleep()](WaveshareInterface::sleep())
    fn reset(&mut self);

    /// Abstraction of setting the delay for simpler calls
    /// 
    /// maximum delay ~65 seconds (u16:max in ms)
    fn delay_ms(&mut self, delay: u16);
}