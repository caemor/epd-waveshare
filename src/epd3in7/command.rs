//! SPI Commands for the Waveshare 3.7" E-Ink Display
use crate::traits;
/// EPD3IN7 commands
///
/// Should rarely (never?) be needed directly.
///
/// For more infos about the addresses and what they are doing look into the pdfs
///
/// The description of the single commands is mostly taken from EDP3IN7 specification
#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub(crate) enum Command {
    ///
    GATE_SETTING = 0x01,
    ///
    POWER_OFF = 0x02,
    ///
    SLEEP_2 = 0x07,
    ///
    GATE_VOLTAGE = 0x03,
    ///
    GATE_VOLTAGE_SOURCE = 0x04,
    ///
    BOOSTER_SOFT_START_CONTROL = 0x0C,
    /// After this command initiated, the chip will enter Deep Sleep Mode,
    /// BUSY pad will keep output high.
    ///
    /// Note: To exit Deep Sleep Mode, User required to send HWRESET to the driver.
    DEEP_SLEEP = 0x10,
    ///
    DATA_ENTRY_SEQUENCE = 0x11,
    /// This command resets commands and parameters to their S/W Reset default values,
    /// except Deep Sleep Mode.
    /// During this operation BUSY pad will keep output high.
    ///
    /// Note: RAM is unaffected by this command.
    SW_RESET = 0x12,
    /// This command selects the Internal or External temperature sensor and offset
    TEMPERATURE_SENSOR_SELECTION = 0x18,
    /// Write to temperature register
    TEMPERATURE_SENSOR_WRITE = 0x1A,
    /// Read from temperature register
    TEMPERATURE_SENSOR_READ = 0x1B,
    /// This command activates Display Update sequence.
    /// The Display Update sequence option is located at R22h.
    ///
    /// Note: BUSY pad will output high during operation. User **should not** interrupt this operation
    /// to avoid corruption of panel images.
    DISPLAY_UPDATE_SEQUENCE = 0x20,
    /// This command sets a Display Update Sequence option.
    DIPSLAY_UPDATE_SEQUENCE_SETTING = 0x22,
    /// This command will transfer its data to B/W RAM, until another command is written
    WRITE_RAM = 0x24,
    /// This command writes VCOM register from MCU interface
    WRITE_VCOM_REGISTER = 0x2C,
    /// This command writes LUT register from MCU interface (105 bytes),
    /// which contains the content of VS [nx-LUT], TP #[nX], RP #[n]
    WRITE_LUT_REGISTER = 0x32,
    ///
    DISPLAY_OPTION = 0x37,
    ///
    BORDER_WAVEFORM_CONTROL = 0x3C,
    /// This command specifies the start/end positions of the window address in the X direction,
    /// by an address unit of RAM.
    SET_RAM_X_ADDRESS_START_END_POSITION = 0x44,
    /// This command specifies the start/end positions of the window address in the Y direction,
    /// by an address unit of RAM.
    SET_RAM_Y_ADDRESS_START_END_POSITION = 0x45,
    ///
    AUTO_WRITE_RED_RAM_REGULAR_PATTERN = 0x46,
    ///
    AUTO_WRITE_BW_RAM_REGULAR_PATTERN = 0x47,
    /// This command makes the initial settings for the RAM X address in the address counter (AC)
    SET_RAM_X_ADDRESS_COUNTER = 0x4E,
    /// This command makes the initial settings for the RAM Y address in the address counter (AC)
    SET_RAM_Y_ADDRESS_COUNTER = 0x4F,
    ///
    SLEEP = 0x50,
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
    }
}
