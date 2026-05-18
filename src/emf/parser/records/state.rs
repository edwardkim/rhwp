//! 상태 레코드 — DC 스택, WorldTransform, Window/Viewport, 색상/모드.

use crate::emf::parser::{
    objects::{PointL, SizeL, XForm},
    Cursor,
};
use crate::emf::Error;

pub fn parse_restore_dc(c: &mut Cursor<'_>) -> Result<i32, Error> {
    c.i32()
}

pub fn parse_set_world_transform(c: &mut Cursor<'_>) -> Result<XForm, Error> {
    XForm::read(c)
}

/// EMR_MODIFYWORLDTRANSFORM: XForm(24B) + iMode(u32).
pub fn parse_modify_world_transform(c: &mut Cursor<'_>) -> Result<(XForm, u32), Error> {
    let x = XForm::read(c)?;
    let mode = c.u32()?;
    Ok((x, mode))
}

pub fn parse_set_window_ext_ex(c: &mut Cursor<'_>) -> Result<SizeL, Error> {
    SizeL::read(c)
}
pub fn parse_set_window_org_ex(c: &mut Cursor<'_>) -> Result<PointL, Error> {
    PointL::read(c)
}
pub fn parse_set_viewport_ext_ex(c: &mut Cursor<'_>) -> Result<SizeL, Error> {
    SizeL::read(c)
}
pub fn parse_set_viewport_org_ex(c: &mut Cursor<'_>) -> Result<PointL, Error> {
    PointL::read(c)
}

pub fn parse_u32_single(c: &mut Cursor<'_>) -> Result<u32, Error> {
    c.u32()
}
