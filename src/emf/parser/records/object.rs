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

/// [#6577] `EMR_EXTCREATEPEN`: ihPen(4) + offBmi(4) + cbBmi(4) + offBits(4) + cbBits(4)
/// + LogPenEx{ PenStyle(4) Width(4) BrushStyle(4) ColorRef(4) BrushHatch(4)
/// NumStyleEntries(4) StyleEntry[] }.
///
/// 이 파일군(156627451 내장 EMF)의 비-스톡 펜은 **전부** 이 레코드에서 온다
/// (`EMR_CREATEPEN` 0건 · `EMR_EXTCREATEPEN` 16건). 종전에는 `Unknown` 으로 버려져
/// 획 색·굵기가 직전 펜에 그대로 묶였다.
///
/// `LogPenEx` 의 색은 `BrushStyle == BS_SOLID(0)` 일 때 `ColorRef` 다. 그 밖의
/// 브러시 스타일(해치·패턴)은 이 단계에서 단색으로 근사한다.
pub fn parse_ext_create_pen(c: &mut Cursor<'_>) -> Result<(u32, LogPen), Error> {
    let handle = c.u32()?;
    // offBmi · cbBmi · offBits · cbBits — 이 단계에서는 브러시 비트맵을 쓰지 않는다.
    for _ in 0..4 {
        let _ = c.u32()?;
    }
    let style = c.u32()?;
    let width = c.u32()? as i32;
    let _brush_style = c.u32()?;
    let color = c.u32()?;
    Ok((
        handle,
        LogPen {
            style,
            width,
            _reserved: 0,
            color,
        },
    ))
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
