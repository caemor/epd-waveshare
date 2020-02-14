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
//! ```rust,ignore
//! use epd_waveshare::{
//!     epd2in9::{EPD2in9, Display2in9},
//!     graphics::{Display, DisplayRotation},
//!     prelude::*,
//! };
//! use embedded_graphics::Drawing;
//!
//! // Setup EPD
//! let mut epd = EPD2in9::new(&mut spi, cs_pin, busy_in, dc, rst, &mut delay).unwrap();
//!
//! // Use display graphics
//! let mut display = Display2in9::default();
//!
//! // Write some hello world in the screenbuffer
//! display.draw(
//!     Font6x8::render_str("Hello World!")
//!         .stroke(Some(Color::Black))
//!         .fill(Some(Color::White))
//!         .translate(Coord::new(5, 50))
//!         .into_iter(),
//! );
//!
//! // Display updated frame
//! epd.update_frame(&mut spi, &display.buffer()).unwrap();
//! epd.display_frame(&mut spi).expect("display frame new graphics");
//!
//! // Set the EPD to sleep
//! epd.sleep(&mut spi).expect("sleep");
//! ```
//!
//!
#![no_std]

#[cfg(feature = "graphics")]
pub mod graphics;

mod traits;

pub mod color;

/// Interface for the physical connection between display and the controlling device
mod interface;

#[cfg(feature = "epd7in5")]
pub mod epd7in5;
#[cfg(feature = "epd7in5_v2")]
pub mod epd7in5_v2;

#[cfg(feature = "epd4in2")]
pub mod epd4in2;

#[cfg(feature = "epd1in54")]
pub mod epd1in54;

#[cfg(feature = "epd1in54b")]
pub mod epd1in54b;

#[cfg(feature = "epd2in9")]
pub mod epd2in9;

#[cfg(any(feature = "epd1in54", feature = "epd2in9"))]
pub(crate) mod type_a;

pub mod prelude {
    pub use crate::color::Color;
    pub use crate::traits::{RefreshLUT, WaveshareDisplay, WaveshareThreeColorDisplay};

    #[cfg(feature = "epd7in5_v2")]
    pub use crate::traits::WaveshareDisplayExt;

    pub use crate::SPI_MODE;

    #[cfg(feature = "graphics")]
    pub use crate::graphics::{Display, DisplayRotation};
}

use embedded_hal::spi::{Mode, Phase, Polarity};

/// SPI mode -
/// For more infos see [Requirements: SPI](index.html#spi)
pub const SPI_MODE: Mode = Mode {
    phase: Phase::CaptureOnFirstTransition,
    polarity: Polarity::IdleLow,
};
