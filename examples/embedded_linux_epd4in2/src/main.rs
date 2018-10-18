// the library for the embedded linux device
extern crate linux_embedded_hal as lin_hal;

// the eink library
extern crate eink_waveshare_rs;


use eink_waveshare_rs::{
    EPD4in2, 
    drawing_old::{Graphics},
    drawing::{DisplayEink42BlackWhite, Display, DisplayRotation},
    color::Color, 
    WaveshareDisplay,
};

extern crate embedded_graphics;
use embedded_graphics::coord::Coord;
use embedded_graphics::fonts::{Font6x8, Font12x16};
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, Line};
use embedded_graphics::Drawing;

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
use embedded_hal::{
    digital::{InputPin},
}; 
use embedded_hal::prelude::*;

struct HackInputPin<'a> {
    pin: &'a Pin
}

impl<'a> HackInputPin<'a> {
    fn new(p : &'a Pin) -> HackInputPin {
        HackInputPin {
            pin: p
        }
    }
}

//TODO: make it safer?? or handle the errors better?
// now it defaults to is_low if an error appears
impl<'a> InputPin for HackInputPin<'a> {
    fn is_low(&self) -> bool {
        self.pin.get_value().unwrap_or(0) == 0
    }

    fn is_high(&self) -> bool {
        self.pin.get_value().unwrap_or(0) == 1
    }
}


/*
*
* BE CAREFUL: this wasn't tested yet, and the pins are also not choosen correctly (just some random ones atm)
*
*/
fn main() {
    run().map_err(|e| println!("{}", e.to_string())).unwrap();
}


