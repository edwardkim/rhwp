//! 밑줄 위치 3종 — HWP5 attr bits 2-3 / HWPX `hh:underline@type`.

use crate::model::style::UnderlineType;

/// HWPX underline type 토큰.
pub const UNDERLINE_TYPE_HWPX: [&str; 3] = ["NONE", "BOTTOM", "TOP"];

pub fn underline_type_str(kind: UnderlineType) -> &'static str {
    match kind {
        UnderlineType::None => "NONE",
        UnderlineType::Bottom => "BOTTOM",
        UnderlineType::Top => "TOP",
    }
}

pub fn underline_type_from_hwpx(token: &str) -> UnderlineType {
    match token {
        "BOTTOM" => UnderlineType::Bottom,
        "TOP" => UnderlineType::Top,
        _ => UnderlineType::None,
    }
}

/// HWP5 bits 2-3. 스펙: 0=없음, 1=아래, 3=위. 2 는 예약.
pub fn underline_type_from_bits(bits: u32) -> UnderlineType {
    match bits & 0x3 {
        1 => UnderlineType::Bottom,
        3 => UnderlineType::Top,
        _ => UnderlineType::None,
    }
}

pub fn underline_type_to_bits(kind: UnderlineType) -> u32 {
    match kind {
        UnderlineType::None => 0,
        UnderlineType::Bottom => 1,
        UnderlineType::Top => 3,
    }
}
