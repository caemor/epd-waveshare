use epd1in54::{DEFAULT_BACKGROUND_COLOR, HEIGHT, WIDTH};

/// Full size buffer for use with the 1in54 EPD
///
/// Can also be manuall constructed:
/// `buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value(); WIDTH / 8 * HEIGHT]`
pub struct Buffer1in54BlackWhite {
    pub buffer: [u8; WIDTH as usize * HEIGHT as usize / 8],
}

impl Default for Buffer1in54BlackWhite {
    fn default() -> Self {
        Buffer1in54BlackWhite {
            buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value();
                WIDTH as usize * HEIGHT as usize / 8],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use color::Color;
    use embedded_graphics::coord::Coord;
    use embedded_graphics::prelude::*;
    use embedded_graphics::primitives::Line;
    use graphics::{Display, DisplayRotation};

    // test buffer length
    #[test]
    fn graphics_size() {
        let mut display1in54 = Buffer1in54BlackWhite::default();
        let display = Display::new(WIDTH, HEIGHT, &mut display1in54.buffer);
        assert_eq!(display.buffer().len(), 5000);
    }

    // test default background color on all bytes
    #[test]
    fn graphics_default() {
        let mut display1in54 = Buffer1in54BlackWhite::default();
        let display = Display::new(WIDTH, HEIGHT, &mut display1in54.buffer);
        for &byte in display.buffer() {
            assert_eq!(byte, DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }

    #[test]
    fn graphics_rotation_0() {
        let mut display1in54 = Buffer1in54BlackWhite::default();
        let mut display = Display::new(WIDTH, HEIGHT, &mut display1in54.buffer);
        display.draw(
            Line::new(Coord::new(0, 0), Coord::new(7, 0))
                .with_stroke(Some(Color::Black))
                .into_iter(),
        );

        let buffer = display.buffer();

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }

    #[test]
    fn graphics_rotation_90() {
        let mut display1in54 = Buffer1in54BlackWhite::default();
        let mut display = Display::new(WIDTH, HEIGHT, &mut display1in54.buffer);
        display.set_rotation(DisplayRotation::Rotate90);
        display.draw(
            Line::new(Coord::new(0, 192), Coord::new(0, 199))
                .with_stroke(Some(Color::Black))
                .into_iter(),
        );

        let buffer = display.buffer();

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }

    #[test]
    fn graphics_rotation_180() {
        let mut display1in54 = Buffer1in54BlackWhite::default();
        let mut display = Display::new(WIDTH, HEIGHT, &mut display1in54.buffer);
        display.set_rotation(DisplayRotation::Rotate180);
        display.draw(
            Line::new(Coord::new(192, 199), Coord::new(199, 199))
                .with_stroke(Some(Color::Black))
                .into_iter(),
        );

        let buffer = display.buffer();

        extern crate std;
        std::println!("{:?}", buffer);

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }

    #[test]
    fn graphics_rotation_270() {
        let mut display1in54 = Buffer1in54BlackWhite::default();
        let mut display = Display::new(WIDTH, HEIGHT, &mut display1in54.buffer);
        display.set_rotation(DisplayRotation::Rotate270);
        display.draw(
            Line::new(Coord::new(199, 0), Coord::new(199, 7))
                .with_stroke(Some(Color::Black))
                .into_iter(),
        );

        let buffer = display.buffer();

        extern crate std;
        std::println!("{:?}", buffer);

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }
}