fn run() -> Result<(), std::io::Error> {

    // Configure SPI
    // Settings are taken from 
    let mut spi = Spidev::open("/dev/spidev0.0").expect("spidev directory");
    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(4_000_000)
        .mode(spidev::SPI_MODE_0)
        .build();
    spi.configure(&options).expect("spi configuration");

    // Configure Digital I/O Pin to be used as Chip Select for SPI
    let cs = Pin::new(26);//BCM7 CE0
    cs.export().expect("cs export");
    while !cs.is_exported() {}
    cs.set_direction(Direction::Out).expect("CS Direction");
    cs.set_value(1).expect("CS Value set to 1");

    let busy = Pin::new(5);//pin 29
    busy.export().expect("busy export");
    while !busy.is_exported() {}
    busy.set_direction(Direction::In).expect("busy Direction");
    //busy.set_value(1).expect("busy Value set to 1");
    let busy_in = HackInputPin::new(&busy);

    let dc = Pin::new(6); //pin 31 //bcm6
    dc.export().expect("dc export");
    while !dc.is_exported() {}
    dc.set_direction(Direction::Out).expect("dc Direction");
    dc.set_value(1).expect("dc Value set to 1");

    let rst = Pin::new(16); //pin 36 //bcm16
    rst.export().expect("rst export");
    while !rst.is_exported() {}
    rst.set_direction(Direction::Out).expect("rst Direction");
    rst.set_value(1).expect("rst Value set to 1");   

    let mut delay = Delay {};

    
 
    

    //TODO: wait for Digital::InputPin
    //fixed currently with the HackInputPin, see further above
    let mut epd4in2 = EPD4in2::new(&mut spi, cs, busy_in, dc, rst, &mut delay).expect("eink initalize error");

    // //let mut buffer =  [0u8, epd4in2.get_width() / 8 * epd4in2.get_height()];
    // let mut buffer = [0u8; 15000];

    // // draw something
    // let mut graphics = Graphics::new(400, 300, &mut buffer);
    // graphics.clear(&Color::White);
    // graphics.draw_line(0,0,400,300, &Color::Black); 

    // graphics.draw_filled_rectangle(200,200, 230, 230, &Color::Black); 
    // graphics.draw_line(202,202,218,228, &Color::White);

    // graphics.draw_circle(200, 150, 130, &Color::Black);

    // graphics.draw_pixel(390, 290, &Color::Black);

    // graphics.draw_horizontal_line(0, 150, 400, &Color::Black);

    // graphics.draw_vertical_line(200, 50, 200, &Color::Black);

    // epd4in2.clear_frame(&mut spi).expect("clear frame error");
    // epd4in2.update_frame(&mut spi, graphics.get_buffer()).expect("update frame error");
    // epd4in2.display_frame(&mut spi)?;

    // println!("Finished basic old graphics test");
 
    // delay.delay_ms(3000u16);

    // epd4in2.clear_frame(&mut spi)?;

    // //Test fast updating a bit more
    // let mut small_buffer = [0x00; 128];
    // let mut circle_graphics = Graphics::new(32,32, &mut small_buffer);
    // circle_graphics.draw_circle(16,16, 10, &Color::Black);

    // epd4in2.update_partial_frame(&mut spi, circle_graphics.get_buffer(), 16,16, 32, 32).expect("update frame error");
    // epd4in2.display_frame(&mut spi)?;

    // epd4in2.update_partial_frame(&mut spi, circle_graphics.get_buffer(), 128,64, 32, 32).expect("update partial frame error");
    // epd4in2.display_frame(&mut spi)?;

    // epd4in2.update_partial_frame(&mut spi, circle_graphics.get_buffer(), 320,24, 32, 32).expect("update partial frame error");
    // epd4in2.display_frame(&mut spi)?;

    // epd4in2.update_partial_frame(&mut spi, circle_graphics.get_buffer(), 160,240, 32, 32).expect("update partial frame error");
    // epd4in2.display_frame(&mut spi)?;

    // println!("Finished partial update test");

    // delay.delay_ms(3000u16);




    // //pub fn draw_string_8x8(&self, buffer: &mut[u8], x0: u16, y0: u16, input: &str, color: &Color) {
    // graphics.draw_string_8x8(16, 16, "hello", &Color::Black);
    // graphics.draw_char_8x8(250, 250, '#', &Color::Black);
    // graphics.draw_char_8x8(300, 16, '7', &Color::Black);
    // epd4in2.update_frame(&mut spi, graphics.get_buffer())?;
    // epd4in2.display_frame(&mut spi)?;

    // println!("Finished draw string test");

    //delay.delay_ms(3000u16);

    println!("Now test new graphics:");

    println!("Now test new graphics with rotate0:");
    let mut display = DisplayEink42BlackWhite::default();
    display.set_rotation(DisplayRotation::Rotate0);
    display.draw(
            Font6x8::render_str("Rotate 0!")
                .with_fill(Some(Color::White))
                .with_stroke(Some(Color::Black))
                .translate(Coord::new(5, 50))
                .into_iter(),
    );
    display.draw(
            Font6x8::render_str("Rotate 0 - inverse!")
                .with_fill(Some(Color::Black))
                .with_stroke(Some(Color::White))
                .translate(Coord::new(50, 70))
                .into_iter(),
    );

    display.draw(
        Line::new(Coord::new(5, 5), Coord::new(15, 15))
            .with_stroke(Some(Color::Black))
            .into_iter(),
    );


    epd4in2.update_frame(&mut spi, &display.buffer()).unwrap();
    epd4in2.display_frame(&mut spi).expect("display frame new graphics");
    delay.delay_ms(2000u16);

    println!("Now test new graphics with rotate90:");
    let mut display = DisplayEink42BlackWhite::default();
    display.set_rotation(DisplayRotation::Rotate90);
    display.draw(
            Font6x8::render_str("Rotate 90!")
                .with_stroke(Some(Color::Black))
                .with_fill(Some(Color::White))
                .translate(Coord::new(5, 50))
                .into_iter(),
    );
    display.draw(
        Line::new(Coord::new(5, 5), Coord::new(15, 15))
            .with_stroke(Some(Color::Black))
            .into_iter(),
    );
    epd4in2.update_frame(&mut spi, &display.buffer()).unwrap();
    epd4in2.display_frame(&mut spi).expect("display frame new graphics");
    delay.delay_ms(2000u16);

    println!("Now test new graphics with rotate180:");
    let mut display = DisplayEink42BlackWhite::default();
    display.set_rotation(DisplayRotation::Rotate180);
    display.draw(
            Font6x8::render_str("Rotate 180!")
                .translate(Coord::new(5, 50))
                .into_iter(),
    );
    display.draw(
        Line::new(Coord::new(5, 5), Coord::new(15, 15))
            .with_stroke(Some(Color::Black))
            .into_iter(),
    );
    epd4in2.update_frame(&mut spi, &display.buffer()).unwrap();
    epd4in2.display_frame(&mut spi).expect("display frame new graphics");
    delay.delay_ms(2000u16);

    println!("Now test new graphics with rotate270:");
    let mut display = DisplayEink42BlackWhite::default();
    display.set_rotation(DisplayRotation::Rotate270);
    display.draw(
            Font6x8::render_str("Rotate 270!")
                .translate(Coord::new(5, 50))
                .into_iter(),
    );
    display.draw(
        Line::new(Coord::new(5, 5), Coord::new(15, 15))
            .with_stroke(Some(Color::Black))
            .into_iter(),
    );
    epd4in2.update_frame(&mut spi, &display.buffer()).unwrap();
    epd4in2.display_frame(&mut spi).expect("display frame new graphics");
    delay.delay_ms(2000u16);


    println!("Now test new graphics with default rotation and some special stuff:");
    let mut display = DisplayEink42BlackWhite::default();
    display.draw(
        Circle::new(Coord::new(64, 64), 64)
            .with_stroke(Some(Color::Black))
            .into_iter(),
    );
    display.draw(
        Line::new(Coord::new(64, 64), Coord::new(0, 64))
            .with_stroke(Some(Color::Black))
            .into_iter(),
    );
    display.draw(
        Line::new(Coord::new(64, 64), Coord::new(80, 80))
            .with_stroke(Some(Color::Black))
            .into_iter(),
    );
    display.draw(
        Font6x8::render_str("It's working-WoB!")
            // Using Style here
            .with_style(Style {
                fill_color: Some(Color::Black),
                stroke_color: Some(Color::White),
                stroke_width: 0u8, // Has no effect on fonts
            })
            .translate(Coord::new(175, 250))
            .into_iter(),
    );

    display.draw(
        Font6x8::render_str("It's working-BoW!")
            // Using Style here
            .with_style(Style {
                fill_color: Some(Color::White),
                stroke_color: Some(Color::Black),
                stroke_width: 0u8, // Has no effect on fonts
            })
            .translate(Coord::new(50, 200))
            .into_iter(),
    );

    display.draw(
        Font12x16::render_str("It's working-BoW!")
            // Using Style here
            .with_style(Style {
                fill_color: Some(Color::White),
                stroke_color: Some(Color::Black),
                stroke_width: 0u8, // Has no effect on fonts
            })
            .translate(Coord::new(50, 22))
            .into_iter(),
    );

    display.draw(
        Line::new(Coord::new(60, 28), Coord::new(120, 28))
            .with_stroke(Some(Color::White))
            .into_iter(),
    );

    display.draw(
        Line::new(Coord::new(90, 14), Coord::new(90, 42))
            .with_stroke(Some(Color::Black))
            .into_iter(),
    );

    

    let mut i = 0;
    let mut limit = 2;
    loop {
        i += 1;
        println!("Moving Hello World. Loop {} from {}", i, limit);

        display.draw(
            Font6x8::render_str("Hello World!")
                .with_style(Style {
                    fill_color: Some(Color::White),
                    stroke_color: Some(Color::Black),
                    stroke_width: 0u8, // Has no effect on fonts
                })
                .translate(Coord::new(5 + i*10, 50))
                .into_iter(),
        );        

        epd4in2.update_frame(&mut spi, &display.buffer()).unwrap();
        epd4in2.display_frame(&mut spi).expect("display frame new graphics");
        if i >= limit {
            
            break;
        }
        delay.delay_ms(1_000u16);
    }


    println!("Finished tests - going to sleep");
    epd4in2.sleep(&mut spi)
}   
