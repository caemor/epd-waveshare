#![deny(warnings)]

use embedded_graphics::{
    mono_font::MonoTextStyleBuilder,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle},
    text::{Baseline, Text, TextStyleBuilder},
};
use embedded_hal::delay::DelayNs;
use epd_waveshare::{
    color::*,
    epd2in13bc::{Display2in13bc, Epd2in13bc},
    graphics::DisplayRotation,
    prelude::*,
};
use linux_embedded_hal::{
    spidev::{self, SpidevOptions},
    sysfs_gpio::Direction,
    Delay, SPIError, SpidevDevice, SysfsPin,
};

// activate spi, gpio in raspi-config
// needs to be run with sudo because of some sysfs_gpio permission problems and follow-up timing problems
// see https://github.com/rust-embedded/rust-sysfs-gpio/issues/5 and follow-up issues
//
// This example first setups SPI communication using the pin layout found
// at https://www.waveshare.com/wiki/2.13inch_e-Paper_HAT_(B). This example uses the layout for the
// Raspberry Pi Zero (RPI Zero). The Chip Select (CS) was taken from the ep2in9 example since CE0 (GPIO8) did
// not seem to work on RPI Zero with 2.13" HAT
//
// The first frame is filled with four texts at different rotations (black on white)
// The second frame uses a buffer for black/white and a seperate buffer for chromatic/white (i.e. red or yellow)
// This example draws a sample clock in black on white and two texts using white on red.
//
// after finishing, put the display to sleep

fn main() -> Result<(), SPIError> {
    let busy = SysfsPin::new(24); // GPIO 24, board J-18
    busy.export().expect("busy export");
    while !busy.is_exported() {}
    busy.set_direction(Direction::In).expect("busy Direction");

    let dc = SysfsPin::new(25); // GPIO 25, board J-22
    dc.export().expect("dc export");
    while !dc.is_exported() {}
    dc.set_direction(Direction::Out).expect("dc Direction");
    // dc.set_value(1).expect("dc Value set to 1");

    let rst = SysfsPin::new(17); // GPIO 17, board J-11
    rst.export().expect("rst export");
    while !rst.is_exported() {}
    rst.set_direction(Direction::Out).expect("rst Direction");
    // rst.set_value(1).expect("rst Value set to 1");

    // Configure Digital I/O Pin to be used as Chip Select for SPI
    let cs = SysfsPin::new(26); // CE0, board J-24, GPIO 8 -> doesn work. use this from 2in19 example which works
    cs.export().expect("cs export");
    while !cs.is_exported() {}
    cs.set_direction(Direction::Out).expect("CS Direction");
    cs.set_value(1).expect("CS Value set to 1");

    // Configure SPI
    // Settings are taken from
    let mut spi = SpidevDevice::open("/dev/spidev0.0").expect("spidev directory");
    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(10_000_000)
        .mode(spidev::SpiModeFlags::SPI_MODE_0)
        .build();
    spi.configure(&options).expect("spi configuration");

    let mut delay = Delay {};

    let mut epd2in13 =
        Epd2in13bc::new(&mut spi, busy, dc, rst, &mut delay, None).expect("eink initalize error");

    println!("Test all the rotations");
    let mut display = Display2in13bc::default();
    display.clear(TriColor::White).ok();

    display.set_rotation(DisplayRotation::Rotate0);
    draw_text(&mut display, "Rotation 0!", 5, 50);

    display.set_rotation(DisplayRotation::Rotate90);
    draw_text(&mut display, "Rotation 90!", 5, 50);

    display.set_rotation(DisplayRotation::Rotate180);
    draw_text(&mut display, "Rotation 180!", 5, 50);

    display.set_rotation(DisplayRotation::Rotate270);
    draw_text(&mut display, "Rotation 270!", 5, 50);

    // Since we only used black and white, we can resort to updating only
    // the bw-buffer of this tri-color screen

    epd2in13
        .update_and_display_frame(&mut spi, display.bw_buffer(), &mut delay)
        .expect("display frame new graphics");

    println!("First frame done. Waiting 5s");
    delay.delay_ms(5000);

    println!("Now test new graphics with default rotation and three colors:");
    display.clear(TriColor::White).ok();

    // draw a analog clock
    let _ = Circle::with_center(Point::new(64, 64), 80)
        .into_styled(PrimitiveStyle::with_stroke(TriColor::Black, 1))
        .draw(&mut display);
    let _ = Line::new(Point::new(64, 64), Point::new(30, 40))
        .into_styled(PrimitiveStyle::with_stroke(TriColor::Black, 4))
        .draw(&mut display);
    let _ = Line::new(Point::new(64, 64), Point::new(80, 40))
        .into_styled(PrimitiveStyle::with_stroke(TriColor::Black, 1))
        .draw(&mut display);

    // draw text white on Red background by using the chromatic buffer
    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_6X10)
        .text_color(TriColor::White)
        .background_color(TriColor::Chromatic)
        .build();
    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

    let _ = Text::with_text_style("It's working-WoB!", Point::new(90, 10), style, text_style)
        .draw(&mut display);

    // use bigger/different font
    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_10X20)
        .text_color(TriColor::White)
        .background_color(TriColor::Chromatic)
        .build();

    let _ = Text::with_text_style("It's working\nWoB!", Point::new(90, 40), style, text_style)
        .draw(&mut display);

    // we used three colors, so we need to update both bw-buffer and chromatic-buffer

    epd2in13.update_color_frame(
        &mut spi,
        &mut delay,
        display.bw_buffer(),
        display.chromatic_buffer(),
    )?;
    epd2in13
        .display_frame(&mut spi, &mut delay)
        .expect("display frame new graphics");

    println!("Second frame done. Waiting 5s");
    delay.delay_ms(5000);

    // clear both bw buffer and chromatic buffer
    display.clear(TriColor::White).ok();
    epd2in13.update_color_frame(
        &mut spi,
        &mut delay,
        display.bw_buffer(),
        display.chromatic_buffer(),
    )?;
    epd2in13.display_frame(&mut spi, &mut delay)?;

    println!("Finished tests - going to sleep");
    epd2in13.sleep(&mut spi, &mut delay)
}

fn draw_text(display: &mut Display2in13bc, text: &str, x: i32, y: i32) {
    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_6X10)
        .text_color(TriColor::White)
        .background_color(TriColor::Black)
        .build();

    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

    let _ = Text::with_text_style(text, Point::new(x, y), style, text_style).draw(display);
}
