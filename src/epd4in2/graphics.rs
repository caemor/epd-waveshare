use crate::epd4in2::{DEFAULT_BACKGROUND_COLOR, HEIGHT, WIDTH};
use crate::graphics::{Display, DisplayRotation};
use crate::prelude::*;
use embedded_graphics::prelude::*;

/// Full size buffer for use with the 4in2 EPD
///
/// Can also be manuall constructed:
/// `buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value(); WIDTH / 8 * HEIGHT]`
pub struct Display4in2 {
    buffer: [u8; WIDTH as usize * HEIGHT as usize / 8],
    rotation: DisplayRotation,
}

impl Default for Display4in2 {
    fn default() -> Self {
        Display4in2 {
            buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value();
                WIDTH as usize * HEIGHT as usize / 8],
            rotation: DisplayRotation::default(),
        }
    }
}

impl Drawing<Color> for Display4in2 {
    fn draw<T>(&mut self, item_pixels: T)
    where
        T: IntoIterator<Item = Pixel<Color>>,
    {
        self.draw_helper(WIDTH, HEIGHT, item_pixels.into_iter());
    }
}

impl Display for Display4in2 {
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
    use crate::epd4in2;
    use crate::graphics::{Display, DisplayRotation};
    use embedded_graphics::coord::Coord;
    use embedded_graphics::primitives::Line;

    // test buffer length
    #[test]
    fn graphics_size() {
        let display = Display4in2::default();
        assert_eq!(display.buffer().len(), 15000);
    }

    // test default background color on all bytes
    #[test]
    fn graphics_default() {
        let display = Display4in2::default();
        for &byte in display.buffer() {
            assert_eq!(byte, epd4in2::DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }

    #[test]
    fn graphics_rotation_0() {
        let mut display = Display4in2::default();
        display.draw(
            Line::new(Coord::new(0, 0), Coord::new(7, 0))
                .with_stroke(Some(Color::Black))
                .into_iter(),
        );

        let buffer = display.buffer();

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, epd4in2::DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }

    #[test]
    fn graphics_rotation_90() {
        let mut display = Display4in2::default();
        display.set_rotation(DisplayRotation::Rotate90);
        display.draw(
            Line::new(Coord::new(0, 392), Coord::new(0, 399))
                .with_stroke(Some(Color::Black))
                .into_iter(),
        );

        let buffer = display.buffer();

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, epd4in2::DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }

    #[test]
    fn graphics_rotation_180() {
        let mut display = Display4in2::default();
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
            assert_eq!(byte, epd4in2::DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }

    #[test]
    fn graphics_rotation_270() {
        let mut display = Display4in2::default();
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
            assert_eq!(byte, epd4in2::DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }
}
