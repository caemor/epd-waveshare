use crate::color::TriColor;
use crate::epd5in83b_v2::{DEFAULT_BACKGROUND_COLOR, HEIGHT, NUM_DISPLAY_BITS, WIDTH};
use crate::graphics::{DisplayColorRendering, DisplayRotation};
use crate::prelude::TriDisplay;
use embedded_graphics_core::prelude::*;

/// Full size buffer for use with the 5in83 EPD
///
/// Can also be manually constructed:
/// `buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value(); 2 * NUM_DISPLAY_BITS as usize]`
pub struct Display5in83 {
    buffer: [u8; 2 * NUM_DISPLAY_BITS as usize],
    rotation: DisplayRotation,
}

impl Default for Display5in83 {
    fn default() -> Self {
        let mut display = Display5in83 {
            buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value(); 2 * NUM_DISPLAY_BITS as usize],
            rotation: DisplayRotation::default(),
        };
        // We need to invert chromatic part to black so it will be render white
        let offset = display.chromatic_offset();
        display.buffer[offset..].fill(0x00);
        display
    }
}

impl DrawTarget for Display5in83 {
    type Color = TriColor;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            self.draw_helper_tri(WIDTH, HEIGHT, pixel, DisplayColorRendering::Negative)?;
        }
        Ok(())
    }
}

impl OriginDimensions for Display5in83 {
    fn size(&self) -> Size {
        Size::new(WIDTH, HEIGHT)
    }
}

impl TriDisplay for Display5in83 {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::TriColor::Black;
    use crate::epd5in83b_v2;
    use crate::graphics::DisplayRotation;
    use embedded_graphics::{
        prelude::*,
        primitives::{Line, PrimitiveStyle},
    };

    // test buffer length
    #[test]
    fn graphics_size() {
        let display = Display5in83::default();
        assert_eq!(display.buffer().len(), 77760); // (77760 = 648 * 480/8) * 2
    }

    // test default background color on all bytes
    #[test]
    fn graphics_default() {
        let display = Display5in83::default();
        for &byte in display.bw_buffer() {
            assert_eq!(
                byte,
                epd5in83b_v2::DEFAULT_BACKGROUND_COLOR.get_byte_value()
            );
        }
        for &byte in display.chromatic_buffer() {
            assert_eq!(byte, 0x00);
        }
    }

    #[test]
    fn graphics_rotation_0() {
        let mut display = Display5in83::default();
        let _ = Line::new(Point::new(0, 0), Point::new(7, 0))
            .into_styled(PrimitiveStyle::with_stroke(Black, 1))
            .draw(&mut display);

        let buffer = display.bw_buffer();

        assert_eq!(buffer[0], Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(
                byte,
                epd5in83b_v2::DEFAULT_BACKGROUND_COLOR.get_byte_value()
            );
        }
    }

    #[test]
    fn graphics_rotation_90() {
        let mut display = Display5in83::default();
        display.set_rotation(DisplayRotation::Rotate90);
        let _ = Line::new(Point::new(0, 640), Point::new(0, 647))
            .into_styled(PrimitiveStyle::with_stroke(Black, 1))
            .draw(&mut display);

        let buffer = display.bw_buffer();

        assert_eq!(buffer[0], Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(
                byte,
                epd5in83b_v2::DEFAULT_BACKGROUND_COLOR.get_byte_value()
            );
        }
    }

    #[test]
    fn graphics_rotation_180() {
        let mut display = Display5in83::default();
        display.set_rotation(DisplayRotation::Rotate180);
        let _ = Line::new(Point::new(640, 479), Point::new(647, 479))
            .into_styled(PrimitiveStyle::with_stroke(Black, 1))
            .draw(&mut display);

        let buffer = display.bw_buffer();

        assert_eq!(buffer[0], Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(
                byte,
                epd5in83b_v2::DEFAULT_BACKGROUND_COLOR.get_byte_value()
            );
        }
    }

    #[test]
    fn graphics_rotation_270() {
        let mut display = Display5in83::default();
        display.set_rotation(DisplayRotation::Rotate270);
        let _ = Line::new(Point::new(479, 0), Point::new(479, 7))
            .into_styled(PrimitiveStyle::with_stroke(Black, 1))
            .draw(&mut display);

        let buffer = display.bw_buffer();

        assert_eq!(buffer[0], Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(
                byte,
                epd5in83b_v2::DEFAULT_BACKGROUND_COLOR.get_byte_value()
            );
        }
    }
}
