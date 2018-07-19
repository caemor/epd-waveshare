//! A simple Driver for the Waveshare 4.2" E-Ink Display via SPI
//! 
//! The other Waveshare E-Ink Displays should be added later on
//! 
//! Build with the help of documentation/code from [Waveshare](https://www.waveshare.com/wiki/4.2inch_e-Paper_Module), 
//! [Ben Krasnows partial Refresh tips](https://benkrasnow.blogspot.de/2017/10/fast-partial-refresh-on-42-e-paper.html) and
//! the driver documents in the `pdfs`-folder as orientation.
//!
//! This driver was built using [`embedded-hal`] traits.
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal/~0.1
//! 
//! # Requirements
//! 
//! ### SPI
//! 
//! - MISO is not connected/available
//! - SPI_MODE_0 is used (CPHL = 0, CPOL = 0)
//! - 8 bits per word, MSB first
//! - Max. Speed tested was 8Mhz but more should be possible
//! 
//! ### Other....
//! 
//! - Buffersize: Wherever a buffer is used it always needs to be of the size: `width / 8 * length`, 
//!   where width and length being either the full e-ink size or the partial update window size
//!
//! # Examples
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
//! 
//! 
//!
//! BE CAREFUL! The Partial Drawing can "destroy" your display.
//! It needs more testing first.


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
    width: u32,
    /// Height
    height: u32,  
    /// Color
    color: Color, 
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
    
    fn get_width(&self) -> u32 {
       self.width
    }

    fn get_height(&self) -> u32 {
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
        let width = WIDTH as u32;
        let height = HEIGHT as u32;

        let interface = ConnectionInterface::new(spi, cs, busy, dc, rst, delay);

        let color = Color::White;

        let mut epd = EPD2in9 {interface, width, height, color};


        epd.init()?;

        Ok(epd)
    }



    fn init(&mut self) -> Result<(), E> {
        self.reset();




        unimplemented!()
    }

    fn sleep(&mut self) -> Result<(), E> {

        self.interface.send_command(Command::DEEP_SLEEP_MODE)?;
        // 0x00 for Normal mode (Power on Reset), 0x01 for Deep Sleep Mode
        //TODO: is 0x00 needed here?
        self.interface.send_data(0x00)?;

        self.interface.wait_until_idle(false);
        Ok(())
    }


    fn reset(&mut self) {
        self.interface.reset()
    }

    fn delay_ms(&mut self, delay: u16) {
        self.interface.delay_ms(delay)
    }

    

    fn update_frame(&mut self, buffer: &[u8]) -> Result<(), E>{
        unimplemented!()
    }

    fn update_partial_frame(&mut self, buffer: &[u8], x: u16, y: u16, width: u16, height: u16) -> Result<(), E>{
        unimplemented!()
    }

    
    fn display_frame(&mut self) -> Result<(), E>{
        unimplemented!()
    }

    // TODO: add this abstraction function
    // fn update_and_display_frame(&mut self, buffer: &[u8]) -> Result<(), E>;
    fn update_and_display_frame(&mut self, buffer: &[u8]) -> Result<(), E>{
        unimplemented!()
    }

    
    fn clear_frame(&mut self) -> Result<(), E>{
        unimplemented!()
    }

    /// Sets the backgroundcolor for various commands like [WaveshareInterface::clear_frame()](clear_frame())
    fn set_background_color(&mut self, color: Color){
        self.color = color;
    }

}


