//! A simple Driver for the Waveshare 2.9" D E-Ink Display via SPI
//!
//!
//! 参考[Waveshare](https://www.waveshare.net/wiki/2.9inch_e-Paper_HAT_%28D%29)的文档/例程进行构建
//!
//! Specification: https://www.waveshare.net/w/upload/b/b5/2.9inch_e-Paper_%28D%29_Specification.pdf

use embedded_hal::{
    blocking::{delay::*, spi::Write},
    digital::v2::*,
};

use crate::interface::DisplayInterface;
use crate::traits::{RefreshLut, WaveshareDisplay};

//The Lookup Tables for the Display
mod constants;
use crate::epd2in9d::constants::*;

/// Width of Epd2in9d in pixels
pub const WIDTH: u32 = 128;
/// Height of Epd2in9d in pixels
pub const HEIGHT: u32 = 296;
/// Default Background Color (white)
pub const DEFAULT_BACKGROUND_COLOR: Color = Color::Black;
const IS_BUSY_LOW: bool = false;
const SINGLE_BYTE_WRITE: bool = true;

use crate::color::Color;

pub(crate) mod command;
use self::command::Command;
use crate::buffer_len;

/// Display with Fullsize buffer for use with the 2in9 EPD D
#[cfg(feature = "graphics")]
pub type Display2in9d = crate::graphics::Display<
    WIDTH,
    HEIGHT,
    false,
    { buffer_len(WIDTH as usize, HEIGHT as usize) },
    Color,
>;

/// Epd2in9d driver
///
pub struct Epd2in9d<SPI, CS, BUSY, DC, RST, DELAY> {
    /// SPI
    interface: DisplayInterface<SPI, CS, BUSY, DC, RST, DELAY, SINGLE_BYTE_WRITE>,
    /// Color
    // background_color: Color,
    color: Color,
    /// Refresh LUT
    refresh: RefreshLut,
}

impl<SPI, CS, BUSY, DC, RST, DELAY> Epd2in9d<SPI, CS, BUSY, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayUs<u32>,
{
    fn init(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.interface.reset(delay, 20_000, 2_000);

        self.interface.cmd(spi, Command::PowerOn)?;
        //waiting for the electronic paper IC to release the idle signal
        self.wait_until_idle(spi, delay)?;

        //panel setting
        //LUT from OTP，KW-BF   KWR-AF	BWROTP 0f	BWOTP 1f
        self.interface
            .cmd_with_data(spi, Command::PanelSetting, &[0x1f])?;

        //resolution setting
        self.interface
            .cmd_with_data(spi, Command::ResolutionSetting, &[0x80, 0x01, 0x28])?;

        //VCOM AND DATA INTERVAL SETTING
        self.interface
            .cmd(spi, Command::VcomAndDataIntervalSetting)?;

        Ok(())
    }
}

