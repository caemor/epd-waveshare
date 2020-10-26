#![deny(warnings)]

use epd_waveshare::{epd1in54::EPD1in54, prelude::*};
use linux_embedded_hal::{
    spidev::{self, SpidevOptions},
    sysfs_gpio::Direction,
    Delay, Pin, Spidev,
};

// activate spi, gpio in raspi-config
// needs to be run with sudo because of some sysfs_gpio permission problems and follow-up timing problems
// see https://github.com/rust-embedded/rust-sysfs-gpio/issues/5 and follow-up issues

fn main() -> Result<(), std::io::Error> {
    // Configure SPI
    // SPI settings are from eink-waveshare-rs documenation
    let mut spi = Spidev::open("/dev/spidev0.0")?;
    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(4_000_000)
        .mode(spidev::SpiModeFlags::SPI_MODE_0)
        .build();
    spi.configure(&options).expect("spi configuration");

    // Configure Digital I/O Pin to be used as Chip Select for SPI
    let cs_pin = Pin::new(26); //BCM7 CE0
    cs_pin.export().expect("cs_pin export");
    while !cs_pin.is_exported() {}
    cs_pin
        .set_direction(Direction::Out)
        .expect("cs_pin Direction");
    cs_pin.set_value(1).expect("cs_pin Value set to 1");

    // Configure Busy Input Pin
    let busy = Pin::new(5); //pin 29
    busy.export().expect("busy export");
    while !busy.is_exported() {}
    busy.set_direction(Direction::In).expect("busy Direction");
    //busy.set_value(1).expect("busy Value set to 1");

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
    let mut delay = Delay {};

    // Setup of the needed pins is finished here
    // Now the "real" usage of the eink-waveshare-rs crate begins
    let mut epd = EPD1in54::new(&mut spi, cs_pin, busy, dc, rst, &mut delay)?;

    // Clear the full screen
    epd.clear_frame(&mut spi)?;
    epd.display_frame(&mut spi)?;

    // Speeddemo
    epd.set_lut(&mut spi, Some(RefreshLUT::QUICK))?;
    let small_buffer = [Color::Black.get_byte_value(); 32]; //16x16
    let number_of_runs = 1;
    for i in 0..number_of_runs {
        let offset = i * 8 % 150;
        epd.update_partial_frame(&mut spi, &small_buffer, 25 + offset, 25 + offset, 16, 16)?;
        epd.display_frame(&mut spi)?;
    }

    // Clear the full screen
    epd.clear_frame(&mut spi)?;
    epd.display_frame(&mut spi)?;

    // Draw some squares
    let small_buffer = [Color::Black.get_byte_value(); 3200]; //160x160
    epd.update_partial_frame(&mut spi, &small_buffer, 20, 20, 160, 160)?;

    let small_buffer = [Color::White.get_byte_value(); 800]; //80x80
    epd.update_partial_frame(&mut spi, &small_buffer, 60, 60, 80, 80)?;

    let small_buffer = [Color::Black.get_byte_value(); 8]; //8x8
    epd.update_partial_frame(&mut spi, &small_buffer, 96, 96, 8, 8)?;

    // Display updated frame
    epd.display_frame(&mut spi)?;
    delay.delay_ms(5000u16);

    // Set the EPD to sleep
    epd.sleep(&mut spi)?;

    Ok(())
}
