// the library for the embedded linux device
extern crate linux_embedded_hal as lin_hal;

// the eink library
extern crate eink_waveshare_rs;


use eink_waveshare_rs::{
    EPD1in54, 
    //drawing::{Graphics},
    color::Color,
    WaveshareInterface,
};

use lin_hal::spidev::{self, SpidevOptions};
use lin_hal::{Pin, Spidev};
use lin_hal::sysfs_gpio::Direction;
use lin_hal::Delay;

// activate spi, gpio in raspi-config
// needs to be run with sudo because of some sysfs_gpio permission problems and follow-up timing problems
// see https://github.com/rust-embedded/rust-sysfs-gpio/issues/5 and follow-up issues


// DigitalIn Hack as long as it's not in the linux_embedded_hal
// from https://github.com/rudihorn/max31865/blob/extra_examples/examples/rpi.rs
// (slightly changed now as OutputPin doesn't provide is_high and is_low anymore)
extern crate embedded_hal;
use embedded_hal::digital::{InputPin};

//TODO: Remove when linux_embedded_hal implements InputPin 
struct HackInputPin<'a> {
    pin: &'a Pin
}

//TODO: Remove when linux_embedded_hal implements InputPin 
impl<'a> HackInputPin<'a> {
    fn new(p : &'a Pin) -> HackInputPin {
        HackInputPin {
            pin: p
        }
    }
}

//TODO: Remove when linux_embedded_hal implements InputPin 
// for now it defaults to is_low if an error appears
// could be handled better!
impl<'a> InputPin for HackInputPin<'a> {
    fn is_low(&self) -> bool {
        self.pin.get_value().unwrap_or(0) == 0
    }

    fn is_high(&self) -> bool {
        !self.is_low()
    }
}


/*
*
* BE CAREFUL: this wasn't tested yet, and the pins are also not choosen correctly (just some random ones atm)
*
*/

fn main() {

    run().unwrap();
}

fn run() -> Result<(), std::io::Error> {
    // Configure SPI
    // SPI settings are from eink-waveshare-rs documenation
    let mut spi = Spidev::open("/dev/spidev0.0").expect("spidev directory");
    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(4_000_000)
        .mode(spidev::SPI_MODE_0)
        .build();
    spi.configure(&options).expect("spi configuration");

    // Configure Digital I/O Pin to be used as Chip Select for SPI
    let cs_pin = Pin::new(26);//BCM7 CE0
    cs_pin.export().expect("cs_pin export");
    while !cs_pin.is_exported() {}
    cs_pin.set_direction(Direction::Out).expect("cs_pin Direction");
    cs_pin.set_value(1).expect("cs_pin Value set to 1");

    // Configure Busy Input Pin
    let busy = Pin::new(5);//pin 29
    busy.export().expect("busy export");
    while !busy.is_exported() {}
    busy.set_direction(Direction::In).expect("busy Direction");
    //busy.set_value(1).expect("busy Value set to 1");
    let busy_in = HackInputPin::new(&busy);

    // Configure Data/Command OutputPin
    let dc = Pin::new(6); //pin 31 //bcm6
    dc.export().expect("dc export");
    while !dc.is_exported() {}
    dc.set_direction(Direction::Out).expect("dc Direction");
    dc.set_value(1).expect("dc Value set to 1");

    // Configure Reset OutputPin
    let rst = Pin::new(16); //pin 36 //bcm16
    rst.export().expect("rst export");
    while !rst.is_exported() {}
    rst.set_direction(Direction::Out).expect("rst Direction");
    rst.set_value(1).expect("rst Value set to 1");   

    // Configure Delay
    let delay = Delay {};


    // Setup of the needed pins is finished here
    // Now the "real" usage of the eink-waveshare-rs crate begins
    let mut epd = EPD1in54::new(spi, cs_pin, busy_in, dc, rst, delay)?;

    // Clear the full screen
    epd.clear_frame();
    epd.display_frame();

    // Speeddemo
    let small_buffer =  [Color::Black.get_byte_value(), 16 as u8 / 8 * 16 as u8];
    let number_of_runs = 100;
    for i in 0..number_of_runs {
        let offset = i * 8 % 150;
        epd.update_partial_frame(&small_buffer, 25 + offset, 25 + offset, 16, 16)?;
        epd.display_frame()?;
    }

    // Clear the full screen
    epd.clear_frame();
    epd.display_frame();

    // Draw some squares
    let mut small_buffer =  [Color::Black.get_byte_value(), 160 as u8 / 8 * 160 as u8];
    epd.update_partial_frame(&small_buffer, 20, 20, 160, 160)?;

    small_buffer =  [Color::White.get_byte_value(), 80 as u8 / 8 * 80 as u8];
    epd.update_partial_frame(&small_buffer, 60, 60, 80, 80)?;

    small_buffer =  [Color::Black.get_byte_value(), 8];
    epd.update_partial_frame(&small_buffer, 96, 96, 8, 8)?;

    // Display updated frame
    epd.display_frame()?;

    // Set the EPD to sleep
    epd.sleep()?;

    Ok(())
}
