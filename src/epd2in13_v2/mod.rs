//! A Driver for the Waveshare 2.13" E-Ink Display (V2) via SPI
//!
//! # References
//!
//! - [Waveshare product page](https://www.waveshare.com/wiki/2.13inch_e-Paper_HAT)
//! - [Waveshare C driver](https://github.com/waveshare/e-Paper/blob/master/RaspberryPi%26JetsonNano/c/lib/e-Paper/EPD_2in13_V2.c)
//! - [Waveshare Python driver](https://github.com/waveshare/e-Paper/blob/master/RaspberryPi%26JetsonNano/python/lib/waveshare_epd/epd2in13_V2.py)
//! - [Controller Datasheet SS1780](http://www.e-paper-display.com/download_detail/downloadsId=682.html)
//!

use embedded_hal::{
    blocking::{delay::*, spi::Write},
    digital::v2::{InputPin, OutputPin},
};

use crate::buffer_len;
use crate::color::Color;
use crate::interface::DisplayInterface;
use crate::traits::{InternalWiAdditions, RefreshLUT, WaveshareDisplay};

pub(crate) mod command;
use self::command::{
    BorderWaveForm, BorderWaveFormFixLevel, BorderWaveFormGS, BorderWaveFormVBD, Command,
    DataEntryModeDir, DataEntryModeIncr, DeepSleepMode, DisplayUpdateControl2, DriverOutput,
    GateDrivingVoltage, I32Ext, SourceDrivingVoltage, VCOM,
};

pub(crate) mod constants;
use self::constants::{LUT_FULL_UPDATE, LUT_PARTIAL_UPDATE};

#[cfg(feature = "graphics")]
mod graphics;
#[cfg(feature = "graphics")]
pub use self::graphics::Display2in13;

/// Width of the display.
pub const WIDTH: u32 = 122;

/// Height of the display
pub const HEIGHT: u32 = 250;

/// Default Background Color
pub const DEFAULT_BACKGROUND_COLOR: Color = Color::White;
const IS_BUSY_LOW: bool = false;

/// EPD2in13 (V2) driver
///
pub struct EPD2in13<SPI, CS, BUSY, DC, RST> {
    /// Connection Interface
    interface: DisplayInterface<SPI, CS, BUSY, DC, RST>,

    sleep_mode: DeepSleepMode,

    /// Background Color
    background_color: Color,
    refresh: RefreshLUT,
}

impl<SPI, CS, BUSY, DC, RST> InternalWiAdditions<SPI, CS, BUSY, DC, RST>
    for EPD2in13<SPI, CS, BUSY, DC, RST>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    fn init<DELAY: DelayMs<u8>>(
        &mut self,
        spi: &mut SPI,
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        // HW reset
        self.interface.reset(delay);

        if self.refresh == RefreshLUT::QUICK {
            self.set_vcom_register(spi, (-9).vcom())?;
            self.wait_until_idle();

            self.set_lut(spi, Some(self.refresh))?;

            // Python code does this, not sure why
            // self.cmd_with_data(spi, Command::WRITE_OTP_SELECTION, &[0, 0, 0, 0, 0x40, 0, 0])?;

            // During partial update, clock/analog are not disabled between 2
            // updates.
            self.set_display_update_control_2(
                spi,
                DisplayUpdateControl2::new().enable_analog().enable_clock(),
            )?;
            self.command(spi, Command::MASTER_ACTIVATION)?;
            self.wait_until_idle();

            self.set_border_waveform(
                spi,
                BorderWaveForm {
                    vbd: BorderWaveFormVBD::GS,
                    fix_level: BorderWaveFormFixLevel::VSS,
                    gs_trans: BorderWaveFormGS::LUT1,
                },
            )?;
        } else {
            self.wait_until_idle();
            self.command(spi, Command::SW_RESET)?;
            self.wait_until_idle();

            self.set_driver_output(
                spi,
                DriverOutput {
                    scan_is_linear: true,
                    scan_g0_is_first: true,
                    scan_dir_incr: true,
                    width: (HEIGHT - 1) as u16,
                },
            )?;

            // These 2 are the reset values
            self.set_dummy_line_period(spi, 0x30)?;
            self.set_gate_scan_start_position(spi, 0)?;

            self.set_data_entry_mode(
                spi,
                DataEntryModeIncr::X_INCR_Y_INCR,
                DataEntryModeDir::X_DIR,
            )?;

            // Use simple X/Y auto increase
            self.set_ram_area(spi, 0, 0, WIDTH - 1, HEIGHT - 1)?;
            self.set_ram_address_counters(spi, 0, 0)?;

            self.set_border_waveform(
                spi,
                BorderWaveForm {
                    vbd: BorderWaveFormVBD::GS,
                    fix_level: BorderWaveFormFixLevel::VSS,
                    gs_trans: BorderWaveFormGS::LUT3,
                },
            )?;

            self.set_vcom_register(spi, (-21).vcom())?;

            self.set_gate_driving_voltage(spi, 190.gate_driving_decivolt())?;
            self.set_source_driving_voltage(
                spi,
                150.source_driving_decivolt(),
                50.source_driving_decivolt(),
                (-150).source_driving_decivolt(),
            )?;

            self.set_gate_line_width(spi, 10)?;

            self.set_lut(spi, Some(self.refresh))?;
        }

        self.wait_until_idle();
        Ok(())
    }
}

