#![deny(unsafe_code)]
#![no_std]
 

extern crate cortex_m;

extern crate f3;

extern crate eink_waveshare_rs;

extern crate embedded_hal as hal;

// For handling 'language item required, but not found: panic_fmt' in #no_std
// see https://os.phil-opp.com/freestanding-rust-binary/ for more infos
extern crate panic_abort;
 

use f3::hal::prelude::*;
use f3::hal::stm32f30x;
use f3::hal::spi::Spi;
use f3::hal::delay::Delay;
use eink_waveshare_rs::{epd4in2::EPD4in2, SPI_MODE};

use hal::digital::{InputPin, OutputPin};


//from https://github.com/rudihorn/max31865/tree/extra_examples/examples
struct HackInputPin<'a> {
    pin: &'a OutputPin
}

impl<'a> HackInputPin<'a> {
    fn new(p : &'a OutputPin) -> HackInputPin {
        HackInputPin {
            pin: p
        }
    }
}

impl<'a> InputPin for HackInputPin<'a> {
    fn is_low(&self) -> bool {
        self.pin.is_low()
    }

    fn is_high(&self) -> bool {
        self.pin.is_high()
    }
}

/*
*
* BE CAREFUL: this wasn't tested yet, and the pins are also not choosen correctly (just some random ones atm)
*
*/

fn main() {
    let cp = cortex_m::Peripherals::take().unwrap();
    let p = stm32f30x::Peripherals::take().unwrap();

    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();
 
    // TRY the other clock configuration
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    // let clocks = rcc.cfgr.sysclk(64.mhz()).pclk1(32.mhz()).freeze(&mut flash.acr);
 
    let mut gpioa = p.GPIOA.split(&mut rcc.ahb);
    let mut gpioe = p.GPIOE.split(&mut rcc.ahb);
 
    let mut cs = gpioe
        .pe3
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    cs.set_high();

    //TODO: Fix when f3::hal includes Digital::InputPin
    //using the hack from rudihorn that Digital::OutputPin basically
    //contains the needed functions for Digital::InputPin
    let busy = gpioe.pe4.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let busy_in = HackInputPin::new(&busy);

    let dc = gpioe.pe5.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let rst = gpioe.pe6.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let delay = Delay::new(cp.SYST, clocks);


 
    // The `L3gd20` abstraction exposed by the `f3` crate requires a specific pin configuration to
    // be used and won't accept any configuration other than the one used here. Trying to use a
    // different pin configuration will result in a compiler error.
    let sck = gpioa.pa5.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let miso = gpioa.pa6.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let mosi = gpioa.pa7.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
 
    let spi = Spi::spi1(
        p.SPI1,
        (sck, miso, mosi),
        SPI_MODE,
        2.mhz(),
        clocks,
        &mut rcc.apb2,
    );

    //TODO wait for f3::hal update to include Digital::InputPin
    let mut epd4in2 = EPD4in2::new(spi, cs, busy_in, dc, rst, delay).unwrap();

    //let mut buffer =  [0u8, epd4in2.get_width() / 8 * epd4in2.get_height()];
    let mut buffer = [0u8; 15000];
    // draw something into the buffer
    buffer[0] = 0xFF;
 
    epd4in2.display_and_transfer_frame(&buffer, None).unwrap();
 
    epd4in2.delay_ms(3000);
   
    epd4in2.clear_frame(None).unwrap();

    epd4in2.sleep().unwrap();
}
