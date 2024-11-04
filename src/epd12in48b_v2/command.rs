//! SPI Commands for the Waveshare 12.48"(B) V2 Ink Display

use crate::traits;

/// Epd12in48 commands
///
#[allow(unused, non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum Command {
    PanelSetting = 0x00,
    PowerOff = 0x02,
    PowerOn = 0x04,
    BoosterSoftStart = 0x06,
    DeepSleep = 0x07,
    DataStartTransmission1 = 0x10,
    DisplayRefresh = 0x12,
    DataStartTransmission2 = 0x13,
    DualSPI = 0x15,
    LutC = 0x20,
    LutWW = 0x21,
    LutKW_LutR = 0x22,
    LutWK_LutW = 0x23,
    LutKK_LutK = 0x24,
    LutBD = 0x25,
    KWLUTOption = 0x2B,
    VcomAndDataIntervalSetting = 0x50,
    TconSetting = 0x60,
    TconResolution = 0x61,
    GetStatus = 0x71,
    PartialWindow = 0x90,
    PartialIn = 0x91,
    PartialOut = 0x92,
    CascadeSetting = 0xE0,
    PowerSaving = 0xE3,
    ForceTemperature = 0xE5,
}

impl traits::Command for Command {
    /// Returns the address of the command
    fn address(self) -> u8 {
        self as u8
    }
}
