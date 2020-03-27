//! B/W Color for EPDs

#[cfg(feature = "graphics")]
use embedded_graphics::pixelcolor::BinaryColor;

#[cfg(feature = "graphics")]
pub use BinaryColor::Off as White;
#[cfg(feature = "graphics")]
pub use BinaryColor::On as Black;

/// Only for the Black/White-Displays
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Color {
    /// Black color
    Black,
    /// White color
    White,
}

/// Only for the Black/White/Color-Displays
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TriColor {
    /// Black color
    Black,
    /// White color
    White,
    /// Chromatic color
    Chromatic,
}

//TODO: Rename get_bit_value to bit() and get_byte_value to byte() ?

impl Color {
    /// Get the color encoding of the color for one bit
    pub fn get_bit_value(self) -> u8 {
        match self {
            Color::White => 1u8,
            Color::Black => 0u8,
        }
    }

    /// Gets a full byte of black or white pixels
    pub fn get_byte_value(self) -> u8 {
        match self {
            Color::White => 0xff,
            Color::Black => 0x00,
        }
    }

    /// Parses from u8 to Color
    fn from_u8(val: u8) -> Self {
        match val {
            0 => Color::Black,
            1 => Color::White,
            e => panic!(
                "DisplayColor only parses 0 and 1 (Black and White) and not `{}`",
                e
            ),
        }
    }

    /// Returns the inverse of the given color.
    ///
    /// Black returns White and White returns Black
    pub fn inverse(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

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
