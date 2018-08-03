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
#![no_std]

//TODO: Make more assertions about buffersizes?


extern crate embedded_hal as hal;

use hal::{
    spi::{Mode, Phase, Polarity},
};

pub mod drawing;
pub mod epd4in2;



pub mod epd2in9;

pub mod interface;

pub mod type_a;



//TODO: test spi mode
/// SPI mode - 
/// For more infos see [Requirements: SPI](index.html#spi)
pub const SPI_MODE: Mode = Mode {
    phase: Phase::CaptureOnFirstTransition,
    polarity: Polarity::IdleLow,
};