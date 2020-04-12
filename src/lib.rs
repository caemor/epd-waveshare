//! A simple Driver for the [Waveshare](https://github.com/waveshare/e-Paper) E-Ink Displays via SPI
//!
//! - Built using [`embedded-hal`] traits.
//! - Graphics support is added through [`embedded-graphics`]
//!
//! [`embedded-graphics`]: https://docs.rs/embedded-graphics/
//! [`embedded-hal`]: https://docs.rs/embedded-hal
//!

//!
//! # Example
//!
//!```rust, no_run
//!# use embedded_hal_mock::*;
//!# fn main() -> Result<(), MockError> {
//!use embedded_graphics::{
//!    pixelcolor::BinaryColor::On as Black, prelude::*, primitives::Line, style::PrimitiveStyle,
//!};
//!use epd_waveshare::{epd1in54::*, prelude::*};
//!#
//!# let expectations = [];
//!# let mut spi = spi::Mock::new(&expectations);
//!# let expectations = [];
//!# let cs_pin = pin::Mock::new(&expectations);
//!# let busy_in = pin::Mock::new(&expectations);
//!# let dc = pin::Mock::new(&expectations);
//!# let rst = pin::Mock::new(&expectations);
//!# let mut delay = delay::MockNoop::new();
//!
//!// Setup EPD
//!let mut epd = EPD1in54::new(&mut spi, cs_pin, busy_in, dc, rst, &mut delay)?;
//!
//!// Use display graphics from embedded-graphics
//!let mut display = Display1in54::default();
//!
//!// Use embedded graphics for drawing a line
//!let _ = Line::new(Point::new(0, 120), Point::new(0, 295))
//!    .into_styled(PrimitiveStyle::with_stroke(Black, 1))
//!    .draw(&mut display);
//!
//!    // Display updated frame
//!epd.update_frame(&mut spi, &display.buffer())?;
//!epd.display_frame(&mut spi)?;
//!
//!// Set the EPD to sleep
//!epd.sleep(&mut spi)?;
//!# Ok(())
//!# }
//!```
//!
//! # Other information and requirements
//!
//! - Buffersize: Wherever a buffer is used it always needs to be of the size: `width / 8 * length`,
//!   where width and length being either the full e-ink size or the partial update window size
//!
//! ### SPI
//!
//! MISO is not connected/available. SPI_MODE_0 is used (CPHL = 0, CPOL = 0) with 8 bits per word, MSB first.
//!
//! Maximum speed tested by myself was 8Mhz but more should be possible (Ben Krasnow used 18Mhz with his implemenation)
//!
#![no_std]
#![deny(missing_docs)]

#[cfg(feature = "graphics")]
pub mod graphics;

mod traits;

pub mod color;

/// Interface for the physical connection between display and the controlling device
mod interface;

pub mod epd1in54;
pub mod epd1in54b;
pub mod epd2in13_v2;
pub mod epd2in9;
pub mod epd2in9bc;
pub mod epd4in2;
pub mod epd7in5;
pub mod epd7in5_v2;
pub mod epd7in5bc;
pub(crate) mod type_a;

/// Includes everything important besides the chosen Display
pub mod prelude {
    pub use crate::color::{Color, TriColor};
    pub use crate::traits::{RefreshLUT, WaveshareDisplay, WaveshareThreeColorDisplay};

    pub use crate::SPI_MODE;

    #[cfg(feature = "graphics")]
    pub use crate::graphics::{Display, DisplayRotation};
}

/// Computes the needed buffer length. Takes care of rounding up in case width
/// is not divisible by 8.
///
///  unused
///  bits        width
/// <----><------------------------>
/// [XXXXX210][76543210]...[76543210] ^
/// [XXXXX210][76543210]...[76543210] | height
/// [XXXXX210][76543210]...[76543210] v
pub const fn buffer_len(width: usize, height: usize) -> usize {
    (width + 7) / 8 * height
}

use embedded_hal::spi::{Mode, Phase, Polarity};

/// SPI mode -
/// For more infos see [Requirements: SPI](index.html#spi)
pub const SPI_MODE: Mode = Mode {
    phase: Phase::CaptureOnFirstTransition,
    polarity: Polarity::IdleLow,
};
