/// The META_RESIZEPALETTE Record redefines the size of the logical palette that
/// is defined in the playback device context.
#[derive(Clone, Debug)]
pub struct META_RESIZEPALETTE {
    /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the number
    /// of WORD structures, defined in [MS-DTYP] section 2.2.61, in the WMF
    /// record.
    pub record_size: crate::wmf::parser::RecordSize,
    /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines this
    /// WMF record type. The lower byte MUST match the lower byte of the
    /// RecordType Enumeration table value META_RESIZEPALETTE.
    pub record_function: u16,
    /// NumberOfEntries (2 bytes): A 16-bit unsigned integer that defines the
    /// number of entries in the logical palette.
    pub number_of_entries: u16,
}

impl META_RESIZEPALETTE {
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
            crate::wmf::parser::RecordType::META_RESIZEPALETTE,
        )?;

        let (number_of_entries, number_of_entries_bytes) =
            crate::wmf::parser::read_u16_from_le_bytes(buf)?;
        record_size.consume(number_of_entries_bytes);

        crate::wmf::parser::records::consume_remaining_bytes(buf, record_size)?;

        Ok(Self {
            record_size,
            record_function,
            number_of_entries,
        })
    }
}
