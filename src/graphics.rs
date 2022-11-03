//! Graphics Support for EPDs

use crate::color::ColorType;
use embedded_graphics_core::prelude::*;
use core::marker::PhantomData;

/// Display rotation, only 90° increment supported
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

/// count the number of bytes per line knowing that it may contains padding bits
const fn line_bytes(width: u32, bits_per_pixel: usize) -> usize {
    // round to upper 8 bit count
    (width as usize * bits_per_pixel + 7) / 8
}

/// Display bffer used for drawing with embedded graphics
/// This can be rendered on EPD using ...
///
/// - WIDTH: width in pixel when display is not rotated
/// - HEIGHT: height in pixel when display is not rotated
/// - BWRBIT: mandatory value of the B/W when chromatic bit is set, can be any value for non
///           tricolor epd
/// - COLOR: color type used by the target display
/// - BYTECOUNT: This is redundant with prvious data and should be removed when const generic
///              expressions are stabilized
///
/// More on BWRBIT:
///
/// Different chromatic displays differently treat the bits in chromatic color planes.
/// Some of them ([crate::epd2in13bc]) will render a color pixel if bit is set for that pixel,
/// which is a [DisplayColorRendering::Positive] mode.
///
/// Other displays, like [crate::epd5in83b_v2] in opposite, will draw color pixel if bit is
/// cleared for that pixel, which is a [DisplayColorRendering::Negative] mode.
///
/// BWRBIT=true: chromatic doesn't override white, white bit cleared for black, white bit set for white, both bits set for chromatic
/// BWRBIT=false: chromatic does override white, both bits cleared for black, white bit set for white, red bit set for black
pub struct Display<const WIDTH: u32, const HEIGHT: u32, const BWRBIT: bool, const BYTECOUNT: usize, COLOR: ColorType> {
    buffer: [u8; BYTECOUNT],
    rotation: DisplayRotation,
    _color: PhantomData<COLOR>,
}

impl<const WIDTH: u32, const HEIGHT: u32, const BWRBIT: bool, const BYTECOUNT: usize, COLOR: ColorType> Default for Display<WIDTH,HEIGHT,BWRBIT,BYTECOUNT,COLOR> {
    /// Initialize display with the color '0', which may not be the same on all device.
    /// Many devices have a bit parameter polarity that should be changed if this is not the right
    /// one.
    /// However, every device driver should implement a DEFAULT_COLOR constant to indicate which
    /// color this represents.
    ///
    /// If you want a specific default color, you can still call clear() to set one.
    // inline is necessary here to allow heap allocation via Box on stack limited programs
    #[inline(always)]
    fn default() -> Self {
        Self {
            // default color must be 0 for every bit in a pixel to make this work everywere
            buffer: [0u8; BYTECOUNT],
            rotation: DisplayRotation::default(),
            _color: PhantomData,
        }
    }
}

/// For use with embedded_grahics
impl<const WIDTH: u32, const HEIGHT: u32, const BWRBIT: bool, const BYTECOUNT: usize, COLOR: ColorType> DrawTarget for Display<WIDTH,HEIGHT,BWRBIT,BYTECOUNT,COLOR> {
    type Color = COLOR;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            self.set_pixel(pixel);
        }
        Ok(())
    }
}

/// For use with embedded_grahics
impl<const WIDTH: u32, const HEIGHT: u32, const BWRBIT: bool, const BYTECOUNT: usize, COLOR: ColorType> OriginDimensions for Display<WIDTH,HEIGHT,BWRBIT,BYTECOUNT,COLOR> {
    fn size(&self) -> Size {
        Size::new(WIDTH, HEIGHT)
    }
}

impl<const WIDTH: u32, const HEIGHT: u32, const BWRBIT: bool, const BYTECOUNT: usize, COLOR: ColorType> Display<WIDTH,HEIGHT,BWRBIT,BYTECOUNT,COLOR> {
    /// get internal buffer to use it (to draw in epd)
    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    /// Set the display rotation.
    ///
    /// This only concerns future drawing made to it. Anything aready drawn
    /// stays as it is in the buffer.
    pub fn set_rotation(&mut self, rotation: DisplayRotation) {
        self.rotation = rotation;
    }

    /// Get current rotation
    pub fn rotation(&self) -> DisplayRotation {
        self.rotation
    }

    /// Get device coordinates from Display coordinate by rotatin appropriately
    fn rotate_coordinates(&self, x: i32, y: i32) -> (i32, i32) {
        match self.rotation {
            // as i32 = never use more than 2 billion pixel per line or per column
            DisplayRotation::Rotate0 => (x,y),
            DisplayRotation::Rotate90 => (WIDTH as i32 - 1 - y, x),
            DisplayRotation::Rotate180 => (WIDTH as i32 - 1 - x, HEIGHT as i32 - 1 - y),
            DisplayRotation::Rotate270 => (y, HEIGHT as i32 - 1 - x),
        }
    }

    /// Set a specific pixel color on this display
    pub fn set_pixel(&mut self, pixel:Pixel<COLOR>) {
        let Pixel(point, color) = pixel;
        // final coordinates
        let (x,y) = self.rotate_coordinates(point.x,point.y);
        // Out of range check
        if (x < 0) || (x >= WIDTH as i32) || (y < 0) || (y > HEIGHT as i32) {
            // don't do anything in case of out of range
            return;
        }

        let index = x as usize * COLOR::BITS_PER_PIXEL_PER_BUFFER/8 + y as usize * line_bytes(WIDTH,COLOR::BITS_PER_PIXEL_PER_BUFFER);
        let (mask, bits) = color.bitmask(BWRBIT, x as u32);

        if COLOR::BUFFER_COUNT == 2 {
            // split buffer is for tricolor displays that use 2 buffer for 2 bits per pixel
            self.buffer[index] = self.buffer[index] & mask | (bits & 1) as u8;
            let index = index + self.buffer.len()/2;
            self.buffer[index] = self.buffer[index] & mask | (bits >> 1) as u8;
        } else {
            self.buffer[index] = self.buffer[index] & mask | bits as u8;
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
