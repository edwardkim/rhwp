/// The META_STRETCHDIB Record specifies the transfer of color data from a block
/// of pixels in device-independent format according to a raster operation, with
/// possible expansion or contraction.
///
/// The source of the color data is a DIB, and the destination of the transfer
/// is the current output region in the playback device context.
#[derive(Clone, Debug)]
pub struct META_STRETCHDIB {
    /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the number
    /// of WORD structures, defined in [MS-DTYP] section 2.2.61, in the record.
    pub record_size: crate::wmf::parser::RecordSize,
    /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines this
    /// WMF record type. The lower byte MUST match the lower byte of the
    /// RecordType Enumeration table value META_STRETCHDIB.
    pub record_function: u16,
    /// RasterOperation (4 bytes): A 32-bit unsigned integer that defines how
    /// the source pixels, the current brush in the playback device context,
    /// and the destination pixels are to be combined to form the new image.
    /// This code MUST be one of the values in the Ternary Raster Operation
    /// Enumeration.
    pub raster_operation: crate::wmf::parser::TernaryRasterOperation,
    /// ColorUsage (2 bytes): A 16-bit unsigned integer that defines whether
    /// the Colors field of the DIB contains explicit RGB values or indexes
    /// into a palette. This value MUST be in the ColorUsage Enumeration.
    pub color_usage: crate::wmf::parser::ColorUsage,
    /// SrcHeight (2 bytes): A 16-bit signed integer that defines the height,
    /// in logical units, of the source rectangle.
    pub src_height: i16,
    /// SrcWidth (2 bytes): A 16-bit signed integer that defines the width, in
    /// logical units, of the source rectangle.
    pub src_width: i16,
    /// YSrc (2 bytes): A 16-bit signed integer that defines the y-coordinate,
    /// in logical units, of the source rectangle.
    pub y_src: i16,
    /// XSrc (2 bytes): A 16-bit signed integer that defines the x-coordinate,
    /// in logical units, of the source rectangle.
    pub x_src: i16,
    /// DestHeight (2 bytes): A 16-bit signed integer that defines the height,
    /// in logical units, of the destination rectangle.
    pub dest_height: i16,
    /// DestWidth (2 bytes): A 16-bit signed integer that defines the width, in
    /// logical units, of the destination rectangle.
    pub dest_width: i16,
    /// yDst (2 bytes): A 16-bit signed integer that defines the y-coordinate,
    /// in logical units, of the upper-left corner of the destination
    /// rectangle.
    pub y_dst: i16,
    /// xDst (2 bytes): A 16-bit signed integer that defines the x-coordinate,
    /// in logical units, of the upper-left corner of the destination
    /// rectangle.
    pub x_dst: i16,
    /// DIB (variable): A variable-sized DeviceIndependentBitmap Object that is
    /// the source of the color data.
    ///
    /// If the image format is JPEG or PNG, the ColorUsage field in this record
    /// MUST be set to DIB_RGB_COLORS, and the RasterOperation field MUST be
    /// set to SRCCOPY.
    pub dib: crate::wmf::parser::DeviceIndependentBitmap,
}

impl META_STRETCHDIB {
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
            crate::wmf::parser::RecordType::META_STRETCHDIB,
        )?;

        let (
            (raster_operation, raster_operation_bytes),
            (color_usage, color_usage_bytes),
            (src_height, src_height_bytes),
            (src_width, src_width_bytes),
            (y_src, y_src_bytes),
            (x_src, x_src_bytes),
            (dest_height, dest_height_bytes),
            (dest_width, dest_width_bytes),
            (y_dst, y_dst_bytes),
            (x_dst, x_dst_bytes),
        ) = (
            crate::wmf::parser::TernaryRasterOperation::parse(buf)?,
            crate::wmf::parser::ColorUsage::parse(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
        );
        record_size.consume(
            raster_operation_bytes
                + color_usage_bytes
                + src_height_bytes
                + src_width_bytes
                + y_src_bytes
                + x_src_bytes
                + dest_height_bytes
                + dest_width_bytes
                + y_dst_bytes
                + x_dst_bytes,
        );

        let (dib, c) =
            crate::wmf::parser::DeviceIndependentBitmap::parse_with_color_usage(buf, color_usage)?;
        record_size.consume(c);

        crate::wmf::parser::records::consume_remaining_bytes(buf, record_size)?;

        Ok(Self {
            record_size,
            record_function,
            raster_operation,
            color_usage,
            src_height,
            src_width,
            y_src,
            x_src,
            dest_height,
            dest_width,
            y_dst,
            x_dst,
            dib,
        })
    }
}
