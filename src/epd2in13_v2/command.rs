//! SPI Commands for the Waveshare 2.13" v2

use crate::traits;

extern crate bit_field;
use bit_field::BitField;

/// EPD2in13 v2
///
/// For more infos about the addresses and what they are doing look into the pdfs
#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub(crate) enum Command {
    DRIVER_OUTPUT_CONTROL = 0x01,
    GATE_DRIVING_VOLTAGE_CTRL = 0x03,
    SOURCE_DRIVING_VOLTAGE_CTRL = 0x04,
    BOOSTER_SOFT_START_CONTROL = 0x0C,
    GATE_SCAN_START_POSITION = 0x0F,
    DEEP_SLEEP_MODE = 0x10,
    DATA_ENTRY_MODE_SETTING = 0x11,
    SW_RESET = 0x12,
    HV_READY_DETECTION = 0x14,
    VCI_DETECTION = 0x15,
    TEMPERATURE_SENSOR_CONTROL_WRITE = 0x1A,
    TEMPERATURE_SENSOR_CONTROL_READ = 0x1B,
    TEMPERATURE_SENSOR_EXT_CONTROL_WRITE = 0x1C,
    MASTER_ACTIVATION = 0x20,
    DISPLAY_UPDATE_CONTROL_1 = 0x21,
    DISPLAY_UPDATE_CONTROL_2 = 0x22,
    WRITE_RAM = 0x24,
    WRITE_RAM_RED = 0x26,
    READ_RAM = 0x27,
    VCOM_SENSE = 0x28,
    VCOM_SENSE_DURATION = 0x29,
    PROGRAM_VCOM_OPT = 0x2A,
    WRITE_VCOM_REGISTER = 0x2C,
    OTP_REGISTER_READ = 0x2D,
    STATUS_BIT_READ = 0x2F,
    PROGRAM_WS_OTP = 0x30,
    LOAD_WS_OTP = 0x31,
    WRITE_LUT_REGISTER = 0x32,
    PROGRAM_OTP_SELECTION = 0x36,
    WRITE_OTP_SELECTION = 0x37,
    SET_DUMMY_LINE_PERIOD = 0x3A,
    SET_GATE_LINE_WIDTH = 0x3B,
    BORDER_WAVEFORM_CONTROL = 0x3C,
    READ_RAM_OPTION = 0x41,
    SET_RAM_X_ADDRESS_START_END_POSITION = 0x44,
    SET_RAM_Y_ADDRESS_START_END_POSITION = 0x45,
    AUTO_WRITE_RED_RAM_REGULAR_PATTERN = 0x46,
    AUTO_WRITE_BW_RAM_REGULAR_PATTERN = 0x47,
    SET_RAM_X_ADDRESS_COUNTER = 0x4E,
    SET_RAM_Y_ADDRESS_COUNTER = 0x4F,
    SET_ANALOG_BLOCK_CONTROL = 0x74,
    SET_DIGITAL_BLOCK_CONTROL = 0x7E,

    NOP = 0x7F,
}

pub(crate) struct DriverOutput {
    pub scan_is_linear: bool,
    pub scan_g0_is_first: bool,
    pub scan_dir_incr: bool,

    pub width: u16,
}

impl DriverOutput {
    pub fn to_bytes(&self) -> [u8; 3] {
        [
            self.width as u8,
            (self.width >> 8) as u8,
            *0u8.set_bit(0, !self.scan_dir_incr)
                .set_bit(1, !self.scan_g0_is_first)
                .set_bit(2, !self.scan_is_linear),
        ]
    }
}

/// These are not directly documented, but the bitfield is easily reversed from
/// documentation and sample code
/// [7|6|5|4|3|2|1|0]
///  | | | | | | | `--- disable clock
///  | | | | | | `----- disable analog
///  | | | | | `------- display
///  | | | | `--------- undocumented and unknown use,
///  | | | |            but used in waveshare reference code
///  | | | `----------- load LUT
///  | | `------------- load temp
///  | `--------------- enable clock
///  `----------------- enable analog

pub(crate) struct DisplayUpdateControl2(pub u8);
#[allow(dead_code)]
impl DisplayUpdateControl2 {
    pub fn new() -> DisplayUpdateControl2 {
        DisplayUpdateControl2(0x00)
    }

    pub fn disable_clock(mut self) -> Self {
        self.0.set_bit(0, true);
        self
    }

    pub fn disable_analog(mut self) -> Self {
        self.0.set_bit(1, true);
        self
    }

    pub fn display(mut self) -> Self {
        self.0.set_bit(2, true);
        self
    }

