use crate::traits::Command;
use core::marker::PhantomData;
use embedded_hal::{
    blocking::{delay::*, spi::Write},
    digital::v2::*,
};

/// The Connection Interface of all (?) Waveshare EPD-Devices
///
pub(crate) struct DisplayInterface<SPI, CS, BUSY, DC, RST> {
    /// SPI
    _spi: PhantomData<SPI>,
    /// CS for SPI
    cs: CS,
    /// Low for busy, Wait until display is ready!
    busy: BUSY,
    /// Data/Command Control Pin (High for data, Low for command)
    dc: DC,
    /// Pin for Reseting
    rst: RST,
}

impl<SPI, CS, BUSY, DC, RST> DisplayInterface<SPI, CS, BUSY, DC, RST>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    pub fn new(cs: CS, busy: BUSY, dc: DC, rst: RST) -> Self {
        DisplayInterface {
            _spi: PhantomData::default(),
            cs,
            busy,
            dc,
            rst,
        }
    }

    /// Basic function for sending [Commands](Command).
    ///
    /// Enables direct interaction with the device with the help of [data()](DisplayInterface::data())
    pub(crate) fn cmd<T: Command>(&mut self, spi: &mut SPI, command: T) -> Result<(), SPI::Error> {
        // low for commands
        let _ = self.dc.set_low();

        // Transfer the command over spi
        self.write(spi, &[command.address()])
    }

    /// Basic function for sending an array of u8-values of data over spi
    ///
    /// Enables direct interaction with the device with the help of [command()](EPD4in2::command())
    pub(crate) fn data(&mut self, spi: &mut SPI, data: &[u8]) -> Result<(), SPI::Error> {
        // high for data
        let _ = self.dc.set_high();

        // Transfer data (u8-array) over spi
        self.write(spi, data)
    }

    /// Basic function for sending [Commands](Command) and the data belonging to it.
    ///
    /// TODO: directly use ::write? cs wouldn't needed to be changed twice than
    pub(crate) fn cmd_with_data<T: Command>(
        &mut self,
        spi: &mut SPI,
        command: T,
        data: &[u8],
    ) -> Result<(), SPI::Error> {
        self.cmd(spi, command)?;
        self.data(spi, data)
    }

    /// Basic function for sending the same byte of data (one u8) multiple times over spi
    ///
    /// Enables direct interaction with the device with the help of [command()](ConnectionInterface::command())
    pub(crate) fn data_x_times(
        &mut self,
        spi: &mut SPI,
        val: u8,
        repetitions: u32,
    ) -> Result<(), SPI::Error> {
        // high for data
        let _ = self.dc.set_high();
        // Transfer data (u8) over spi
        for _ in 0..repetitions {
            self.write(spi, &[val])?;
        }
        Ok(())
    }

    // spi write helper/abstraction function
    fn write(&mut self, spi: &mut SPI, data: &[u8]) -> Result<(), SPI::Error> {
        // activate spi with cs low
        let _ = self.cs.set_low();

        // transfer spi data
        // Be careful!! Linux has a default limit of 4096 bytes per spi transfer
        // see https://raspberrypi.stackexchange.com/questions/65595/spi-transfer-fails-with-buffer-size-greater-than-4096
        if cfg!(target_os = "linux") {
            for data_chunk in data.chunks(4096) {
                spi.write(data_chunk)?;
            }
        } else {
            spi.write(data)?;
        }

        // deativate spi with cs high
        let _ = self.cs.set_high();

        Ok(())
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
        //tested: worked without the delay for all tested devices
        //self.delay_ms(1);

        while self.is_busy(is_busy_low) {
            //tested: REMOVAL of DELAY: it's only waiting for the signal anyway and should continue work asap
            //old: shorten the time? it was 100 in the beginning
            //self.delay_ms(5);
        }
    }

    /// Checks if device is still busy
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
    pub(crate) fn is_busy(&self, is_busy_low: bool) -> bool {
        (is_busy_low && self.busy.is_low().unwrap_or(false))
            || (!is_busy_low && self.busy.is_high().unwrap_or(false))
    }

    /// Resets the device.
    ///
    /// Often used to awake the module from deep sleep. See [EPD4in2::sleep()](EPD4in2::sleep())
    ///
    /// TODO: Takes at least 400ms of delay alone, can it be shortened?
    pub(crate) fn reset<DELAY: DelayMs<u8>>(&mut self, delay: &mut DELAY) {
        let _ = self.rst.set_low();
        //TODO: why 200ms? (besides being in the arduino version)
        delay.delay_ms(200);
        let _ = self.rst.set_high();
        //TODO: same as 3 lines above
        delay.delay_ms(200);
    }
}
