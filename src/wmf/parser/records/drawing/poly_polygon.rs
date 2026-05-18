/// The META_POLYPOLYGON Record paints a series of closed polygons. Each polygon
/// is outlined by using the pen and filled by using the brush and polygon fill
/// mode; these are defined in the playback device context. The polygons drawn
/// by this function can overlap.
#[derive(Clone, Debug)]
pub struct META_POLYPOLYGON {
    /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the number
    /// of WORD structures, defined in [MS-DTYP] section 2.2.61, in the WMF
    /// record.
    pub record_size: crate::wmf::parser::RecordSize,
    /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines this
    /// WMF record type. The lower byte MUST match the lower byte of the
    /// RecordType Enumeration table value META_POLYPOLYGON.
    pub record_function: u16,
    /// PolyPolygon (variable): A variable-sized PolyPolygon Object that
    /// defines the point information.
    pub poly_polygon: crate::wmf::parser::PolyPolygon,
}

impl META_POLYPOLYGON {
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
            crate::wmf::parser::RecordType::META_POLYPOLYGON,
        )?;

        let (poly_polygon, poly_polygon_bytes) = crate::wmf::parser::PolyPolygon::parse(buf)?;
        record_size.consume(poly_polygon_bytes);

        crate::wmf::parser::records::consume_remaining_bytes(buf, record_size)?;

        Ok(Self {
            record_size,
            record_function,
            poly_polygon,
        })
    }
}
