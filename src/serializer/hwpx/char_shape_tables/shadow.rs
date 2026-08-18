//! 그림자 3종 — HWP5 attr bits 11-12 (2비트) / HWPX `hh:shadow@type`.

pub const SHADOW_TYPE_HWPX: [&str; 3] = ["NONE", "DROP", "CONTINUOUS"];

/// 계약 밖(예약값 3)은 NONE. 종전 CONTINUOUS 폴백은 그림자 없음을 있음으로 둔갑했다 (#3038).
pub fn shadow_type_str(kind: u8) -> &'static str {
    SHADOW_TYPE_HWPX
        .get(kind as usize)
        .copied()
        .unwrap_or("NONE")
}

pub fn shadow_type_id(token: &str) -> u8 {
    SHADOW_TYPE_HWPX
        .iter()
        .position(|name| *name == token)
        .unwrap_or(0) as u8
}
