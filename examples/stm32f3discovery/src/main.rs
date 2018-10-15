//! Example for EInk-Waveshare and STM32F3
#![deny(unsafe_code)]
#![no_main]
#![no_std]
//#![deny(warnings)]


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

//entry!(main);


// the eink library
extern crate eink_waveshare_rs;


use eink_waveshare_rs::{
    EPD1in54,
    SPI_MODE,
    //drawing_old::{Graphics},
    color::Color,
    WaveshareDisplay,
};




// use lin_hal::spidev::{self, SpidevOptions};
// use lin_hal::{Pin, Spidev};
// use lin_hal::sysfs_gpio::Direction;
// use lin_hal::Delay;

// activate spi, gpio in raspi-config
// needs to be run with sudo because of some sysfs_gpio permission problems and follow-up timing problems
// see https://github.com/rust-embedded/rust-sysfs-gpio/issues/5 and follow-up issues

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

    // // Configure Digital I/O Pin to be used as Chip Select for SPI
    let mut cs = gpioe
        .pe3
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    cs.set_high();

    // // Configure Busy Input Pin
    let busy = gpioe
        .pe4
        .into_floating_input(&mut gpioe.moder, &mut gpioe.pupdr);
   //     .pe4
    //    .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    
    // let busy = Pin::new(5);//pin 29
    // busy.export().expect("busy export");
    // while !busy.is_exported() {}
    // busy.set_direction(Direction::In).expect("busy Direction");
    // //busy.set_value(1).expect("busy Value set to 1");
    // let busy_in = HackInputPin::new(&busy);

    // // Configure Data/Command OutputPin
    let mut dc = gpioe
        .pe5
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    dc.set_high();
    // let dc = Pin::new(6); //pin 31 //bcm6
    // dc.export().expect("dc export");
    // while !dc.is_exported() {}
    // dc.set_direction(Direction::Out).expect("dc Direction");
    // dc.set_value(1).expect("dc Value set to 1");

    let mut rst = gpioe
        .pe6
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    rst.set_high();
    // // Configure Reset OutputPin
    // let rst = Pin::new(16); //pin 36 //bcm16
    // rst.export().expect("rst export");
    // while !rst.is_exported() {}
    // rst.set_direction(Direction::Out).expect("rst Direction");
    // rst.set_value(1).expect("rst Value set to 1");   

    // // Configure Delay
    let mut delay = Delay::new(cp.SYST, clocks);

    // copied from the l3gd20 example
    // The `L3gd20` abstraction exposed by the `f3` crate requires a specific pin configuration to
    // be used and won't accept any configuration other than the one used here. Trying to use a
    // different pin configuration will result in a compiler error.
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
    let small_buffer =  [Color::Black.get_byte_value(), 16 as u8 / 8 * 16 as u8];
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
    let mut small_buffer =  [Color::Black.get_byte_value(), 160 as u8 / 8 * 160 as u8];
    epd.update_partial_frame(&mut spi, &small_buffer, 20, 20, 160, 160).unwrap();

    small_buffer =  [Color::White.get_byte_value(), 80 as u8 / 8 * 80 as u8];
    epd.update_partial_frame(&mut spi, &small_buffer, 60, 60, 80, 80).unwrap();

    small_buffer =  [Color::Black.get_byte_value(), 8];
    epd.update_partial_frame(&mut spi, &small_buffer, 96, 96, 8, 8).unwrap();

    // Display updated frame
    epd.display_frame(&mut spi).unwrap();

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





    


