//! 객체 레코드 — 펜/브러시/폰트 생성·선택·삭제.

use crate::emf::parser::{
    objects::{LogBrush, LogFontW, LogPen},
    Cursor,
};
use crate::emf::Error;

/// EMR_CREATEPEN: ihPen(u32) + LogPen(16B) = 20B.
pub fn parse_create_pen(c: &mut Cursor<'_>) -> Result<(u32, LogPen), Error> {
    let handle = c.u32()?;
    let pen = LogPen::read(c)?;
    Ok((handle, pen))
}

/// EMR_CREATEBRUSHINDIRECT: ihBrush(u32) + LogBrush(12B) = 16B.
pub fn parse_create_brush_indirect(c: &mut Cursor<'_>) -> Result<(u32, LogBrush), Error> {
    let handle = c.u32()?;
    let brush = LogBrush::read(c)?;
    Ok((handle, brush))
}

/// EMR_EXTCREATEFONTINDIRECTW: ihFont(u32) + LogFontW(92B) + 선택적 DV 확장.
///
/// 확장부(LogFontExDv)는 단계 11에서 파싱하지 않고 스킵한다.
pub fn parse_ext_create_font_indirect_w(
    c: &mut Cursor<'_>,
    payload_len: usize,
) -> Result<(u32, LogFontW), Error> {
    let handle = c.u32()?;
    let font = LogFontW::read(c)?;
    // 남은 페이로드(확장): 4(handle) + 92(LogFontW) = 96 소비. 남으면 스킵.
    let consumed = 4 + 92;
    if payload_len > consumed {
        let _ = c.take(payload_len - consumed)?;
    }
    Ok((handle, font))
}

/// EMR_SELECTOBJECT: ihObject(u32).
pub fn parse_select_object(c: &mut Cursor<'_>) -> Result<u32, Error> {
    c.u32()
}

/// EMR_DELETEOBJECT: ihObject(u32).
pub fn parse_delete_object(c: &mut Cursor<'_>) -> Result<u32, Error> {
    c.u32()
}
