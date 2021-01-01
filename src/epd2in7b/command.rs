//! SPI Commands for the Waveshare 2.7" B 3 color E-Ink Display
use crate::traits;

/// EPD2IN7B commands
///
/// More information can be found in the [specification](https://www.waveshare.com/w/upload/d/d8/2.7inch-e-paper-b-specification.pdf)
#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub(crate) enum Command {
    /// Set Resolution, LUT selection, BWR pixels, gate scan direction, source shift direction, booster switch, soft reset
    PANEL_SETTING = 0x00,
    /// Selecting internal and external power
    POWER_SETTING = 0x01,
    POWER_OFF = 0x02,
    /// Setting Power OFF sequence
    POWER_OFF_SEQUENCE_SETTING = 0x03,
    POWER_ON = 0x04,
    /// This command enables the internal bandgap, which will be cleared by the next POF.
    POWER_ON_MEASURE = 0x05,
    /// Starting data transmission
    ///
    /// ```ignore
    /// self.send_data(&[0x07, 0x07, 0x17])?;
    /// ```
    BOOSTER_SOFT_START = 0x06,
    /// After this command is transmitted, the chip would enter the deep-sleep mode to save power.
    ///
    /// The deep sleep mode would return to standby by hardware reset.
    ///
    /// The only one parameter is a check code, the command would be excuted if check code = 0xA5.
    DEEP_SLEEP = 0x07,
    /// This command starts transmitting data and write them into SRAM. To complete data transmission, command DSP (Data
    /// transmission Stop) must be issued. Then the chip will start to send data/VCOM for panel.
    ///
    /// - In B/W mode, this command writes “OLD” data to SRAM.
    /// - In B/W/Red mode, this command writes “B/W” data to SRAM.
    DATA_START_TRANSMISSION_1 = 0x10,
    /// Stopping data transmission
    DATA_STOP = 0x11,
    /// After this command is issued, driver will refresh display (data/VCOM) according to SRAM data and LUT.
    DISPLAY_REFRESH = 0x12,
    /// This command starts transmitting data and write them into SRAM. To complete data transmission, command DSP (Data
    /// transmission Stop) must be issued. Then the chip will start to send data/VCOM for panel.
    /// - In B/W mode, this command writes “NEW” data to SRAM.
    /// - In B/W/Red mode, this command writes “RED” data to SRAM.
    DATA_START_TRANSMISSION_2 = 0x13,
    /// The command define as follows: The register is indicates that user start to transmit data, then write to SRAM. While data transmission
    /// complete, user must send command DSP (Data transmission Stop). Then chip will start to send data/VCOM for panel.
    ///
    /// - In B/W mode, this command writes “OLD” data to SRAM.
    /// - In B/W/Red mode, this command writes “B/W” data to SRAM.
    PARTIAL_DATA_START_TRANSMISSION_1 = 0x14,
    /// The command define as follows: The register is indicates that user start to transmit data, then write to SRAM. While data transmission
    /// complete, user must send command DSP (Data transmission Stop). Then chip will start to send data/VCOM for panel.
    ///
    /// - In B/W mode, this command writes “NEW” data to SRAM.
    /// - In B/W/Red mode, this command writes “RED” data to SRAM.
    PARTIAL_DATA_START_TRANSMISSION_2 = 0x15,
    /// While user sent this command, driver will refresh display (data/VCOM) base on SRAM data and LUT.
    ///
    /// Only the area (X,Y, W, L) would update, the others pixel output would follow VCOM LUT
    PARTIAL_DISPLAY_REFRESH = 0x16,
    /// This command builds the Look-up table for VCOM
    LUT_FOR_VCOM = 0x20,
    LUT_WHITE_TO_WHITE = 0x21,
    LUT_BLACK_TO_WHITE = 0x22,
    LUT_WHITE_TO_BLACK = 0x23,
    LUT_BLACK_TO_BLACK = 0x24,
    /// The command controls the PLL clock frequency.
    PLL_CONTROL = 0x30,
    /// This command reads the temperature sensed by the temperature sensor.
    ///
    /// Doesn't work! Waveshare doesn't connect the read pin
    TEMPERATURE_SENSOR_COMMAND = 0x40,
    /// This command selects Internal or External temperature sensor.
    TEMPERATURE_SENSOR_CALIBRATION = 0x41,
    /// Write External Temperature Sensor
    TEMPERATURE_SENSOR_WRITE = 0x42,
    /// Read External Temperature Sensor
    ///
    /// Doesn't work! Waveshare doesn't connect the read pin
    TEMPERATURE_SENSOR_READ = 0x43,
    /// This command indicates the interval of Vcom and data output. When setting the vertical back porch, the total blanking will be kept (20 Hsync)
    VCOM_AND_DATA_INTERVAL_SETTING = 0x50,
    /// This command indicates the input power condition. Host can read this flag to learn the battery condition.
    LOW_POWER_DETECTION = 0x51,
    /// This command defines non-overlap period of Gate and Source.
    TCON_SETTING = 0x60,
    /// This command defines alternative resolution and this setting is of higher priority than the RES\[1:0\] in R00H (PSR).
    RESOLUTION_SETTING = 0x61,
    SOURCE_AND_GATE_SETTING = 0x62,
    /// This command reads the IC status.
    ///
    /// Doesn't work! Waveshare doesn't connect the read pin
    GET_STATUS = 0x71,
    /// Automatically measure VCOM. This command reads the IC status
    AUTO_MEASUREMENT_VCOM = 0x80,
    /// This command gets the VCOM value
    ///
    /// Doesn't work! Waveshare doesn't connect the read pin
    READ_VCOM_VALUE = 0x81,
    /// This command sets VCOM_DC value.
    VCM_DC_SETTING = 0x82,
    /// After this command is issued, the chip would enter the program mode.
    ///
    /// After the programming procedure completed, a hardware reset is necessary for leaving program mode.
    ///
    /// The only one parameter is a check code, the command would be excuted if check code = 0xA5.
    PROGRAM_MODE = 0xA0,
    /// After this command is issued, the chip would enter the program mode.
    ACTIVE_PROGRAMMING = 0xA1,
    /// The command is used for reading the content of OTP for checking the data of programming.
    ///
    /// The value of (n) is depending on the amount of programmed data, tha max address = 0xFFF.
    READ_OTP = 0xA2,
    /// Not shown in commands table, but used in init sequence
    POWER_OPTIMIZATION = 0xf8,
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
        assert_eq!(Command::PANEL_SETTING.address(), 0x00);
        assert_eq!(Command::DISPLAY_REFRESH.address(), 0x12);
    }
}
