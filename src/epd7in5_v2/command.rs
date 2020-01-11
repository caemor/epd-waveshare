//! SPI Commands for the Waveshare 7.5" E-Ink Display

use crate::traits;

/// EPD7in5 commands
///
/// Should rarely (never?) be needed directly.
///
/// For more infos about the addresses and what they are doing look into the PDFs.
#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub(crate) enum Command {
    /// Set Resolution, LUT selection, BWR pixels, gate scan direction, source shift
    /// direction, booster switch, soft reset.
    PANEL_SETTING = 0x00,

    /// Selecting internal and external power
    POWER_SETTING = 0x01,

    /// After the Power Off command, the driver will power off following the Power Off
    /// Sequence; BUSY signal will become "0". This command will turn off charge pump,
    /// T-con, source driver, gate driver, VCOM, and temperature sensor, but register
    /// data will be kept until VDD becomes OFF. Source Driver output and Vcom will remain
    /// as previous condition, which may have 2 conditions: 0V or floating.
    POWER_OFF = 0x02,

    /// Setting Power OFF sequence
    POWER_OFF_SEQUENCE_SETTING = 0x03,

    /// Turning On the Power
    ///
    /// After the Power ON command, the driver will power on following the Power ON
    /// sequence. Once complete, the BUSY signal will become "1".
    POWER_ON = 0x04,

    /// Starting data transmission
    BOOSTER_SOFT_START = 0x06,

    /// This command makes the chip enter the deep-sleep mode to save power.
    ///
    /// The deep sleep mode would return to stand-by by hardware reset.
    ///
    /// The only one parameter is a check code, the command would be excuted if check code = 0xA5.
    DEEP_SLEEP = 0x07,

    /// This command starts transmitting data and write them into SRAM. To complete data
    /// transmission, command DSP (Data Stop) must be issued. Then the chip will start to
    /// send data/VCOM for panel.
    ///
    /// BLACK/WHITE or OLD_DATA
    DATA_START_TRANSMISSION_1 = 0x10,

    /// To stop data transmission, this command must be issued to check the `data_flag`.
    ///
    /// After this command, BUSY signal will become "0" until the display update is
    /// finished.
    DATA_STOP = 0x11,

    /// After this command is issued, driver will refresh display (data/VCOM) according to
    /// SRAM data and LUT.
    ///
    /// After Display Refresh command, BUSY signal will become "0" until the display
    /// update is finished.
    DISPLAY_REFRESH = 0x12,

    /// RED or NEW_DATA
    DATA_START_TRANSMISSION_2 = 0x13,

    /// Dual SPI - what for?
    DUAL_SPI = 0x15,

    /// This command builds the VCOM Look-Up Table (LUTC).
    LUT_FOR_VCOM = 0x20,
    /// This command builds the Black Look-Up Table (LUTB).
    LUT_BLACK = 0x21,
    /// This command builds the White Look-Up Table (LUTW).
    LUT_WHITE = 0x22,
    /// This command builds the Gray1 Look-Up Table (LUTG1).
    LUT_GRAY_1 = 0x23,
    /// This command builds the Gray2 Look-Up Table (LUTG2).
    LUT_GRAY_2 = 0x24,
    /// This command builds the Red0 Look-Up Table (LUTR0).
    LUT_RED_0 = 0x25,
    /// This command builds the Red1 Look-Up Table (LUTR1).
    LUT_RED_1 = 0x26,
    /// This command builds the Red2 Look-Up Table (LUTR2).
    LUT_RED_2 = 0x27,
    /// This command builds the Red3 Look-Up Table (LUTR3).
    LUT_RED_3 = 0x28,
    /// This command builds the XON Look-Up Table (LUTXON).
    LUT_XON = 0x29,

    /// The command controls the PLL clock frequency.
    PLL_CONTROL = 0x30,

    /// This command reads the temperature sensed by the temperature sensor.
    TEMPERATURE_SENSOR_COMMAND = 0x40,
    /// This command selects the Internal or External temperature sensor.
    TEMPERATURE_CALIBRATION = 0x41,
    /// This command could write data to the external temperature sensor.
    TEMPERATURE_SENSOR_WRITE = 0x42,
    /// This command could read data from the external temperature sensor.
    TEMPERATURE_SENSOR_READ = 0x43,

    /// This command indicates the interval of Vcom and data output. When setting the
    /// vertical back porch, the total blanking will be kept (20 Hsync).
    VCOM_AND_DATA_INTERVAL_SETTING = 0x50,
    /// This command indicates the input power condition. Host can read this flag to learn
    /// the battery condition.
    LOW_POWER_DETECTION = 0x51,

    /// This command defines non-overlap period of Gate and Source.
    TCON_SETTING = 0x60,
    /// This command defines alternative resolution and this setting is of higher priority
    /// than the RES\[1:0\] in R00H (PSR).
    TCON_RESOLUTION = 0x61,
    /// This command defines MCU host direct access external memory mode.
    SPI_FLASH_CONTROL = 0x65,

    /// The LUT_REV / Chip Revision is read from OTP address = 25001 and 25000.
    REVISION = 0x70,
    /// This command reads the IC status.
    GET_STATUS = 0x71,

    /// This command implements related VCOM sensing setting.
    AUTO_MEASUREMENT_VCOM = 0x80,
    /// This command gets the VCOM value.
    READ_VCOM_VALUE = 0x81,
    /// This command sets `VCOM_DC` value.
    VCM_DC_SETTING = 0x82,

    // /// This is in all the Waveshare controllers for EPD7in5, but it's not documented
    // /// anywhere in the datasheet `¯\_(ツ)_/¯`
    // FLASH_MODE = 0xE5,
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
        assert_eq!(Command::PANEL_SETTING.address(), 0x00);
        assert_eq!(Command::DISPLAY_REFRESH.address(), 0x12);
    }
}
