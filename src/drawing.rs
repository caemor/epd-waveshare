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
    fn get_buffer(&self) -> &[u8];
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
    fn get_buffer(&self) -> &[u8] {
        &self.buffer
    }
    fn set_rotation(&mut self, rotation: DisplayRotation) {
        self.rotation = rotation;
    }
    fn rotation(&self) -> DisplayRotation {
        self.rotation
    }
}

impl Drawing<Color> for DisplayEink42BlackWhite {
    fn draw<T>(&mut self, item_pixels: T)
    where
        T: Iterator<Item = Pixel<Color>>
    {
        use epd4in2::constants::{DEFAULT_BACKGROUND_COLOR, WIDTH, HEIGHT};
        for Pixel(UnsignedCoord(x,y), color) in item_pixels {
            let (idx, bit) = match self.rotation {
                DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => (
                    (x as usize / 8 + (WIDTH as usize / 8) * y as usize),
                    0x80 >> (x % 8),
                ),
                DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => (
                    y as usize / 8 * WIDTH as usize + x as usize,
                    0x80 >> (y % 8),
                ),
            };

            if idx >= self.buffer.len() {
                return;
            }

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