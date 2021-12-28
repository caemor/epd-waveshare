//! Graphics Support for EPDs

use crate::buffer_len;
use crate::color::{Color, OctColor, TriColor};
use embedded_graphics_core::pixelcolor::BinaryColor;
use embedded_graphics_core::prelude::*;

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

impl Default for DisplayRotation {
    fn default() -> Self {
        DisplayRotation::Rotate0
    }
}

/// Display specific pixel output configuration
///
/// Different chromatic displays differently treat the bits in chromatic color planes.
/// Some of them ([crate::epd2in13bc]) will render a color pixel if bit is set for that pixel,
/// which is a [DisplayColorRendering::Positive] mode.
///
/// Other displays, like [crate::epd5in83b_v2] in opposite, will draw color pixel if bit is
/// cleared for that pixel, which is a [DisplayColorRendering::Negative] mode.
///
#[derive(Clone, Copy)]
pub enum DisplayColorRendering {
    /// Positive: chromatic doesn't override white, white bit cleared for black, white bit set for white, both bits set for chromatic
    Positive,
    /// Negative: chromatic does override white, both bits cleared for black, white bit set for white, red bit set for black
    Negative,
}

/// Necessary traits for all displays to implement for drawing
///
/// Adds support for:
/// - Drawing (With the help of DrawTarget/Embedded Graphics)
/// - Rotations
/// - Clearing
pub trait Display: DrawTarget<Color = BinaryColor> {
    /// Clears the buffer of the display with the chosen background color
    fn clear_buffer(&mut self, background_color: Color) {
        for elem in self.get_mut_buffer().iter_mut() {
            *elem = background_color.get_byte_value();
        }
    }

    /// Returns the buffer
    fn buffer(&self) -> &[u8];

    /// Returns a mutable buffer
    fn get_mut_buffer(&mut self) -> &mut [u8];

    /// Sets the rotation of the display
    fn set_rotation(&mut self, rotation: DisplayRotation);

    /// Get the current rotation of the display
    fn rotation(&self) -> DisplayRotation;

    /// Helperfunction for the Embedded Graphics draw trait
    ///
    /// Becomes uneccesary when const_generics become stablised
    fn draw_helper(
        &mut self,
        width: u32,
        height: u32,
        pixel: Pixel<BinaryColor>,
    ) -> Result<(), Self::Error> {
        let rotation = self.rotation();
        let buffer = self.get_mut_buffer();

        let Pixel(point, color) = pixel;
        if outside_display(point, width, height, rotation) {
            return Ok(());
        }

        // Give us index inside the buffer and the bit-position in that u8 which needs to be changed
        let (index, bit) = find_position(point.x as u32, point.y as u32, width, height, rotation);
        let index = index as usize;

        // "Draw" the Pixel on that bit
        match color {
            // Black
            BinaryColor::On => {
                buffer[index] &= !bit;
            }
            // White
            BinaryColor::Off => {
                buffer[index] |= bit;
            }
        }
        Ok(())
    }
}

/// Necessary traits for all displays to implement for drawing
///
/// Adds support for:
/// - Drawing (With the help of DrawTarget/Embedded Graphics)
/// - Rotations
/// - Clearing
pub trait TriDisplay: DrawTarget<Color = TriColor> {
    /// Clears the buffer of the display with the chosen background color
    fn clear_buffer(&mut self, background_color: TriColor) {
        for elem in self.get_mut_buffer().iter_mut() {
            *elem = background_color.get_byte_value();
        }
    }

    /// Returns the buffer
    fn buffer(&self) -> &[u8];

    /// Returns a mutable buffer
    fn get_mut_buffer(&mut self) -> &mut [u8];

    /// Sets the rotation of the display
    fn set_rotation(&mut self, rotation: DisplayRotation);

    /// Get the current rotation of the display
    fn rotation(&self) -> DisplayRotation;

    /// Get the offset into buffer where chromatic data starts
    fn chromatic_offset(&self) -> usize;

    /// return the b/w part of the buffer
    fn bw_buffer(&self) -> &[u8];

    /// return the chromatic part of the buffer
    fn chromatic_buffer(&self) -> &[u8];

