use alloc::boxed::Box;
use alloc::fmt;
use alloc::string::String;
use core::result;
use std::error::Error as StdError;
use std::io::Error as IoError;

use crate::ColorTransform;

pub type Result<T> = result::Result<T, Error>;

/// An enumeration over JPEG features (currently) unsupported by this library.
///
/// Support for features listed here may be included in future versions of this library.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UnsupportedFeature {
    /// Hierarchical JPEG.
    Hierarchical,
    /// JPEG using arithmetic entropy coding instead of Huffman coding.
    ArithmeticEntropyCoding,
    /// Sample precision in bits. 8 bit sample precision is what is currently supported in non-lossless coding process.
    SamplePrecision(u8),
    /// Number of components in an image. 1, 3 and 4 components are currently supported.
    ComponentCount(u8),
    /// An image can specify a zero height in the frame header and use the DNL (Define Number of
    /// Lines) marker at the end of the first scan to define the number of lines in the frame.
    DNL,
    /// Subsampling ratio.
    SubsamplingRatio,
    /// A subsampling ratio not representable as an integer.
    NonIntegerSubsamplingRatio,
    /// Colour transform
    ColorTransform(ColorTransform),
}

/// Errors that can occur while decoding a JPEG image.
#[derive(Debug)]
pub enum Error {
    /// A request reservation was denied, overflowed or failed.
    AllocationLimit,
    /// The image is not formatted properly. The string contains detailed information about the
    /// error.
    Format(String),
    /// The image makes use of a JPEG feature not (currently) supported by this library.
    Unsupported(UnsupportedFeature),
    /// An I/O error occurred while decoding the image.
    Io(IoError),
    /// An internal error occurred while decoding the image.
    Internal(Box<dyn StdError + Send + Sync + 'static>), //TODO: not used, can be removed with the next version bump
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::AllocationLimit => f.write_str("JPEG allocation limit"),
            Error::Format(ref desc) => write!(f, "invalid JPEG format: {}", desc),
            Error::Unsupported(ref feat) => write!(f, "unsupported JPEG feature: {:?}", feat),
            Error::Io(ref err) => err.fmt(f),
            Error::Internal(ref err) => err.fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            Error::Io(ref err) => Some(err),
            Error::Internal(ref err) => Some(&**err),
            _ => None,
        }
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Error::Io(err)
    }
}
