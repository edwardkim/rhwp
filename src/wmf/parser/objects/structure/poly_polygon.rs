use crate::wmf::imports::*;

/// The PolyPolygon Object defines a series of closed polygons.
#[derive(Clone, Debug)]
pub struct PolyPolygon {
    /// NumberOfPolygons (2 bytes): A 16-bit unsigned integer that defines the
    /// number of polygons in the object.
    pub number_of_polygons: u16,
    /// aPointsPerPolygon (variable): A NumberOfPolygons array of 16-bit
    /// unsigned integers that define the number of points for each polygon in
    /// the object.
    pub a_points_per_polygon: Vec<u16>,
    /// aPoints (variable): An array of PointS values that define the
    /// coordinates of the polygons. The length of the array is equal to the
    /// sum of all 16-bit integers in the aPointsPerPolygon array.
    pub a_points: Vec<crate::wmf::parser::PointS>,
}

impl PolyPolygon {
    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip_all,
        err(level = tracing::Level::ERROR, Display),
    ))]
    pub fn parse<R: crate::wmf::Read>(
        buf: &mut R,
    ) -> Result<(Self, usize), crate::wmf::parser::ParseError> {
        let (number_of_polygons, mut consumed_bytes) =
            crate::wmf::parser::read_u16_from_le_bytes(buf)?;
        let mut number_of_points = 0;
        let mut a_points_per_polygon = Vec::with_capacity(number_of_polygons as usize);
        let mut a_points = Vec::with_capacity(number_of_points as usize);

        for _ in 0..number_of_polygons {
            let (v, c) = crate::wmf::parser::read_u16_from_le_bytes(buf)?;

            consumed_bytes += c;
            number_of_points += v;
            a_points_per_polygon.push(v);
        }

        for _ in 0..number_of_points {
            let (v, c) = crate::wmf::parser::PointS::parse(buf)?;

            consumed_bytes += c;
            a_points.push(v);
        }

        Ok((
            Self {
                number_of_polygons,
                a_points_per_polygon,
                a_points,
            },
            consumed_bytes,
        ))
    }
}
