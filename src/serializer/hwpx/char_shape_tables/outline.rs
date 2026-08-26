//! 외곽선 8종 — HWP5 attr bits 8-10 (3비트) / HWPX `hh:outline@type`.

pub const OUTLINE_TYPE_HWPX: [&str; 8] = [
    "NONE",
    "SOLID",
    "DASH",
    "DOT",
    "DASH_DOT",
    "DASH_DOT_DOT",
    "LONG_DASH",
    "CIRCLE",
];

pub fn outline_type_str(kind: u8) -> &'static str {
    OUTLINE_TYPE_HWPX
        .get(kind as usize)
        .copied()
        .unwrap_or("NONE")
}

pub fn outline_type_id(token: &str) -> u8 {
    OUTLINE_TYPE_HWPX
        .iter()
        .position(|name| *name == token)
        .unwrap_or(0) as u8
}