impl<SPI, CS, BUSY, DC, RST> WaveshareDisplay<SPI, CS, BUSY, DC, RST>
    for EPD2in13<SPI, CS, BUSY, DC, RST>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    fn new<DELAY: DelayMs<u8>>(
        spi: &mut SPI,
        cs: CS,
        busy: BUSY,
        dc: DC,
        rst: RST,
        delay: &mut DELAY,
    ) -> Result<Self, SPI::Error> {
        let mut epd = EPD2in13 {
            interface: DisplayInterface::new(cs, busy, dc, rst),
            sleep_mode: DeepSleepMode::MODE_1,
            background_color: DEFAULT_BACKGROUND_COLOR,
            refresh: RefreshLUT::FULL,
        };

        epd.init(spi, delay)?;
        Ok(epd)
    }

    fn wake_up<DELAY: DelayMs<u8>>(
        &mut self,
        spi: &mut SPI,
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        self.init(spi, delay)
    }

    fn sleep(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        self.wait_until_idle();

        // All sample code enables and disables analog/clocks...
        self.set_display_update_control_2(
            spi,
            DisplayUpdateControl2::new()
                .enable_analog()
                .enable_clock()
                .disable_analog()
                .disable_clock(),
        )?;
        self.command(spi, Command::MASTER_ACTIVATION)?;

        self.set_sleep_mode(spi, self.sleep_mode)?;
        Ok(())
    }

    fn update_frame(&mut self, spi: &mut SPI, buffer: &[u8]) -> Result<(), SPI::Error> {
        assert!(buffer.len() == buffer_len(WIDTH as usize, HEIGHT as usize));
        self.set_ram_area(spi, 0, 0, WIDTH - 1, HEIGHT - 1)?;
        self.set_ram_address_counters(spi, 0, 0)?;

        self.cmd_with_data(spi, Command::WRITE_RAM, buffer)?;

        if self.refresh == RefreshLUT::FULL {
            // Always keep the base buffer equal to current if not doing partial refresh.
            self.set_ram_area(spi, 0, 0, WIDTH - 1, HEIGHT - 1)?;
            self.set_ram_address_counters(spi, 0, 0)?;

            self.cmd_with_data(spi, Command::WRITE_RAM_RED, buffer)?;
        }
        Ok(())
    }

    /// Updating only a part of the frame is not supported when using the
    /// partial refresh feature. The function will panic if called when set to
    /// use partial refresh.
    fn update_partial_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), SPI::Error> {
        assert!((width * height / 8) as usize == buffer.len());

        // This should not be used when doing partial refresh. The RAM_RED must
        // be updated with the last buffer having been displayed. Doing partial
        // update directly in RAM makes this update impossible (we can't read
        // RAM content). Using this function will most probably make the actual
        // display incorrect as the controler will compare with something
        // incorrect.
        assert!(self.refresh == RefreshLUT::FULL);

        self.set_ram_area(spi, x, y, x + width, y + height)?;
        self.set_ram_address_counters(spi, x, y)?;

        self.cmd_with_data(spi, Command::WRITE_RAM, buffer)?;

        if self.refresh == RefreshLUT::FULL {
            // Always keep the base buffer equals to current if not doing partial refresh.
            self.set_ram_area(spi, x, y, x + width, y + height)?;
            self.set_ram_address_counters(spi, x, y)?;

            self.cmd_with_data(spi, Command::WRITE_RAM_RED, buffer)?;
        }

        Ok(())
    }

    /// Never use directly this function when using partial refresh, or also
    /// keep the base buffer in syncd using `set_partial_base_buffer` function.
    fn display_frame(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        if self.refresh == RefreshLUT::FULL {
            self.set_display_update_control_2(
                spi,
                DisplayUpdateControl2::new()
                    .enable_clock()
                    .enable_analog()
                    .display()
                    .disable_analog()
                    .disable_clock(),
            )?;
        } else {
            self.set_display_update_control_2(spi, DisplayUpdateControl2::new().display())?;
        }
        self.command(spi, Command::MASTER_ACTIVATION)?;
        self.wait_until_idle();

        Ok(())
    }

    fn update_and_display_frame(&mut self, spi: &mut SPI, buffer: &[u8]) -> Result<(), SPI::Error> {
        self.update_frame(spi, buffer)?;
        self.display_frame(spi)?;

        if self.refresh == RefreshLUT::QUICK {
            self.set_partial_base_buffer(spi, buffer)?;
        }
        Ok(())
    }

    fn clear_frame(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        let color = self.background_color.get_byte_value();

        self.set_ram_area(spi, 0, 0, WIDTH - 1, HEIGHT - 1)?;
        self.set_ram_address_counters(spi, 0, 0)?;

        self.command(spi, Command::WRITE_RAM)?;
        self.interface.data_x_times(
            spi,
            color,
            buffer_len(WIDTH as usize, HEIGHT as usize) as u32,
        )?;

        // Always keep the base buffer equals to current if not doing partial refresh.
        if self.refresh == RefreshLUT::FULL {
            self.set_ram_area(spi, 0, 0, WIDTH - 1, HEIGHT - 1)?;
            self.set_ram_address_counters(spi, 0, 0)?;

            self.command(spi, Command::WRITE_RAM_RED)?;
            self.interface.data_x_times(
                spi,
                color,
                buffer_len(WIDTH as usize, HEIGHT as usize) as u32,
            )?;
        }
        Ok(())
    }

    fn set_background_color(&mut self, background_color: Color) {
        self.background_color = background_color;
    }

    fn background_color(&self) -> &Color {
        &self.background_color
    }

    fn width(&self) -> u32 {
        WIDTH
    }

    fn height(&self) -> u32 {
        HEIGHT
    }

    fn set_lut(
        &mut self,
        spi: &mut SPI,
        refresh_rate: Option<RefreshLUT>,
    ) -> Result<(), SPI::Error> {
        let buffer = match refresh_rate {
            Some(RefreshLUT::FULL) | None => &LUT_FULL_UPDATE,
            Some(RefreshLUT::QUICK) => &LUT_PARTIAL_UPDATE,
        };

        self.cmd_with_data(spi, Command::WRITE_LUT_REGISTER, buffer)
    }

    fn is_busy(&self) -> bool {
        self.interface.is_busy(IS_BUSY_LOW)
    }
}

