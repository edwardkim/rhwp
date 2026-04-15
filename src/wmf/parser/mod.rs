mod constants;
mod objects;
mod records;

pub use self::{constants::*, objects::*, records::*};
use crate::wmf::imports::*;

#[derive(Clone, Debug, snafu::prelude::Snafu)]
pub enum ParseError {
    #[snafu(display("failed to read buffer: {cause}"))]
    FailedReadBuffer { cause: ReadError },
    #[snafu(display("not supported: {cause}"))]
    NotSupported { cause: String },
    #[snafu(display("unexpected enum value: {cause}"))]
    UnexpectedEnumValue { cause: String },
    #[snafu(display("unexpected bytes pattern: {cause}"))]
    UnexpectedPattern { cause: String },
}

impl From<ReadError> for ParseError {
    fn from(err: ReadError) -> Self {
        Self::FailedReadBuffer { cause: err }
    }
}

#[derive(Clone, Debug, snafu::prelude::Snafu)]
#[snafu(display("failed to read buffer: {cause}"))]
pub struct ReadError {
    cause: String,
}

impl ReadError {
    pub fn new(err: impl core::fmt::Display) -> Self {
        Self {
            cause: err.to_string(),
        }
    }
}

pub fn read<R: crate::wmf::Read, const N: usize>(
    buf: &mut R,
) -> Result<([u8; N], usize), ReadError> {
    let mut buffer = [0u8; N];

    match buf.read(&mut buffer) {
        Ok(bytes_read) if bytes_read == N => Ok((buffer, N)),
        Ok(bytes_read) => Err(ReadError::new(format!(
            "expected {N} bytes read, but {bytes_read} bytes read"
        ))),
        Err(err) => Err(ReadError::new(format!("{err:?}"))),
    }
}

pub fn read_variable<R: crate::wmf::Read>(
    buf: &mut R,
    len: usize,
) -> Result<(Vec<u8>, usize), ReadError> {
    if len == 0 {
        return Ok((vec![0u8; 0], 0));
    }

    let mut buffer = vec![0u8; len];

    match buf.read(&mut buffer) {
        Ok(bytes_read) if bytes_read == len => Ok((buffer, len)),
        Ok(bytes_read) => Err(ReadError::new(format!(
            "expected {len} bytes read, but {bytes_read} bytes read"
        ))),
        Err(err) => Err(ReadError::new(format!("{err:?}"))),
    }
}

macro_rules! impl_from_le_bytes {
    ($(($t:ty, $n:expr)),+) => {
        paste::paste!{
            $(
                pub fn [<read_ $t _from_le_bytes>]<R: $crate::wmf::Read>(
                    buf: &mut R,
                ) -> Result<($t, usize), ReadError> {
                    let (bytes, consumed_bytes) = read::<R, $n>(buf)?;

                    Ok((<$t>::from_le_bytes(bytes), consumed_bytes))
                }
            )*
        }
    };
}

impl_from_le_bytes! {(i8, 1), (i16, 2), (i32, 4), (u8, 1), (u16, 2), (u32, 4) }

/// Converts the given byte slice to a UTF-8 string using the specified
/// character set.
///
/// # Arguments
///
/// - `bytes` - The byte slice to convert.
/// - `charset` - The character set indicating the encoding of the byte slice.
///
/// # Returns
///
/// - On success, returns a UTF-8 string.
/// - On failure to decode, returns a `ParseError`.
///
/// If `SYMBOL_CHARSET` is specified, the function uses the symbol charset table
/// for conversion. Otherwise, it decodes using the provided encoding and
/// removes any null ( `\0` ) characters from the result.
fn bytes_into_utf8(
    bytes: &[u8],
    charset: crate::wmf::parser::CharacterSet,
) -> Result<String, crate::wmf::parser::ParseError> {
    if charset == crate::wmf::parser::CharacterSet::SYMBOL_CHARSET {
        Ok(bytes
            .iter()
            .filter_map(|v| crate::wmf::parser::symbol_charset_table().get(v).copied())
            .collect::<String>()
            .replace('\0', ""))
    } else {
        let encoding: &'static encoding_rs::Encoding = charset.into();
        let (cow, _, had_errors) = encoding.decode(bytes);

        if had_errors {
            return Err(crate::wmf::parser::ParseError::UnexpectedPattern {
                cause: "Failed to decode string with specified charset".to_string(),
            });
        }

        Ok(cow.replace('\0', ""))
    }
}
