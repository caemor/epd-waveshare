//! B/W Color for EPDs

#[cfg(feature = "graphics")]
use embedded_graphics::pixelcolor::BinaryColor;
#[cfg(feature = "graphics")]
use embedded_graphics::pixelcolor::PixelColor;

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

/// For the 5in65 7 Color Display
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum OctColor {
    /// Black Color
    Black = 0x00,
    /// White Color
    White = 0x01,
    /// Green Color
    Green = 0x02,
    /// Blue Color
    Blue = 0x03,
    /// Red Color
    Red = 0x04,
    /// Yellow Color
    Yellow = 0x05,
    /// Orange Color
    Orange = 0x06,
    /// HiZ / Clean Color
    HiZ = 0x07,
}

impl From<()> for OctColor {
    fn from(_: ()) -> OctColor {
        OctColor::White
    }
}

#[cfg(feature = "graphics")]
impl PixelColor for OctColor {
    type Raw = ();
}

impl OctColor {
    /// Gets the Nibble representation of the Color as needed by the display
    pub fn get_nibble(self) -> u8 {
        self as u8
    }
    /// Converts two colors into a single byte for the Display
    pub fn colors_byte(a: OctColor, b: OctColor) -> u8 {
        a.get_nibble() << 4 | b.get_nibble()
    }

    ///Take the nibble (lower 4 bits) and convert to an OctColor if possible
    pub fn from_nibble(nibble: u8) -> Result<OctColor, ()> {
        match nibble & 0xf {
            0x00 => Ok(OctColor::Black),
            0x01 => Ok(OctColor::White),
            0x02 => Ok(OctColor::Green),
            0x03 => Ok(OctColor::Blue),
            0x04 => Ok(OctColor::Red),
            0x05 => Ok(OctColor::Yellow),
            0x06 => Ok(OctColor::Orange),
            0x07 => Ok(OctColor::HiZ),
            _ => Err(()),
        }
    }
    ///Split the nibbles of a single byte and convert both to an OctColor if possible
    pub fn split_byte(byte: u8) -> Result<(OctColor, OctColor), ()> {
        let low = OctColor::from_nibble(byte & 0xf)?;
        let high = OctColor::from_nibble((byte >> 4) & 0xf)?;
        Ok((high, low))
    }
    /// Converts to limited range of RGB values.
    pub fn rgb(self) -> (u8, u8, u8) {
        match self {
            OctColor::White => (0xff, 0xff, 0xff),
            OctColor::Black => (0x00, 0x00, 0x00),
            OctColor::Green => (0x00, 0xff, 0x00),
            OctColor::Blue => (0x00, 0x00, 0xff),
            OctColor::Red => (0xff, 0x00, 0x00),
            OctColor::Yellow => (0xff, 0xff, 0x00),
            OctColor::Orange => (0xff, 0x80, 0x00),
            OctColor::HiZ => (0x80, 0x80, 0x80), /* looks greyish */
        }
    }
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

    #[test]
    fn test_oct() {
        let left = OctColor::Red;
        let right = OctColor::Green;
        assert_eq!(
            OctColor::split_byte(OctColor::colors_byte(left, right)),
            Ok((left, right))
        );
    }
}
