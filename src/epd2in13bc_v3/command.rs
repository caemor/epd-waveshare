
use crate::traits;

/// Epd7in5b_v3 commands
///
/// Should rarely (never?) be needed directly.
///
/// For more infos about the addresses and what they are doing look into the PDFs.
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub(crate) enum Command {
    /// Set Resolution, LUT selection, BWR pixels, gate scan direction, source shift
    /// direction, booster switch, soft reset.
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

    /// Starting data transmission
    BoosterSoftStart = 0x06,

    /// This command makes the chip enter the deep-sleep mode to save power.
    ///
    /// The deep sleep mode would return to stand-by by hardware reset.
    ///
    /// The only one parameter is a check code, the command would be excuted if check code = 0xA5.
    DeepSleep = 0x07,

    /// This command starts transmitting data and write them into SRAM. To complete data
    /// transmission, command DSP (Data Stop) must be issued. Then the chip will start to
    /// send data/VCOM for panel.
    ///
    /// BLACK/WHITE or OLD_DATA
    DataStartTransmissionBlackWhite = 0x10,

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
    DisplayRefresh = 0x12,

    /// RED or NEW_DATA
    DataStartTransmissionChromatic = 0x13,

    /// Dual SPI - what for?
    DualSpi = 0x15,

    /// This command builds the VCOM Look-Up Table (LUTC).
    LutForVcom = 0x20,
    /// This command builds the White to White Look-Up Table (LUTWW).
    LutWhiteToWhite = 0x21,
    /// This command builds the Black to White Look-Up Table (LUTKW).
    LutBlackToWhite = 0x22,
    /// This command builds the White to Black Look-Up Table (LUTWK).
    LutWhiteToBlack = 0x23,
    /// This command builds the Black to Black Look-Up Table (LUTKK).
    LutBlackToBlack = 0x24,
    /// This command builds the Border? Look-Up Table (LUTR0).
    LutBorder = 0x25,

    /// This command builds the XON Look-Up Table (LUTXON).
    LutXon = 0x2A,

    /// The command controls the PLL clock frequency.
    PllControl = 0x30,

    /// This command reads the temperature sensed by the temperature sensor.
    TemperatureSensor = 0x40,

    /// This command indicates the interval of Vcom and data output. When setting the
    /// vertical back porch, the total blanking will be kept (20 Hsync).
    VcomAndDataIntervalSetting = 0x50,

    /// This command defines non-overlap period of Gate and Source.
    TconSetting = 0x60,
    /// This command defines alternative resolution and this setting is of higher priority
    /// than the RES\[1:0\] in R00H (PSR).
    TconResolution = 0x61,
    //TODO /// This command defines MCU host direct access external memory mode.
    GateSourceStart = 0x65,

    /// The LUT_REV / Chip Revision is read from OTP address = 25001 and 25000.
    Revision = 0x70,

    /// This command sets `VCOM_DC` value.
    VcmDcSetting = 0x82,

    // TODO
    ProgramMode = 0xA0,
    //
    // TODO
    ActiveProgram = 0xA1,

    // TODO
    ReadOtpData = 0xA2,
    // /// This is in all the Waveshare controllers for Epd7in5, but it's not documented
    // /// anywhere in the datasheet `¯\_(ツ)_/¯`
    // FlashMode = 0xE5,
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
