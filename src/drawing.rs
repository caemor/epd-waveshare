use color::Color;
use embedded_graphics::prelude::*;

/// Displayrotation 
#[derive(Clone, Copy)]
pub enum DisplayRotation {
    /// No rotation
    Rotate0,
    /// Rotate by 90 degrees clockwise
    Rotate90,
    /// Rotate by 180 degrees clockwise
    Rotate180,
    /// Rotate 270 degrees clockwise
    Rotate270,
}
impl Default for DisplayRotation {
    fn default() -> Self {
        DisplayRotation::Rotate0
    }
}

pub trait Display {
    fn buffer(&self) -> &[u8];
    fn set_rotation(&mut self, rotation: DisplayRotation);
    fn rotation(&self) -> DisplayRotation;
}


pub struct DisplayEink42BlackWhite {
    buffer: [u8; 400 * 300 / 8],
    rotation: DisplayRotation, //TODO: check embedded_graphics for orientation
}

impl Default for DisplayEink42BlackWhite {
    fn default() -> Self {
        use epd4in2::constants::{DEFAULT_BACKGROUND_COLOR, WIDTH, HEIGHT};
        DisplayEink42BlackWhite {
            buffer: [
                DEFAULT_BACKGROUND_COLOR.get_byte_value();
                WIDTH as usize * HEIGHT as usize / 8                
            ],
            rotation: DisplayRotation::default()
        }
    }
}

impl Display for DisplayEink42BlackWhite {
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

//TODO: add more tests for the rotation maybe? or test it at least once in real!
impl Drawing<Color> for DisplayEink42BlackWhite {
    fn draw<T>(&mut self, item_pixels: T)
    where
        T: Iterator<Item = Pixel<Color>>
    {
        use epd4in2::constants::{WIDTH, HEIGHT};
        for Pixel(UnsignedCoord(x,y), color) in item_pixels {
            match self.rotation {
                DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => {
                    if x >= WIDTH as u32 || y >= HEIGHT as u32 {
                        return;
                    }
                },
                DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => {
                    if y >= WIDTH as u32 || x >= HEIGHT as u32 {
                        return;
                    } 
                }
            }

            let (idx, bit) = match self.rotation {
                DisplayRotation::Rotate0 => (
                    x as usize / 8 + (WIDTH as usize / 8) * y as usize,
                    0x80 >> (x % 8),
                ),
                DisplayRotation::Rotate90 => (
                    (WIDTH as usize - 1 - y as usize) / 8 + (WIDTH as usize / 8) * x as usize,
                    0x01 << (y % 8),
                ),
                DisplayRotation::Rotate180 => (
                    ((WIDTH as usize / 8) * HEIGHT as usize - 1) - (x as usize / 8 + (WIDTH as usize / 8) * y as usize),
                    0x01 << (x % 8),
                ),
                DisplayRotation::Rotate270 => (
                    y as usize / 8 + (HEIGHT as usize - 1 - x as usize) * (WIDTH as usize / 8),
                    0x80 >> (y % 8),
                ),
            };

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

//TODO: write tests
#[cfg(test)]
mod tests {
    use super::*;
    use epd4in2;
    use embedded_graphics::coord::Coord;
    use embedded_graphics::primitives::Line;

    #[test]
    fn from_u8() {
        assert_eq!(Color::Black, Color::from(0u8));
        assert_eq!(Color::White, Color::from(1u8));
    }

    // test all values aside from 0 and 1 which all should panic
    #[test]
    fn from_u8_panic() {
        for val in 2..=u8::max_value() {
            extern crate std;
            let result = std::panic::catch_unwind(|| Color::from(val));
            assert!(result.is_err());
        }        
    }

    // test buffer length
    #[test]
    fn graphics_4in2_size() {
        let display = DisplayEink42BlackWhite::default();
        assert_eq!(display.buffer().len(), 15000);
    }
    
    // test default background color on all bytes
    #[test]
    fn graphics_4in2_default() {
        let display = DisplayEink42BlackWhite::default();
        use epd4in2;
        for &byte in display.buffer() {
            assert_eq!(byte, epd4in2::constants::DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }

    #[test]
    fn graphics_4in2_rotation_0() {
        let mut display = DisplayEink42BlackWhite::default();
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
    fn graphics_4in2_rotation_90() {
        let mut display = DisplayEink42BlackWhite::default();
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
    fn graphics_4in2_rotation_180() {
        let mut display = DisplayEink42BlackWhite::default();
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
    fn graphics_4in2_rotation_270() {
                let mut display = DisplayEink42BlackWhite::default();
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