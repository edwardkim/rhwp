use crate::wmf::imports::*;

/// The META_EOF Record indicates the end of the WMF metafile.
#[derive(Clone, Debug)]
pub struct META_EOF {
    /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the number
    /// of 16-bit WORD structures, defined in [MS-DTYP] section 2.2.61, in the
    /// record.
    pub record_size: crate::wmf::parser::RecordSize,
    /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines the
    /// type of this record. For META_EOF, this value MUST be 0x0000, as
    /// specified in the RecordType Enumeration table.
    pub record_function: u16,
}

impl META_EOF {
    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip_all,
        fields(
            %record_size,
            record_function = %format!("{record_function:#06X}"),
        ),
        err(level = tracing::Level::ERROR, Display),
    ))]
    pub fn parse<R: crate::wmf::Read>(
        _buf: &mut R,
        record_size: crate::wmf::parser::RecordSize,
        record_function: u16,
    ) -> Result<Self, crate::wmf::parser::ParseError> {
        if record_size.word_size() != 3 {
            return Err(crate::wmf::parser::ParseError::UnexpectedPattern {
                cause: "The record_size must be `3`".to_owned(),
            });
        }

        if record_function != 0x0000 {
            return Err(crate::wmf::parser::ParseError::UnexpectedPattern {
                cause: "The record_function field must be `0x0000`".to_owned(),
            });
        }

        Ok(Self {
            record_size,
            record_function,
        })
    }
}
