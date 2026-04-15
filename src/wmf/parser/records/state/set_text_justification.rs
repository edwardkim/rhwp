/// The META_SETTEXTJUSTIFICATION Record defines the amount of space to add to
/// break characters in a string of justified text.
#[derive(Clone, Debug)]
pub struct META_SETTEXTJUSTIFICATION {
    /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the number
    /// of WORD structures, defined in [MS-DTYP] section 2.2.61, in the WMF
    /// record.
    pub record_size: crate::wmf::parser::RecordSize,
    /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines this
    /// WMF record type. The lower byte MUST match the lower byte of the
    /// RecordType Enumeration table value META_SETTEXTJUSTIFICATION.
    pub record_function: u16,
    /// BreakCount (2 bytes): A 16-bit unsigned integer that specifies the
    /// number of space characters in the line.
    pub break_count: u16,
    /// BreakExtra (2 bytes): A 16-bit unsigned integer that specifies the
    /// total extra space, in logical units, to be added to the line of text.
    /// If the current mapping mode is not MM_TEXT, the value identified by the
    /// BreakExtra member is transformed and rounded to the nearest pixel. For
    /// details about setting the mapping mode, see META_SETMAPMODE Record.
    pub break_extra: u16,
}

impl META_SETTEXTJUSTIFICATION {
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
            crate::wmf::parser::RecordType::META_SETTEXTJUSTIFICATION,
        )?;

        let ((break_count, break_count_bytes), (break_extra, break_extra_bytes)) = (
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
        );
        record_size.consume(break_count_bytes + break_extra_bytes);

        crate::wmf::parser::records::consume_remaining_bytes(buf, record_size)?;

        Ok(Self {
            record_size,
            record_function,
            break_count,
            break_extra,
        })
    }
}
