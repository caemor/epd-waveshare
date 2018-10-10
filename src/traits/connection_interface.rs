use hal::{
    blocking::{delay::*, spi::Write},
    digital::*,
};

use traits::Command;

/// The Connection Interface of all (?) Waveshare EPD-Devices
///
pub(crate) struct ConnectionInterface<SPI, CS, BUSY, DC, RST> {
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
}

impl<SPI, CS, BUSY, DC, RST, ERR>
    ConnectionInterface<SPI, CS, BUSY, DC, RST>
where
    SPI: Write<u8, Error = ERR>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    pub fn new(spi: SPI, cs: CS, busy: BUSY, dc: DC, rst: RST) -> Self {
        ConnectionInterface {
            spi,
            cs,
            busy,
            dc,
            rst,
        }
    }

    /// Basic function for sending [Commands](Command).
    ///
    /// Enables direct interaction with the device with the help of [data()](ConnectionInterface::data())
    /// 
    /// //TODO: make public?
    pub(crate) fn cmd<T: Command>(&mut self, command: T) -> Result<(), ERR> {
        // low for commands
        self.dc.set_low();

        // Transfer the command over spi
        self.with_cs(|epd| epd.spi.write(&[command.address()]))
    }

    /// Basic function for sending an array of u8-values of data over spi
    ///
    /// Enables direct interaction with the device with the help of [command()](EPD4in2::command())
    /// 
    /// //TODO: make public?
    pub(crate) fn data(&mut self, data: &[u8]) -> Result<(), ERR> {
        // high for data
        self.dc.set_high();

        // Transfer data (u8-array) over spi
        self.with_cs(|epd| epd.spi.write(data))
    }

    /// Basic function for sending [Commands](Command) and the data belonging to it.
    /// 
    /// //TODO: make public?
    pub(crate) fn cmd_with_data<T: Command>(&mut self, command: T, data: &[u8]) -> Result<(), ERR> {
       self.cmd(command)?;
       self.data(data)
    }

    // spi write helper/abstraction function
    fn with_cs<F>(&mut self, f: F) -> Result<(), ERR>
    where
        F: FnOnce(&mut Self) -> Result<(), ERR>,
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
    /// //TODO: use the #cfg feature to make this compile the right way for the certain types
    pub(crate) fn wait_until_idle(&mut self, is_busy_low: bool) {
        // TODO: removal of delay. TEST! 
        //self.delay_ms(1);
        //low: busy, high: idle
        while (is_busy_low && self.busy.is_low()) || (!is_busy_low && self.busy.is_high()) {
            //TODO: REMOVAL of DELAY: it's only waiting for the signal anyway and should continue work asap
            //old: shorten the time? it was 100 in the beginning
            //self.delay_ms(5);
        }
    }

    /// Resets the device.
    ///
    /// Often used to awake the module from deep sleep. See [EPD4in2::sleep()](EPD4in2::sleep())
    ///
    /// TODO: Takes at least 400ms of delay alone, can it be shortened?
    pub(crate) fn reset<DELAY: DelayMs<u8>>(&mut self, delay: &mut DELAY) {
        self.rst.set_low();
        //TODO: why 200ms? (besides being in the arduino version)
        delay.delay_ms(200);
        self.rst.set_high();
        //TODO: same as 3 lines above
        delay.delay_ms(200);
    }
}
