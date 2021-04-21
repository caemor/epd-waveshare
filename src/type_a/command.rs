//! SPI Commands for the Waveshare 2.9" and 1.54" E-Ink Display

use crate::traits;

/// Epd1in54 and EPD2IN9 commands
///
/// Should rarely (never?) be needed directly.
///
/// For more infos about the addresses and what they are doing look into the pdfs
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub(crate) enum Command {
    /// Driver Output control
    ///     3 Databytes:
    ///     A[7:0]
    ///     0.. A[8]
    ///     0.. B[2:0]
    ///     Default: Set A[8:0] = 0x127 and B[2:0] = 0x0
    DriverOutputControl = 0x01,
    /// Booster Soft start control
    ///     3 Databytes:
    ///     1.. A[6:0]
    ///     1.. B[6:0]
    ///     1.. C[6:0]
    ///     Default: A[7:0] = 0xCF, B[7:0] = 0xCE, C[7:0] = 0x8D
    BoosterSoftStartControl = 0x0C,
    GateScanStartPosition = 0x0F,
    //TODO: useful?
    // GateScanStartPosition = 0x0F,
    /// Deep Sleep Mode Control
    ///     1 Databyte:
    ///     0.. A[0]
    ///     Values:
    ///         A[0] = 0: Normal Mode (POR)
    ///         A[0] = 1: Enter Deep Sleep Mode
    DeepSleepMode = 0x10,
    // /// Data Entry mode setting
    DataEntryModeSetting = 0x11,

    SwReset = 0x12,

    TemperatureSensorControl = 0x1A,

    MasterActivation = 0x20,

    DisplayUpdateControl1 = 0x21,

    DisplayUpdateControl2 = 0x22,

    WriteRam = 0x24,

    WriteRam2 = 0x26,

    WriteVcomRegister = 0x2C,

    WriteLutRegister = 0x32,

    WriteOtpSelection = 0x37,

    SetDummyLinePeriod = 0x3A,

    SetGateLineWidth = 0x3B,

    BorderWaveformControl = 0x3C,

    SetRamXAddressStartEndPosition = 0x44,

    SetRamYAddressStartEndPosition = 0x45,

    SetRamXAddressCounter = 0x4E,

    SetRamYAddressCounter = 0x4F,

    Nop = 0xFF,
}

impl traits::Command for Command {
    /// Returns the address of the command
    fn address(self) -> u8 {
        self as u8
    }
}

#[cfg(test)]
mod tests {
    use super::Command;
    use crate::traits::Command as CommandTrait;

    #[test]
    fn command_addr() {
        assert_eq!(Command::DriverOutputControl.address(), 0x01);

        assert_eq!(Command::SetRamXAddressCounter.address(), 0x4E);

        assert_eq!(Command::Nop.address(), 0xFF);
    }
}
