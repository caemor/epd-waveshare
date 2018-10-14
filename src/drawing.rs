use color::Color;

/// Displayrotation 
#[derive(Clone, Copy)]
pub enum DisplayRotation {
    /// No rotation
    Rotate0,
    /// Rotate by 90 degrees clockwise
    Rotate90,
    /// Rotate by 180 degrees clockwise
    Rotate180,
    /// Rotate 270 degrees clockwise
    Rotate270,
}
impl Default for Displayorientation {
    fn default() -> Self {
        Displayorientation::Rotate0
    }
}

pub enum Display {
    Eink42BlackWhite,
}
impl Display {
    /// Gets the Dimensions of a dipslay in the following order:
    /// - Width
    /// - Height
    /// - Neccessary Buffersize
    pub fn get_dimensions(&self) -> (u16, u16, u16) {
        match self {
            Display::Eink42BlackWhite => (400, 300, 15000),
        }
    }
}

pub trait Buffer {
    fn get_buffer(&self) -> &[u8];
}

pub struct DisplayEink42BlackWhite {
    buffer: [u8; 400 * 300 / 8],
    rotation: Displayorientation, //TODO: check embedded_graphics for orientation
}
impl Default for DisplayEink42BlackWhite {
    fn default() -> Self {
        use epd4in2::constants::*;
        DisplayEink42BlackWhite {
            buffer: [
                DEFAULT_BACKGROUND_COLOR.get_full_byte(),
                WIDTH * HEIGHT / 8                
            ],
            rotation: DisplayRotation::default()
        }
    }
}
impl Buffer for DisplayEink42BlackWhite {
    fn get_buffer(&self) -> &[u8] {
        &self.buffer
    }
}
impl Drawing<u8> for DisplayEink42BlackWhite {
    fn draw<T>(&mut self, item_pixels: T)
    where
        T: Iterator<Item = Pixel<u8>>
    {
        for Pixel(UnsignedCoord(x,y), color) in item_pixels {
            let (idx, bit) = match self.rotation {
                Displayorientation::Rotate0 | Displayorientation::Rotate180 => (
                    (x as usize / 8 + (self.width as usize / 8) * y as usize),
                    0x80 >> (x % 8),
                ),
                Displayorientation::Rotate90 | Displayorientation::Rotate270 => (
                    y as usize / 8 * self.width as usize + x as usize,
                    0x80 >> (y % 8),
                ),
            };

            if idx >= self.buffer.len() {
                return;
            }

            match color {
                Color::Black => {
                    self.buffer[idx] &= !bit;
                }
                Color::White => {
                    self.buffer[idx] |= bit;
                }
            }            
        }
    }
}

// impl Drawing<u8> for DisplayRibbonLeft {
//     fn draw<T>(&mut self, item_pixels: T)
//     where
//         T: Iterator<Item = Pixel<u8>>,
//     {
//         for Pixel(UnsignedCoord(x, y), color) in item_pixels {
//             if y > 127 || x > 295 {
//                 continue;
//             }
//             let cell = &mut self.0[y as usize / 8 + (295 - x as usize) * 128 / 8];
//             let bit = 7 - y % 8;
//             if color != 0 {
//                 *cell &= !(1 << bit);
//             } else {
//                 *cell |= 1 << bit;
//             }
//         }
//     }
// }



    // /// Draw a single Pixel with `color`
    // ///
    // /// limited to i16::max images (buffer_size) at the moment
    // pub fn draw_pixel(&mut self, x: u16, y: u16, color: &Color) {
    //     let (idx, bit) = match self.rotation {
    //         Displayorientation::Rotate0 | Displayorientation::Rotate180 => (
    //             (x as usize / 8 + (self.width as usize / 8) * y as usize),
    //             0x80 >> (x % 8),
    //         ),
    //         Displayorientation::Rotate90 | Displayorientation::Rotate270 => (
    //             y as usize / 8 * self.width as usize + x as usize,
    //             0x80 >> (y % 8),
    //         ),
    //     };

    //     if idx >= self.buffer.len() {
    //         return;
    //     }

    //     match color {
    //         Color::Black => {
    //             self.buffer[idx] &= !bit;
    //         }
    //         Color::White => {
    //             self.buffer[idx] |= bit;
    //         }
    //     }
    // }

    // /// Draw a single Pixel with `color`
    // ///
    // /// limited to i16::max images (buffer_size) at the moment
    // #[allow(dead_code)]
    // fn draw_byte(&mut self, x: u16, y: u16, filling: u8, color: &Color) {
    //     let idx = match self.rotation {
    //         Displayorientation::Rotate0 | Displayorientation::Rotate180 => {
    //             x as usize / 8 + (self.width as usize / 8) * y as usize
    //         },
    //         Displayorientation::Rotate90 | Displayorientation::Rotate270 => {
    //             y as usize / 8 + (self.width as usize / 8) * x as usize
    //         },
    //     };

    //     if idx >= self.buffer.len() {
    //         return;
    //     }

    //     match color {
    //         Color::Black => {
    //             self.buffer[idx] = !filling;
    //         },
    //         Color::White => {
    //             self.buffer[idx] = filling;
    //         }
    //     }
    // }

//TODO: write tests