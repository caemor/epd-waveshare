use epd2in9::{DEFAULT_BACKGROUND_COLOR, WIDTH, HEIGHT};

pub struct Buffer2in9 {
    pub buffer: [u8; WIDTH as usize * HEIGHT as usize / 8],
}

impl Default for Buffer2in9 {
    fn default() -> Self {
        Buffer2in9 {
            buffer: [
                DEFAULT_BACKGROUND_COLOR.get_byte_value();
                WIDTH as usize * HEIGHT as usize / 8                
            ]
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use graphics::Display;

    // test buffer length
    #[test]
    fn graphics_size() {
        let mut buffer = Buffer2in9::default();
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