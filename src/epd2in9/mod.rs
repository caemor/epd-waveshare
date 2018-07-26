//! A simple Driver for the Waveshare 2.9" E-Ink Display via SPI
//! 
//!
//! # Examples from the 4.2" Display. It should work the same for the 2.9" one.
//!
//! ```ignore
//! let mut epd4in2 = EPD4in2::new(spi, cs, busy, dc, rst, delay).unwrap();
//!
//! let mut buffer =  [0u8, epd4in2.get_width() / 8 * epd4in2.get_height()];
//! 
//! // draw something into the buffer
//! 
//! epd4in2.display_and_transfer_buffer(buffer, None);
//! 
//! // wait and look at the image
//! 
//! epd4in2.clear_frame(None);
//! 
//! epd4in2.sleep();
//! ```


use hal::{
    blocking::{
        spi::Write,
        delay::*
    },
    digital::*
};

mod constants;
use self::constants::*;

use drawing::color::Color;

pub mod command;
pub use self::command::Command;

use interface::*;

use interface::connection_interface::ConnectionInterface;



/// EPD2in9 driver
///
pub struct EPD2in9<SPI, CS, BUSY, DC, RST, D> {
    /// SPI
    interface: ConnectionInterface<SPI, CS, BUSY, DC, RST, D>,
    /// Width
    width: u16,
    /// Height
    height: u16,  
    /// Color
    background_color: Color, 
}

impl<SPI, CS, BUSY, DC, RST, D, E> EPD2in9<SPI, CS, BUSY, DC, RST, D>
where 
    SPI: Write<u8, Error = E>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    D: DelayUs<u16> + DelayMs<u16>
{

}


impl<SPI, CS, BUSY, DC, RST, D, E> WaveshareInterface<SPI, CS, BUSY, DC, RST, D, E> for EPD2in9<SPI, CS, BUSY, DC, RST, D>
where 
    SPI: Write<u8, Error = E>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    D: DelayUs<u16> + DelayMs<u16>,
{ 
    
    fn get_width(&self) -> u16 {
        self.width
    }

    fn get_height(&self) -> u16 {
        self.height
    }


    fn new(
        spi: SPI, 
        cs: CS, 
        busy: BUSY, 
        dc: DC, 
        rst: RST, 
        delay: D
    ) -> Result<Self, E> {                
        let width = WIDTH as u16;
        let height = HEIGHT as u16;

        let interface = ConnectionInterface::new(spi, cs, busy, dc, rst, delay);

        let background_color = Color::White;

        let mut epd = EPD2in9 {interface, width, height, background_color};


        epd.init()?;

        Ok(epd)
    }



    fn init(&mut self) -> Result<(), E> {
        
        
        self.reset();

        // 3 Databytes:
        // A[7:0]
        // 0.. A[8]
        // 0.. B[2:0]
        // Default Values: A = Height of Screen (0x127), B = 0x00 (GD, SM and TB=0?)
        self.interface.send_command(Command::DRIVER_OUTPUT_CONTROL)?;
        self.interface.send_data(HEIGHT as u8)?;
        self.interface.send_data((HEIGHT >> 8) as u8)?;
        self.interface.send_data(0x00)?;

        // 3 Databytes: (and default values from datasheet and arduino)
        // 1 .. A[6:0]  = 0xCF | 0xD7
        // 1 .. B[6:0]  = 0xCE | 0xD6
        // 1 .. C[6:0]  = 0x8D | 0x9D
        //TODO: test
        self.interface.send_command(Command::BOOSTER_SOFT_START_CONTROL)?;
        self.interface.send_data(0xD7)?;
        self.interface.send_data(0xD6)?;
        self.interface.send_data(0x9D)?;

        // One Databyte with value 0xA8 for 7V VCOM
        self.interface.send_command(Command::WRITE_VCOM_REGISTER)?;
        self.interface.send_data(0xA8)?;

        // One Databyte with default value 0x1A for 4 dummy lines per gate
        self.interface.send_command(Command::SET_DUMMY_LINE_PERIOD)?;
        self.interface.send_data(0x1A)?;

        // One Databyte with default value 0x08 for 2us per line
        self.interface.send_command(Command::SET_GATE_LINE_WIDTH)?;
        self.interface.send_data(0x08)?;

        // One Databyte with default value 0x03
        //  -> address: x increment, y increment, address counter is updated in x direction
        self.interface.send_command(Command::DATA_ENTRY_MODE_SETTING)?;
        self.interface.send_data(0x03)?;

        self.set_lut()
    }

    fn sleep(&mut self) -> Result<(), E> {

        self.interface.send_command(Command::DEEP_SLEEP_MODE)?;
        // 0x00 for Normal mode (Power on Reset), 0x01 for Deep Sleep Mode
        //TODO: is 0x00 needed here?
        self.interface.send_data(0x00)?;

        self.wait_until_idle();
        Ok(())
    }


    fn reset(&mut self) {
        self.interface.reset()
    }

    fn delay_ms(&mut self, delay: u16) {
        self.interface.delay_ms(delay)
    }

    

    fn update_frame(&mut self, buffer: &[u8]) -> Result<(), E>{
        self.use_full_frame()?;

        self.interface.send_command(Command::WRITE_RAM)?;
        self.interface.send_multiple_data(buffer)
    }

    //TODO: update description: last 3 bits will be ignored for width and x_pos
    fn update_partial_frame(&mut self, buffer: &[u8], x: u16, y: u16, width: u16, height: u16) -> Result<(), E>{
        self.set_ram_area(x, y, x + width, y + height)?;
        self.set_ram_counter(x, y)?;

        self.interface.send_command(Command::WRITE_RAM)?;
        self.interface.send_multiple_data(buffer)
    }

    
    fn display_frame(&mut self) -> Result<(), E>{
        // enable clock signal, enable cp, display pattern -> 0xC4 (tested with the arduino version)
        //TODO: test control_1 or control_2 with default value 0xFF (from the datasheet)
        self.interface.send_command(Command::DISPLAY_UPDATE_CONTROL_2)?;
        self.interface.send_data(0xC4)?;

        self.interface.send_command(Command::MASTER_ACTIVATION)?;
        // MASTER Activation should not be interupted to avoid currption of panel images
        // therefore a terminate command is send
        self.interface.send_command(Command::TERMINATE_COMMANDS_AND_FRAME_WRITE)
    }

    
    fn update_and_display_frame(&mut self, buffer: &[u8]) -> Result<(), E>{
        self.update_frame(buffer)?;
        self.display_frame()
    }

    
    fn clear_frame(&mut self) -> Result<(), E>{
        self.use_full_frame()?;

        // clear the ram with the background color
        let color = self.background_color.get_byte_value();

        self.interface.send_command(Command::WRITE_RAM)?;        
        self.interface.send_data_x_times(color, WIDTH / 8 * HEIGHT)
    }

    /// Sets the backgroundcolor for various commands like [WaveshareInterface::clear_frame()](clear_frame())
    fn set_background_color(&mut self, background_color: Color){
        self.background_color = background_color;
    }

}

