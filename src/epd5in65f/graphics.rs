use crate::epd5in65f::{DEFAULT_BACKGROUND_COLOR, HEIGHT, WIDTH};
use crate::graphics::{OctDisplay, DisplayRotation};
use embedded_graphics::prelude::*;
use crate::color::OctColor;

/// Full size buffer for use with the 5in65f EPD
///
/// Can also be manually constructed:
/// `buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value(); WIDTH / 2 * HEIGHT]`
pub struct Display5in65f {
    buffer: [u8; WIDTH as usize * HEIGHT as usize / 2],
    rotation: DisplayRotation,
}

impl Default for Display5in65f {
    fn default() -> Self {
        Display5in65f {
            buffer: [OctColor::colors_byte(DEFAULT_BACKGROUND_COLOR, DEFAULT_BACKGROUND_COLOR);
                WIDTH as usize * HEIGHT as usize / 2],
            rotation: DisplayRotation::default(),
        }
    }
}

impl DrawTarget<OctColor> for Display5in65f {
    type Error = core::convert::Infallible;

    fn draw_pixel(&mut self, pixel: Pixel<OctColor>) -> Result<(), Self::Error> {
        self.draw_helper(WIDTH, HEIGHT, pixel)
    }

    fn size(&self) -> Size {
        Size::new(WIDTH, HEIGHT)
    }
}

impl OctDisplay for Display5in65f {
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
    use crate::epd5in65f;
    use crate::graphics::{OctDisplay, DisplayRotation};
    use embedded_graphics::{primitives::Line, style::PrimitiveStyle};

    // test buffer length
    #[test]
    fn graphics_size() {
        let display = Display5in65f::default();
        assert_eq!(display.buffer().len(), 448*600 / 2);
    }

    // test default background color on all bytes
    #[test]
    fn graphics_default() {
        let display = Display5in65f::default();
        for &byte in display.buffer() {
            assert_eq!(byte, OctColor::colors_byte(
                epd5in65f::DEFAULT_BACKGROUND_COLOR,
                epd5in65f::DEFAULT_BACKGROUND_COLOR,
            ));
        }
    }

    #[test]
    fn graphics_rotation_0() {
        let mut display = Display5in65f::default();

        let _ = Line::new(Point::new(0, 0), Point::new(1, 0))
            .into_styled(PrimitiveStyle::with_stroke(OctColor::Black, 1))
            .draw(&mut display);

        let buffer = display.buffer();
        
        for &byte in buffer.iter().take(1) {
            assert_eq!(OctColor::split_byte(byte), Ok((OctColor::Black, OctColor::Black)));
        }

        for &byte in buffer.iter().skip(1) {
            assert_eq!(
                OctColor::split_byte(byte),
                Ok((epd5in65f::DEFAULT_BACKGROUND_COLOR, epd5in65f::DEFAULT_BACKGROUND_COLOR))
            );
        }
    }

    #[test]
    fn graphics_rotation_90() {
        let mut display = Display5in65f::default();
        display.set_rotation(DisplayRotation::Rotate90);

        let _ = Line::new(Point::new(0, WIDTH as i32 - 2), Point::new(0, WIDTH as i32- 1))
            .into_styled(PrimitiveStyle::with_stroke(OctColor::Black, 1))
            .draw(&mut display);

        let buffer = display.buffer();

        for &byte in buffer.iter().take(1) {
            assert_eq!(OctColor::split_byte(byte), Ok((OctColor::Black, OctColor::Black)));
        }

        for &byte in buffer.iter().skip(1) {
            assert_eq!(
                OctColor::split_byte(byte),
                Ok((epd5in65f::DEFAULT_BACKGROUND_COLOR, epd5in65f::DEFAULT_BACKGROUND_COLOR))
            );
        }
    }

    #[test]
    fn graphics_rotation_180() {
        let mut display = Display5in65f::default();
        display.set_rotation(DisplayRotation::Rotate180);

        let _ = Line::new(Point::new(WIDTH as i32 - 2, HEIGHT as i32 - 1),
                          Point::new(WIDTH as i32 - 1, HEIGHT as i32 - 1))
            .into_styled(PrimitiveStyle::with_stroke(OctColor::Black, 1))
            .draw(&mut display);

        let buffer = display.buffer();
        
        for &byte in buffer.iter().take(1) {
            assert_eq!(OctColor::split_byte(byte), Ok((OctColor::Black, OctColor::Black)));
        }

        for &byte in buffer.iter().skip(1) {
            assert_eq!(
                OctColor::split_byte(byte),
                Ok((epd5in65f::DEFAULT_BACKGROUND_COLOR, epd5in65f::DEFAULT_BACKGROUND_COLOR))
            );
        }
    }

    #[test]
    fn graphics_rotation_270() {
        let mut display = Display5in65f::default();
        display.set_rotation(DisplayRotation::Rotate270);

        let _ = Line::new(Point::new(HEIGHT as i32 -1, 0),
                          Point::new(HEIGHT as i32 -1, 1))
            .into_styled(PrimitiveStyle::with_stroke(OctColor::Black, 1))
            .draw(&mut display);

        let buffer = display.buffer();

        for &byte in buffer.iter().take(1) {
            assert_eq!(OctColor::split_byte(byte), Ok((OctColor::Black, OctColor::Black)));
        }

        for &byte in buffer.iter().skip(1) {
            assert_eq!(
                OctColor::split_byte(byte),
                Ok((epd5in65f::DEFAULT_BACKGROUND_COLOR, epd5in65f::DEFAULT_BACKGROUND_COLOR))
            );
        }
    }
}
