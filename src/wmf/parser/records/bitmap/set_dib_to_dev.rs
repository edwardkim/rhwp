/// The META_SETDIBTODEV Record sets a block of pixels in the playback device
/// context using device-independent color data.
///
/// The source of the color data is a DIB.
#[derive(Clone, Debug)]
pub struct META_SETDIBTODEV {
    /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the number
    /// of WORD structures, defined in [MS-DTYP] section 2.2.61, in the record.
    pub record_size: crate::wmf::parser::RecordSize,
    /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines this
    /// WMF record type. The lower byte MUST match the lower byte of the
    /// RecordType Enumeration table value META_SETDIBTODEV.
    pub record_function: u16,
    /// ColorUsage (2 bytes): A 16-bit unsigned integer that defines whether
    /// the Colors field of the DIB contains explicit RGB values or indexes
    /// into a palette. This MUST be one of the values in the ColorUsage
    /// Enumeration.
    pub color_usage: crate::wmf::parser::ColorUsage,
    /// ScanCount (2 bytes): A 16-bit unsigned integer that defines the number
    /// of scan lines in the source.
    pub scan_count: u16,
    /// StartScan (2 bytes): A 16-bit unsigned integer that defines the
    /// starting scan line in the source.
    pub start_scan: u16,
    /// yDib (2 bytes): A 16-bit unsigned integer that defines the
    /// y-coordinate, in logical units, of the source rectangle.
    pub y_dib: u16,
    /// xDib (2 bytes): A 16-bit unsigned integer that defines the
    /// x-coordinate, in logical units, of the source rectangle.
    pub x_dib: u16,
    /// Height (2 bytes): A 16-bit unsigned integer that defines the height, in
    /// logical units, of the source and destination rectangles.
    pub height: u16,
    /// Width (2 bytes): A 16-bit unsigned integer that defines the width, in
    /// logical units, of the source and destination rectangles.
    pub width: u16,
    /// yDest (2 bytes): A 16-bit unsigned integer that defines the
    /// y-coordinate, in logical units, of the upper-left corner of the
    /// destination rectangle.
    pub y_dest: u16,
    /// xDest (2 bytes): A 16-bit unsigned integer that defines the
    /// x-coordinate, in logical units, of the upper-left corner of the
    /// destination rectangle.
    pub x_dest: u16,
    /// DIB (variable): A variable-sized DeviceIndependentBitmap Object that is
    /// the source of the color data.
    pub dib: crate::wmf::parser::DeviceIndependentBitmap,
}

impl META_SETDIBTODEV {
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
            crate::wmf::parser::RecordType::META_SETDIBTODEV,
        )?;

        let (
            (color_usage, color_usage_bytes),
            (scan_count, scan_count_bytes),
            (start_scan, start_scan_bytes),
            (y_dib, y_dib_bytes),
            (x_dib, x_dib_bytes),
            (height, height_bytes),
            (width, width_bytes),
            (y_dest, y_dest_bytes),
            (x_dest, x_dest_bytes),
        ) = (
            crate::wmf::parser::ColorUsage::parse(buf)?,
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
        );
        record_size.consume(
            color_usage_bytes
                + scan_count_bytes
                + start_scan_bytes
                + y_dib_bytes
                + x_dib_bytes
                + height_bytes
                + width_bytes
                + y_dest_bytes
                + x_dest_bytes,
        );

        let (dib, c) =
            crate::wmf::parser::DeviceIndependentBitmap::parse_with_color_usage(buf, color_usage)?;
        record_size.consume(c);

        crate::wmf::parser::records::consume_remaining_bytes(buf, record_size)?;

        Ok(Self {
            record_size,
            record_function,
            color_usage,
            scan_count,
            start_scan,
            y_dib,
            x_dib,
            height,
            width,
            y_dest,
            x_dest,
            dib,
        })
    }
}
