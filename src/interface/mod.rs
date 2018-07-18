use hal::{
    blocking::{
        spi::Write,
        delay::*
    },
    spi::{Mode, Phase, Polarity},
    digital::*
};

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
    fn address(&self) -> u8;
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
    fn get_height(&self) -> u32;
    fn new(
        spi: SPI, 
        cs: CS, 
        busy: BUSY, 
        dc: DC, 
        rst: RST, 
        delay: D
    ) -> Result<Self, E>
        where Self: Sized;
    fn init(&mut self) -> Result<(), E>;
    fn sleep(&mut self) -> Result<(), E>;
    fn reset(&mut self);
    fn wait_until_idle(&mut self);
    fn delay_ms(&mut self, delay: u32);

    /*
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


pub trait TestInterface
{
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
    

}

struct testStruct {
    width: u32,
    height: u32,
}

impl TestInterface for testStruct {
    fn get_width(&self) -> u32 {
        self.width
    }

    fn get_height(&self) -> u32 {
        self.height
    }


}

