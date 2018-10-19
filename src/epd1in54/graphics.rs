use graphics::{
    outside_display,
    rotation,
    DisplayRotation, 
    Display
};
use color::Color;
use embedded_graphics::prelude::*;

use epd1in54::{DEFAULT_BACKGROUND_COLOR, WIDTH, HEIGHT};

pub struct DisplayEink1in54BlackWhite {    
    buffer: [u8; WIDTH as usize * HEIGHT as usize / 8],
    rotation: DisplayRotation,
}

impl Default for DisplayEink1in54BlackWhite {
    fn default() -> Self {
        DisplayEink1in54BlackWhite {
            buffer: [
                DEFAULT_BACKGROUND_COLOR.get_byte_value();
                WIDTH as usize * HEIGHT as usize / 8                
            ],
            rotation: DisplayRotation::default()
        }
    }
}

impl Display for DisplayEink1in54BlackWhite {
    fn buffer(&self) -> &[u8] {
        &self.buffer
    }
    fn set_rotation(&mut self, rotation: DisplayRotation) {
        self.rotation = rotation;
    }
    fn rotation(&self) -> DisplayRotation {
        self.rotation
    }
}


impl Drawing<Color> for DisplayEink1in54BlackWhite {
    fn draw<T>(&mut self, item_pixels: T)
    where
        T: Iterator<Item = Pixel<Color>>
    {
        let width = WIDTH as u32;
        let height = HEIGHT as u32;

        for Pixel(UnsignedCoord(x,y), color) in item_pixels {
            if outside_display(x, y, width, height, self.rotation) {
                return;
            }

            let (idx, bit) = rotation(x, y, width, height, self.rotation);

            let idx = idx as usize;
            match color {
                Color::Black => {
                    self.buffer[idx] &= !bit;
                }
                Color::White => {
                    self.buffer[idx] |= bit;
                }
            }            
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use epd1in54::{DEFAULT_BACKGROUND_COLOR};
    use embedded_graphics::coord::Coord;
    use embedded_graphics::primitives::Line;

    // test buffer length
    #[test]
    fn graphics_size() {
        let display = DisplayEink1in54BlackWhite::default();
        assert_eq!(display.buffer().len(), 5000);
    }
    
    // test default background color on all bytes
    #[test]
    fn graphics_default() {
        let display = DisplayEink1in54BlackWhite::default();
        for &byte in display.buffer() {
            assert_eq!(byte, DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }

    #[test]
    fn graphics_rotation_0() {
        let mut display = DisplayEink1in54BlackWhite::default();
        display.draw(
            Line::new(Coord::new(0, 0), Coord::new(7, 0))
                .with_stroke(Some(Color::Black))
                .into_iter(),
        );
        
        let buffer = display.buffer();

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }

    #[test]
    fn graphics_rotation_90() {
        let mut display = DisplayEink1in54BlackWhite::default();
        display.set_rotation(DisplayRotation::Rotate90);
        display.draw(
            Line::new(Coord::new(0, 192), Coord::new(0, 199))
                .with_stroke(Some(Color::Black))
                .into_iter(),
        );
        
        let buffer = display.buffer();

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }

    #[test]
    fn graphics_rotation_180() {
        let mut display = DisplayEink1in54BlackWhite::default();
        display.set_rotation(DisplayRotation::Rotate180);
        display.draw(
            Line::new(Coord::new(192, 199), Coord::new(199, 199))
                .with_stroke(Some(Color::Black))
                .into_iter(),
        );
        
        let buffer = display.buffer();

        extern crate std;
        std::println!("{:?}", buffer);

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
        
    }

    #[test]
    fn graphics_rotation_270() {
                let mut display = DisplayEink1in54BlackWhite::default();
        display.set_rotation(DisplayRotation::Rotate270);
        display.draw(
            Line::new(Coord::new(199, 0), Coord::new(199, 7))
                .with_stroke(Some(Color::Black))
                .into_iter(),
        );
        
        let buffer = display.buffer();

        extern crate std;
        std::println!("{:?}", buffer);

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
        
    }
}