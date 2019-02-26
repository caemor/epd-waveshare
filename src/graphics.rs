//! Graphics Support for EPDs

use crate::color::Color;
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
    fn clear_buffer(&mut self, background_color: Color) {
        for elem in self.get_mut_buffer().iter_mut() {
            *elem = background_color.get_byte_value();
        }
    }

    fn buffer(&self) -> &[u8];

    fn get_mut_buffer<'a>(&'a mut self) -> &'a mut [u8];

    /// Sets the rotation of the display
    fn set_rotation(&mut self, rotation: DisplayRotation);
    /// Get the current rotation of the display
    fn rotation(&self) -> DisplayRotation;

    fn draw_helper<T>(&mut self, width: u32, height: u32, item_pixels: T)
    where
        T: Iterator<Item = Pixel<Color>>,
    {
        let rotation = self.rotation();
        let buffer = self.get_mut_buffer();
        for Pixel(UnsignedCoord(x, y), color) in item_pixels {
            if outside_display(x, y, width, height, rotation) {
                continue;
            }

            // Give us index inside the buffer and the bit-position in that u8 which needs to be changed
            let (index, bit) = find_position(x, y, width, height, rotation);
            let index = index as usize;



            // "Draw" the Pixel on that bit
            match color {
                Color::Black => {
                    buffer[index] &= !bit;
                }
                Color::White => {
                    buffer[index] |= bit;
                }
            }
        }
    }
}

/// A variable Display without a predefined buffer
/// 
/// The buffer can be created as following:
/// buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value(); WIDTH / 8 * HEIGHT]
pub struct VarDisplay<'a> {
    width: u32,
    height: u32,
    rotation: DisplayRotation,
    buffer: &'a mut [u8], //buffer: Box<u8>//[u8; 15000]
}

impl<'a> VarDisplay<'a> {
    pub fn new(width: u32, height: u32, buffer: &'a mut [u8]) -> VarDisplay<'a> {
        let len = buffer.len() as u32;
        assert!(width / 8 * height >= len);
        VarDisplay {
            width,
            height,
            rotation: DisplayRotation::default(),
            buffer,
        }
    }
}

impl<'a> Drawing<Color> for VarDisplay<'a> {
    fn draw<T>(&mut self, item_pixels: T)
    where
        T: Iterator<Item = Pixel<Color>>,
    {
        self.draw_helper(self.width, self.height, item_pixels);
    }
}

impl<'a> Display for VarDisplay<'a> {
    fn buffer(&self) -> &[u8] {
        &self.buffer
    }
    
    fn get_mut_buffer<'b>(&'b mut self) -> &'b mut [u8] {
        &mut self.buffer
    }

    fn set_rotation(&mut self, rotation: DisplayRotation) {
        self.rotation = rotation;
    }

    fn rotation(&self) -> DisplayRotation {
        self.rotation
    }
}

// Checks if a pos is outside the defined display
fn outside_display(x: u32, y: u32, width: u32, height: u32, rotation: DisplayRotation) -> bool {
    match rotation {
        DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => {
            if x >= width || y >= height {
                return true;
            }
        }
        DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => {
            if y >= width || x >= height {
                return true;
            }
        }
    }
    false
}

#[rustfmt::skip]
//returns index position in the u8-slice and the bit-position inside that u8
fn find_position(x: u32, y: u32, width: u32, height: u32, rotation: DisplayRotation) -> (u32, u8) {
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
    use super::{outside_display, find_position, VarDisplay, Display, DisplayRotation};
    use crate::color::Color;
    use embedded_graphics::coord::Coord;
    use embedded_graphics::prelude::*;
    use embedded_graphics::primitives::Line;

    #[test]
    fn buffer_clear() {
        use crate::epd4in2::{HEIGHT, WIDTH};

        let mut buffer = [Color::Black.get_byte_value(); WIDTH as usize / 8 * HEIGHT as usize];
        let mut display = VarDisplay::new(WIDTH, HEIGHT, &mut buffer);

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
        use crate::epd4in2::{HEIGHT, WIDTH};
        let width = WIDTH as u32;
        let height = HEIGHT as u32;
        test_rotation_overflow(width, height, DisplayRotation::Rotate0);
        test_rotation_overflow(width, height, DisplayRotation::Rotate90);
        test_rotation_overflow(width, height, DisplayRotation::Rotate180);
        test_rotation_overflow(width, height, DisplayRotation::Rotate270);
    }

    fn test_rotation_overflow(width: u32, height: u32, rotation2: DisplayRotation) {
        let max_value = width / 8 * height;
        for x in 0..(width + height) {
            //limit x because it runs too long
            for y in 0..(u32::max_value()) {
                if outside_display(x, y, width, height, rotation2) {
                    break;
                } else {
                    let (idx, _) = find_position(x, y, width, height, rotation2);
                    assert!(idx < max_value);
                }
            }
        }
    }

    #[test]
    fn graphics_rotation_0() {
        use crate::epd2in9::DEFAULT_BACKGROUND_COLOR;
        let width = 128;
        let height = 296;

        let mut buffer = [DEFAULT_BACKGROUND_COLOR.get_byte_value(); 128 / 8 * 296];
        let mut display = VarDisplay::new(width, height, &mut buffer);

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
        use crate::epd2in9::DEFAULT_BACKGROUND_COLOR;
        let width = 128;
        let height = 296;

        let mut buffer = [DEFAULT_BACKGROUND_COLOR.get_byte_value(); 128 / 8 * 296];
        let mut display = VarDisplay::new(width, height, &mut buffer);

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
