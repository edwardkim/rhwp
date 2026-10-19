//! LogPen — MS-EMF 2.2.19 (16바이트).

use crate::emf::parser::Cursor;
use crate::emf::Error;

/// 원시 폭의 단위. CREATEPEN의 양수 폭과 EXTLOGPEN의 cosmetic 폭은 다르다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PenWidthKind {
    Logical,
    DevicePixel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogPen {
    pub style: u32, // PenStyle flags
    pub width: i32, // 원시 lopnWidth.x 또는 elpWidth
    pub width_kind: PenWidthKind,
    pub _reserved: i32, // lopnWidth.y (사용 안 함)
    pub color: u32,     // COLORREF: 0x00BBGGRR
}

impl LogPen {
    pub fn read(c: &mut Cursor<'_>) -> Result<Self, Error> {
        let style = c.u32()?;
        let width = c.i32()?;
        Ok(Self {
            style,
            width,
            width_kind: if width == 0 {
                PenWidthKind::DevicePixel
            } else {
                PenWidthKind::Logical
            },
            _reserved: c.i32()?,
            color: c.u32()?,
        })
    }
}