impl<SPI, CS, BUSY, DC, RST, DELAY> WaveshareDisplay<SPI, CS, BUSY, DC, RST, DELAY>
    for Epd2in9d<SPI, CS, BUSY, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayUs<u32>,
{
    type DisplayColor = Color;
    fn width(&self) -> u32 {
        WIDTH
    }

    fn height(&self) -> u32 {
        HEIGHT
    }

    fn new(
        spi: &mut SPI,
        cs: CS,
        busy: BUSY,
        dc: DC,
        rst: RST,
        delay: &mut DELAY,
        delay_us: Option<u32>,
    ) -> Result<Self, SPI::Error> {
        let interface = DisplayInterface::new(cs, busy, dc, rst, delay_us);
        let color = DEFAULT_BACKGROUND_COLOR;

        let mut epd = Epd2in9d {
            interface,
            color,
            refresh: RefreshLut::Full,
        };

        epd.init(spi, delay)?;

        Ok(epd)
    }

    fn sleep(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.interface
            .cmd_with_data(spi, Command::VcomAndDataIntervalSetting, &[0xf7])?;
        self.interface.cmd(spi, Command::PowerOff)?;
        self.wait_until_idle(spi, delay)?;
        self.interface
            .cmd_with_data(spi, Command::DeepSleep, &[0xA5])?;

        //TODO: 这还有一个命令没实现，先放着等下回头写
        // DigitalWrite(reset_pin, LOW);

        Ok(())
    }

    fn wake_up(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.init(spi, delay)?;
        Ok(())
    }

    // 对应的是Display函数
    fn update_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        self.wait_until_idle(spi, delay)?;

        self.interface.cmd(spi, Command::DataStartTransmission1)?;
        self.interface.data_x_times(spi, 0x00, WIDTH / 8 * HEIGHT)?;

        // self.interface.cmd(spi,Command::DataStartTransmission2)?;
        // for j in 0..h {
        //     for i in 0..w {
        //         let mut miao = (i+j*w) as u8;
        //         self.interface.data(spi, &buffer[miao])?;
        //     }
        // }
        //TODO: 不太确定这样写对不对
        self.interface
            .cmd_with_data(spi, Command::DataStartTransmission2, buffer)?;
        Ok(())
    }

    // 这个是DisplayPart
    fn update_partial_frame(
        &mut self,
        spi: &mut SPI,
        delay: &mut DELAY,
        buffer: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), SPI::Error> {
        self.wait_until_idle(spi, delay)?;

        self.set_part_reg(spi, delay)?;
        self.interface.cmd(spi, Command::PartialIn)?;

        // 这部分按照例程来说应该是下面这样
        // self.interface.cmd_with_data(
        //     spi,
        //     Command::PartialWindow,
        //     &[
        //         0x00,
        //         (WIDTH - 1) as u8,
        //         0x00,
        //         0x00,
        //         (HEIGHT / 256) as u8,
        //         (HEIGHT % 256 - 1) as u8,
        //         0x28,
        //     ],
        // )?;
        // 但看了下隔壁4in2的代码，决定抄抄
        self.interface.cmd(spi, Command::PartialWindow)?;
        self.interface.data(spi, &[(x >> 8) as u8])?;
        let tmp = x & 0xf8;
        self.interface.data(spi, &[tmp as u8])?; // x should be the multiple of 8, the last 3 bit will always be ignored
        let tmp = tmp + width - 1;
        self.interface.data(spi, &[(tmp >> 8) as u8])?;
        self.interface.data(spi, &[(tmp | 0x07) as u8])?;

        self.interface.data(spi, &[(y >> 8) as u8])?;
        self.interface.data(spi, &[y as u8])?;

        self.interface.data(spi, &[((y + height - 1) >> 8) as u8])?;
        self.interface.data(spi, &[(y + height - 1) as u8])?;

        self.interface.data(spi, &[0x01])?; // Gates scan both inside and outside of the partial window. (default)

        self.interface
            .cmd_with_data(spi, Command::DataStartTransmission2, buffer)?;

        self.turn_on_display(spi, delay)?;
        Ok(())
    }

    /// actually is the "Turn on Display" sequence
    fn display_frame(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.wait_until_idle(spi, delay)?;
        self.interface.cmd(spi, Command::DisplayRefresh)?;
        Ok(())

        // 其实也可以是下面这样
        // self.wait_until_idle(spi, delay)?;
        // self.turn_on_display(spi, delay)?;
    }

    fn update_and_display_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        self.update_frame(spi, buffer, delay)?;
        self.display_frame(spi, delay)?;
        Ok(())
    }

    fn clear_frame(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.wait_until_idle(spi, delay)?;

        // let mut w = if WIDTH % 8 == 0 {
        //     WIDTH / 8
        // } else {
        //     WIDTH / 8 + 1
        // };
        // let mut h = HEIGHT;

        self.interface.cmd(spi, Command::DataStartTransmission1)?;
        // for _ in 0..h {
        //     for _ in 0..w {
        //         self.interface.data(spi, &[0x00])?;
        //     }
        // }
        self.interface.data_x_times(spi, 0x00, WIDTH / 8 * HEIGHT)?;

        self.interface.cmd(spi, Command::DataStartTransmission2)?;
        // for _ in 0..h {
        //     for _ in 0..w {
        //         self.interface.data(spi, &[0xFF])?;
        //     }
        // }
        self.interface.data_x_times(spi, 0xFF, WIDTH / 8 * HEIGHT)?;

        self.turn_on_display(spi, delay)?;

        Ok(())
    }

    fn set_background_color(&mut self, background_color: Color) {
        self.color = background_color;
    }

    fn background_color(&self) -> &Color {
        &self.color
    }

    fn set_lut(
        &mut self,
        spi: &mut SPI,
        delay: &mut DELAY,
        refresh_rate: Option<RefreshLut>,
    ) -> Result<(), SPI::Error> {
        if let Some(refresh_lut) = refresh_rate {
            self.refresh = refresh_lut;
        }
        self.set_lut_helper(
            spi, delay, &LUT_VCOM1, &LUT_WW1, &LUT_BW1, &LUT_WB1, &LUT_BB1,
        )?;

        Ok(())
    }

    fn wait_until_idle(&mut self, _spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.interface.wait_until_idle(delay, IS_BUSY_LOW);
        Ok(())
    }
}

