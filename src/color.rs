
use embedded_graphics::prelude::*;

/// Only for the B/W Displays atm
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Color {
    Black,
    White,
}

impl Color {
    /// Get the color encoding of the color for one bit
    pub fn get_bit_value(&self) -> u8 {
        match self {
            Color::White => 0u8,
            Color::Black => 1u8,
        }
    }

    /// Gets a full byte of black or white pixels
    pub fn get_byte_value(&self) -> u8 {
        match self {
            Color::White => 0x00,
            Color::Black => 0xff,
        }
    }

    fn from_u8(val: u8) -> Self {
        match val {
            1 => Color::Black,
            0 => Color::White,
            e => panic!("DisplayColor only parses 0 and 1 (Black and White) and not `{}`", e),
        }
    }

    /// Get the color encoding of a specific bit in a byte
    ///
    /// input is the byte where one bit is gonna be selected
    /// pos is counted from the left (highest value) from 0 to 7
    /// remember: 1 is white, 0 is black
    /// Color is the color you want to draw with in the foreground
    pub(crate) fn get_color(input: u8, pos: u8, color: &Color) -> Color {
        if Color::is_drawable_pixel(input, pos) {
            Color::normal_color(color)
        } else {
            Color::inverse_color(color)
        }
    }

    // Inverses the given color from Black to White or from White to Black
    fn inverse_color(color: &Color) -> Color {
        match color {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    // Gives you a new owned copy of the color
    //TODO: just use clone?
    fn normal_color(color: &Color) -> Color {
        match color {
            Color::White => Color::White,
            Color::Black => Color::Black,
        }
    }

    //position counted from the left (highest value) from 0 to 7
    //remember: 1 is white, 0 is black
    pub(crate) fn is_drawable_pixel(input: u8, pos: u8) -> bool {
        ((input >> (7 - pos)) & 1u8) > 0u8
    }

    //TODO: does basically the same as get_color, so remove one of them?
    pub(crate) fn convert_color(input: u8, pos: u8, foreground_color: &Color) -> Color {
        //match color:
        //      - white for "nothing to draw"/background drawing
        //      - black for pixel to draw
        //
        //foreground color is the color you want to have in the foreground
        if Color::is_drawable_pixel(input, pos) {
            Color::normal_color(foreground_color)
        } else {
            Color::inverse_color(foreground_color)
        }
    }
}

impl PixelColor for Color {}

impl From<u8> for Color {
    fn from(value: u8) -> Self {
        Color::from_u8(value)
    }
}



#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn u8_conversion_black() {
        assert_eq!(Color::from(Color::Black.get_bit_value()), Color::Black);
        assert_eq!(Color::from(0u8).get_bit_value(), 0u8);
    }

    #[test]
    fn u8_conversion_white() {
        assert_eq!(Color::from(Color::White.get_bit_value()), Color::White);
        assert_eq!(Color::from(1u8).get_bit_value(), 1u8);
    }
}
