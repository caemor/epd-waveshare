use core::fmt::{Debug, Display, Formatter};

use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal_async::{digital::Wait, spi::SpiDevice};

use crate::traits::Error;

/// Epd error type
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ErrorKind<SPI, BUSY, DC, RST>
where
    SPI: SpiDevice,
    SPI::Error: Copy,
    BUSY: InputPin,
    BUSY::Error: Copy,
    DC: OutputPin,
    DC::Error: Copy,
    RST: OutputPin,
    RST::Error: Copy,
{
    /// Encountered an SPI error
    SpiError(SPI::Error),

    /// Encountered an error on Busy GPIO
    BusyError(BUSY::Error),

    /// Encountered an error on DC GPIO
    DcError(DC::Error),

    /// Encountered an error on RST GPIO
    RstError(RST::Error),

    /// Anything else
    Other,
}

impl<SPI, BUSY, DC, RST> Clone for ErrorKind<SPI, BUSY, DC, RST>
where
    SPI: SpiDevice,
    SPI::Error: Copy,
    BUSY: InputPin,
    BUSY::Error: Copy,
    DC: OutputPin,
    DC::Error: Copy,
    RST: OutputPin,
    RST::Error: Copy,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<SPI, BUSY, DC, RST> Copy for ErrorKind<SPI, BUSY, DC, RST>
where
    SPI: SpiDevice,
    SPI::Error: Copy,
    BUSY: InputPin,
    BUSY::Error: Copy,
    DC: OutputPin,
    DC::Error: Copy,
    RST: OutputPin,
    RST::Error: Copy,
{
}

impl<SPI, BUSY, DC, RST> Display for ErrorKind<SPI, BUSY, DC, RST>
where
    SPI: SpiDevice,
    SPI::Error: Copy + Debug + Display,
    BUSY: InputPin + Wait,
    BUSY::Error: Copy + Debug + Display,
    DC: OutputPin,
    DC::Error: Copy + Debug + Display,
    RST: OutputPin,
    RST::Error: Copy + Debug + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::SpiError(err) => Display::fmt(&err, f),
            Self::BusyError(err) => Display::fmt(&err, f),
            Self::DcError(err) => Display::fmt(&err, f),
            Self::RstError(err) => Display::fmt(&err, f),
            Self::Other => write!(
                f,
                "A different error occurred. The original error may contain more information"
            ),
        }
    }
}

impl<SPI, BUSY, DC, RST> Debug for ErrorKind<SPI, BUSY, DC, RST>
where
    SPI: SpiDevice,
    SPI::Error: Copy + Debug + Display,
    BUSY: InputPin + Wait,
    BUSY::Error: Copy + Debug + Display,
    DC: OutputPin,
    DC::Error: Copy + Debug + Display,
    RST: OutputPin,
    RST::Error: Copy + Debug + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::SpiError(err) => Debug::fmt(&err, f),
            Self::BusyError(err) => Debug::fmt(&err, f),
            Self::DcError(err) => Debug::fmt(&err, f),
            Self::RstError(err) => Debug::fmt(&err, f),
            Self::Other => write!(
                f,
                "A different error occurred. The original error may contain more information"
            ),
        }
    }
}

impl<SPI, BUSY, DC, RST> Error<SPI, BUSY, DC, RST> for ErrorKind<SPI, BUSY, DC, RST>
where
    SPI: SpiDevice,
    SPI::Error: Copy + Debug + Display,
    BUSY: InputPin + Wait,
    BUSY::Error: Copy + Debug + Display,
    DC: OutputPin,
    DC::Error: Copy + Debug + Display,
    RST: OutputPin,
    RST::Error: Copy + Debug + Display,
{
    fn kind(&self) -> &ErrorKind<SPI, BUSY, DC, RST> {
        self
    }
}
