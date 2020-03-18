#![no_main]
#![no_std]

// set the panic handler
#[allow(unused_imports)]
use panic_semihosting;

use cortex_m_rt::entry;
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::{delay, spi};

use embedded_graphics::{
    fonts::{Font12x16, Font6x8, Text},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, Line},
    style::{PrimitiveStyle, Styled},
    text_style, DrawTarget,
};
use epd_waveshare::{
    color::*,
    epd4in2::Display4in2,
    graphics::{Display, DisplayRotation},
    prelude::*,
};

#[entry]
fn main() -> ! {
    let core = cortex_m::Peripherals::take().unwrap();
    let device = stm32f1xx_hal::stm32::Peripherals::take().unwrap();
    let mut rcc = device.RCC.constrain();
    let mut flash = device.FLASH.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(72.mhz())
        .pclk1(36.mhz())
        .freeze(&mut flash.acr);

    let mut gpioa = device.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = device.GPIOB.split(&mut rcc.apb2);

    let mut delay = delay::Delay::new(core.SYST, clocks);

    // spi setup
    let sck = gpiob.pb13.into_alternate_push_pull(&mut gpiob.crh);
    let miso = gpiob.pb14;
    let mosi = gpiob.pb15.into_alternate_push_pull(&mut gpiob.crh);
    let mut spi = spi::Spi::spi2(
        device.SPI2,
        (sck, miso, mosi),
        epd_waveshare::SPI_MODE,
        4.mhz(),
        clocks,
        &mut rcc.apb1,
    );
    // epd setup
    let mut epd4in2 = epd_waveshare::epd4in2::EPD4in2::new(
        &mut spi,
        gpiob.pb12.into_push_pull_output(&mut gpiob.crh),
        gpioa.pa10.into_floating_input(&mut gpioa.crh),
        gpioa.pa8.into_push_pull_output(&mut gpioa.crh),
        gpioa.pa9.into_push_pull_output(&mut gpioa.crh),
        &mut delay,
    )
    .unwrap();
    epd4in2.set_lut(&mut spi, Some(RefreshLUT::QUICK)).unwrap();
    epd4in2.clear_frame(&mut spi).unwrap();

    //println!("Test all the rotations");
    let mut display = Display4in2::default();

    display.set_rotation(DisplayRotation::Rotate0);
    draw_text(&mut display, "Rotate 0!", 5, 50);

    display.set_rotation(DisplayRotation::Rotate90);
    draw_text(&mut display, "Rotate 90!", 5, 50);

    display.set_rotation(DisplayRotation::Rotate180);
    draw_text(&mut display, "Rotate 180!", 5, 50);

    display.set_rotation(DisplayRotation::Rotate270);
    draw_text(&mut display, "Rotate 270!", 5, 50);

    epd4in2.update_frame(&mut spi, &display.buffer()).unwrap();
    epd4in2
        .display_frame(&mut spi)
        .expect("display frame new graphics");
    delay.delay_ms(5000u16);

    //println!("Now test new graphics with default rotation and some special stuff:");
    display.clear_buffer(Color::White);

    // draw a analog clock
    Circle::new(Point::new(64, 64), 64)
        .into_styled(PrimitiveStyle::with_stroke(Black, 1))
        .draw(&mut display);
    Line::new(Point::new(64, 64), Point::new(0, 64))
        .into_styled(PrimitiveStyle::with_stroke(Black, 1))
        .draw(&mut display);
    Line::new(Point::new(64, 64), Point::new(80, 80))
        .into_styled(PrimitiveStyle::with_stroke(Black, 1))
        .draw(&mut display);

    // draw white on black background
    let _ = Text::new("It's working-WoB!", Point::new(175, 250))
        .into_styled(text_style!(
            font = Font6x8,
            text_color = White,
            background_color = Black
        ))
        .draw(&mut display);

    // use bigger/different font
    let _ = Text::new("It's working-WoB!", Point::new(50, 200))
        .into_styled(text_style!(
            font = Font12x16,
            text_color = White,
            background_color = Black
        ))
        .draw(&mut display);

    // a moving `Hello World!`
    let limit = 10;
    epd4in2.set_lut(&mut spi, Some(RefreshLUT::QUICK)).unwrap();
    epd4in2.clear_frame(&mut spi).unwrap();
    for i in 0..limit {
        //println!("Moving Hello World. Loop {} from {}", (i + 1), limit);

        draw_text(&mut display, "  Hello World! ", 5 + i * 12, 50);

        epd4in2.update_frame(&mut spi, &display.buffer()).unwrap();
        epd4in2
            .display_frame(&mut spi)
            .expect("display frame new graphics");

        delay.delay_ms(1_000u16);
    }

    //println!("Finished tests - going to sleep");
    epd4in2.sleep(&mut spi).expect("epd goes to sleep");

    loop {
        // sleep
        cortex_m::asm::wfi();
    }
}

fn draw_text(display: &mut Display4in2, text: &str, x: i32, y: i32) {
    let _ = Text::new(text, Point::new(x, y))
        .into_styled(text_style!(
            font = Font6x8,
            text_color = Black,
            background_color = White
        ))
        .draw(display);
}
