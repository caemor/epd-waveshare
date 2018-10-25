//! Example for EInk-Waveshare and STM32F3
#![deny(unsafe_code)]
#![no_main]
#![no_std]
#![deny(warnings)]


extern crate cortex_m_rt as rt;
extern crate cortex_m;
extern crate stm32f30x_hal;
extern crate panic_semihosting;

use rt::*;
//use cortex_m::asm;
use stm32f30x_hal::prelude::*;
use stm32f30x_hal::spi::Spi;
use stm32f30x_hal::stm32f30x;
use stm32f30x_hal::delay::Delay;
use rt::ExceptionFrame;



// the eink library
extern crate eink_waveshare_rs;


use eink_waveshare_rs::{
    epd1in54::{
        EPD1in54, 
        Buffer1in54,
    },
    graphics::{Display, DisplayRotation},
    prelude::*,
    SPI_MODE,
};

extern crate embedded_graphics;
use embedded_graphics::coord::Coord;
use embedded_graphics::fonts::{Font6x8};
use embedded_graphics::prelude::*;
//use embedded_graphics::primitives::{Circle, Line};
use embedded_graphics::Drawing;

/*
*
* BE CAREFUL: this wasn't tested yet, and the pins are also not choosen correctly (just some random ones atm)
*
*/
#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let p = stm32f30x::Peripherals::take().unwrap();

    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();

    // clock configuration using the default settings (all clocks run at 8 MHz)
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    // TRY this alternate clock configuration (all clocks run at 16 MHz)
    // let clocks = rcc.cfgr.sysclk(16.mhz()).freeze(&mut flash.acr);

    let mut gpioa = p.GPIOA.split(&mut rcc.ahb);
    let mut gpioe = p.GPIOE.split(&mut rcc.ahb);

    // Configure Digital I/O Pin to be used as Chip Select for SPI
    let mut cs = gpioe
        .pe3
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    cs.set_high();

    // Configure Busy Input Pin
    let busy = gpioe
        .pe4
        .into_floating_input(&mut gpioe.moder, &mut gpioe.pupdr);

    // Configure Data/Command OutputPin
    let mut dc = gpioe
        .pe5
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    dc.set_high();

    // Configure Reset OutputPin
    let mut rst = gpioe
        .pe6
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    rst.set_high();

    // Configure Delay
    let mut delay = Delay::new(cp.SYST, clocks);

    // copied from the l3gd20 example
    // The `L3gd20` abstraction exposed by the `f3` crate requires a specific pin configuration to
    // be used and won't accept any configuration other than the one used here. Trying to use a
    // different pin configuration will result in a compiler error.
    //TODO: test if this isn't also working with external connections/read datasheet if it doesn't :-D
    let sck = gpioa.pa5.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let miso = gpioa.pa6.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let mosi = gpioa.pa7.into_af5(&mut gpioa.moder, &mut gpioa.afrl);

    let mut spi = Spi::spi1(
        p.SPI1,
        (sck, miso, mosi),
        SPI_MODE,
        4.mhz(),
        clocks,
        &mut rcc.apb2,
    );

    // // Setup of the needed pins is finished here
    // // Now the "real" usage of the eink-waveshare-rs crate begins
    let mut epd = EPD1in54::new(&mut spi, cs, busy, dc, rst, &mut delay).unwrap();    

    // Clear the full screen
    epd.clear_frame(&mut spi).unwrap();
    epd.display_frame(&mut spi).unwrap();

    // Speeddemo
    let small_buffer =  [Color::Black.get_byte_value(); 16 / 8 * 16];
    let number_of_runs = 100;
    for i in 0..number_of_runs {
        let offset = i * 8 % 150;
        epd.update_partial_frame(&mut spi, &small_buffer, 25 + offset, 25 + offset, 16, 16).unwrap();
        epd.display_frame(&mut spi).unwrap();
    }

    // Clear the full screen
    epd.clear_frame(&mut spi).unwrap();
    epd.display_frame(&mut spi).unwrap();

    // Draw some squares
    let small_buffer =  [Color::Black.get_byte_value(); 160  / 8 * 160 ];
    epd.update_partial_frame(&mut spi, &small_buffer, 20, 20, 160, 160).unwrap();

    let small_buffer =  [Color::White.get_byte_value(); 80  / 8 * 80 ];
    epd.update_partial_frame(&mut spi, &small_buffer, 60, 60, 80, 80).unwrap();

    let small_buffer =  [Color::Black.get_byte_value(); 8];
    epd.update_partial_frame(&mut spi, &small_buffer, 96, 96, 8, 8).unwrap();

    // Display updated frame
    epd.display_frame(&mut spi).expect("display frame new graphics");
    delay.delay_ms(5000u16);


    
    let mut buffer = Buffer1in54::default();
    let mut display = Display::new(epd.width(), epd.height(), &mut buffer.buffer);
    display.set_rotation(DisplayRotation::Rotate0);
    display.draw(
            Font6x8::render_str("Rotate 0!")
                .with_stroke(Some(Color::Black))
                .with_fill(Some(Color::White))                
                .translate(Coord::new(5, 50))
                .into_iter(),
    );

    display.set_rotation(DisplayRotation::Rotate90);
    display.draw(
            Font6x8::render_str("Rotate 90!")
                .with_stroke(Some(Color::Black))
                .with_fill(Some(Color::White))
                .translate(Coord::new(5, 50))
                .into_iter(),
    );

    display.set_rotation(DisplayRotation::Rotate180);
    display.draw(
            Font6x8::render_str("Rotate 180!")
                .with_stroke(Some(Color::Black))
                .with_fill(Some(Color::White))
                .translate(Coord::new(5, 50))
                .into_iter(),
    );

    display.set_rotation(DisplayRotation::Rotate270);
    display.draw(
            Font6x8::render_str("Rotate 270!")
                .with_stroke(Some(Color::Black))
                .with_fill(Some(Color::White))
                .translate(Coord::new(5, 50))
                .into_iter(),
    );

    // Display updated frame
    epd.update_frame(&mut spi, &display.buffer()).unwrap();
    epd.display_frame(&mut spi).expect("display frame new graphics");
    delay.delay_ms(5000u16);

    // Set the EPD to sleep
    epd.sleep(&mut spi).unwrap();

    loop {}
}

//exception!(HardFault, hard_fault);
#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}

//exception!(*, default_handler);
#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}





    


