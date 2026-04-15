/// The META_EXTFLOODFILL Record fills an area with the brush that is defined in
/// the playback device context.
#[derive(Clone, Debug)]
pub struct META_EXTFLOODFILL {
    /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the number
    /// of WORD structures, defined in [MS-DTYP] section 2.2.61, in the WMF
    /// record.
    pub record_size: crate::wmf::parser::RecordSize,
    /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines this
    /// WMF record type. The lower byte MUST match the lower byte of the
    /// RecordType Enumeration table value META_EXTFLOODFILL.
    pub record_function: u16,
    /// Mode (2 bytes): A 16-bit unsigned integer that defines the fill
    /// operation to be performed. This member MUST be one of the values in the
    /// FloodFill Enumeration table.
    pub mode: crate::wmf::parser::FloodFill,
    /// ColorRef (4 bytes): A 32-bit ColorRef Object that defines the color
    /// value.
    pub color_ref: i16,
    /// Y (2 bytes): A 16-bit signed integer that defines the y-coordinate, in
    /// logical units, of the point to be set.
    pub y: i16,
    /// X (2 bytes): A 16-bit signed integer that defines the x-coordinate, in
    /// logical units, of the point to be set.
    pub x: i16,
}

impl META_EXTFLOODFILL {
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
            crate::wmf::parser::RecordType::META_EXTFLOODFILL,
        )?;

        let ((mode, mode_bytes), (color_ref, color_ref_bytes), (y, y_bytes), (x, x_bytes)) = (
            crate::wmf::parser::FloodFill::parse(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
        );
        record_size.consume(mode_bytes + color_ref_bytes + y_bytes + x_bytes);

        crate::wmf::parser::records::consume_remaining_bytes(buf, record_size)?;

        Ok(Self {
            record_size,
            record_function,
            mode,
            color_ref,
            y,
            x,
        })
    }
}
