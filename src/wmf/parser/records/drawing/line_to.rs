/// The META_LINETO Record draws a line from the drawing position that is
/// defined in the playback device context up to, but not including, the
/// specified point.
#[derive(Clone, Debug)]
pub struct META_LINETO {
    /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the number
    /// of WORD structures, defined in [MS-DTYP] section 2.2.61, in the WMF
    /// record.
    pub record_size: crate::wmf::parser::RecordSize,
    /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines this
    /// WMF record type. The lower byte MUST match the lower byte of the
    /// RecordType Enumeration table value META_LINETO.
    pub record_function: u16,
    /// Y (2 bytes): A 16-bit signed integer that defines the vertical
    /// component of the drawing destination position, in logical units.
    pub y: i16,
    /// X (2 bytes): A 16-bit signed integer that defines the horizontal
    /// component of the drawing destination position, in logical units.
    pub x: i16,
}

impl META_LINETO {
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
            crate::wmf::parser::RecordType::META_LINETO,
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
