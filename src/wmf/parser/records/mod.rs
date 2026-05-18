//! Implementation of the definitions in Section 2.3 of the WMF specifications.

mod bitmap;
mod control;
mod drawing;
mod escape;
mod object;
mod state;

pub use self::{bitmap::*, control::*, drawing::*, escape::*, object::*, state::*};

/// Check lower byte MUST match the lower byte of the RecordType Enumeration
/// table value.
fn check_lower_byte_matches(
    record_function: u16,
    record_type: crate::wmf::parser::RecordType,
) -> Result<(), crate::wmf::parser::ParseError> {
    if record_function & 0x00FF != (record_type as u16) & 0x00FF {
        return Err(crate::wmf::parser::ParseError::UnexpectedPattern {
            cause: format!(
                "The low-order byte of record_function \
                 `{record_function:#06X}` field must be same as `{:#06X}`",
                record_type as u16
            ),
        });
    }

    Ok(())
}

fn consume_remaining_bytes<R: crate::wmf::Read>(
    buf: &mut R,
    record_size: crate::wmf::parser::RecordSize,
) -> Result<(crate::wmf::imports::Vec<u8>, usize), crate::wmf::parser::ParseError> {
    crate::wmf::parser::read_variable(buf, record_size.remaining_bytes())
        .map_err(|err| crate::wmf::parser::ParseError::FailedReadBuffer { cause: err })
}

/// A 32-bit unsigned integer that defines the number of 16-bit WORD structures,
/// defined in [MS-DTYP] section 2.2.61, in the record.
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct RecordSize(u32, usize);

impl RecordSize {
    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip_all,
        err(level = tracing::Level::ERROR, Display),
    ))]
    pub fn parse<R: crate::wmf::Read>(buf: &mut R) -> Result<Self, crate::wmf::parser::ParseError> {
        let (v, c) = crate::wmf::parser::read_u32_from_le_bytes(buf)?;

        Ok(Self(v, c))
    }
}

impl From<u32> for RecordSize {
    fn from(v: u32) -> Self {
        Self(v, 0)
    }
}

impl From<RecordSize> for u32 {
    fn from(v: RecordSize) -> Self {
        v.0
    }
}

impl core::fmt::Display for RecordSize {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#010X}", self.0)
    }
}

impl core::fmt::Debug for RecordSize {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "RecordSize(size: {:#010X}, consumed_bytes: {})",
            self.0, self.1
        )
    }
}

impl RecordSize {
    pub fn byte_count(&self) -> usize {
        (self.0 * 2) as usize
    }

    pub fn word_size(&self) -> usize {
        self.0 as usize
    }

    pub fn consume(&mut self, consumed_bytes: usize) {
        self.1 += consumed_bytes;
    }

    pub fn consumed_bytes(&self) -> usize {
        self.1
    }

    pub fn remaining(&self) -> bool {
        self.remaining_bytes() > 0
    }

    pub fn remaining_bytes(&self) -> usize {
        self.byte_count() - self.1
    }
}
