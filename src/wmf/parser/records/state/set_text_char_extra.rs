/// The META_SETTEXTCHAREXTRA Record defines inter-character spacing for text
/// justification in the playback device context. Spacing is added to the white
/// space between each character, including break characters, when a line of
/// justified text is output.
#[derive(Clone, Debug)]
pub struct META_SETTEXTCHAREXTRA {
    /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the number
    /// of WORD structures, defined in [MS-DTYP] section 2.2.61, in the WMF
    /// record.
    pub record_size: crate::wmf::parser::RecordSize,
    /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines this
    /// WMF record type. The lower byte MUST match the lower byte of the
    /// RecordType Enumeration table value META_SETTEXTCHAREXTRA.
    pub record_function: u16,
    /// CharExtra (2 bytes): A 16-bit unsigned integer that defines the amount
    /// of extra space, in logical units, to be added to each character. If the
    /// current mapping mode is not MM_TEXT, this value is transformed and
    /// rounded to the nearest pixel. For details about setting the mapping
    /// mode, see META_SETMAPMODE Record.
    pub char_extra: u16,
}

impl META_SETTEXTCHAREXTRA {
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
            crate::wmf::parser::RecordType::META_SETTEXTCHAREXTRA,
        )?;

        let (char_extra, char_extra_bytes) = crate::wmf::parser::read_u16_from_le_bytes(buf)?;
        record_size.consume(char_extra_bytes);

        crate::wmf::parser::records::consume_remaining_bytes(buf, record_size)?;

        Ok(Self {
            record_size,
            record_function,
            char_extra,
        })
    }
}