    pub fn load_lut(mut self) -> Self {
        self.0.set_bit(4, true);
        self
    }

    pub fn load_temp(mut self) -> Self {
        self.0.set_bit(5, true);
        self
    }

    pub fn enable_clock(mut self) -> Self {
        self.0.set_bit(6, true);
        self
    }

    pub fn enable_analog(mut self) -> Self {
        self.0.set_bit(7, true);
        self
    }
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub(crate) enum DataEntryModeIncr {
    X_DECR_Y_DECR = 0x0,
    X_INCR_Y_DECR = 0x1,
    X_DECR_Y_INCR = 0x2,
    X_INCR_Y_INCR = 0x3,
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub(crate) enum DataEntryModeDir {
    X_DIR = 0x0,
    Y_DIR = 0x4,
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub(crate) enum BorderWaveFormVBD {
    GS = 0x0,
    FIX_LEVEL = 0x1,
    VCOM = 0x2,
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub(crate) enum BorderWaveFormFixLevel {
    VSS = 0x0,
    VSH1 = 0x1,
    VSL = 0x2,
    VSH2 = 0x3,
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub(crate) enum BorderWaveFormGS {
    LUT0 = 0x0,
    LUT1 = 0x1,
    LUT2 = 0x2,
    LUT3 = 0x3,
}

pub(crate) struct BorderWaveForm {
    pub vbd: BorderWaveFormVBD,
    pub fix_level: BorderWaveFormFixLevel,
    pub gs_trans: BorderWaveFormGS,
}

impl BorderWaveForm {
    pub fn to_u8(&self) -> u8 {
        *0u8.set_bits(6..8, self.vbd as u8)
            .set_bits(4..6, self.fix_level as u8)
            .set_bits(0..2, self.gs_trans as u8)
    }
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub enum DeepSleepMode {
    // Sleeps and keeps access to RAM and controller
    NORMAL = 0x00,

    // Sleeps without access to RAM/controller but keeps RAM content
    MODE_1 = 0x01,

    // Same as MODE_1 but RAM content is not kept
    MODE_2 = 0x11,
}

pub(crate) struct GateDrivingVoltage(pub u8);
pub(crate) struct SourceDrivingVoltage(pub u8);
pub(crate) struct VCOM(pub u8);

pub(crate) trait I32Ext {
    fn vcom(self) -> VCOM;
    fn gate_driving_decivolt(self) -> GateDrivingVoltage;
    fn source_driving_decivolt(self) -> SourceDrivingVoltage;
}

impl I32Ext for i32 {
    // This is really not very nice. Until I find something better, this will be
    // a placeholder.
    fn vcom(self) -> VCOM {
        assert!(self >= -30 && self <= -2);
        let u = match -self {
            2 => 0x08,
            3 => 0x0B,
            4 => 0x10,
            5 => 0x14,
            6 => 0x17,
            7 => 0x1B,
            8 => 0x20,
            9 => 0x24,
            10 => 0x28,
            11 => 0x2C,
            12 => 0x2F,
            13 => 0x34,
            14 => 0x37,
            15 => 0x3C,
            16 => 0x40,
            17 => 0x44,
            18 => 0x48,
            19 => 0x4B,
            20 => 0x50,
            21 => 0x54,
            22 => 0x58,
            23 => 0x5B,
            24 => 0x5F,
            25 => 0x64,
            26 => 0x68,
            27 => 0x6C,
            28 => 0x6F,
            29 => 0x73,
            30 => 0x78,
            _ => 0,
        };
        VCOM(u)
    }

    fn gate_driving_decivolt(self) -> GateDrivingVoltage {
        assert!(self >= 100 && self <= 210 && self % 5 == 0);
        GateDrivingVoltage(((self - 100) / 5 + 0x03) as u8)
    }

    fn source_driving_decivolt(self) -> SourceDrivingVoltage {
        assert!(
            (self >= 24 && self <= 88)
                || (self >= 90 && self <= 180 && self % 5 == 0)
                || (self >= -180 && self <= -90 && self % 5 == 0)
        );

        if self >= 24 && self <= 88 {
            SourceDrivingVoltage(((self - 24) + 0x8E) as u8)
        } else if self >= 90 && self <= 180 {
            SourceDrivingVoltage(((self - 90) / 2 + 0x23) as u8)
        } else {
            SourceDrivingVoltage((((-self - 90) / 5) * 2 + 0x1A) as u8)
        }
    }
}

impl traits::Command for Command {
    /// Returns the address of the command
    fn address(self) -> u8 {
        self as u8
    }
}