impl<SPI, CS, BUSY, DC, RST> EPD2in13<SPI, CS, BUSY, DC, RST>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    /// When using partial refresh, the controller uses the provided buffer for
    /// comparison with new buffer.
    pub fn set_partial_base_buffer(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
    ) -> Result<(), SPI::Error> {
        assert!(buffer_len(WIDTH as usize, HEIGHT as usize) == buffer.len());
        self.set_ram_area(spi, 0, 0, WIDTH - 1, HEIGHT - 1)?;
        self.set_ram_address_counters(spi, 0, 0)?;

        self.cmd_with_data(spi, Command::WRITE_RAM_RED, buffer)?;
        Ok(())
    }

    /// Selects which sleep mode will be used when triggering the deep sleep.
    pub fn set_deep_sleep_mode(&mut self, mode: DeepSleepMode) {
        self.sleep_mode = mode;
    }

    /// Sets the refresh mode. When changing mode, the screen will be
    /// re-initialized accordingly.
    pub fn set_refresh<DELAY: DelayMs<u8>>(
        &mut self,
        spi: &mut SPI,
        delay: &mut DELAY,
        refresh: RefreshLUT,
    ) -> Result<(), SPI::Error> {
        if self.refresh != refresh {
            self.refresh = refresh;
            self.init(spi, delay)?;
        }
        Ok(())
    }

    fn set_gate_scan_start_position(
        &mut self,
        spi: &mut SPI,
        start: u16,
    ) -> Result<(), SPI::Error> {
        assert!(start <= 295);
        self.cmd_with_data(
            spi,
            Command::GATE_SCAN_START_POSITION,
            &[(start & 0xFF) as u8, ((start >> 8) & 0x1) as u8],
        )
    }

    fn set_border_waveform(
        &mut self,
        spi: &mut SPI,
        borderwaveform: BorderWaveForm,
    ) -> Result<(), SPI::Error> {
        self.cmd_with_data(
            spi,
            Command::BORDER_WAVEFORM_CONTROL,
            &[borderwaveform.to_u8()],
        )
    }

    fn set_vcom_register(&mut self, spi: &mut SPI, vcom: VCOM) -> Result<(), SPI::Error> {
        self.cmd_with_data(spi, Command::WRITE_VCOM_REGISTER, &[vcom.0])
    }

    fn set_gate_driving_voltage(
        &mut self,
        spi: &mut SPI,
        voltage: GateDrivingVoltage,
    ) -> Result<(), SPI::Error> {
        self.cmd_with_data(spi, Command::GATE_DRIVING_VOLTAGE_CTRL, &[voltage.0])
    }

    fn set_dummy_line_period(
        &mut self,
        spi: &mut SPI,
        number_of_lines: u8,
    ) -> Result<(), SPI::Error> {
        assert!(number_of_lines <= 127);
        self.cmd_with_data(spi, Command::SET_DUMMY_LINE_PERIOD, &[number_of_lines])
    }

    fn set_gate_line_width(&mut self, spi: &mut SPI, width: u8) -> Result<(), SPI::Error> {
        self.cmd_with_data(spi, Command::SET_GATE_LINE_WIDTH, &[width & 0x0F])
    }

    /// Sets the source driving voltage value
    fn set_source_driving_voltage(
        &mut self,
        spi: &mut SPI,
        vsh1: SourceDrivingVoltage,
        vsh2: SourceDrivingVoltage,
        vsl: SourceDrivingVoltage,
    ) -> Result<(), SPI::Error> {
        self.cmd_with_data(
            spi,
            Command::SOURCE_DRIVING_VOLTAGE_CTRL,
            &[vsh1.0, vsh2.0, vsl.0],
        )
    }

    /// Prepare the actions that the next master activation command will
    /// trigger.
    fn set_display_update_control_2(
        &mut self,
        spi: &mut SPI,
        value: DisplayUpdateControl2,
    ) -> Result<(), SPI::Error> {
        self.cmd_with_data(spi, Command::DISPLAY_UPDATE_CONTROL_2, &[value.0])
    }

    /// Triggers the deep sleep mode
    fn set_sleep_mode(&mut self, spi: &mut SPI, mode: DeepSleepMode) -> Result<(), SPI::Error> {
        self.cmd_with_data(spi, Command::DEEP_SLEEP_MODE, &[mode as u8])
    }

    fn set_driver_output(&mut self, spi: &mut SPI, output: DriverOutput) -> Result<(), SPI::Error> {
        self.cmd_with_data(spi, Command::DRIVER_OUTPUT_CONTROL, &output.to_bytes())
    }

    /// Sets the data entry mode (ie. how X and Y positions changes when writing
    /// data to RAM)
    fn set_data_entry_mode(
        &mut self,
        spi: &mut SPI,
        counter_incr_mode: DataEntryModeIncr,
        counter_direction: DataEntryModeDir,
    ) -> Result<(), SPI::Error> {
        let mode = counter_incr_mode as u8 | counter_direction as u8;
        self.cmd_with_data(spi, Command::DATA_ENTRY_MODE_SETTING, &[mode])
    }

    /// Sets both X and Y pixels ranges
    fn set_ram_area(
        &mut self,
        spi: &mut SPI,
        start_x: u32,
        start_y: u32,
        end_x: u32,
        end_y: u32,
    ) -> Result<(), SPI::Error> {
        self.cmd_with_data(
            spi,
            Command::SET_RAM_X_ADDRESS_START_END_POSITION,
            &[(start_x >> 3) as u8, (end_x >> 3) as u8],
        )?;

        self.cmd_with_data(
            spi,
            Command::SET_RAM_Y_ADDRESS_START_END_POSITION,
            &[
                start_y as u8,
                (start_y >> 8) as u8,
                end_y as u8,
                (end_y >> 8) as u8,
            ],
        )
    }

    /// Sets both X and Y pixels counters when writing data to RAM
    fn set_ram_address_counters(
        &mut self,
        spi: &mut SPI,
        x: u32,
        y: u32,
    ) -> Result<(), SPI::Error> {
        self.wait_until_idle();
        self.cmd_with_data(spi, Command::SET_RAM_X_ADDRESS_COUNTER, &[(x >> 3) as u8])?;

        self.cmd_with_data(
            spi,
            Command::SET_RAM_Y_ADDRESS_COUNTER,
            &[y as u8, (y >> 8) as u8],
        )?;
        Ok(())
    }

    fn command(&mut self, spi: &mut SPI, command: Command) -> Result<(), SPI::Error> {
        self.interface.cmd(spi, command)
    }

    fn cmd_with_data(
        &mut self,
        spi: &mut SPI,
        command: Command,
        data: &[u8],
    ) -> Result<(), SPI::Error> {
        self.interface.cmd_with_data(spi, command, data)
    }

    fn wait_until_idle(&mut self) {
        self.interface.wait_until_idle(IS_BUSY_LOW)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epd_size() {
        assert_eq!(WIDTH, 122);
        assert_eq!(HEIGHT, 250);
        assert_eq!(DEFAULT_BACKGROUND_COLOR, Color::White);
    }
}
