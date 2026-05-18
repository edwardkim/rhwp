//! XForm — 2×3 affine 변환 (MS-EMF 2.2.28).

use crate::emf::parser::Cursor;
use crate::emf::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct XForm {
    pub m11: f32,
    pub m12: f32,
    pub m21: f32,
    pub m22: f32,
    pub dx: f32,
    pub dy: f32,
}

impl XForm {
    /// identity 변환.
    #[must_use]
    pub const fn identity() -> Self {
        Self {
            m11: 1.0,
            m12: 0.0,
            m21: 0.0,
            m22: 1.0,
            dx: 0.0,
            dy: 0.0,
        }
    }

    pub fn read(c: &mut Cursor<'_>) -> Result<Self, Error> {
        let m11 = f32::from_le_bytes(c.take(4)?.try_into().unwrap());
        let m12 = f32::from_le_bytes(c.take(4)?.try_into().unwrap());
        let m21 = f32::from_le_bytes(c.take(4)?.try_into().unwrap());
        let m22 = f32::from_le_bytes(c.take(4)?.try_into().unwrap());
        let dx = f32::from_le_bytes(c.take(4)?.try_into().unwrap());
        let dy = f32::from_le_bytes(c.take(4)?.try_into().unwrap());
        Ok(Self {
            m11,
            m12,
            m21,
            m22,
            dx,
            dy,
        })
    }
}

impl Default for XForm {
    fn default() -> Self {
        Self::identity()
    }
}
