//! EMF 기본 좌표 구조체 — MS-EMF 2.2.13/2.2.14/2.2.20.

use crate::emf::parser::Cursor;
use crate::emf::Error;

/// 32bit 정수 좌표 사각형 (left, top, right, bottom).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RectL {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl RectL {
    pub fn read(c: &mut Cursor<'_>) -> Result<Self, Error> {
        Ok(Self {
            left: c.i32()?,
            top: c.i32()?,
            right: c.i32()?,
            bottom: c.i32()?,
        })
    }
    #[must_use]
    pub const fn width(&self) -> i32 {
        self.right - self.left
    }
    #[must_use]
    pub const fn height(&self) -> i32 {
        self.bottom - self.top
    }
}

/// 32bit 정수 좌표 점.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PointL {
    pub x: i32,
    pub y: i32,
}

impl PointL {
    pub fn read(c: &mut Cursor<'_>) -> Result<Self, Error> {
        Ok(Self {
            x: c.i32()?,
            y: c.i32()?,
        })
    }
}

/// 32bit 정수 크기 (cx, cy).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SizeL {
    pub cx: i32,
    pub cy: i32,
}

impl SizeL {
    pub fn read(c: &mut Cursor<'_>) -> Result<Self, Error> {
        Ok(Self {
            cx: c.i32()?,
            cy: c.i32()?,
        })
    }
}