    /// Helperfunction for the Embedded Graphics draw trait
    ///
    /// Becomes uneccesary when const_generics become stablised
    ///
    /// # Arguments
    ///
    /// * `width` - Screen width in pixels
    /// * `height` - Screen height in pixels
    /// * `pixel` - Pixel to draw
    /// * `rendering` - Chooses rendering mode for the color plane,
    ///  whether it is positive or negative. Check [DisplayColorRendering] for details.
    ///  This is a hardware defined setting, that needs to be checked from the datasheet.
    fn draw_helper_tri(
        &mut self,
        width: u32,
        height: u32,
        pixel: Pixel<TriColor>,
        rendering: DisplayColorRendering,
    ) -> Result<(), Self::Error> {
        let rotation = self.rotation();

        let Pixel(point, color) = pixel;
        if outside_display(point, width, height, rotation) {
            return Ok(());
        }

        // Give us index inside the buffer and the bit-position in that u8 which needs to be changed
        let (index, bit) = find_position(point.x as u32, point.y as u32, width, height, rotation);
        let index = index as usize;
        let offset = self.chromatic_offset();

        let buffer = self.get_mut_buffer();

        // "Draw" the Pixel on that bit
        match color {
            TriColor::Black => {
                // clear bit in bw-buffer -> black
                buffer[index] &= !bit;
                match rendering {
                    DisplayColorRendering::Positive => {
                        // set bit in chromatic-buffer -> white
                        buffer[index + offset] |= bit;
                    }
                    DisplayColorRendering::Negative => {
                        // clear bit in chromatic-buffer -> white
                        buffer[index + offset] &= !bit;
                    }
                }
            }
            TriColor::White => {
                // set bit in bw-buffer -> white
                buffer[index] |= bit;
                match rendering {
                    DisplayColorRendering::Positive => {
                        // set bit in chromatic-buffer -> white
                        buffer[index + offset] |= bit;
                    }
                    DisplayColorRendering::Negative => {
                        // clear bit in chromatic-buffer -> white
                        buffer[index + offset] &= !bit;
                    }
                }
            }
            TriColor::Chromatic => {
                match rendering {
                    DisplayColorRendering::Positive => {
                        // set bit in b/w buffer (white)
                        buffer[index] |= bit;
                        // clear bit in chromatic buffer -> chromatic
                        buffer[index + offset] &= !bit;
                    }
                    DisplayColorRendering::Negative => {
                        // set bit in b/w buffer (white)
                        buffer[index] |= bit;
                        // set bit in chromatic-buffer -> chromatic
                        buffer[index + offset] |= bit;
                    }
                }
            }
        }
        Ok(())
    }
}

/// Necessary traits for all displays to implement for drawing
///
/// Adds support for:
/// - Drawing (With the help of DrawTarget/Embedded Graphics)
/// - Rotations
/// - Clearing
pub trait OctDisplay: DrawTarget<Color = OctColor> {
    /// Clears the buffer of the display with the chosen background color
    fn clear_buffer(&mut self, background_color: OctColor) {
        for elem in self.get_mut_buffer().iter_mut() {
            *elem = OctColor::colors_byte(background_color, background_color);
        }
    }

    /// Returns the buffer
    fn buffer(&self) -> &[u8];

    /// Returns a mutable buffer
    fn get_mut_buffer(&mut self) -> &mut [u8];

    /// Sets the rotation of the display
    fn set_rotation(&mut self, rotation: DisplayRotation);

    /// Get the current rotation of the display
    fn rotation(&self) -> DisplayRotation;

    /// Helperfunction for the Embedded Graphics draw trait
    ///
    /// Becomes uneccesary when const_generics become stablised
    fn draw_helper(
        &mut self,
        width: u32,
        height: u32,
        pixel: Pixel<OctColor>,
    ) -> Result<(), Self::Error> {
        let rotation = self.rotation();
        let buffer = self.get_mut_buffer();

        let Pixel(point, color) = pixel;
        if outside_display(point, width, height, rotation) {
            return Ok(());
        }

        // Give us index inside the buffer and the bit-position in that u8 which needs to be changed
        let (index, upper) =
            find_oct_position(point.x as u32, point.y as u32, width, height, rotation);
        let index = index as usize;

        // "Draw" the Pixel on that bit
        let (mask, color_nibble) = if upper {
            (0x0f, color.get_nibble() << 4)
        } else {
            (0xf0, color.get_nibble())
        };
        buffer[index] = (buffer[index] & mask) | color_nibble;
        Ok(())
    }
}

