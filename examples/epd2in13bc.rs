#![deny(warnings)]

use embedded_graphics::{
    fonts::{Font12x16, Font6x8, Text},
    prelude::*,
    primitives::{Circle, Line},
    style::PrimitiveStyle,
    text_style,
};
use embedded_hal::prelude::*;
use epd_waveshare::{
    color::*,
    epd2in13bc::{Display2in13bc, Epd2in13bc},
    graphics::{Display, DisplayRotation},
    prelude::*,
};
use linux_embedded_hal::{
    spidev::{self, SpidevOptions},
    sysfs_gpio::Direction,
    Delay, Pin, Spidev,
};

// activate spi, gpio in raspi-config
// needs to be run with sudo because of some sysfs_gpio permission problems and follow-up timing problems
// see https://github.com/rust-embedded/rust-sysfs-gpio/issues/5 and follow-up issues

fn main() -> Result<(), std::io::Error> {
    let busy = Pin::new(24); // GPIO 24, board J-18
    busy.export().expect("busy export");
    while !busy.is_exported() {}
    busy.set_direction(Direction::In).expect("busy Direction");

    let dc = Pin::new(25); // GPIO 25, board J-22
    dc.export().expect("dc export");
    while !dc.is_exported() {}
    dc.set_direction(Direction::Out).expect("dc Direction");
    // dc.set_value(1).expect("dc Value set to 1");

    let rst = Pin::new(17); // GPIO 17, board J-11
    rst.export().expect("rst export");
    while !rst.is_exported() {}
    rst.set_direction(Direction::Out).expect("rst Direction");
    // rst.set_value(1).expect("rst Value set to 1");

    // Configure Digital I/O Pin to be used as Chip Select for SPI
    let cs = Pin::new(26); // CE0, board J-24, GPIO 8 -> doesn work. use this from 2in19 example which works
    cs.export().expect("cs export");
    while !cs.is_exported() { }
    cs.set_direction(Direction::Out).expect("CS Direction");
    cs.set_value(1).expect("CS Value set to 1");

    // Configure SPI
    // Settings are taken from
    let mut spi = Spidev::open("/dev/spidev0.0").expect("spidev directory");
    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(10_000_000)
        .mode(spidev::SpiModeFlags::SPI_MODE_0)
        .build();
    spi.configure(&options).expect("spi configuration");

    let mut delay = Delay {};

    let mut epd2in13 =
        Epd2in13bc::new(&mut spi, cs, busy, dc, rst, &mut delay).expect("eink initalize error");

    println!("Test all the rotations");
    let mut display = Display2in13bc::default();
    let mut display_chromatic = Display2in13bc::default();

    display.set_rotation(DisplayRotation::Rotate0);
    draw_text(&mut display, "Rotation 0!", 5, 50);

    display.set_rotation(DisplayRotation::Rotate90);
    draw_text(&mut display, "Rotation 90!", 5, 50);

    display.set_rotation(DisplayRotation::Rotate180);
    draw_text(&mut display, "Rotation 180!", 5, 50);

    display.set_rotation(DisplayRotation::Rotate270);
    draw_text(&mut display, "Rotation 270!", 5, 50);

    epd2in13.update_and_display_frame(&mut spi, &display.buffer(), &mut delay)
        .expect("display frame new graphics");

    println!("First frame done. Waiting 5s");
    delay.delay_ms(5000u16);

    println!("Now test new graphics with default rotation:");
    display.clear_buffer(Color::White);
    display_chromatic.clear_buffer(Color::White);
    // keep both displays on same rotation
    display_chromatic.set_rotation(DisplayRotation::Rotate270); 

    // draw a analog clock
    let _ = Circle::new(Point::new(64, 	64), 40)
        .into_styled(PrimitiveStyle::with_stroke(Black, 1))
        .draw(&mut display);
    let _ = Line::new(Point::new(64, 64), Point::new(30, 40))
        .into_styled(PrimitiveStyle::with_stroke(Black, 4))
        .draw(&mut display);
    let _ = Line::new(Point::new(64, 64), Point::new(80, 40))
        .into_styled(PrimitiveStyle::with_stroke(Black, 1))
        .draw(&mut display);

    // draw white on Red background
    let _ = Text::new("It's working-WoB!", Point::new(90, 10))
        .into_styled(text_style!(
            font = Font6x8,
            text_color = White,
            background_color = Black
        ))
        .draw(&mut display_chromatic);

    // use bigger/different font
    let _ = Text::new("It's working-WoB!", Point::new(90, 40))
        .into_styled(text_style!(
            font = Font12x16,
            text_color = White,
            background_color = Black
        ))
        .draw(&mut display_chromatic);

    epd2in13.update_color_frame(&mut spi, &display.buffer(), &display_chromatic.buffer())?;
    epd2in13
        .display_frame(&mut spi, &mut delay)
        .expect("display frame new graphics");
    println!("Second frame done. Waiting 5s");
    delay.delay_ms(5000u16);

    display.clear_buffer(Color::White);
    display_chromatic.clear_buffer(Color::White);
    epd2in13.update_color_frame(&mut spi, &display.buffer(), &display_chromatic.buffer())?;
    epd2in13.display_frame(&mut spi, &mut delay)?;

    println!("Finished tests - going to sleep"); 
    epd2in13.sleep(&mut spi, &mut delay)
}

fn draw_text(display: &mut Display2in13bc, text: &str, x: i32, y: i32) {
    let _ = Text::new(text, Point::new(x, y))
        .into_styled(text_style!(
            font = Font6x8,
            text_color = Black,
            background_color = White
        ))
        .draw(display);
}
