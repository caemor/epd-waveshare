//! A simple Driver for the Waveshare E-Ink Displays via SPI
//! 
//! The other Waveshare E-Ink Displays should be added later on, atm it's only the 4.2"-Display
//! 
//! Build with the help of documentation/code from [Waveshare](https://www.waveshare.com/wiki/4.2inch_e-Paper_Module), 
//! [Ben Krasnows partial Refresh tips](https://benkrasnow.blogspot.de/2017/10/fast-partial-refresh-on-42-e-paper.html) and
//! the driver documents in the `pdfs`-folder as orientation.
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
//! - Max. Speed tested was 8Mhz but more should be possible
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
//! BE CAREFUL! The Partial Drawing can "destroy" your display.
//! It needs more testing first.
#![no_std]


extern crate embedded_hal as hal;

use hal::{
    spi::{Mode, Phase, Polarity},
};

pub mod drawing;
pub mod epd4in2;
use epd4in2::*;

pub mod epd2in9;

pub mod interface;



//TODO: test spi mode
/// SPI mode - 
/// For more infos see [Requirements: SPI](index.html#spi)
pub const SPI_MODE: Mode = Mode {
    phase: Phase::CaptureOnFirstTransition,
    polarity: Polarity::IdleLow,
};