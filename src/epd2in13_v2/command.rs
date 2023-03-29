//! SPI Commands for the Waveshare 2.13" v2

use crate::traits;

extern crate bit_field;
use bit_field::BitField;

/// Epd2in13 v2
///
/// For more infos about the addresses and what they are doing look into the pdfs
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub(crate) enum Command {
    DriverOutputControl = 0x01,
    GateDrivingVoltageCtrl = 0x03,
    SourceDrivingVoltageCtrl = 0x04,
    BoosterSoftStartControl = 0x0C,
    GateScanStartPosition = 0x0F,
    DeepSleepMode = 0x10,
    DataEntryModeSetting = 0x11,
    SwReset = 0x12,
    HvReadyDetection = 0x14,
    VciDetection = 0x15,
    TemperatureSensorControlWrite = 0x1A,
    TemperatureSensorControlRead = 0x1B,
    TemperatureSensorExtControlWrite = 0x1C,
    MasterActivation = 0x20,
    DisplayUpdateControl1 = 0x21,
    DisplayUpdateControl2 = 0x22,
    WriteRam = 0x24,
    WriteRamRed = 0x26,
    ReadRam = 0x27,
    VcomSense = 0x28,
    VcomSenseDuration = 0x29,
    ProgramVcomOpt = 0x2A,
    WriteVcomRegister = 0x2C,
    OtpRegisterRead = 0x2D,
    StatusBitRead = 0x2F,
    ProgramWsOtp = 0x30,
    LoadWsOtp = 0x31,
    WriteLutRegister = 0x32,
    ProgramOtpSelection = 0x36,
    WriteOtpSelection = 0x37,
    SetDummyLinePeriod = 0x3A,
    SetGateLineWidth = 0x3B,
    BorderWaveformControl = 0x3C,
    ReadRamOption = 0x41,
    SetRamXAddressStartEndPosition = 0x44,
    SetRamYAddressStartEndPosition = 0x45,
    AutoWriteRedRamRegularPattern = 0x46,
    AutoWriteBwRamRegularPattern = 0x47,
    SetRamXAddressCounter = 0x4E,
    SetRamYAddressCounter = 0x4F,
    SetAnalogBlockControl = 0x74,
    SetDigitalBlockControl = 0x7E,

    Nop = 0x7F,
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

#[allow(dead_code, clippy::enum_variant_names)]
pub(crate) enum DataEntryModeIncr {
    XDecrYDecr = 0x0,
    XIncrYDecr = 0x1,
    XDecrYIncr = 0x2,
    XIncrYIncr = 0x3,
}

#[allow(dead_code)]
pub(crate) enum DataEntryModeDir {
    XDir = 0x0,
    YDir = 0x4,
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub(crate) enum BorderWaveFormVbd {
    Gs = 0x0,
    FixLevel = 0x1,
    Vcom = 0x2,
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub(crate) enum BorderWaveFormFixLevel {
    Vss = 0x0,
    Vsh1 = 0x1,
    Vsl = 0x2,
    Vsh2 = 0x3,
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub(crate) enum BorderWaveFormGs {
    Lut0 = 0x0,
    Lut1 = 0x1,
    Lut2 = 0x2,
    Lut3 = 0x3,
}

pub(crate) struct BorderWaveForm {
    pub vbd: BorderWaveFormVbd,
    pub fix_level: BorderWaveFormFixLevel,
    pub gs_trans: BorderWaveFormGs,
}

impl BorderWaveForm {
    pub fn to_u8(&self) -> u8 {
        *0u8.set_bits(6..8, self.vbd as u8)
            .set_bits(4..6, self.fix_level as u8)
            .set_bits(0..2, self.gs_trans as u8)
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum DeepSleepMode {
    // Sleeps and keeps access to RAM and controller
    Normal = 0x00,

    // Sleeps without access to RAM/controller but keeps RAM content
    Mode1 = 0x01,

    // Same as MODE_1 but RAM content is not kept
    Mode2 = 0x11,
}

pub(crate) struct GateDrivingVoltage(pub u8);
pub(crate) struct SourceDrivingVoltage(pub u8);
pub(crate) struct Vcom(pub u8);

pub(crate) trait I32Ext {
    fn vcom(self) -> Vcom;
    fn gate_driving_decivolt(self) -> GateDrivingVoltage;
    fn source_driving_decivolt(self) -> SourceDrivingVoltage;
}

impl I32Ext for i32 {
    // This is really not very nice. Until I find something better, this will be
    // a placeholder.
    fn vcom(self) -> Vcom {
        assert!((-30..=-2).contains(&self));
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
        Vcom(u)
    }

    fn gate_driving_decivolt(self) -> GateDrivingVoltage {
        assert!((100..=210).contains(&self) && self % 5 == 0);
        GateDrivingVoltage(((self - 100) / 5 + 0x03) as u8)
    }

    fn source_driving_decivolt(self) -> SourceDrivingVoltage {
        assert!((24..=88).contains(&self) || (self % 5 == 0 && (90..=180).contains(&self.abs())));

        if (24..=88).contains(&self) {
            SourceDrivingVoltage(((self - 24) + 0x8E) as u8)
        } else if (90..=180).contains(&self) {
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