/// A variable Display without a predefined buffer
///
/// The buffer can be created as following:
/// buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value(); WIDTH / 8 * HEIGHT]
/// If WIDTH is not a multiple of 8, don't forget to round it up (ie. (WIDTH + 7) / 8)
///
/// Example:
/// ```rust,no_run
/// # use epd_waveshare::epd2in9::DEFAULT_BACKGROUND_COLOR;
/// # use epd_waveshare::prelude::*;
/// # use epd_waveshare::graphics::VarDisplay;
/// # use epd_waveshare::color::Black;
/// # use embedded_graphics::prelude::*;
/// # use embedded_graphics::primitives::{Circle, Line, PrimitiveStyle};
/// let width = 128;
/// let height = 296;
///
/// let mut buffer = [DEFAULT_BACKGROUND_COLOR.get_byte_value(); 128 / 8 * 296];
/// let mut display = VarDisplay::new(width, height, &mut buffer);
///
/// display.set_rotation(DisplayRotation::Rotate90);
///
/// let _ = Line::new(Point::new(0, 120), Point::new(0, 295))
///         .into_styled(PrimitiveStyle::with_stroke(Black, 1))
///         .draw(&mut display);
/// ```
pub struct VarDisplay<'a> {
    width: u32,
    height: u32,
    rotation: DisplayRotation,
    buffer: &'a mut [u8], //buffer: Box<u8>//[u8; 15000]
}

impl<'a> VarDisplay<'a> {
    /// Create a new variable sized display.
    ///
    /// Buffersize must be at least (width + 7) / 8 * height bytes.
    pub fn new(width: u32, height: u32, buffer: &'a mut [u8]) -> VarDisplay<'a> {
        let len = buffer.len() as u32;
        assert!(buffer_len(width as usize, height as usize) >= len as usize);
        VarDisplay {
            width,
            height,
            rotation: DisplayRotation::default(),
            buffer,
        }
    }
}

impl<'a> DrawTarget for VarDisplay<'a> {
    type Color = BinaryColor;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            self.draw_helper(self.width, self.height, pixel)?;
        }
        Ok(())
    }
}

impl<'a> OriginDimensions for VarDisplay<'a> {
    fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }
}

impl<'a> Display for VarDisplay<'a> {
    fn buffer(&self) -> &[u8] {
        self.buffer
    }

    fn get_mut_buffer(&mut self) -> &mut [u8] {
        self.buffer
    }

    fn set_rotation(&mut self, rotation: DisplayRotation) {
        self.rotation = rotation;
    }

    fn rotation(&self) -> DisplayRotation {
        self.rotation
    }
}

// Checks if a pos is outside the defined display
fn outside_display(p: Point, width: u32, height: u32, rotation: DisplayRotation) -> bool {
    if p.x < 0 || p.y < 0 {
        return true;
    }
    let (x, y) = (p.x as u32, p.y as u32);
    match rotation {
        DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => {
            if x >= width || y >= height {
                return true;
            }
        }
        DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => {
            if y >= width || x >= height {
                return true;
            }
        }
    }
    false
}

fn find_rotation(x: u32, y: u32, width: u32, height: u32, rotation: DisplayRotation) -> (u32, u32) {
    let nx;
    let ny;
    match rotation {
        DisplayRotation::Rotate0 => {
            nx = x;
            ny = y;
        }
        DisplayRotation::Rotate90 => {
            nx = width - 1 - y;
            ny = x;
        }
        DisplayRotation::Rotate180 => {
            nx = width - 1 - x;
            ny = height - 1 - y;
        }
        DisplayRotation::Rotate270 => {
            nx = y;
            ny = height - 1 - x;
        }
    }
    (nx, ny)
}

