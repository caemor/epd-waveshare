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
    DRIVER_OUTPUT_CONTROL = 0x01,

    /// Set gate driving voltage
    GATE_DRIVING_VOLTAGE_CONTROL = 0x03,

    /// Set source driving voltage
    SOURCE_DRIVING_VOLTAGE_CONTROL = 0x04,

    SOFT_START = 0x0C,

    /// Set the scanning start position of the gate driver.
    /// The valid range is from 0 to 679.
    GATE_SCAN_START_POSITION = 0x0F,

    /// Deep sleep mode control
    DEEP_SLEEP = 0x10,

    /// Define data entry sequence
    DATA_ENTRY = 0x11,

    /// resets the commands and parameters to their S/W Reset default values except R10h-Deep Sleep Mode.
    /// During operation, BUSY pad will output high.
    /// Note: RAM are unaffected by this command.
    SW_RESET = 0x12,

    /// After this command initiated, HV Ready detection starts.
    /// BUSY pad will output high during detection.
    /// The detection result can be read from the Status Bit Read (Command 0x2F).
    HV_READY_DETECTION = 0x14,

    /// After this command initiated, VCI detection starts.
    /// BUSY pad will output high during detection.
    /// The detection result can be read from the Status Bit Read (Command 0x2F).
    VCI_DETECTION = 0x15,

    /// Temperature Sensor Selection
    TEMPERATURE_SENSOR_CONTROL = 0x18,

    /// Write to temperature register
    TEMPERATURE_SENSOR_WRITE = 0x1A,

    /// Read from temperature register
    TEMPERATURE_SENSOR_READ = 0x1B,

    /// Write Command to External temperature sensor.
    TEMPERATURE_SENSOR_WRITE_EXTERNAL = 0x1C,

    /// Activate Display Update Sequence
    MASTER_ACTIVATION = 0x20,

    /// RAM content option for Display Update
    DISPLAY_UPDATE_CONTROL_1 = 0x21,

    /// Display Update Sequence Option
    DISPLAY_UPDATE_CONTROL_2 = 0x22,

    /// After this command, data entries will be written into the BW RAM until another command is written
    WRITE_RAM_BW = 0x24,

    /// After this command, data entries will be written into the RED RAM until another command is written
    WRITE_RAM_RED = 0x26,

    /// Fetch data from RAM
    READ_RAM = 0x27,

    /// Enter VCOM sensing conditions
    VCOM_SENSE = 0x28,

    /// Enter VCOM sensing conditions
    VCOM_SENSE_DURATION = 0x29,

    /// Program VCOM register into OTP
    VCOM_PROGRAM_OTP = 0x2A,

    /// Reduces a glitch when ACVCOM is toggled
    VCOM_CONTROL = 0x2B,

    /// Write VCOM register from MCU interface
    VCOM_WRITE = 0x2C,

    /// Read Register for Display Option
    OTP_READ = 0x2D,

    /// CRC calculation command for OTP content validation
    CRC_CALCULATION = 0x34,

    /// CRC Status Read
    CRC_READ = 0x35,

    /// Program OTP Selection according to the OTP Selection Control
    PROGRAM_SELECTION = 0x36,

    /// Write Register for Display Option
    DISPLAY_OPTION_WRITE = 0x37,

    /// Write register for User ID
    USER_ID_WRITE = 0x38,

    /// Select border waveform for VBD
    VBD_CONTROL = 0x3C,

    /// Read RAM Option
    READ_RAM_OPTION = 0x41,

    /// Specify the start/end positions of the window address in the X direction by an address unit for RAM
    SET_RAM_X_START_END = 0x44,

    /// Specify the start/end positions of the window address in the Y direction by an address unit for RAM
    SET_RAM_Y_START_END = 0x45,

    /// Auto write RED RAM for regular pattern
    AUTO_WRITE_RED = 0x46,

    /// Auto write B/W RAM for regular pattern
    AUTO_WRITE_BW = 0x47,

    /// Make initial settings for the RAM X address in the address counter (AC)
    SET_RAM_X_AC = 0x4E,

    /// Make initial settings for the RAM Y address in the address counter (AC)
    SET_RAM_Y_AC = 0x4F,

    /// This command is an empty command; it does not have any effect on the display module.
    /// However, it can be used to terminate Frame Memory Write or Read Commands.
    NOP = 0x7F,
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
        // assert_eq!(Command::PANEL_SETTING.address(), 0x00);
        // assert_eq!(Command::DISPLAY_REFRESH.address(), 0x12);
    }
}
