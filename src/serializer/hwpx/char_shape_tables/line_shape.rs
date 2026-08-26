//! 선 종류 13종 — 밑줄·취소선 `shape` 속성 (한컴 표 27).

/// HWP5 선 종류 id → HWPX `shape` 토큰.
pub const LINE_SHAPE_HWPX: [&str; 13] = [
    "SOLID",
    "DASH",
    "DOT",
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

/// 선 종류 설명.
pub const LINE_SHAPE_LABELS_KO: [&str; 13] = [
    "실선",
    "긴 점선",
    "점선",
    "일점쇄선",
    "이점쇄선",
    "긴 대시",
    "원형 점선",
    "가는 이중선",
    "가늘고 굵은 이중선",
    "굵고 가는 이중선",
    "가늘고 굵고 가는 삼중선",
    "물결",
    "이중 물결",
];

/// 취소선이 꺼진 상태의 HWPX 토큰. 파서는 이 값을 no-strike 로 읽는다.
pub const STRIKE_SHAPE_NONE: &str = "NONE";

/// id → HWPX. 계약 밖 값은 실선.
pub fn line_shape_str(shape: u8) -> &'static str {
    LINE_SHAPE_HWPX
        .get(shape as usize)
        .copied()
        .unwrap_or("SOLID")
}

/// 취소선 방출. 꺼져 있으면 반드시 `NONE`.
pub fn strike_shape_str(strikethrough: bool, shape: u8) -> &'static str {
    if strikethrough {
        line_shape_str(shape)
    } else {
        STRIKE_SHAPE_NONE
    }
}

/// HWPX 토큰 → id. `NONE` 과 미지 값은 `None`.
pub fn line_shape_id(token: &str) -> Option<u8> {
    LINE_SHAPE_HWPX
        .iter()
        .position(|name| *name == token)
        .map(|i| i as u8)
}
