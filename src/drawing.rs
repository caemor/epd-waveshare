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

fn outside_display(x: u32, y: u32, width: u32, height: u32, rotation: DisplayRotation) -> bool {
    match rotation {
        DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => {
            if x >= width || y >= height {
                return true;
            }
        },
        DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => {
            if y >= width || x >= height {
                return true;
            } 
        }
    }
    false
}

fn rotation(x: u32, y: u32, width: u32, height: u32, rotation: DisplayRotation) -> (u32, u8) {
    match rotation {
        DisplayRotation::Rotate0 => (
            x / 8 + (width / 8) * y,
            0x80 >> (x % 8),
        ),
        DisplayRotation::Rotate90 => (
            (width - 1 - y) / 8 + (width / 8) * x,
            0x01 << (y % 8),
        ),
        DisplayRotation::Rotate180 => (
            ((width / 8) * height - 1) - (x / 8 + (width / 8) * y),
            0x01 << (x % 8),
        ),
        DisplayRotation::Rotate270 => (
            y / 8 + (height - 1 - x) * (width / 8),
            0x80 >> (y % 8),
        ),
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

    #[test]
    fn rotation_overflow() {
        use epd4in2::constants::{WIDTH, HEIGHT};
        let width = WIDTH as u32;
        let height = HEIGHT as u32;
        test_rotation_overflow(width, height, DisplayRotation::Rotate0);
        test_rotation_overflow(width, height, DisplayRotation::Rotate90);
        test_rotation_overflow(width, height, DisplayRotation::Rotate180);
        test_rotation_overflow(width, height, DisplayRotation::Rotate270);
        
    }

    fn test_rotation_overflow(width: u32, height: u32, rotation2: DisplayRotation) {
        let max_value = width / 8 * height;
        for x in 0..(width + height) { //limit x because it runs too long 
            for y in 0..(u32::max_value()) {
                if outside_display(x, y, width, height, rotation2) {
                    break;
                } else {
                    let (idx, _) = rotation(x, y, width, height, rotation2);
                    assert!(idx < max_value);
                }
            }
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