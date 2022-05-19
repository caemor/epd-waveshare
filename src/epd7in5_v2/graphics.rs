use crate::color::TriColor;
use crate::epd7in5_v2::{DEFAULT_BACKGROUND_COLOR, HEIGHT, WIDTH, NUM_DISPLAY_BITS};
use crate::graphics::{TriDisplay, DisplayRotation};
use embedded_graphics_core::pixelcolor::BinaryColor;
use embedded_graphics_core::prelude::*;

/// Full size buffer for use with the 7in5 EPD
///
/// Can also be manually constructed:
/// `buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value(); WIDTH / 8 * HEIGHT]`
pub struct Display7in5 {
    buffer: [u8; 2 * NUM_DISPLAY_BITS as usize],
    rotation: DisplayRotation,
}

impl Default for Display7in5 {
    fn default() -> Self {
        Display7in5 {
            buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value();
                2 * NUM_DISPLAY_BITS as usize],
            rotation: DisplayRotation::default(),
        }
    }
}

impl DrawTarget for Display7in5 {
    type Color = TriColor;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            self.draw_helper_tri(WIDTH, HEIGHT, pixel)?;
        }
        Ok(())
    }
}

impl OriginDimensions for Display7in5 {
    fn size(&self) -> Size {
        Size::new(WIDTH, HEIGHT)
    }
}

impl TriDisplay for Display7in5 {
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

    fn chromatic_offset(&self) -> usize {
        NUM_DISPLAY_BITS as usize
    }

    fn bw_buffer(&self) -> &[u8] {
        &self.buffer[0..self.chromatic_offset()]
    }

    fn chromatic_buffer(&self) -> &[u8] {
        &self.buffer[self.chromatic_offset()..]
    }
    /*fn buffer(&self) -> &[u8] {
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
    }*/
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::{Black, Color};
    use crate::epd7in5_v2;
    use crate::graphics::{Display, DisplayRotation};
    use embedded_graphics::{
        prelude::*,
        primitives::{Line, PrimitiveStyle},
    };

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

        let _ = Line::new(Point::new(0, 0), Point::new(7, 0))
            .into_styled(PrimitiveStyle::with_stroke(Black, 1))
            .draw(&mut display);

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

        let _ = Line::new(Point::new(0, 792), Point::new(0, 799))
            .into_styled(PrimitiveStyle::with_stroke(Black, 1))
            .draw(&mut display);

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

        let _ = Line::new(Point::new(792, 479), Point::new(799, 479))
            .into_styled(PrimitiveStyle::with_stroke(Black, 1))
            .draw(&mut display);

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

        let _ = Line::new(Point::new(479, 0), Point::new(479, 7))
            .into_styled(PrimitiveStyle::with_stroke(Black, 1))
            .draw(&mut display);

        let buffer = display.buffer();

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, epd7in5_v2::DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }
}
