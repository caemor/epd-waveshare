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
    DriverOutputControl = 0x01,

    /// Set gate driving voltage
    GateDrivingVoltageControl = 0x03,

    /// Set source driving voltage
    SourceDrivingVoltageControl = 0x04,

    SoftStart = 0x0C,

    /// Set the scanning start position of the gate driver.
    /// The valid range is from 0 to 679.
    GateScanStartPosition = 0x0F,

    /// Deep sleep mode control
    DeepSleep = 0x10,

    /// Define data entry sequence
    DataEntry = 0x11,

    /// resets the commands and parameters to their S/W Reset default values except R10h-Deep Sleep Mode.
    /// During operation, BUSY pad will output high.
    /// Note: RAM are unaffected by this command.
    SwReset = 0x12,

    /// After this command initiated, HV Ready detection starts.
    /// BUSY pad will output high during detection.
    /// The detection result can be read from the Status Bit Read (Command 0x2F).
    HvReadyDetection = 0x14,

    /// After this command initiated, VCI detection starts.
    /// BUSY pad will output high during detection.
    /// The detection result can be read from the Status Bit Read (Command 0x2F).
    VciDetection = 0x15,

    /// Temperature Sensor Selection
    TemperatureSensorControl = 0x18,

    /// Write to temperature register
    TemperatureSensorWrite = 0x1A,

    /// Read from temperature register
    TemperatureSensorRead = 0x1B,

    /// Write Command to External temperature sensor.
    TemperatureSensorWriteExternal = 0x1C,

    /// Activate Display Update Sequence
    MasterActivation = 0x20,

    /// RAM content option for Display Update
    DisplayUpdateControl1 = 0x21,

    /// Display Update Sequence Option
    DisplayUpdateControl2 = 0x22,

    /// After this command, data entries will be written into the BW RAM until another command is written
    WriteRamBw = 0x24,

    /// After this command, data entries will be written into the RED RAM until another command is written
    WriteRamRed = 0x26,

    /// Fetch data from RAM
    ReadRam = 0x27,

    /// Enter VCOM sensing conditions
    VcomSense = 0x28,

    /// Enter VCOM sensing conditions
    VcomSenseDuration = 0x29,

    /// Program VCOM register into OTP
    VcomProgramOtp = 0x2A,

    /// Reduces a glitch when ACVCOM is toggled
    VcomControl = 0x2B,

    /// Write VCOM register from MCU interface
    VcomWrite = 0x2C,

    /// Read Register for Display Option
    OtpRead = 0x2D,

    /// CRC calculation command for OTP content validation
    CrcCalculation = 0x34,

    /// CRC Status Read
    CrcRead = 0x35,

    /// Program OTP Selection according to the OTP Selection Control
    ProgramSelection = 0x36,

    /// Write Register for Display Option
    DisplayOptionWrite = 0x37,

    /// Write register for User ID
    UserIdWrite = 0x38,

    /// Select border waveform for VBD
    VbdControl = 0x3C,

    /// Read RAM Option
    ReadRamOption = 0x41,

    /// Specify the start/end positions of the window address in the X direction by an address unit for RAM
    SetRamXStartEnd = 0x44,

    /// Specify the start/end positions of the window address in the Y direction by an address unit for RAM
    SetRamYStartEnd = 0x45,

    /// Auto write RED RAM for regular pattern
    AutoWriteRed = 0x46,

    /// Auto write B/W RAM for regular pattern
    AutoWriteBw = 0x47,

    /// Make initial settings for the RAM X address in the address counter (AC)
    SetRamXAc = 0x4E,

    /// Make initial settings for the RAM Y address in the address counter (AC)
    SetRamYAc = 0x4F,

    /// This command is an empty command; it does not have any effect on the display module.
    /// However, it can be used to terminate Frame Memory Write or Read Commands.
    Nop = 0x7F,
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
        assert_eq!(Command::MasterActivation.address(), 0x20);
        assert_eq!(Command::SwReset.address(), 0x12);
        assert_eq!(Command::DisplayUpdateControl2.address(), 0x22);
    }
}
