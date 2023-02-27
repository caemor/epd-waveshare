#![allow(dead_code)]
///! SPI Commands for the SSD1675B driver chip
use crate::traits;

#[derive(Copy, Clone)]
pub(crate) enum Command {
    DriverOutputControl = 0x01,
    GateDrivingVoltageControl = 0x02,
    SourceDrivingVoltageControl = 0x04,
    ProgramOTPInitialCodeSetting = 0x08,
    WriteRegisterForInitialCodeSetting = 0x09,
    ReadRegisterForInitiaslCodeSetting = 0x0a,
    BoosterSoftstartControl = 0x0c,
    GateScanStartPosition = 0x0f,
    DeepSleepMode = 0x10,
    DataEntryMode = 0x11,
    Reset = 0x12,
    HVReadyDetection = 0x14,
    VCIDetection = 0x15,
    TemperatureSensorSelection = 0x18,
    WriteTemperatureRegister = 0x1a,
    ReadTemperatureRegister = 0x1b,
    ExternalTemperatureSensorWriteCommand = 0x1c,
    MasterActivation = 0x20,
    DisplayUpdateControl1 = 0x21,
    DisplayUpdateControl2 = 0x22,
    WriteBlackWhiteRAM = 0x24,
    WriteRedRAM = 0x26,
    ReadRAM = 0x27,
    SenseVCOM = 0x28,
    VCOMSenseDuration = 0x29,
    ProgramOTPVCOM = 0x2a,
    WriteRegisterForVCOMControl = 0x2b,
    WriteVCOMRegister = 0x2c,
    ReadOTPDisplayOptions = 0x2d,
    ReadOTPUserId = 0x2e,
    ReadStatusBits = 0x2f,
    ProgramOTPWaveformSetting = 0x30,
    LoadOTPWaveformSetting = 0x31,
    WriteLUTRegister = 0x32,
    CalculateCRC = 0x34,
    ReadCRC = 0x35,
    ProgramOTPSelection = 0x36,
    WriteRegisterForDisplayOption = 0x37,
    WriteRegisterForUserID = 0x38,
    OTPProgramMode = 0x39,
    SetDummyLinePeriod = 0x3a,
    SetGateLineWidth = 0x3b,
    BorderWaveformControl = 0x3c,
    RAMReadOption = 0x41,
    SetXAddressRange = 0x44,
    SetYAddressRange = 0x45,
    RedRAMTestPattern = 0x46,
    BlackWhiteRAMTestPattern = 0x47,
    SetXAddressCounter = 0x4e,
    SetYAddressCounter = 0x4f,
    SetAnalogBlockControl = 0x74,
    SetDigitalBlockControl = 0x7e,
    Nop = 0x7f,
}

impl traits::Command for Command {
    /// Returns the address of the command
    fn address(self) -> u8 {
        self as u8
    }
}

pub(crate) enum DataEntrySign {
    DecYDecX = 0b00,
    DecYIncX = 0b01,
    IncYDecX = 0b10,
    IncYIncX = 0b11,
}
pub(crate) enum DataEntryRow {
    XMinor = 0b000,
    YMinor = 0b100,
}

pub(crate) enum WriteMode {
    Normal = 0b0000,
    ForceZero = 0b0100,
    Invert = 0b1000,
}
pub(crate) enum OutputSource {
    S0ToS175 = 0x00,
    S8ToS167 = 0x80,
}

pub(crate) enum DeepSleep {
    Awake = 0b00,
    SleepKeepingRAM = 0b01,
    SleepLosingRAM = 0b11,
}

pub(crate) enum PatH {
    H8 = 0b000_0000,
    H16 = 0b001_0000,
    H32 = 0b010_0000,
    H64 = 0b011_0000,
    H128 = 0b100_0000,
    H256 = 0b101_0000,
    H296 = 0b110_0000,
}
pub(crate) enum PatW {
    W8 = 0b000,
    W16 = 0b001,
    W32 = 0b010,
    W64 = 0b011,
    W128 = 0b100,
    W160 = 0b101,
}
pub(crate) enum StartWith {
    Zero = 0x00,
    One = 0x80,
}
