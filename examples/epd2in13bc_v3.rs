#![deny(warnings)]

use embedded_graphics::{
    mono_font::MonoTextStyleBuilder,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle},
    text::{Baseline, Text, TextStyleBuilder},
};
use embedded_hal::prelude::*;
use epd_waveshare::{
    color::TriColor,
    epd2in13bc_v3::{Display2in13bc, Epd2in13bc},
    graphics::DisplayRotation,
    prelude::*,
};
use linux_embedded_hal::{
    spidev::{self, SpidevOptions},
    sysfs_gpio::Direction,
    Delay, Pin, Spidev,
};

// The pins in this example are for the Universal e-Paper Raw Panel Driver HAT
// activate spi, gpio in raspi-config
// needs to be run with sudo because of some sysfs_gpio permission problems and follow-up timing problems
// see https://github.com/rust-embedded/rust-sysfs-gpio/issues/5 and follow-up issues

fn main() -> Result<(), std::io::Error> {
    // Configure SPI
    // Settings are taken from
    let mut spi = Spidev::open("/dev/spidev0.0").expect("spidev directory");
    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(4_000_000)
        .mode(spidev::SpiModeFlags::SPI_MODE_0)
        .build();
    spi.configure(&options).expect("spi configuration");

    // Configure Digital I/O Pin to be used as Chip Select for SPI
    let cs = Pin::new(26); //BCM7 CE0
    cs.export().expect("cs export");
    while !cs.is_exported() {}
    cs.set_direction(Direction::Out).expect("CS Direction");
    cs.set_value(1).expect("CS Value set to 1");

    let busy = Pin::new(24); // GPIO 24, board J-18
    busy.export().expect("busy export");
    while !busy.is_exported() {}
    busy.set_direction(Direction::In).expect("busy Direction");
    //busy.set_value(1).expect("busy Value set to 1");

    let dc = Pin::new(25); // GPIO 25, board J-22
    dc.export().expect("dc export");
    while !dc.is_exported() {}
    dc.set_direction(Direction::Out).expect("dc Direction");
    dc.set_value(1).expect("dc Value set to 1");

    let rst = Pin::new(17); // GPIO 17, board J-11
    rst.export().expect("rst export");
    while !rst.is_exported() {}
    rst.set_direction(Direction::Out).expect("rst Direction");
    rst.set_value(1).expect("rst Value set to 1");

    let mut delay = Delay {};

    let mut epd2in13 =
        Epd2in13bc::new(&mut spi, cs, busy, dc, rst, &mut delay).expect("eink initalize error");

    println!("Test all the rotations");
    let mut display = Display2in13bc::default();

    display.set_rotation(DisplayRotation::Rotate0);
    draw_text(&mut display, "Rotate 0!", 5, 5);

    display.set_rotation(DisplayRotation::Rotate90);
    draw_text(&mut display, "Rotate 90!", 5, 5);

    display.set_rotation(DisplayRotation::Rotate180);
    draw_text(&mut display, "Rotate 180!", 5, 5);

    display.set_rotation(DisplayRotation::Rotate270);
    draw_text(&mut display, "Rotate 270!", 5, 5);

    epd2in13.update_frame(&mut spi, display.buffer(), &mut delay)?;
    epd2in13
        .display_frame(&mut spi, &mut delay)
        .expect("display frame new graphics");
    delay.delay_ms(5000u16);

    display.clear_buffer(TriColor::White);
    println!("Now test new graphics with default rotation and some special stuff:");

    // draw a analog clock
    let _ = Circle::with_center(Point::new(52, 52), 80)
        .into_styled(PrimitiveStyle::with_fill(TriColor::Chromatic))
        .draw(&mut display);
    let _ = Circle::with_center(Point::new(52, 52), 80)
        .into_styled(PrimitiveStyle::with_stroke(TriColor::Black, 3))
        .draw(&mut display);
    let _ = Line::new(Point::new(52, 52), Point::new(18, 28))
        .into_styled(PrimitiveStyle::with_stroke(TriColor::White, 5))
        .draw(&mut display);
    let _ = Line::new(Point::new(52, 52), Point::new(68, 28))
        .into_styled(PrimitiveStyle::with_stroke(TriColor::White, 1))
        .draw(&mut display);

    // draw white on black background
    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_6X10)
        .text_color(TriColor::White)
        .background_color(TriColor::Chromatic)
        .build();
    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

    let _ = Text::with_text_style("Hello World!", Point::new(112, 10), style, text_style)
        .draw(&mut display);

    // use bigger/different font
    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_10X20)
        .text_color(TriColor::White)
        .background_color(TriColor::Chromatic)
        .build();

    let _ = Text::with_text_style("Hello\nWorld!", Point::new(112, 40), style, text_style)
        .draw(&mut display);

    epd2in13.update_color_frame(&mut spi, display.bw_buffer(), display.chromatic_buffer()).unwrap();
    epd2in13.display_frame(&mut spi, &mut delay).unwrap();

    println!("Finished tests - going to sleep");
    epd2in13.clear_frame(&mut spi, &mut delay).unwrap();
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
