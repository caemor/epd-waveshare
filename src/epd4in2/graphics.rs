use epd4in2::constants::{DEFAULT_BACKGROUND_COLOR, WIDTH, HEIGHT};
use graphics::DisplayDimension;

pub struct DisplayEink4in2BlackWhite {
    width: u32,
    height: u32,
    pub buffer: [u8; WIDTH as usize * HEIGHT as usize / 8],
}

impl Default for DisplayEink4in2BlackWhite {
    fn default() -> Self {
        DisplayEink4in2BlackWhite {
            width: WIDTH,
            height: HEIGHT,        
            buffer: [
                DEFAULT_BACKGROUND_COLOR.get_byte_value();
                WIDTH as usize * HEIGHT as usize / 8                
            ]
        }
    }
}

impl DisplayDimension for DisplayEink4in2BlackWhite {
    fn buffer(&mut self) -> &mut [u8] {
        &mut self.buffer
    }
    fn width(&self) -> u32 {
        self.width
    }
    fn height(&self) -> u32 {
        self.height
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use epd4in2;
    use graphics::{DisplayRotation, Display};
    use embedded_graphics::coord::Coord;
    use embedded_graphics::primitives::Line;
    use color::Color;
    use embedded_graphics::prelude::*;

    // test buffer length
    #[test]
    fn graphics_size() {
        let mut display4in2 = DisplayEink4in2BlackWhite::default();
        let display = Display::new(WIDTH, HEIGHT, &mut display4in2.buffer);
        assert_eq!(display.buffer().len(), 15000);
    }
    
    // test default background color on all bytes
    #[test]
    fn graphics_default() {
        let mut display4in2 = DisplayEink4in2BlackWhite::default();
        let display = Display::new(WIDTH, HEIGHT, &mut display4in2.buffer);
        use epd4in2;
        for &byte in display.buffer() {
            assert_eq!(byte, epd4in2::constants::DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }

    #[test]
    fn graphics_rotation_0() {
        let mut display4in2 = DisplayEink4in2BlackWhite::default();
        let mut display = Display::new(WIDTH, HEIGHT, &mut display4in2.buffer);
        display.draw(
            Line::new(Coord::new(0, 0), Coord::new(7, 0))
                .with_stroke(Some(Color::Black))
                .into_iter(),
        );
        
        let buffer = display.buffer();

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, epd4in2::constants::DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }

    #[test]
    fn graphics_rotation_90() {
        let mut display4in2 = DisplayEink4in2BlackWhite::default();
        let mut display = Display::new(WIDTH, HEIGHT, &mut display4in2.buffer);
        display.set_rotation(DisplayRotation::Rotate90);
        display.draw(
            Line::new(Coord::new(0, 392), Coord::new(0, 399))
                .with_stroke(Some(Color::Black))
                .into_iter(),
        );
        
        let buffer = display.buffer();

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, epd4in2::constants::DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }

    #[test]
    fn graphics_rotation_180() {
        let mut display4in2 = DisplayEink4in2BlackWhite::default();
        let mut display = Display::new(WIDTH, HEIGHT, &mut display4in2.buffer);
        display.set_rotation(DisplayRotation::Rotate180);
        display.draw(
            Line::new(Coord::new(392, 299), Coord::new(399, 299))
                .with_stroke(Some(Color::Black))
                .into_iter(),
        );
        
        let buffer = display.buffer();

        extern crate std;
        std::println!("{:?}", buffer);

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, epd4in2::constants::DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
        
    }

    #[test]
    fn graphics_rotation_270() {
        let mut display4in2 = DisplayEink4in2BlackWhite::default();
        let mut display = Display::new(WIDTH, HEIGHT, &mut display4in2.buffer);
        display.set_rotation(DisplayRotation::Rotate270);
        display.draw(
            Line::new(Coord::new(299, 0), Coord::new(299, 7))
                .with_stroke(Some(Color::Black))
                .into_iter(),
        );
        
        let buffer = display.buffer();

        extern crate std;
        std::println!("{:?}", buffer);

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, epd4in2::constants::DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
        
    }
}

