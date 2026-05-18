//! LogPen — MS-EMF 2.2.19 (16바이트).

use crate::emf::parser::Cursor;
use crate::emf::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogPen {
    pub style: u32,     // PenStyle flags
    pub width: i32,     // lopnWidth.x (pixel)
    pub _reserved: i32, // lopnWidth.y (사용 안 함)
    pub color: u32,     // COLORREF: 0x00BBGGRR
}

impl LogPen {
    pub fn read(c: &mut Cursor<'_>) -> Result<Self, Error> {
        Ok(Self {
            style: c.u32()?,
            width: c.i32()?,
            _reserved: c.i32()?,
            color: c.u32()?,
        })
    }
}
