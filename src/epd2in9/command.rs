//! SPI Commands for the Waveshare 2.9" E-Ink Display

use interface;


/// EPD2IN9 commands
/// 
/// Should rarely (never?) be needed directly.
/// 
/// For more infos about the addresses and what they are doing look into the pdfs 
/// 
/// The description of the single commands is mostly taken from IL0398.pdf
//#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub enum Command {
    /// Driver Output control 	
	/// 	3 Databytes: 
	/// 	A[7:0]
	/// 	0.. A[8]
	/// 	0.. B[2:0]
	/// 	Default: Set A[8:0] = 0x127 and B[2:0] = 0x0
	DRIVER_OUTPUT_CONTROL = 0x01,
	/// Booster Soft start control
	/// 	3 Databytes:
	/// 	1.. A[6:0]
	/// 	1.. B[6:0]
	/// 	1.. C[6:0]
	/// 	Default: A[7:0] = 0xCF, B[7:0] = 0xCE, C[7:0] = 0x8D
	BOOSTER_SOFT_START_CONTROL = 0x0C,
	//TODO: useful?
	// GATE_SCAN_START_POSITION = 0x0F,
	/// Deep Sleep Mode Control
	/// 	1 Databyte: 
	/// 	0.. A[0]
	/// 	Values: 
	/// 		A[0] = 0: Normal Mode (POR)
	/// 		A[0] = 1: Enter Deep Sleep Mode
	DEEP_SLEEP_MODE = 0x10,
	// /// Data Entry mode setting
	DATA_ENTRY_MODE_SETTING = 0x11,

	SW_RESET = 0x12,

	TEMPERATURE_SENSOR_CONTROL = 0x1A,

	MASTER_ACTIVATION = 0x20,

	DISPLAY_UPDATE_CONTROL_1 = 0x21,

	DISPLAY_UPDATE_CONTROL_2 = 0x22,

	WRITE_RAM = 0x24,

	WRITE_VCOM_REGISTER = 0x2C,

	WRITE_LUT_REGISTER = 0x32,

	SET_DUMMY_LINE_PERIOD = 0x3A,

	SET_GATE_TIME = 0x3B,

	BORDER_WAVEFORM_CONTROL = 0x3C,

	SET_RAM_X_ADDRESS_START_END_POSITION = 0x44,

	SET_RAM_Y_ADDRESS_START_END_POSITION = 0x45,

	SET_RAM_X_ADDRESS_COUNTER = 0x4E,

	SET_RAM_Y_ADDRESS_COUNTER = 0x4F,

	TERMINATE_COMMANDS_AND_FRAME_WRITE = 0xFF
}



impl interface::Command for Command {
	/// Returns the address of the command
	fn address(self) -> u8 {
	    self as u8
	}
}


#[cfg(test)]
mod tests {
    use super::Command;
	use interface::Command as CommandTrait;

    #[test]
    fn command_addr() {
		assert_eq!(Command::DRIVER_OUTPUT_CONTROL.address(), 0x01);

		//assert_eq!(Command::PANEL_SETTING.addr(), 0x00);

		//assert_eq!(Command::DISPLAY_REFRESH.addr(), 0x12);        
    }
}