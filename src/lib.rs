//! A simple Driver for the Waveshare E-Ink Displays via SPI
//!
//! This driver was built using [`embedded-hal`] traits.
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal/~0.1
//!
//! # Requirements
//!
//! ### SPI
//!
//! - MISO is not connected/available
//! - SPI_MODE_0 is used (CPHL = 0, CPOL = 0)
//! - 8 bits per word, MSB first
//! - Max. Speed tested by myself was 8Mhz but more should be possible (Ben Krasnow used 18Mhz with his implemenation)
//!
//! ### Other....
//!
//! - Buffersize: Wherever a buffer is used it always needs to be of the size: `width / 8 * length`,
//!   where width and length being either the full e-ink size or the partial update window size
//!
//! # Examples
//!
//! ```ignore
//! use eink-waveshare-rs::epd4in2::EPD4in2;
//!
//! let mut epd4in2 = EPD4in2::new(spi, cs, busy, dc, rst, delay).unwrap();
//!
//! let mut buffer =  [0u8, epd4in2.get_width() / 8 * epd4in2.get_height()];
//!
//! // draw something into the buffer
//!
//! epd4in2.display_and_transfer_buffer(buffer, None);
//!
//! // wait and look at the image
//!
//! epd4in2.clear_frame(None);
//!
//! epd4in2.sleep();
//! ```
//!
//!
#![no_std]

//TODO: Make more assertions about buffersizes?

extern crate embedded_hal as hal;

use hal::spi::{Mode, Phase, Polarity};

#[cfg(feature = "graphics")]
pub mod drawing;

mod traits;
pub use traits::{WaveshareDisplay};

pub mod color;

/// Interface for the physical connection between display and the controlling device
mod interface;

#[cfg(feature = "epd4in2")]
mod epd4in2;
#[cfg(feature = "epd4in2")]
pub use epd4in2::EPD4in2;

#[cfg(feature = "epd1in54")]
mod epd1in54;
#[cfg(feature = "epd1in54")]
pub use epd1in54::EPD1in54;

#[cfg(feature = "epd2in9")]
mod epd2in9;
///2in9 eink
#[cfg(feature = "epd2in9")]
///2in9 eink
pub use epd2in9::EPD2in9;

#[cfg(any(feature = "epd1in54", feature = "epd2in9"))]
pub(crate) mod type_a;

use embedded-graphics;

//TODO: test spi mode
/// SPI mode -
/// For more infos see [Requirements: SPI](index.html#spi)
pub const SPI_MODE: Mode = Mode {
    phase: Phase::CaptureOnFirstTransition,
    polarity: Polarity::IdleLow,
};
