use hal::{
    blocking::{
        spi::Write,
        delay::*
    },
    digital::*
};

use interface::Command;

/// The Connection Interface of all (?) Waveshare EPD-Devices
///
pub(crate) struct ConnectionInterface<SPI, CS, BUSY, DC, RST, D> {
    /// SPI
    spi: SPI,
    /// CS for SPI
    cs: CS,
    /// Low for busy, Wait until display is ready!
    busy: BUSY,
    /// Data/Command Control Pin (High for data, Low for command)
    dc: DC,
    /// Pin for Reseting
    rst: RST,
    /// The concrete Delay implementation
    delay: D,
}


impl<SPI, CS, BUSY, DataCommand, RST, Delay, ErrorSpeziale> ConnectionInterface<SPI, CS, BUSY, DataCommand, RST, Delay>
where 
    SPI: Write<u8, Error = ErrorSpeziale>,
    CS: OutputPin,
    BUSY: InputPin,
    DataCommand: OutputPin,
    RST: OutputPin,
    Delay: DelayUs<u16> + DelayMs<u16>,
{
    pub(crate) fn new(spi: SPI, cs: CS, busy: BUSY, dc: DataCommand, rst: RST, delay: Delay) -> Self {
        ConnectionInterface {spi, cs, busy, dc, rst, delay }
    }
    
    /// Basic function for sending [Commands](Command). 
    /// 
    /// Enables direct interaction with the device with the help of [send_data()](ConnectionInterface::send_data())
    /// Should rarely be needed!
    /// //TODO: make public? 
    pub(crate) fn send_command<T: Command>(&mut self, command: T) -> Result<(), ErrorSpeziale> {
        // low for commands
        self.dc.set_low(); 

        // Transfer the command over spi
        self.with_cs(|epd| {
            epd.spi.write(&[command.address()])
        })
    }

    /// Basic function for sending a single u8 of data over spi
    /// 
    /// Enables direct interaction with the device with the help of [Esend_command()](ConnectionInterface::send_command())
    /// 
    /// Should rarely be needed!
    /// //TODO: make public? 
    pub(crate) fn send_data(&mut self, val: u8) -> Result<(), ErrorSpeziale> {
        // high for data
        self.dc.set_high();

        // Transfer data (u8) over spi
        self.with_cs(|epd| {
            epd.spi.write(&[val])
        })
    }

    /// Basic function for sending a single u8 of data over spi
    /// 
    /// Enables direct interaction with the device with the help of [Esend_command()](ConnectionInterface::send_command())
    /// 
    /// Should rarely be needed!
    /// //TODO: make public? 
    pub(crate) fn send_data_x_times(&mut self, val: u8, repetitions: u16) -> Result<(), ErrorSpeziale> {
        // high for data
        self.dc.set_high();

        // Transfer data (u8) over spi
        self.with_cs(|epd| {
            for _ in 0..repetitions {
                epd.spi.write(&[val])?;
            }
            Ok(())           
        })
    }

    /// Basic function for sending an array of u8-values of data over spi
    /// 
    /// Enables direct interaction with the device with the help of [send_command()](EPD4in2::send_command())
    /// 
    /// Should rarely be needed!
    /// //TODO: make public? 
    pub(crate) fn send_multiple_data(&mut self, data: &[u8]) -> Result<(), ErrorSpeziale> {
        // high for data
        self.dc.set_high();

        // Transfer data (u8-array) over spi
        self.with_cs(|epd| {
            epd.spi.write(data)
        })
    }

    // spi write helper/abstraction function
    pub(crate) fn with_cs<F>(&mut self, f: F) -> Result<(), ErrorSpeziale>
    where 
        F: FnOnce(&mut Self) -> Result<(), ErrorSpeziale>,
    {
        // activate spi with cs low
        self.cs.set_low();
        // transfer spi data
        let result = f(self);
        // deativate spi with cs high
        self.cs.set_high();
        // return result
        result
    }


    /// Waits until device isn't busy anymore (busy == HIGH)
    /// 
    /// This is normally handled by the more complicated commands themselves,
    /// but in the case you send data and commands directly you might need to check
    /// if the device is still busy
    /// 
    /// is_busy_low
    /// 
    ///  - TRUE for epd4in2, epd2in13, epd2in7, epd5in83, epd7in5
    ///  - FALSE for epd2in9, epd1in54 (for all Display Type A ones?)
    /// 
    /// Most likely there was a mistake with the 2in9 busy connection
    pub(crate) fn wait_until_idle(&mut self, is_busy_low: bool) {
        self.delay_ms(1);
        //low: busy, high: idle
        while (is_busy_low && self.busy.is_low()) || (!is_busy_low && self.busy.is_high()) {
            //TODO: shorten the time? it was 100 in the beginning
            self.delay_ms(5);
        }
    }


    /// Abstraction of setting the delay for simpler calls
    /// 
    /// maximum delay ~65 seconds (u16:max in ms)
    pub(crate) fn delay_ms(&mut self, delay: u16) {
        self.delay.delay_ms(delay);
    }

    /// Resets the device.
    /// 
    /// Often used to awake the module from deep sleep. See [EPD4in2::sleep()](EPD4in2::sleep())
    /// 
    /// TODO: Takes at least 400ms of delay alone, can it be shortened?
    pub(crate) fn reset(&mut self) {
        self.rst.set_low();

        //TODO: why 200ms? (besides being in the waveshare code)
        self.delay_ms(200);

        self.rst.set_high();

        //TODO: same as 3 lines above
        self.delay_ms(200);
    }

}