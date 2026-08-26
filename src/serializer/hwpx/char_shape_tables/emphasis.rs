//! 강조점 7종 — HWP5 attr bits 21-24 / HWPX `hh:charPr@symMark`.

pub const EMPHASIS_HWPX: [&str; 7] = [
    "NONE",
    "DOT_ABOVE",
    "RING_ABOVE",
    "TILDE",
    "CARON",
    "SIDE",
    "COLON",
];

pub const EMPHASIS_LABELS_KO: [&str; 7] = [
    "없음",
    "위 검은 점",
    "위 고리",
    "물결",
    "역 곡절",
    "옆점",
    "쌍점",
];

pub fn sym_mark_str(emphasis: u8) -> &'static str {
    EMPHASIS_HWPX
        .get(emphasis as usize)
        .copied()
        .unwrap_or("NONE")
}

pub fn emphasis_id(token: &str) -> u8 {
    EMPHASIS_HWPX
        .iter()
        .position(|name| *name == token)
        .unwrap_or(0) as u8
}
