//! Graphics Support for EPDs

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

pub struct Display<'a> {
    width: u32,
    height: u32,
    rotation: DisplayRotation,
    buffer: &'a mut [u8], //buffer: Box<u8>//[u8; 15000]
}

impl<'a> Display<'a> {
    pub fn new(width: u32, height: u32, buffer: &'a mut [u8]) -> Display<'a> {
        let len = buffer.len() as u32;
        assert!(width / 8 * height >= len);
        Display {
            width,
            height,
            rotation: DisplayRotation::default(),
            buffer,
        }
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    pub fn clear_buffer(&mut self, background_color: Color) {
        for elem in &mut self.buffer.iter_mut() {
            *elem = background_color.get_byte_value();
        }
    }

    pub fn set_rotation(&mut self, rotation: DisplayRotation) {
        self.rotation = rotation;
    }

    pub fn rotation(&self) -> DisplayRotation {
        self.rotation
    }
}


impl<'a> Drawing<Color> for Display<'a> {
    fn draw<T>(&mut self, item_pixels: T)
    where
        T: Iterator<Item = Pixel<Color>>
    {
        for Pixel(UnsignedCoord(x,y), color) in item_pixels {

            if outside_display(x, y, self.width, self.height, self.rotation) {
                continue;
            }

            // Give us index inside the buffer and the bit-position in that u8 which needs to be changed
            let (index, bit) = rotation(x, y, self.width, self.height, self.rotation);
            let index = index as usize;

            // "Draw" the Pixel on that bit
            match color {
                Color::Black => {
                    self.buffer[index] &= !bit;
                }
                Color::White => {
                    self.buffer[index] |= bit;
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



#[cfg(test)]
mod tests {
    use super::{DisplayRotation, outside_display, rotation, Display};
    use color::Color;
    use embedded_graphics::coord::Coord;
    use embedded_graphics::primitives::Line;
    use embedded_graphics::prelude::*;

    #[test]
    fn buffer_clear() {
        use epd4in2::{WIDTH, HEIGHT};

        let mut buffer = [Color::Black.get_byte_value(); WIDTH as usize / 8 * HEIGHT as usize];
        let mut display = Display::new(WIDTH, HEIGHT, &mut buffer);

        for &byte in display.buffer.iter() {
            assert_eq!(byte, Color::Black.get_byte_value());
        }

        display.clear_buffer(Color::White);

        for &byte in display.buffer.iter() {
            assert_eq!(byte, Color::White.get_byte_value());
        }        
    }


    #[test]
    fn rotation_overflow() {
        use epd4in2::{WIDTH, HEIGHT};
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



    #[test]
    fn graphics_rotation_0() {
        use epd2in9::{DEFAULT_BACKGROUND_COLOR};        
        let width = 128;
        let height = 296;
        
        let mut buffer = [DEFAULT_BACKGROUND_COLOR.get_byte_value(); 128 / 8 * 296];
        let mut display = Display::new(width, height, &mut buffer);

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
        use epd2in9::{DEFAULT_BACKGROUND_COLOR};        
        let width = 128;
        let height = 296;
        
        let mut buffer = [DEFAULT_BACKGROUND_COLOR.get_byte_value(); 128 / 8 * 296];
        let mut display = Display::new(width, height, &mut buffer);

        display.set_rotation(DisplayRotation::Rotate90);

        display.draw(
            Line::new(Coord::new(0, 120), Coord::new(0, 295))
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