impl<SPI, CS, BUSY, DC, RST, D, E> EPD2in9<SPI, CS, BUSY, DC, RST, D>
where 
    SPI: Write<u8, Error = E>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    D: DelayUs<u16> + DelayMs<u16>,
{
    fn wait_until_idle(&mut self) {
        self.interface.wait_until_idle(false);
    }
    
    pub(crate) fn use_full_frame(&mut self) -> Result<(), E> {
        // choose full frame/ram
        self.set_ram_area(0, 0, WIDTH - 1, HEIGHT - 1)?;

        // start from the beginning
        self.set_ram_counter(0,0)
    }
    
    pub(crate) fn set_ram_area(&mut self, start_x: u16, start_y: u16, end_x: u16, end_y: u16) -> Result<(), E> {
        assert!(start_x < end_x);
        assert!(start_y < end_y);

        // x is positioned in bytes, so the last 3 bits which show the position inside a byte in the ram
        // aren't relevant        
        self.interface.send_command(Command::SET_RAM_X_ADDRESS_START_END_POSITION)?;
        self.interface.send_data((start_x >> 3) as u8)?;
        self.interface.send_data((end_x >> 3) as u8)?;

        // 2 Databytes: A[7:0] & 0..A[8] for each - start and end
        self.interface.send_command(Command::SET_RAM_Y_ADDRESS_START_END_POSITION)?;
        self.interface.send_data(start_y as u8)?;
        self.interface.send_data((start_y >> 8) as u8)?;
        self.interface.send_data(end_y as u8)?;
        self.interface.send_data((end_y >> 8) as u8)
    }

    pub(crate) fn set_ram_counter(&mut self, x: u16, y: u16) -> Result<(), E> {
        // x is positioned in bytes, so the last 3 bits which show the position inside a byte in the ram
        // aren't relevant
        self.interface.send_command(Command::SET_RAM_X_ADDRESS_COUNTER)?;
        self.interface.send_data((x >> 3) as u8)?;

        // 2 Databytes: A[7:0] & 0..A[8]
        self.interface.send_command(Command::SET_RAM_Y_ADDRESS_COUNTER)?;
        self.interface.send_data(y as u8)?;
        self.interface.send_data((y >> 8) as u8)?;

        self.wait_until_idle();
        Ok(())
    }

    /// Uses the slower full update 
    pub fn set_lut(&mut self) -> Result<(), E> {
        self.set_lut_helper(&LUT_FULL_UPDATE)
    }

    /// Uses the quick partial refresh 
    pub fn set_lut_quick(&mut self) -> Result<(), E> {
        self.set_lut_helper(&LUT_PARTIAL_UPDATE)
    }

    //TODO: assert length for LUT is exactly 30
    fn set_lut_manual(&mut self, buffer: &[u8]) -> Result<(), E> {
        self.set_lut_helper(buffer)
    }


    fn set_lut_helper(&mut self, buffer: &[u8]) -> Result<(), E> {
        assert!(buffer.len() == 30);
        self.interface.send_command(Command::WRITE_LUT_REGISTER)?;
        self.interface.send_multiple_data(buffer)
    }

}