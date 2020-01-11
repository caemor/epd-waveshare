use crate::epd7in5_v2::{DEFAULT_BACKGROUND_COLOR, HEIGHT, WIDTH};
use crate::graphics::{Display, DisplayRotation};
use crate::prelude::*;
use embedded_graphics::prelude::*;

/// Full size buffer for use with the 7in5 EPD
///
/// Can also be manually constructed:
/// `buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value(); WIDTH / 8 * HEIGHT]`
pub struct Display7in5 {
    buffer: [u8; WIDTH as usize * HEIGHT as usize / 8],
    rotation: DisplayRotation,
}

impl Default for Display7in5 {
    fn default() -> Self {
        Display7in5 {
            buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value();
                WIDTH as usize * HEIGHT as usize / 8],
            rotation: DisplayRotation::default(),
        }
    }
}

impl Drawing<Color> for Display7in5 {
    fn draw<T>(&mut self, item_pixels: T)
    where
        T: IntoIterator<Item = Pixel<Color>>,
    {
        self.draw_helper(WIDTH, HEIGHT, item_pixels);
    }
}

impl Display for Display7in5 {
    fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    fn get_mut_buffer(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    fn set_rotation(&mut self, rotation: DisplayRotation) {
        self.rotation = rotation;
    }

    fn rotation(&self) -> DisplayRotation {
        self.rotation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;
    use crate::epd7in5_v2;
    use crate::graphics::{Display, DisplayRotation};
    use embedded_graphics::coord::Coord;
    use embedded_graphics::primitives::Line;

    // test buffer length
    #[test]
    fn graphics_size() {
        let display = Display7in5::default();
        assert_eq!(display.buffer().len(), 48000);
    }

    // test default background color on all bytes
    #[test]
    fn graphics_default() {
        let display = Display7in5::default();
        for &byte in display.buffer() {
            assert_eq!(byte, epd7in5_v2::DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }

    #[test]
    fn graphics_rotation_0() {
        let mut display = Display7in5::default();
        display.draw(
            Line::new(Coord::new(0, 0), Coord::new(7, 0))
                .stroke(Some(Color::Black))
                .into_iter(),
        );

        let buffer = display.buffer();

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, epd7in5_v2::DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }

    #[test]
    fn graphics_rotation_90() {
        let mut display = Display7in5::default();
        display.set_rotation(DisplayRotation::Rotate90);
        display.draw(
            Line::new(Coord::new(0, 792), Coord::new(0, 799))
                .stroke(Some(Color::Black))
                .into_iter(),
        );

        let buffer = display.buffer();

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, epd7in5_v2::DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }

    #[test]
    fn graphics_rotation_180() {
        let mut display = Display7in5::default();
        display.set_rotation(DisplayRotation::Rotate180);
        display.draw(
            Line::new(Coord::new(792, 479), Coord::new(799, 479))
                .stroke(Some(Color::Black))
                .into_iter(),
        );

        let buffer = display.buffer();

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, epd7in5_v2::DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }

    #[test]
    fn graphics_rotation_270() {
        let mut display = Display7in5::default();
        display.set_rotation(DisplayRotation::Rotate270);
        display.draw(
            Line::new(Coord::new(479, 0), Coord::new(479, 7))
                .stroke(Some(Color::Black))
                .into_iter(),
        );

        let buffer = display.buffer();

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, epd7in5_v2::DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }
}
