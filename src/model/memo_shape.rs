//! 메모 모양(HWP DocInfo `MEMO_SHAPE`, HWPX `hh:memoProperties/hh:memoPr`).
//!
//! 한컴 12.30 Mac이 저장한 파일로 확인한 형식이다.
//! - 레코드 22바이트: 너비 u32(HWPUNIT), 선 종류 u8, 선 굵기 u8, 선·배경·선택 색(COLORREF), 메모 종류 u32.
//! - 선 종류는 메모 모양 창 목록의 차례(0 선 없음 … 11 삼중선)이고, HWPX `lineType`은 같은 차례의
//!   OWPML 이름이다(둘째 그림인 파선이 `DOT`, 셋째 그림인 점선이 `DASH`).
//! - 선 굵기는 창의 굵기 목록(0.1 … 5.0mm, 0.5mm 없음) 순번이다.
//! - 구역은 HWPX `secPr@memoShapeIDRef`, HWP 구역 정의의 언어 다음 u16으로 모양을 1부터 가리키고
//!   0이면 기본 모양을 쓴다. 메모 필드는 65535(구역의 모양)를 가리킨다.

use super::document::{DocInfo, RawRecord};
use crate::parser::tags::HWPTAG_MEMO_SHAPE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoShape {
    pub width: u32,
    pub line_type: u8,
    pub line_width: u8,
    /// COLORREF(0x00BBGGRR)
    pub line_color: u32,
    pub fill_color: u32,
    pub active_color: u32,
    pub memo_type: u32,
}

impl Default for MemoShape {
    /// 한컴이 메모 모양이 없는 문서에 쓰는 값(너비 55.0mm, 실선 0.12mm, 연두색).
    fn default() -> Self {
        Self {
            width: 15591,
            line_type: 1,
            line_width: 1,
            line_color: 0x00AE_D7B6,
            fill_color: 0x00E9_FFF0,
            active_color: 0x00C7_F1CF,
            memo_type: 0,
        }
    }
}

/// HWPX `lineType` 이름. 순번이 HWP 값이다.
pub const LINE_TYPES: [&str; 14] = [
    "NONE",
    "SOLID",
    "DOT",
    "DASH",
    "DASH_DOT",
    "DASH_DOT_DOT",
    "LONG_DASH",
    "CIRCLE",
    "DOUBLE_SLIM",
    "SLIM_THICK",
    "THICK_SLIM",
    "SLIM_THICK_SLIM",
    "WAVE",
    "DOUBLE_WAVE",
];

/// HWPX `lineType` 이름의 HWP 값. 모르는 이름은 0(선 없음)이다.
pub fn line_type_from_name(name: &str) -> u8 {
    LINE_TYPES
        .iter()
        .position(|known| *known == name)
        .unwrap_or(0) as u8
}

impl MemoShape {
    pub fn from_record(data: &[u8]) -> Option<Self> {
        if data.len() < 22 {
            return None;
        }
        let at = |i: usize| u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);
        Some(Self {
            width: at(0),
            line_type: data[4],
            line_width: data[5],
            line_color: at(6),
            fill_color: at(10),
            active_color: at(14),
            memo_type: at(18),
        })
    }

    pub fn to_record(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(22);
        data.extend_from_slice(&self.width.to_le_bytes());
        data.push(self.line_type);
        data.push(self.line_width);
        data.extend_from_slice(&self.line_color.to_le_bytes());
        data.extend_from_slice(&self.fill_color.to_le_bytes());
        data.extend_from_slice(&self.active_color.to_le_bytes());
        data.extend_from_slice(&self.memo_type.to_le_bytes());
        data
    }

    fn line_type_name(&self) -> &'static str {
        LINE_TYPES
            .get(usize::from(self.line_type))
            .copied()
            .unwrap_or("SOLID")
    }
}

/// COLORREF를 `#RRGGBB`로.
pub fn color_hex(color: u32) -> String {
    format!(
        "#{:02X}{:02X}{:02X}",
        color & 0xFF,
        (color >> 8) & 0xFF,
        (color >> 16) & 0xFF
    )
}

/// `#RRGGBB`를 COLORREF로. 형식이 맞지 않으면 None이다.
pub fn color_from_hex(hex: &str) -> Option<u32> {
    let digits = hex.strip_prefix('#')?;
    if digits.len() != 6 {
        return None;
    }
    let rgb = u32::from_str_radix(digits, 16).ok()?;
    Some(((rgb & 0xFF) << 16) | (rgb & 0xFF00) | ((rgb >> 16) & 0xFF))
}

/// 문서의 메모 모양 목록. 순번+1이 모양 번호다. 읽지 못하는 레코드는 기본 모양으로 자리를 지킨다.
pub fn memo_shapes(doc_info: &DocInfo) -> Vec<MemoShape> {
    doc_info
        .extra_records
        .iter()
        .filter(|record| record.tag_id == HWPTAG_MEMO_SHAPE)
        .map(|record| MemoShape::from_record(&record.data).unwrap_or_default())
        .collect()
}

/// 메모 모양 목록을 바꾼다. HWP 레코드는 원래 자리(없으면 한컴처럼 스타일 바로 뒤)에 두고,
/// HWPX는 원문 대신 이 목록으로 다시 쓴다.
pub fn set_memo_shapes(doc_info: &mut DocInfo, shapes: &[MemoShape]) {
    let at = doc_info
        .extra_records
        .iter()
        .position(|record| record.tag_id == HWPTAG_MEMO_SHAPE)
        .unwrap_or(0);
    doc_info
        .extra_records
        .retain(|record| record.tag_id != HWPTAG_MEMO_SHAPE);
    let at = at.min(doc_info.extra_records.len());
    doc_info.extra_records.splice(
        at..at,
        shapes.iter().map(|shape| RawRecord {
            tag_id: HWPTAG_MEMO_SHAPE,
            level: 1,
            data: shape.to_record(),
        }),
    );
    doc_info.memo_shape_count = shapes.len() as u32;
    doc_info.memo_properties_xml = None;
    doc_info.raw_stream_dirty = true;
}

/// HWPX `<hh:memoProperties>`. 한컴처럼 메모 종류는 `NOMAL`로 쓴다.
pub fn memo_properties_xml(shapes: &[MemoShape]) -> String {
    let mut xml = format!(r#"<hh:memoProperties itemCnt="{}">"#, shapes.len());
    for (index, shape) in shapes.iter().enumerate() {
        xml.push_str(&format!(
            r#"<hh:memoPr id="{}" width="{}" lineWidth="{}" lineType="{}" lineColor="{}" fillColor="{}" activeColor="{}" memoType="NOMAL"/>"#,
            index + 1,
            shape.width,
            shape.line_width,
            shape.line_type_name(),
            color_hex(shape.line_color),
            color_hex(shape.fill_color),
            color_hex(shape.active_color),
        ));
    }
    xml.push_str("</hh:memoProperties>");
    xml
}
