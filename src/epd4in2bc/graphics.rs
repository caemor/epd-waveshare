use crate::color::TriColor;
use crate::epd4in2bc::{DEFAULT_BACKGROUND_COLOR, HEIGHT, NUM_DISPLAY_BITS, WIDTH};
use crate::graphics::{DisplayRotation, DisplayColorRendering};
use crate::graphics::TriDisplay;
use embedded_graphics_core::prelude::*;

/// Full size buffer for use with the 4in2 EPD
///
/// Can also be manuall constructed:
/// `buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value(); WIDTH / 8 * HEIGHT]`
pub struct Display4in2bc {
    buffer: [u8; 2 * NUM_DISPLAY_BITS as usize],
	rotation: DisplayRotation,
}

impl Default for Display4in2bc {
	fn default() -> Self {
		Display4in2bc {
            buffer: [DEFAULT_BACKGROUND_COLOR.get_byte_value(); 2 * NUM_DISPLAY_BITS as usize],
			rotation: DisplayRotation::default(),
		}
	}
}

impl DrawTarget for Display4in2bc {
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

impl OriginDimensions for Display4in2bc {
	fn size(&self) -> Size {
		Size::new(WIDTH, HEIGHT)
	}
}

impl TriDisplay for Display4in2bc {
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::color::Black;
	use crate::color::Color;
	use crate::epd4in2;
	use crate::graphics::{Display, DisplayRotation};
	use embedded_graphics::{
		prelude::*,
		primitives::{Line, PrimitiveStyle},
	};

	// test buffer length
	#[test]
	fn graphics_size() {
		let display = Display4in2bc::default();
		assert_eq!(display.buffer().len(), 15000);
	}

	// test default background color on all bytes
	#[test]
	fn graphics_default() {
		let display = Display4in2bc::default();
		for &byte in display.buffer() {
			assert_eq!(byte, epd4in2::DEFAULT_BACKGROUND_COLOR.get_byte_value());
		}
	}

	#[test]
	fn graphics_rotation_0() {
		let mut display = Display4in2bc::default();
		let _ = Line::new(Point::new(0, 0), Point::new(7, 0))
			.into_styled(PrimitiveStyle::with_stroke(Black, 1))
			.draw(&mut display);

		let buffer = display.buffer();

		assert_eq!(buffer[0], Color::Black.get_byte_value());

		for &byte in buffer.iter().skip(1) {
			assert_eq!(byte, epd4in2::DEFAULT_BACKGROUND_COLOR.get_byte_value());
		}
	}

	#[test]
	fn graphics_rotation_90() {
		let mut display = Display4in2bc::default();
		display.set_rotation(DisplayRotation::Rotate90);
		let _ = Line::new(Point::new(0, 392), Point::new(0, 399))
			.into_styled(PrimitiveStyle::with_stroke(Black, 1))
			.draw(&mut display);

		let buffer = display.buffer();

		assert_eq!(buffer[0], Color::Black.get_byte_value());

		for &byte in buffer.iter().skip(1) {
			assert_eq!(byte, epd4in2::DEFAULT_BACKGROUND_COLOR.get_byte_value());
		}
	}

	#[test]
	fn graphics_rotation_180() {
		let mut display = Display4in2bc::default();
		display.set_rotation(DisplayRotation::Rotate180);

		let _ = Line::new(Point::new(392, 299), Point::new(399, 299))
			.into_styled(PrimitiveStyle::with_stroke(Black, 1))
			.draw(&mut display);

		let buffer = display.buffer();

		extern crate std;
		std::println!("{:?}", buffer);

		assert_eq!(buffer[0], Color::Black.get_byte_value());

		for &byte in buffer.iter().skip(1) {
			assert_eq!(byte, epd4in2::DEFAULT_BACKGROUND_COLOR.get_byte_value());
		}
	}

	#[test]
	fn graphics_rotation_270() {
		let mut display = Display4in2bc::default();
		display.set_rotation(DisplayRotation::Rotate270);
		let _ = Line::new(Point::new(299, 0), Point::new(299, 7))
			.into_styled(PrimitiveStyle::with_stroke(Black, 1))
			.draw(&mut display);

		let buffer = display.buffer();

		extern crate std;
		std::println!("{:?}", buffer);

		assert_eq!(buffer[0], Color::Black.get_byte_value());

		for &byte in buffer.iter().skip(1) {
			assert_eq!(byte, epd4in2::DEFAULT_BACKGROUND_COLOR.get_byte_value());
		}
	}
}
