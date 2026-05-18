use crate::wmf::imports::*;

/// The META_POLYGON Record paints a polygon consisting of two or more vertices
/// connected by straight lines. The polygon is outlined by using the pen and
/// filled by using the brush and polygon fill mode that are defined in the
/// playback device context.
#[derive(Clone, Debug)]
pub struct META_POLYGON {
    /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the number
    /// of WORD structures, defined in [MS-DTYP] section 2.2.61, in the WMF
    /// record.
    pub record_size: crate::wmf::parser::RecordSize,
    /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines this
    /// WMF record type. The lower byte MUST match the lower byte of the
    /// RecordType Enumeration table value META_POLYGON.
    pub record_function: u16,
    /// NumberOfPoints (2 bytes): A 16-bit signed integer that defines the
    /// number of points in the array. This value must be greater than or equal
    /// to 2.
    pub number_of_points: i16,
    /// aPoints (variable): A NumberOfPoints array of 32-bit PointS Objects, in
    /// logical units.
    pub a_points: Vec<crate::wmf::parser::PointS>,
}

impl META_POLYGON {
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
            crate::wmf::parser::RecordType::META_POLYGON,
        )?;

        let (number_of_points, number_of_points_bytes) =
            crate::wmf::parser::read_i16_from_le_bytes(buf)?;
        record_size.consume(number_of_points_bytes);

        let mut a_points = Vec::with_capacity(number_of_points as usize);

        for _ in 0..number_of_points {
            let (v, c) = crate::wmf::parser::PointS::parse(buf)?;

            record_size.consume(c);
            a_points.push(v);
        }

        crate::wmf::parser::records::consume_remaining_bytes(buf, record_size)?;

        Ok(Self {
            record_size,
            record_function,
            number_of_points,
            a_points,
        })
    }
}