impl<SPI, CS, BUSY, DC, RST, DELAY> Epd2in9d<SPI, CS, BUSY, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayUs<u32>,
{
    fn turn_on_display(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        //DISPLAY REFRESH
        self.interface.cmd(spi, Command::DisplayRefresh)?;
        //The delay here is necessary, 200uS at least!!!
        delay.delay_us(200);

        self.wait_until_idle(spi, delay)?;
        Ok(())
    }

    /// 唤醒屏幕
    ///
    /// 在屏幕执行sleep之后，会进入深度睡眠模式。在深度睡眠模式下若需要刷新屏幕，必须先执行awaken()
    /// 唤醒屏幕
    // fn awaken(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
    //     // reset the device
    //     self.interface.reset(delay, 20_000, 2_000);
    //     self.wait_until_idle(spi, delay)?;

    //     // panel setting
    //     // LUT from OTP，KW-BF   KWR-AF	BWROTP 0f	BWOTP 1f
    //     self.interface
    //         .cmd_with_data(spi, Command::PanelSetting, &[0x1f])?;

    //     // resolution setting
    //     self.interface
    //         .cmd_with_data(spi, Command::ResolutionSetting, &[0x80, 0x01, 0x28])?;

    //     // VCOM AND DATA INTERVAL SETTING
    //     self.interface
    //         .cmd(spi, Command::VcomAndDataIntervalSetting)?;
    //     Ok(())
    // }

