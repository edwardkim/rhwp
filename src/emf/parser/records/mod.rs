//! EMF 레코드 enum + 개별 파서 모듈.

pub mod bitmap;
pub mod drawing;
pub mod header;
pub mod object;
pub mod path;
pub mod state;
pub mod text;

use super::objects::{Header, LogBrush, LogFontW, LogPen, PointL, RectL, SizeL, XForm};
pub use bitmap::StretchDIBits;
pub use text::ExtTextOut;

/// 파싱된 EMF 레코드.
#[derive(Debug, Clone)]
pub enum Record {
    // 제어 (단계 10)
    Header(Header),
    Eof,

    // 객체 (단계 11)
    CreatePen {
        handle: u32,
        pen: LogPen,
    },
    CreateBrushIndirect {
        handle: u32,
        brush: LogBrush,
    },
    ExtCreateFontIndirectW {
        handle: u32,
        font: LogFontW,
    },
    SelectObject {
        handle: u32,
    },
    DeleteObject {
        handle: u32,
    },

    // 상태 — DC 스택 (단계 11)
    SaveDC,
    RestoreDC {
        relative: i32,
    },
    SetWorldTransform(XForm),
    ModifyWorldTransform {
        xform: XForm,
        mode: u32,
    },

    // 상태 — 좌표계 (단계 11)
    SetMapMode(u32),
    SetWindowExtEx(SizeL),
    SetWindowOrgEx(PointL),
    SetViewportExtEx(SizeL),
    SetViewportOrgEx(PointL),

    // 상태 — 색상/모드 (단계 11)
    SetBkMode(u32),
    SetTextAlign(u32),
    SetTextColor(u32),
    SetBkColor(u32),

    // 드로잉 (단계 12)
    MoveToEx(PointL),
    LineTo(PointL),
    Rectangle(RectL),
    RoundRect {
        rect: RectL,
        corner_w: i32,
        corner_h: i32,
    },
    Ellipse(RectL),
    Arc {
        rect: RectL,
        start: PointL,
        end: PointL,
    },
    Chord {
        rect: RectL,
        start: PointL,
        end: PointL,
    },
    Pie {
        rect: RectL,
        start: PointL,
        end: PointL,
    },
    Polyline16 {
        bounds: RectL,
        points: Vec<(i16, i16)>,
    },
    Polygon16 {
        bounds: RectL,
        points: Vec<(i16, i16)>,
    },
    PolyBezier16 {
        bounds: RectL,
        points: Vec<(i16, i16)>,
    },
    /// [#6577] `EMR_POLYLINETO16` — 현재 위치에서 이어지는 선분열.
    /// 패스 기반 EMF(Office·Illustrator 내장본)의 주 구성 레코드다.
    PolylineTo16 {
        bounds: RectL,
        points: Vec<(i16, i16)>,
    },
    /// [#6577] `EMR_POLYBEZIERTO16` — 현재 위치에서 이어지는 3차 베지에열.
    PolyBezierTo16 {
        bounds: RectL,
        points: Vec<(i16, i16)>,
    },

    /// [#6577] `EMR_INTERSECTCLIPRECT` — 현재 클립을 이 사각형과 교차시킨다.
    IntersectClipRect(RectL),
    /// `EMR_EXTSELECTCLIPRGN` — `has_region` 이 false 이고 `mode` 가 `RGN_COPY`(5) 면
    /// 클립을 기본(무제한)으로 되돌리라는 뜻이다.
    ExtSelectClipRgn {
        mode: u32,
        has_region: bool,
    },

    // 패스 (단계 12)
    BeginPath,
    EndPath,
    CloseFigure,
    FillPath(RectL),
    StrokePath(RectL),
    StrokeAndFillPath(RectL),

    // 텍스트 (단계 13)
    ExtTextOutW(ExtTextOut),

    // 비트맵 (단계 13)
    StretchDIBits(StretchDIBits),

    /// 미분기 레코드. `payload`는 type/size 8B를 제외한 나머지.
    Unknown {
        record_type: u32,
        payload: Vec<u8>,
    },
}
