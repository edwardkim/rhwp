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
    let count = c.u32()? as usize;
    let mut pts = Vec::with_capacity(count);
    for _ in 0..count {
        let x = c.u16()? as i16;
        let y = c.u16()? as i16;
        pts.push((x, y));
    }
    Ok((bounds, pts))
}