    fn set_part_reg(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        // 重置EPD驱动电路
        //TODO: 这里在微雪的例程中反复刷新了3次，后面有显示问题再进行修改
        self.interface.reset(delay, 20_000, 2_000);

        // 电源设置
        //TODO: 文档中的数据为[0x03,0x00,0x2b,0x2b,0x09]
        self.interface.cmd_with_data(
            spi,
            Command::PowerSetting,
            &[0x03, 0x00, 0x2b, 0x2b, 0x03],
        )?;

        // 软启动
        self.interface
            .cmd_with_data(spi, Command::BoosterSoftStart, &[0x17, 0x17, 0x17])?;

        // 开启电源
        // self.interface.cmd_with_data(
        //     spi,
        //     Command::PowerOn,
        //     &[0x04],
        // );
        self.interface.cmd(spi, Command::PowerOn)?;

        // 获取BUSY电平，高电平继续执行，低电平则等待屏幕响应
        //TODO: 这里是文档推荐的步骤，但我看其他屏幕的也没等待就先忽略了
        // self.wait_until_idle(spi, delay)?;

        // 面板设置
        self.interface
            .cmd_with_data(spi, Command::PanelSetting, &[0xbf])?;

        // 设置刷新率
        // 3a 100HZ | 29 150Hz | 39 200HZ | 31 171HZ
        // 例程中使用3a
        self.interface
            .cmd_with_data(spi, Command::PllControl, &[0x3a])?;

        // 分辨率设置
        self.interface
            .cmd_with_data(spi, Command::ResolutionSetting, &[0x80, 0x01, 0x28])?;

        // vcom_DC设置
        self.interface
            .cmd_with_data(spi, Command::VcmDcSetting, &[0x12])?;

        // vcom和数据间隔设置
        self.interface
            .cmd_with_data(spi, Command::VcomAndDataIntervalSetting, &[0x97])?;

        self.set_lut(spi, delay, None)?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn set_lut_helper(
        &mut self,
        spi: &mut SPI,
        delay: &mut DELAY,
        lut_vcom: &[u8],
        lut_ww: &[u8],
        lut_bw: &[u8],
        lut_wb: &[u8],
        lut_bb: &[u8],
    ) -> Result<(), SPI::Error> {
        self.wait_until_idle(spi, delay)?;
        // LUT VCOM
        self.interface
            .cmd_with_data(spi, Command::LutForVcom, lut_vcom)?;

        // LUT WHITE to WHITE
        self.interface
            .cmd_with_data(spi, Command::LutWhiteToWhite, lut_ww)?;

        // LUT BLACK to WHITE
        self.interface
            .cmd_with_data(spi, Command::LutBlackToWhite, lut_bw)?;

        // LUT WHITE to BLACK
        self.interface
            .cmd_with_data(spi, Command::LutWhiteToBlack, lut_wb)?;

        // LUT BLACK to BLACK
        self.interface
            .cmd_with_data(spi, Command::LutBlackToBlack, lut_bb)?;
        Ok(())
    }
}

// impl<SPI, CS, BUSY, DC, RST, DELAY> QuickRefresh<SPI, CS, BUSY, DC, RST, DELAY>
//     for Epd2in9d<SPI, CS, BUSY, DC, RST, DELAY>
// where
//     SPI: Write<u8>,
//     CS: OutputPin,
//     BUSY: InputPin,
//     DC: OutputPin,
//     RST: OutputPin,
//     DELAY: DelayUs<u32>,
// {
//     /// To be followed immediately by `update_new_frame`.
//     fn update_old_frame(
//         &mut self,
//         spi: &mut SPI,
//         buffer: &[u8],
//         delay: &mut DELAY,
//     ) -> Result<(), SPI::Error> {
//         self.wait_until_idle(spi, delay)?;
//         self.interface
//             .cmd_with_data(spi, Command::WriteRam2, buffer)
//     }

//     /// To be used immediately after `update_old_frame`.
//     fn update_new_frame(
//         &mut self,
//         spi: &mut SPI,
//         buffer: &[u8],
//         delay: &mut DELAY,
//     ) -> Result<(), SPI::Error> {
//         self.wait_until_idle(spi, delay)?;
//         self.interface.reset(delay, 10_000, 2_000);

//         self.set_lut_helper(spi, delay, &LUT_PARTIAL_2IN9)?;
//         self.interface.cmd_with_data(
//             spi,
//             Command::WriteOtpSelection,
//             &[0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00],
//         )?;
//         self.interface
//             .cmd_with_data(spi, Command::BorderWaveformControl, &[0x80])?;
//         self.interface
//             .cmd_with_data(spi, Command::DisplayUpdateControl2, &[0xC0])?;
//         self.interface.cmd(spi, Command::MasterActivation)?;

//         self.wait_until_idle(spi, delay)?;

//         self.use_full_frame(spi, delay)?;

//         self.interface
//             .cmd_with_data(spi, Command::WriteRam, buffer)?;
//         Ok(())
//     }

//     /// For a quick refresh of the new updated frame. To be used immediately after `update_new_frame`
//     fn display_new_frame(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
//         self.wait_until_idle(spi, delay)?;
//         self.interface
//             .cmd_with_data(spi, Command::DisplayUpdateControl2, &[0x0F])?;
//         self.interface.cmd(spi, Command::MasterActivation)?;
//         self.wait_until_idle(spi, delay)?;
//         Ok(())
//     }

//     /// Updates and displays the new frame.
//     fn update_and_display_new_frame(
//         &mut self,
//         spi: &mut SPI,
//         buffer: &[u8],
//         delay: &mut DELAY,
//     ) -> Result<(), SPI::Error> {
//         self.update_new_frame(spi, buffer, delay)?;
//         self.display_new_frame(spi, delay)?;
//         Ok(())
//     }

//     /// Partial quick refresh not supported yet
//     #[allow(unused)]
//     fn update_partial_old_frame(
//         &mut self,
//         spi: &mut SPI,
//         delay: &mut DELAY,
//         buffer: &[u8],
//         x: u32,
//         y: u32,
//         width: u32,
//         height: u32,
//     ) -> Result<(), SPI::Error> {
//         //TODO supported by display?
//         unimplemented!()
//     }

//     /// Partial quick refresh not supported yet
//     #[allow(unused)]
//     fn update_partial_new_frame(
//         &mut self,
//         spi: &mut SPI,
//         delay: &mut DELAY,
//         buffer: &[u8],
//         x: u32,
//         y: u32,
//         width: u32,
//         height: u32,
//     ) -> Result<(), SPI::Error> {
//         //TODO supported by display?
//         unimplemented!()
//     }

//     /// Partial quick refresh not supported yet
//     #[allow(unused)]
//     fn clear_partial_frame(
//         &mut self,
//         spi: &mut SPI,
//         delay: &mut DELAY,
//         x: u32,
//         y: u32,
//         width: u32,
//         height: u32,
//     ) -> Result<(), SPI::Error> {
//         //TODO supported by display?
//         unimplemented!()
//     }
// }
