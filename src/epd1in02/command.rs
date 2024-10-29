//! SPI Commands for the Waveshare 1.02" E-Ink Display

use crate::traits;

/// Epd1in02 commands
///
/// Should rarely (never?) be needed directly.
///
/// For more infos about the addresses and what they are doing look into the PDFs.
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub(crate) enum Command {
    /// Set Resolution, LUT selection, gate scan direction, source shift
    /// direction, charge pump switch, soft reset.
    PanelSetting = 0x00,

    /// Selecting internal and external power
    PowerSetting = 0x01,

    /// After the Power Off command, the driver will power off following the Power Off
    /// Sequence; BUSY signal will become "0". This command will turn off charge pump,
    /// T-con, source driver, gate driver, VCOM, and temperature sensor, but register
    /// data will be kept until VDD becomes OFF. Source Driver output and Vcom will remain
    /// as previous condition, which may have 2 conditions: 0V or floating.
    PowerOff = 0x02,

    /// Setting Power OFF sequence
    PowerOffSequenceSetting = 0x03,

    /// Turning On the Power
    ///
    /// After the Power ON command, the driver will power on following the Power ON
    /// sequence. Once complete, the BUSY signal will become "1".
    PowerOn = 0x04,

    /// Setting charge pump time interval, driving strength and frequency
    ChargePumpSetting = 0x06,

    /// This command makes the chip enter the deep-sleep mode to save power.
    ///
    /// The deep sleep mode would return to stand-by by hardware reset.
    ///
    /// The only one parameter is a check code, the command would be excuted if check code = 0xA5.
    DeepSleep = 0x07,

    /// This command starts transmitting B/W data and write them into SRAM. To complete data
    /// transmission, commands Display Refresh or Data Start Transmission2 must be issued. Then the chip will start to
    /// send data/VCOM for panel.
    DataStartTransmission1 = 0x10,

    /// This command starts transmitting RED data and write them into SRAM. To complete data
    /// transmission, command Display refresh must be issued. Then the chip will start to
    /// send data/VCOM for panel.
    DataStartTransmission2 = 0x13,

    /// To stop data transmission, this command must be issued to check the `data_flag`.
    ///
    /// After this command, BUSY signal will become "0" until the display update is
    /// finished.
    DataStop = 0x11,

    /// After this command is issued, driver will refresh display (data/VCOM) according to
    /// SRAM data and LUT.
    ///
    /// After Display Refresh command, BUSY signal will become "0" until the display
    /// update is finished.
    /// The waiting interval from BUSY falling to the first FLG command must be longer than 200us.
    DisplayRefresh = 0x12,

    /// This command stores white Look-Up Table
    SetWhiteLut = 0x23,

    /// This command stores black Look-Up Table
    SetBlackLut = 0x24,

    /// This command sets XON and the options of LUT.
    LutOption = 0x2A,

    /// The command controls the PLL clock frequency.
    PllControl = 0x30,

    /// This command reads the temperature sensed by the temperature sensor.
    TemperatureSensorCalibration = 0x40,
    /// This command selects temperature option.
    TemperatureSensorSelection = 0x41,

    /// This command indicates the interval of Vcom and data output. When setting the
    /// vertical back porch, the total blanking will be kept (20 Hsync).
    VcomAndDataIntervalSetting = 0x50,
    /// This command indicates the input power condition. Host can read this flag to learn
    /// the battery condition.
    LowPowerDetection = 0x51,

    /// This command defines non-overlap period of Gate and Source.
    TconSetting = 0x60,

    /// This command defines alternative resolution and this setting is of higher priority
    /// than the RES\[1:0\] in R00H (PSR).
    TconResolution = 0x61,

    /// The command reads LUT revision and chip revision.
    Revision = 0x70,
    /// This command reads the IC status.
    GetStatus = 0x71,

    /// This command reads Cyclic redundancy check (CRC) result.
    ///
    /// The calculation only incudes image data (DTM1 & DTM2), and don't containt DTM1(0x10) & DTM2(0x13).
    /// Polynomial = x^16 + x^12 + x^5 + 1, initial value: 0xFFFF
    ///
    /// The result will be reset after this command.
    CyclicRedundancyCheck = 0x72,

    /// This command implements related VCOM sensing setting.
    AutoMeasurementVcom = 0x80,
    /// This command gets the VCOM value.
    ReadVcomValue = 0x81,
    /// This command sets `VCOM_DC` value.
    VcomDcSetting = 0x82,

    /// Sets window size for the partial update
    PartialWindow = 0x90,
    /// Sets chip into partial update mode
    PartialIn = 0x91,
    /// Quits partial update mode
    PartialOut = 0x92,

    /// After this command is issued, the chip would enter the program mode.
    /// After the programming procedure completed, a hardware reset is necessary for leaving program mode.
    ProgramMode = 0xA0,
    /// After this command is transmitted, the programming state machine would be activated.
    /// The BUSY flag would fall to 0 until the programming is completed.
    ActiveProgramming = 0xA1,
    /// The command is used for reading the content of OTP for checking the data of programming.
    /// The value of (n) is depending on the amount of programmed data, tha max address = 0xFFF.
    ReadOtp = 0xA2,

    /// This command is set for saving power during refresh period.
    /// If the output voltage of VCOM / Source is from negative to positive or
    /// from positive to negative, the power saving mechanism will be activated.
    /// The active period width is defined by the following two parameters.
    PowerSaving = 0xE3,
}

impl traits::Command for Command {
    /// Returns the address of the command
    fn address(self) -> u8 {
        self as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Command as CommandTrait;

    #[test]
    fn command_addr() {
        assert_eq!(Command::PanelSetting.address(), 0x00);
        assert_eq!(Command::DisplayRefresh.address(), 0x12);
    }
}
