//! 드로잉 레코드 — 선/사각형/타원/호/폴리라인16.

use crate::emf::parser::{
    objects::{PointL, RectL},
    Cursor,
};
use crate::emf::Error;

pub fn parse_point(c: &mut Cursor<'_>) -> Result<PointL, Error> {
    PointL::read(c)
}

pub fn parse_rect(c: &mut Cursor<'_>) -> Result<RectL, Error> {
    RectL::read(c)
}

/// EMR_ROUNDRECT: RectL + SizeL(corner width/height)
pub fn parse_round_rect(c: &mut Cursor<'_>) -> Result<(RectL, i32, i32), Error> {
    let rect = RectL::read(c)?;
    let cx = c.i32()?;
    let cy = c.i32()?;
    Ok((rect, cx, cy))
}

/// EMR_ARC/CHORD/PIE: RectL + PointL start + PointL end.
pub fn parse_arc_like(c: &mut Cursor<'_>) -> Result<(RectL, PointL, PointL), Error> {
    let rect = RectL::read(c)?;
    let start = PointL::read(c)?;
    let end = PointL::read(c)?;
    Ok((rect, start, end))
}

/// POINTS 배열 레코드 공통: RectL bounds + u32 count + POINTS[count] (4B/point).
pub fn parse_points16(c: &mut Cursor<'_>) -> Result<(RectL, Vec<(i16, i16)>), Error> {
    let bounds = RectL::read(c)?;
    // count는 EMF 레코드에서 온 검증되지 않은 u32다. 포인트 1개당 4바이트이므로
    // 남은 바이트로 상한을 씌워 Vec::with_capacity가 거대한 값을 그대로
    // 예약하지 않게 한다 (doc_info.rs tab_count류와 동일한 클래스).
    let count = (c.u32()? as usize).min(c.remaining() / 4);
    let mut pts = Vec::with_capacity(count);
    for _ in 0..count {
        let x = c.u16()? as i16;
        let y = c.u16()? as i16;
        pts.push((x, y));
    }
    Ok((bounds, pts))
}

/// MS-EMF 2.3.5.29: 전체 점 배열을 다각형별 점 수로 나눈다.
/// 선언 수를 검증한 뒤 할당해 손상된 수치가 메모리 예약으로 이어지지 않게 한다.
pub fn parse_poly_polygon16(c: &mut Cursor<'_>) -> Result<(RectL, Vec<Vec<(i16, i16)>>), Error> {
    let bounds = RectL::read(c)?;
    let polygons = c.u32()? as usize;
    let points = c.u32()? as usize;
    if polygons > c.remaining() / 4 {
        return Err(Error::UnexpectedEof {
            at: c.position(),
            need: polygons.saturating_mul(4),
        });
    }
    let mut counts = Vec::with_capacity(polygons);
    let mut total = 0usize;
    for _ in 0..polygons {
        let count = c.u32()? as usize;
        total = total.saturating_add(count);
        counts.push(count);
    }
    if total != points {
        return Err(Error::InvalidPolygonPointCount {
            declared: points,
            actual: total,
        });
    }
    if points > c.remaining() / 4 {
        return Err(Error::UnexpectedEof {
            at: c.position(),
            need: points.saturating_mul(4),
        });
    }
    let mut result = Vec::with_capacity(polygons);
    for count in counts {
        let mut polygon = Vec::with_capacity(count);
        for _ in 0..count {
            polygon.push((c.u16()? as i16, c.u16()? as i16));
        }
        result.push(polygon);
    }
    Ok((bounds, result))
}
