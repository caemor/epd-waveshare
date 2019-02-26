use crate::epd2in9::{DEFAULT_BACKGROUND_COLOR, HEIGHT, WIDTH};
use crate::graphics::Display;
use crate::prelude::*;
use embedded_graphics::prelude::*;

/// Full size buffer for use with the 2in9 EPD
///
/// Can also be manuall constructed:
/// `buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value(); WIDTH / 8 * HEIGHT]`
pub struct Display2in9 {
    pub buffer: [u8; WIDTH as usize * HEIGHT as usize / 8],
    display: Display,
}

impl Default for Display2in9 {
    fn default() -> Self {
        Display2in9 {
            buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value();
                WIDTH as usize * HEIGHT as usize / 8],
            display: Display::default(),
        }
    }
}

impl Drawing<Color> for Display2in9 {
    fn draw<T>(&mut self, item_pixels: T)
    where
        T: Iterator<Item = Pixel<Color>>,
    {
        self.display.draw(&mut self.buffer, WIDTH, HEIGHT, item_pixels);
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::Display;

    // test buffer length
    #[test]
    fn graphics_size() {
        let mut display = Display2in9::default();
        let display = Display::new(WIDTH, HEIGHT, &mut buffer.buffer);
        assert_eq!(display.buffer().len(), 4736);
    }

    // test default background color on all bytes
    #[test]
    fn graphics_default() {
        let mut buffer = Buffer2in9::default();
        let display = Display::new(WIDTH, HEIGHT, &mut buffer.buffer);
        for &byte in display.buffer() {
            assert_eq!(byte, DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }
}
