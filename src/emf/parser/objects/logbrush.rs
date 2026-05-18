//! LogBrush — MS-EMF 2.2.12 (LogBrush32, 12바이트).

use crate::emf::parser::Cursor;
use crate::emf::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogBrush {
    pub style: u32, // BrushStyle
    pub color: u32, // COLORREF
    pub hatch: u32, // HatchStyle (style=Hatched일 때만)
}

impl LogBrush {
    pub fn read(c: &mut Cursor<'_>) -> Result<Self, Error> {
        Ok(Self {
            style: c.u32()?,
            color: c.u32()?,
            hatch: c.u32()?,
        })
    }
}
