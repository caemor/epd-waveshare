//! SPI Commands for the Waveshare 1.54" red E-Ink Display
use crate::traits;

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub(crate) enum Command {
    PANEL_SETTING = 0x00,

    POWER_SETTING = 0x01,
    POWER_OFF = 0x02,
    POWER_ON = 0x04,
    BOOSTER_SOFT_START = 0x06,
    DATA_START_TRANSMISSION_1 = 0x10,
    DISPLAY_REFRESH = 0x12,
    DATA_START_TRANSMISSION_2 = 0x13,

    LUT_FOR_VCOM = 0x20,
    LUT_WHITE_TO_WHITE = 0x21,
    LUT_BLACK_TO_WHITE = 0x22,
    LUT_G0 = 0x23,
    LUT_G1 = 0x24,
    LUT_RED_VCOM = 0x25,
    LUT_RED0 = 0x26,
    LUT_RED1 = 0x27,

    PLL_CONTROL = 0x30,
    TEMPERATURE_SENSOR_COMMAND = 0x40,
    TEMPERATURE_SENSOR_SELECTION = 0x41,
    VCOM_AND_DATA_INTERVAL_SETTING = 0x50,
    RESOLUTION_SETTING = 0x61,
    VCM_DC_SETTING = 0x82,
    POWER_SAVING = 0xE3,
}

impl traits::Command for Command {
    /// Returns the address of the command
    fn address(self) -> u8 {
        self as u8
    }
}
