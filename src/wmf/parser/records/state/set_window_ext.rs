/// The META_SETWINDOWEXT Record defines the horizontal and vertical extents of
/// the output window in the playback device context.
#[derive(Clone, Debug)]
pub struct META_SETWINDOWEXT {
    /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the number
    /// of WORD structures, defined in [MS-DTYP] section 2.2.61, in the WMF
    /// record.
    pub record_size: crate::wmf::parser::RecordSize,
    /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines this
    /// WMF record type. The lower byte MUST match the lower byte of the
    /// RecordType Enumeration table value META_SETWINDOWEXT.
    pub record_function: u16,
    /// Y (2 bytes): A 16-bit signed integer that defines the vertical extent
    /// of the window in logical units.
    pub y: i16,
    /// X (2 bytes): A 16-bit signed integer that defines the horizontal extent
    /// of the window in logical units.
    pub x: i16,
}

impl META_SETWINDOWEXT {
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
            crate::wmf::parser::RecordType::META_SETWINDOWEXT,
        )?;

        let ((y, y_bytes), (x, x_bytes)) = (
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
        );
        record_size.consume(y_bytes + x_bytes);

        crate::wmf::parser::records::consume_remaining_bytes(buf, record_size)?;

        Ok(Self {
            record_size,
            record_function,
            y,
            x,
        })
    }
}
