use crate::color::TriColor;
use crate::epd2in13bc::{DEFAULT_BACKGROUND_COLOR, HEIGHT, NUM_DISPLAY_BITS, WIDTH};
use crate::graphics::{DisplayColorRendering, DisplayRotation, TriDisplay};
use embedded_graphics_core::prelude::*;

/// Full size buffer for use with the 2.13" b/c EPD
///
/// Can also be manually constructed and be used together with VarDisplay
pub struct Display2in13bc {
    // one buffer for both b/w and for chromatic:
    // * &buffer[0..NUM_DISPLAY_BITS] for b/w buffer and
    // * &buffer[NUM_DISPLAY_BITS..2*NUM_DISPLAY_BITS] for chromatic buffer
    buffer: [u8; 2 * NUM_DISPLAY_BITS as usize],
    rotation: DisplayRotation,
}

impl Default for Display2in13bc {
    fn default() -> Self {
        Display2in13bc {
            buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value(); 2 * NUM_DISPLAY_BITS as usize],
            rotation: DisplayRotation::default(),
        }
    }
}

impl DrawTarget for Display2in13bc {
    type Color = TriColor;
    type Error = core::convert::Infallible;
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            self.draw_helper_tri(WIDTH, HEIGHT, pixel, DisplayColorRendering::Positive)?;
        }
        Ok(())
    }
}

impl OriginDimensions for Display2in13bc {
    fn size(&self) -> Size {
        Size::new(WIDTH, HEIGHT)
    }
}

impl TriDisplay for Display2in13bc {
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
