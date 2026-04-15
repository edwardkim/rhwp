/// The META_FRAMEREGION Record draws a border around a specified region using a
/// specified brush.
#[derive(Clone, Debug)]
pub struct META_FRAMEREGION {
    /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the number
    /// of WORD structures, defined in [MS-DTYP] section 2.2.61, in the WMF
    /// record.
    pub record_size: crate::wmf::parser::RecordSize,
    /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines this
    /// WMF record type. The lower byte MUST match the lower byte of the
    /// RecordType Enumeration table value META_FRAMEREGION.
    pub record_function: u16,
    /// Region (2 bytes): A 16-bit unsigned integer used to index into the WMF
    /// Object Table to get the region to be framed.
    pub region: u16,
    /// Brush (2 bytes): A 16-bit unsigned integer used to index into the WMF
    /// Object Table to get the Brush to use for filling the region.
    pub brush: u16,
    /// Height (2 bytes): A 16-bit signed integer that defines the height, in
    /// logical units, of the region frame.
    pub height: i16,
    /// Width (2 bytes): A 16-bit signed integer that defines the width, in
    /// logical units, of the region frame.
    pub width: i16,
}

impl META_FRAMEREGION {
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
            crate::wmf::parser::RecordType::META_FRAMEREGION,
        )?;

        let (
            (region, region_bytes),
            (brush, brush_bytes),
            (height, height_bytes),
            (width, width_bytes),
        ) = (
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
        );
        record_size.consume(region_bytes + brush_bytes + height_bytes + width_bytes);

        crate::wmf::parser::records::consume_remaining_bytes(buf, record_size)?;

        Ok(Self {
            record_size,
            record_function,
            region,
            brush,
            height,
            width,
        })
    }
}
