use crate::color::TriColor;
use crate::epd7in5b_v2::{DEFAULT_BACKGROUND_COLOR, HEIGHT, NUM_DISPLAY_BYTES, WIDTH};
use crate::graphics::{DisplayRotation, TriDisplay};
use embedded_graphics_core::prelude::*;

/// Full size buffer for use with the 7in5 EPD
///
/// Can also be manually constructed:
/// `buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value(); 2 * NUM_DISPLAY_BYTES]`
pub struct Display7in5 {
    buffer: [u8; 2 * NUM_DISPLAY_BYTES],
    rotation: DisplayRotation,
}

impl Default for Display7in5 {
    // inline is necessary here to allow heap allocation via Box on stack limited programs
    #[inline(always)]
    fn default() -> Self {
        Display7in5 {
            // This way of initializing doesn't work for bicolor buffer
            buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value(); 2*NUM_DISPLAY_BYTES],
            rotation: DisplayRotation::default(),
        }
    }
}

impl Display7in5 {
    /// Please call me after creating a default Display7in5 for bicolor display, otherwise
    /// the default color won't be correct
    pub fn init(&mut self) {
        match DEFAULT_BACKGROUND_COLOR {
            // white and chromatic are both 1
            TriColor::White => self.buffer[NUM_DISPLAY_BYTES..].fill(0x00),
            TriColor::Black => {},
            TriColor::Chromatic => self.buffer[NUM_DISPLAY_BYTES..].fill(0xFF),
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
            self.draw_helper_tri(
                WIDTH,
                HEIGHT,
                pixel,
                crate::graphics::DisplayColorRendering::Negative,
            )?;
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
        NUM_DISPLAY_BYTES
    }

    fn bw_buffer(&self) -> &[u8] {
        &self.buffer[0..self.chromatic_offset()]
    }

    fn chromatic_buffer(&self) -> &[u8] {
        &self.buffer[self.chromatic_offset()..]
    }

    fn clear_buffer(&mut self, background_color: TriColor) {
        let offset = self.chromatic_offset();

        for (i, elem) in self.get_mut_buffer().iter_mut().enumerate() {
            if i < offset {
                *elem = background_color.get_byte_value();
            }
            // for V3, white in the BW buffer is 255. But in the chromatic buffer 255 is red.
            // This means that the chromatic buffer needs to be inverted when clearing
            else {
                *elem = background_color.get_byte_value() ^ 0xFF;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::epd7in5_v3;

    // test buffer length
    #[test]
    fn graphics_size() {
        let display = Display7in5::default();
        assert_eq!(display.buffer().len(), 96000);
    }

    // test default background color on all bytes
    #[test]
    fn graphics_default() {
        let display = Display7in5::default();
        for &byte in display.buffer() {
            assert_eq!(byte, epd7in5_v3::DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }
}
