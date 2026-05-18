//! 패스 레코드 — BeginPath/EndPath/CloseFigure/FillPath/StrokePath.

use crate::emf::parser::{objects::RectL, Cursor};
use crate::emf::Error;

pub fn parse_path_bounds(c: &mut Cursor<'_>) -> Result<RectL, Error> {
    RectL::read(c)
}
