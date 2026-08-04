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
        // [정적분석] u16 를 그대로 누적하면 다각형 총 점 개수가 65535 를 넘을 때
        // (예: 다각형 2개, 각 65000점) debug 빌드는 오버플로 패닉, release 빌드는
        // wrap 되어 aPointsPerPolygon 이 요구하는 점 개수보다 적게 읽는 스트림 desync 로
        // 이어진다. u32 로 누적하고 초과 시 파싱을 명시적으로 거부한다.
        let mut number_of_points: u32 = 0;
        let mut a_points_per_polygon = Vec::with_capacity(number_of_polygons as usize);

        for _ in 0..number_of_polygons {
            let (v, c) = crate::wmf::parser::read_u16_from_le_bytes(buf)?;

            consumed_bytes += c;
            number_of_points = number_of_points.checked_add(v as u32).ok_or_else(|| {
                crate::wmf::parser::ParseError::NotSupported {
                    cause: "PolyPolygon: aPointsPerPolygon 총합이 u32 범위를 초과합니다"
                        .to_string(),
                }
            })?;
            a_points_per_polygon.push(v);
        }

        let mut a_points = Vec::with_capacity((number_of_points as usize).min(1 << 20));

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_point_count_above_u16_does_not_wrap() {
        // 두 polygon의 점 수 합계는 65,536이다. 이전 u16 누산은 debug에서는
        // overflow panic, release에서는 0으로 wrap되어 점을 읽지 않고 성공했다.
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&32_768u16.to_le_bytes());
        bytes.extend_from_slice(&32_768u16.to_le_bytes());
        bytes.resize(2 + 2 * 2 + 65_536 * 4, 0); // PointS { x: 0, y: 0 } × 65,536
        let mut input = bytes.as_slice();

        let (polygon, consumed) =
            PolyPolygon::parse(&mut input).expect("u16을 넘는 총점도 모든 PointS를 읽어야 함");
        assert!(
            polygon.a_points.len() == 65_536,
            "총점이 65,536인데 aPoints가 wrap되면 안 됨"
        );
        assert_eq!(consumed, bytes.len());
    }
}
