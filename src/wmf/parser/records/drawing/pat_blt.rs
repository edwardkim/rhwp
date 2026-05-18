/// The META_PATBLT Record paints a specified rectangle using the brush that is
/// defined in the playback device context. The brush color and the surface
/// color or colors are combined using the specified raster operation.
#[derive(Clone, Debug)]
pub struct META_PATBLT {
    /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the number
    /// of WORD structures, defined in [MS-DTYP] section 2.2.61, in the WMF
    /// record.
    pub record_size: crate::wmf::parser::RecordSize,
    /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines this
    /// WMF record type. The lower byte MUST match the lower byte of the
    /// RecordType Enumeration table value META_PATBLT.
    pub record_function: u16,
    /// RasterOperation (4 bytes): A 32-bit unsigned integer that defines the
    /// raster operation code. This code MUST be one of the values in the
    /// TernaryRasterOperation Enumeration table.
    pub raster_operation: crate::wmf::parser::TernaryRasterOperation,
    /// Height (2 bytes): A 16-bit signed integer that defines the height, in
    /// logical units, of the rectangle.
    pub height: i16,
    /// Width (2 bytes): A 16-bit signed integer that defines the width, in
    /// logical units, of the rectangle.
    pub width: i16,
    /// YLeft (2 bytes): A 16-bit signed integer that defines the y-coordinate,
    /// in logical units, of the upper-left corner of the rectangle to be
    /// filled.
    pub y_left: i16,
    /// XLeft (2 bytes): A 16-bit signed integer that defines the x-coordinate,
    /// in logical units, of the upper-left corner of the rectangle to be
    /// filled.
    pub x_left: i16,
}

impl META_PATBLT {
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
            crate::wmf::parser::RecordType::META_PATBLT,
        )?;

        let (
            (raster_operation, raster_operation_bytes),
            (height, height_bytes),
            (width, width_bytes),
            (y_left, y_left_bytes),
            (x_left, x_left_bytes),
        ) = (
            crate::wmf::parser::TernaryRasterOperation::parse(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
        );
        record_size.consume(
            raster_operation_bytes + height_bytes + width_bytes + y_left_bytes + x_left_bytes,
        );

        crate::wmf::parser::records::consume_remaining_bytes(buf, record_size)?;

        Ok(Self {
            record_size,
            record_function,
            raster_operation,
            height,
            width,
            y_left,
            x_left,
        })
    }
}
