//! SPI Commands for the Waveshare 2.9" (B/C) E-Ink Display
use crate::traits;

#[derive(Copy, Clone)]
pub(crate) enum Command {
    SwReset = 0x12,
    DriverOutputControl = 0x01,
    DataEntryMode = 0x11,
    BorderWavefrom = 0x3c,
    DisplayUpdateControl = 0x21,
    TurnOnDisplay = 0x22,
    ActivateDisplayUpdateSequence = 0x20,
    ReadBuiltInTemperatureSensor = 0x18,
    RamXPosition = 0x44,
    RamYPosition = 0x45,
    RamXAddressCount = 0x4e,
    RamYAddressCount = 0x4f,
    WriteBlackData = 0x24,
    WriteRedData = 0x26,
    DeepSleep = 0x10,
}

impl traits::Command for Command {
    /// Returns the address of the command
    fn address(self) -> u8 {
        self as u8
    }
}
