use crate::buffer_len;
use crate::epd2in13_v2::{DEFAULT_BACKGROUND_COLOR, HEIGHT, WIDTH};
use crate::graphics::{Display, DisplayRotation};
use embedded_graphics_core::pixelcolor::BinaryColor;
use embedded_graphics_core::prelude::*;

/// Full size buffer for use with the 2in13 v2 EPD
///
/// Can also be manually constructed:
/// `buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value(); WIDTH / 8 * HEIGHT]`
pub struct Display2in13 {
    buffer: [u8; buffer_len(WIDTH as usize, HEIGHT as usize)],
    rotation: DisplayRotation,
}

impl Default for Display2in13 {
    fn default() -> Self {
        Display2in13 {
            buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value();
                buffer_len(WIDTH as usize, HEIGHT as usize)],
            rotation: DisplayRotation::default(),
        }
    }
}

impl DrawTarget for Display2in13 {
    type Color = BinaryColor;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            self.draw_helper(WIDTH, HEIGHT, pixel)?;
        }
        Ok(())
    }
}

impl OriginDimensions for Display2in13 {
    fn size(&self) -> Size {
        Size::new(WIDTH, HEIGHT)
    }
}

impl Display for Display2in13 {
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
    use crate::color::{Black, Color};
    use crate::epd2in13_v2;
    use crate::graphics::{Display, DisplayRotation};
    use embedded_graphics::{
        prelude::*,
        primitives::{Line, PrimitiveStyle},
    };

    // test buffer length
    #[test]
    fn graphics_size() {
        let display = Display2in13::default();
        assert_eq!(display.buffer().len(), 4000);
    }

    // test default background color on all bytes
    #[test]
    fn graphics_default() {
        let display = Display2in13::default();
        for &byte in display.buffer() {
            assert_eq!(byte, epd2in13_v2::DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }

    #[test]
    fn graphics_rotation_0() {
        let mut display = Display2in13::default();

        let _ = Line::new(Point::new(0, 0), Point::new(7, 0))
            .into_styled(PrimitiveStyle::with_stroke(Black, 1))
            .draw(&mut display);

        let buffer = display.buffer();

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, epd2in13_v2::DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }

    #[test]
    fn graphics_rotation_90() {
        let mut display = Display2in13::default();
        display.set_rotation(DisplayRotation::Rotate90);

        let _ = Line::new(
            Point::new(0, (WIDTH - 8) as i32),
            Point::new(0, (WIDTH - 1) as i32),
        )
        .into_styled(PrimitiveStyle::with_stroke(Black, 1))
        .draw(&mut display);

        let buffer = display.buffer();

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, epd2in13_v2::DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }

    #[test]
    fn graphics_rotation_180() {
        let mut display = Display2in13::default();
        display.set_rotation(DisplayRotation::Rotate180);

        let _ = Line::new(
            Point::new((WIDTH - 8) as i32, (HEIGHT - 1) as i32),
            Point::new((WIDTH - 1) as i32, (HEIGHT - 1) as i32),
        )
        .into_styled(PrimitiveStyle::with_stroke(Black, 1))
        .draw(&mut display);

        let buffer = display.buffer();

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, epd2in13_v2::DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }

    #[test]
    fn graphics_rotation_270() {
        let mut display = Display2in13::default();
        display.set_rotation(DisplayRotation::Rotate270);

        let _ = Line::new(
            Point::new((HEIGHT - 1) as i32, 0),
            Point::new((HEIGHT - 1) as i32, 7),
        )
        .into_styled(PrimitiveStyle::with_stroke(Black, 1))
        .draw(&mut display);

        let buffer = display.buffer();

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, epd2in13_v2::DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }
}
