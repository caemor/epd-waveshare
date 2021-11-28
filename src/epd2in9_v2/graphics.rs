use crate::epd2in9::{DEFAULT_BACKGROUND_COLOR, HEIGHT, WIDTH};
use crate::graphics::{Display, DisplayRotation};
use embedded_graphics_core::pixelcolor::BinaryColor;
use embedded_graphics_core::prelude::*;

/// Display with Fullsize buffer for use with the 2in9 EPD V2
///
/// Can also be manuall constructed:
/// `buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value(); WIDTH / 8 * HEIGHT]`
pub struct Display2in9 {
    buffer: [u8; WIDTH as usize * HEIGHT as usize / 8],
    rotation: DisplayRotation,
}

impl Default for Display2in9 {
    fn default() -> Self {
        Display2in9 {
            buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value();
                WIDTH as usize * HEIGHT as usize / 8],
            rotation: DisplayRotation::default(),
        }
    }
}

impl DrawTarget for Display2in9 {
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

impl OriginDimensions for Display2in9 {
    fn size(&self) -> Size {
        Size::new(WIDTH, HEIGHT)
    }
}

impl Display for Display2in9 {
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

    // test buffer length
    #[test]
    fn graphics_size() {
        let display = Display2in9::default();
        assert_eq!(display.buffer().len(), 4736);
    }

    // test default background color on all bytes
    #[test]
    fn graphics_default() {
        let display = Display2in9::default();
        for &byte in display.buffer() {
            assert_eq!(byte, DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }
}