/*


impl<SPI, CS, BUSY, DC, RST, D, E> EPD2in9<SPI, CS, BUSY, DC, RST, D>
where 
    SPI: Write<u8, Error = E>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    D: DelayUs<u16> + DelayMs<u16>,
{
    /// Get the width of the display
    pub fn get_width(&self) -> u16 {
        self.width
    }

    /// Get the height of the display
    pub fn get_height(&self) -> u16 {
        self.height
    }
    
    
    /// Creates a new driver from a SPI peripheral, CS Pin, Busy InputPin, DC
    /// 
    /// This already initialises the device. That means [EPD4in2::init()](EPD4in2::init()) isn't needed directly afterwards
    /// 
    /// # Example
    /// 
    /// ```ignore
    /// //buffer = some image data;
    /// 
    /// let mut epd4in2 = EPD4in2::new(spi, cs, busy, dc, rst, delay);
    /// 
    /// epd4in2.display_and_transfer_frame(buffer, None);
    /// 
    /// epd4in2.sleep();
    /// ```
    pub fn new(spi: SPI, cs: CS, busy: BUSY, dc: DC, rst: RST, delay: D) -> Result<Self, E> {
        //TODO: width und height anpassbar machen?
        let width = WIDTH as u16;
        let height = HEIGHT as u16;

        let mut epd4in2 = EPD4in2 {spi, cs, busy, dc, rst, delay, width, height };

        epd4in2.init()?;

        Ok(epd4in2)
    }



    /// This initialises the EPD and powers it up
    /// 
    /// This function is already called from [EPD4in2::new()](EPD4in2::new())
    /// 
    /// This function calls [EPD4in2::reset()](EPD4in2::reset()),
    /// so you don't need to call reset your self when trying to wake your device up
    /// after setting it to sleep.
    pub fn init(&mut self) -> Result<(), E> {
        // reset the device
        self.reset();

        // set the power settings
        self.send_command(Command::POWER_SETTING)?;
        self.send_data(0x03)?; //VDS_EN, VDG_EN
        self.send_data(0x00)?; //VCOM_HV, VGHL_LV[1], VGHL_LV[0]
        self.send_data(0x2b)?; //VDH
        self.send_data(0x2b)?; //VDL
        self.send_data(0xff)?; //VDHR

        // start the booster
        self.send_command(Command::BOOSTER_SOFT_START)?;
        for _ in 0..3 {
            self.send_data(0x17)?; //07 0f 17 1f 27 2F 37 2f 
        }

        // power on
        self.send_command(Command::POWER_ON)?;
        self.wait_until_idle();

        // set the panel settings
        self.send_command(Command::PANEL_SETTING)?;
        // 0x0F Red Mode, LUT from OTP
        // 0x1F B/W Mode, LUT from OTP
        // 0x2F Red Mode, LUT set by registers
        // 0x3F B/W Mode, LUT set by registers
        self.send_data(0x3F)?;

        // the values used by waveshare before for the panel settings
        // instead of our one liner:
        // SendData(0xbf);    // KW-BF   KWR-AF  BWROTP 0f
        // SendData(0x0b);

        // Set Frequency, 200 Hz didn't work on my board
        // 150Hz and 171Hz wasn't tested yet
        // TODO: Test these other frequencies
        // 3A 100HZ   29 150Hz 39 200HZ  31 171HZ DEFAULT: 3c 50Hz 
        self.send_command(Command::PLL_CONTROL)?;
        self.send_data(0x3A)?;

        Ok(())
    }





    
    /// Transmit partial data to the SRAM of the EPD,
    /// the final parameter dtm chooses between the 2
    /// internal buffers 
    /// 
    /// Normally it should be dtm2, so use false
    /// 
    /// BUFFER needs to be of size: w / 8 * l !
    pub fn set_partial_window(&mut self, buffer: &[u8], x: u16, y: u16, w: u16, l: u16, is_dtm1: bool) -> Result<(), E> {
        if buffer.len() as u16 != w / 8 * l {
            //TODO: panic!! or sth like that
            //return Err("Wrong buffersize");
        }

        self.send_command(Command::PARTIAL_IN)?;
        self.send_command(Command::PARTIAL_WINDOW)?;
        self.send_data((x >> 8) as u8)?;
        let tmp = x & 0xf8;
        self.send_data(tmp as u8)?; // x should be the multiple of 8, the last 3 bit will always be ignored
        let tmp = tmp + w - 1;
        self.send_data((tmp >> 8) as u8)?;
        self.send_data((tmp | 0x07) as u8)?;

        self.send_data((y >> 8) as u8)?;
        self.send_data(y as u8)?;

        self.send_data(((y + l - 1) >> 8) as u8)?;
        self.send_data((y + l - 1) as u8)?;

        self.send_data(0x01)?; // Gates scan both inside and outside of the partial window. (default) 

        if is_dtm1 {
            self.send_command(Command::DATA_START_TRANSMISSION_1)?
        } else {
            self.send_command(Command::DATA_START_TRANSMISSION_2)?
        }

        self.send_multiple_data(buffer)?;

        self.send_command(Command::PARTIAL_OUT)
    }

    

    // void DisplayFrame(const unsigned char* frame_buffer);
    /// Display the frame data from SRAM
    /// Uses the SLOW!! full update/refresh
    /// Default color: 0xff
    /// 
    pub fn display_and_transfer_frame(&mut self, buffer: &[u8], color: Option<u8>) -> Result<(), E>{
        let color = color.unwrap_or(0xff);

        self.send_resolution()?;

        self.send_command(Command::VCM_DC_SETTING)?;
        self.send_data(0x12)?;

        self.send_command(Command::VCOM_AND_DATA_INTERVAL_SETTING)?;
        //TODO: this was a send_command instead of a send_data. check if it's alright and doing what it should do (setting the default values)
        //oldTODO is this really a command here or shouldn't that be data?
        //self.send_command_u8(0x97)?; //VBDF 17|D7 VBDW 97  VBDB 57  VBDF F7  VBDW 77  VBDB 37  VBDR B7
        self.send_data(0x97)?;


        self.send_command(Command::DATA_START_TRANSMISSION_1)?;
        for _ in 0..(buffer.len()) {
            self.send_data(color)?;
        }
        self.delay_ms(2);

        self.send_command(Command::DATA_START_TRANSMISSION_2)?;
        //self.send_multiple_data(buffer)?;
        for &elem in buffer.iter() {
            self.send_data(elem)?;
        }
        self.delay_ms(2);

        self.set_lut()?;

        self.send_command(Command::DISPLAY_REFRESH)?;
        //TODO: adapt time, is this long delay really needed?
        self.delay_ms(10);
        self.wait_until_idle();

        Ok(())
    }

    fn send_resolution(&mut self) -> Result<(), E> {
        let w = self.get_width();
        let h = self.get_height();

        self.send_command(Command::RESOLUTION_SETTING)?;
        self.send_data((w >> 8) as u8)?;
        self.send_data(w as u8)?;
        self.send_data((h >> 8) as u8)?;
        self.send_data(h as u8)
    }

    /// Displays the frame data from SRAM
    pub fn display_frame(&mut self) -> Result<(), E> {
        self.set_lut()?;
        self.send_command(Command::DISPLAY_REFRESH)?;

        self.delay_ms(100);
        self.wait_until_idle();
        Ok(())
    }

    /// Same as display_frame(), but with nearly no delay
    /// and uses the fast/partial refresh LUT
    /// needs more testing!!!
    /// maybe delay can be fully removed as wait_until_idle should do
    /// the necessary stuff
    /// TODO: check delay!!!
    /// Displays the frame data from SRAM
    pub fn display_frame_quick(&mut self) -> Result<(), E> {
        self.set_lut_quick()?;
        self.send_command(Command::DISPLAY_REFRESH)?;

        self.delay_ms(1);
        self.wait_until_idle();
        Ok(())
    }

    
    /// Clears the frame from the buffer
    /// 
    /// Set a reset_color if you want a different from the default 0xff
    /// 
    /// TODO: should that option be removed? E.g. the struct contains an additional default background value
    /// which is settable?
    pub fn clear_frame(&mut self, reset_color: Option<Color>) -> Result<(), E> {
        let reset_color: Color = reset_color.unwrap_or(Color::White);

        self.send_resolution()?;

        let size = self.width / 8 * self.height;

        self.send_command(Command::DATA_START_TRANSMISSION_1)?;
        self.delay_ms(2);
        for _ in 0..size {
            self.send_data(reset_color.get_byte_value())?;
        }

        self.delay_ms(2);

        self.send_command(Command::DATA_START_TRANSMISSION_2)?;
        self.delay_ms(2);
        for _ in 0..size {
            self.send_data(reset_color.get_byte_value())?;
        }
        Ok(())
    }

    /// Let the device enter deep-sleep mode to save power. 
    /// 
    /// The deep sleep mode returns to standby with a hardware reset. 
    /// But you can also use [EPD4in2::reset()](EPD4in2::reset()) to awaken.
    /// But as you need to power it up once more anyway you can also just directly use [EPD4in2::init()](EPD4in2::init()) for resetting
    /// and initialising which already contains the reset
    pub fn sleep(&mut self) -> Result<(), E> {
        self.send_command(Command::VCOM_AND_DATA_INTERVAL_SETTING)?;
        self.send_data(0x17)?; //border floating
        self.send_command(Command::VCM_DC_SETTING)?; // VCOM to 0V
        self.send_command(Command::PANEL_SETTING)?;
        self.delay_ms(100);

        self.send_command(Command::POWER_SETTING)?; //VG&VS to 0V fast
        for _ in 0..4 {
            self.send_data(0x00)?;
        }
        self.delay_ms(100);

        self.send_command(Command::POWER_OFF)?;
        self.wait_until_idle();
        self.send_command(Command::DEEP_SLEEP)?;
        self.send_data(0xA5)?;

        Ok(())
    }





    /// Fill the look-up table for the EPD
    //TODO: make public? 
    fn set_lut(&mut self) -> Result<(), E> {
        self.set_lut_helper(
            &LUT_VCOM0,
            &LUT_WW,
            &LUT_BW,
            &LUT_WB,
            &LUT_BB)
    }

    /// Fill the look-up table for a quick display (partial refresh)
    /// 
    /// Is automatically done by [EPD4in2::display_frame_quick()](EPD4in2::display_frame_quick()) 
    /// //TODO: make public? 
    fn set_lut_quick(&mut self) -> Result<(), E> {
        self.set_lut_helper(
            &LUT_VCOM0_QUICK,
            &LUT_WW_QUICK,
            &LUT_BW_QUICK,
            &LUT_WB_QUICK,
            &LUT_BB_QUICK)
    }

    fn set_lut_helper(&mut self, 
            lut_vcom: &[u8],
            lut_ww: &[u8],
            lut_bw: &[u8],
            lut_wb: &[u8],
            lut_bb: &[u8]) -> Result<(), E> 
    {
        //vcom
        self.send_command(Command::LUT_FOR_VCOM)?;
        self.send_multiple_data(lut_vcom)?;

        //ww --
        self.send_command(Command::LUT_WHITE_TO_WHITE)?;
        self.send_multiple_data(lut_ww)?;

        //bw r
        self.send_command(Command::LUT_BLACK_TO_WHITE)?;
        self.send_multiple_data(lut_bw)?;

        //wb w
        self.send_command(Command::LUT_WHITE_TO_BLACK)?;
        self.send_multiple_data(lut_wb)?;

        //bb b
        self.send_command(Command::LUT_BLACK_TO_BLACK)?;
        self.send_multiple_data(lut_bb)?;

        Ok(())
    }


}



*/