#[rustfmt::skip]
//returns index position in the u8-slice and the bit-position inside that u8
fn find_oct_position(x: u32, y: u32, width: u32, height: u32, rotation: DisplayRotation) -> (u32, bool) {
    let (nx, ny) = find_rotation(x, y, width, height, rotation);
    (
        /* what byte address is this? */
        nx / 2 + (width / 2) * ny,
        /* is this the lower nibble (within byte)? */
        (nx & 0x1) == 0,
    )
}

#[rustfmt::skip]
//returns index position in the u8-slice and the bit-position inside that u8
fn find_position(x: u32, y: u32, width: u32, height: u32, rotation: DisplayRotation) -> (u32, u8) {
    let (nx, ny) = find_rotation(x, y, width, height, rotation);
    (
        nx / 8 + ((width + 7) / 8) * ny,
        0x80 >> (nx % 8),
    )
}

#[cfg(test)]
mod tests {
    use super::{buffer_len, find_position, outside_display, Display, DisplayRotation, VarDisplay};
    use crate::color::Black;
    use crate::color::Color;
    use embedded_graphics::{
        prelude::*,
        primitives::{Line, PrimitiveStyle},
    };

    #[test]
    fn buffer_clear() {
        use crate::epd4in2::{HEIGHT, WIDTH};

        let mut buffer =
            [Color::Black.get_byte_value(); buffer_len(WIDTH as usize, HEIGHT as usize)];
        let mut display = VarDisplay::new(WIDTH, HEIGHT, &mut buffer);

        for &byte in display.buffer.iter() {
            assert_eq!(byte, Color::Black.get_byte_value());
        }

        display.clear_buffer(Color::White);

        for &byte in display.buffer.iter() {
            assert_eq!(byte, Color::White.get_byte_value());
        }
    }

    #[test]
    fn rotation_overflow() {
        use crate::epd4in2::{HEIGHT, WIDTH};
        let width = WIDTH as u32;
        let height = HEIGHT as u32;
        test_rotation_overflow(width, height, DisplayRotation::Rotate0);
        test_rotation_overflow(width, height, DisplayRotation::Rotate90);
        test_rotation_overflow(width, height, DisplayRotation::Rotate180);
        test_rotation_overflow(width, height, DisplayRotation::Rotate270);
    }

    fn test_rotation_overflow(width: u32, height: u32, rotation2: DisplayRotation) {
        let max_value = width / 8 * height;
        for x in 0..(width + height) {
            //limit x because it runs too long
            for y in 0..(u32::max_value()) {
                if outside_display(Point::new(x as i32, y as i32), width, height, rotation2) {
                    break;
                } else {
                    let (idx, _) = find_position(x, y, width, height, rotation2);
                    assert!(idx < max_value);
                }
            }
        }
    }

    #[test]
    fn graphics_rotation_0() {
        use crate::epd2in9::DEFAULT_BACKGROUND_COLOR;
        let width = 128;
        let height = 296;

        let mut buffer = [DEFAULT_BACKGROUND_COLOR.get_byte_value(); 128 / 8 * 296];
        let mut display = VarDisplay::new(width, height, &mut buffer);

        let _ = Line::new(Point::new(0, 0), Point::new(7, 0))
            .into_styled(PrimitiveStyle::with_stroke(Black, 1))
            .draw(&mut display);

        let buffer = display.buffer();

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }

    #[test]
    fn graphics_rotation_90() {
        use crate::epd2in9::DEFAULT_BACKGROUND_COLOR;
        let width = 128;
        let height = 296;

        let mut buffer = [DEFAULT_BACKGROUND_COLOR.get_byte_value(); 128 / 8 * 296];
        let mut display = VarDisplay::new(width, height, &mut buffer);

        display.set_rotation(DisplayRotation::Rotate90);

        let _ = Line::new(Point::new(0, 120), Point::new(0, 295))
            .into_styled(PrimitiveStyle::with_stroke(Black, 1))
            .draw(&mut display);

        let buffer = display.buffer();

        extern crate std;
        std::println!("{:?}", buffer);

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, DEFAULT_BACKGROUND_COLOR.get_byte_value());
        }
    }
}
