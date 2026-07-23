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
        self.0 as usize * 2
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
        self.byte_count().saturating_sub(self.1)
    }
}

#[cfg(test)]
mod tests {
    use super::RecordSize;

    /// 악의적 WMF는 RecordSize(WORD 개수)를 실제 고정 필드보다 작게 선언할 수 있다.
    /// 이때 `remaining_bytes()`의 `byte_count() - self.1`이 usize 언더플로를 일으켜
    /// 디버그 빌드에서 "attempt to subtract with overflow" 패닉,
    /// 릴리스 빌드에서 거대한 값이 `read_variable`의 `Vec::with_capacity`로 전달되어
    /// 할당 패닉/OOM을 유발한다. 수정 후에는 0으로 포화되어 패닉하지 않아야 한다.
    #[test]
    fn remaining_bytes_does_not_underflow_when_consumed_exceeds_declared_size() {
        // word_size = 2 → byte_count = 4 바이트.
        let mut record_size = RecordSize::from(2u32);
        // record_function(2) + 고정 필드(4) = 6 바이트 소비 (선언된 4 초과).
        record_size.consume(6);

        assert_eq!(
            record_size.remaining_bytes(),
            0,
            "consumed가 byte_count를 초과하면 남은 바이트는 0으로 포화되어야 한다"
        );
        assert!(!record_size.remaining());
    }

    /// RecordSize(u32) 값이 매우 크면 `byte_count()`의 `self.0 * 2`가
    /// u32 산술에서 오버플로되어 디버그 빌드에서 패닉한다.
    /// 수정 후에는 usize 산술로 계산되어 패닉하지 않아야 한다.
    #[test]
    fn byte_count_does_not_overflow_on_large_word_size() {
        let record_size = RecordSize::from(0x8000_0000u32);

        assert_eq!(record_size.byte_count(), 0x1_0000_0000usize);
    }
}
