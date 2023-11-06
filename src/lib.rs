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
//!# use embedded_hal_mock::eh1::*;
//!# fn main() -> Result<(), embedded_hal::spi::ErrorKind> {
//!use embedded_graphics::{
//!    pixelcolor::BinaryColor::On as Black, prelude::*, primitives::{Line, PrimitiveStyle},
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
//!# let mut delay = delay::NoopDelay::new();
//!
//!// Setup EPD
//!let mut epd = Epd1in54::new(&mut spi, busy_in, dc, rst, None)?;
//!
//!// Use display graphics from embedded-graphics
//!let mut display = Display1in54::default();
//!
//!// Use embedded graphics for drawing a line
//!
//!let _ = Line::new(Point::new(0, 120), Point::new(0, 295))
//!    .into_styled(PrimitiveStyle::with_stroke(Color::Black, 1))
//!    .draw(&mut display);
//!
//!    // Display updated frame
//!epd.update_frame(&mut spi, &display.buffer(), &mut delay)?;
//!epd.display_frame(&mut spi, &mut delay)?;
//!
//!// Set the EPD to sleep
//!epd.sleep(&mut spi, &mut delay)?;
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
#![allow(stable_features, incomplete_features, async_fn_in_trait)]
#![cfg_attr(feature = "async", feature(async_fn_in_trait, impl_trait_projections))]
#![cfg_attr(feature = "async", allow(incomplete_features, async_fn_in_trait))]

#[cfg(feature = "graphics")]
pub mod graphics;

mod error;
mod traits;

pub mod color;

/// Interface for the physical connection between display and the controlling device
mod interface;

pub mod epd1in54;
pub mod epd1in54_v2;
pub mod epd1in54b;
pub mod epd1in54c;
pub mod epd2in13_v2;
pub mod epd2in13bc;
pub mod epd2in66b;
pub mod epd2in7b;
pub mod epd2in9;
pub mod epd2in9_v2;
pub mod epd2in9bc;
pub mod epd2in9d;
pub mod epd3in7;
pub mod epd4in2;
pub mod epd5in65f;
pub mod epd5in83_v2;
pub mod epd5in83b_v2;
pub mod epd7in3f;
pub mod epd7in5;
pub mod epd7in5_hd;
pub mod epd7in5_v2;
pub mod epd7in5b_v2;
pub use epd7in5b_v2 as epd7in5b_v3;

pub(crate) mod type_a;

/// Includes everything important besides the chosen Display
pub mod prelude {
    pub use crate::color::{Color, OctColor, TriColor};
    pub use crate::traits::{
        QuickRefresh, RefreshLut, WaveshareDisplay, WaveshareThreeColorDisplay,
    };

    pub use crate::error::*;

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
/// \[XXXXX210\]\[76543210\]...\[76543210\] ^
/// \[XXXXX210\]\[76543210\]...\[76543210\] | height
/// \[XXXXX210\]\[76543210\]...\[76543210\] v
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
