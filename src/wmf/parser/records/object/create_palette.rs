/// The META_CREATEPALETTE Record creates a Palette Object.
#[derive(Clone, Debug)]
pub struct META_CREATEPALETTE {
    /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the number
    /// of WORD structures, defined in [MS-DTYP] section 2.2.61, in the WMF
    /// record.
    pub record_size: crate::wmf::parser::RecordSize,
    /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines this
    /// WMF record type. The lower byte MUST match the lower byte of the
    /// RecordType Enumeration table value META_CREATEPALETTE.
    pub record_function: u16,
    /// Palette (variable): Palette Object data that defines the palette to
    /// create. The Start field in the Palette Object MUST be set to 0x0300.
    pub palette: crate::wmf::parser::Palette,
}

impl META_CREATEPALETTE {
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
        buf: &mut R,
        mut record_size: crate::wmf::parser::RecordSize,
        record_function: u16,
    ) -> Result<Self, crate::wmf::parser::ParseError> {
        crate::wmf::parser::records::check_lower_byte_matches(
            record_function,
            crate::wmf::parser::RecordType::META_CREATEPALETTE,
        )?;

        let (palette, palette_bytes) = crate::wmf::parser::Palette::parse(buf)?;
        record_size.consume(palette_bytes);

        if palette.start != 0x0300 {
            return Err(crate::wmf::parser::ParseError::UnexpectedPattern {
                cause: format!(
                    "The start field in the palette object must be `0x0300`, \
                     but `{:#06X}`",
                    palette.start
                ),
            });
        }

        crate::wmf::parser::records::consume_remaining_bytes(buf, record_size)?;

        Ok(Self {
            record_size,
            record_function,
            palette,
        })
    }
}
