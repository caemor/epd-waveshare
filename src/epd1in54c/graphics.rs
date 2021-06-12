use crate::epd1in54c::{DEFAULT_BACKGROUND_COLOR, HEIGHT, NUM_DISPLAY_BITS, WIDTH};
use crate::graphics::{Display, DisplayRotation};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;

/// Full size buffer for use with the 1in54c EPD
///
/// Can also be manually constructed and be used together with VarDisplay
pub struct Display1in54c {
    buffer: [u8; NUM_DISPLAY_BITS as usize],
    rotation: DisplayRotation,
}

impl Default for Display1in54c {
    fn default() -> Self {
        Display1in54c {
            buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value(); NUM_DISPLAY_BITS as usize],
            rotation: DisplayRotation::default(),
        }
    }
}

impl DrawTarget for Display1in54c {
    type Color = BinaryColor;
    type Error = core::convert::Infallible;

    fn draw_pixel(&mut self, pixel: Pixel<BinaryColor>) -> Result<(), Self::Error> {
        self.draw_helper(WIDTH, HEIGHT, pixel)
    }

    fn size(&self) -> Size {
        Size::new(WIDTH, HEIGHT)
    }
}

impl Display for Display1in54c {
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
