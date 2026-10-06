//! [#7407] 칸 문단의 줄 상자도 **문단 좌우 여백**만큼 들여 잡는다.
//!
//! # 무엇이 깨져 있었나
//!
//! 저장 줄이 없는 문단을 다시 조판할 때, 본문 문단은 `ParagraphBox::body_for_style` 로
//! 문단 여백만큼 들여 잡는데 **표 칸 문단은 `content_width_px` 로 칸 안쪽 폭 전체**를
//! 받았다. 같은 문단이 본문일 때와 칸일 때 다른 상자를 받은 것이다 —
//! `ParagraphBox::body` 의 주석이 "One paragraph must not get two different boxes
//! depending on which route reached it" 라고 적어 둔 바로 그 상태다.
//!
//! 그래서 줄이 문단 여백만큼 넓어지고 줄마다 글자가 더 들어갔다.
//!
//! # 기대값의 출처 — 한/글이 스스로 적은 값
//!
//! 두 입력은 **같은 문서**다.
//!
//! - `issue6639-hancom-160.hwpx` — 한/글이 저장한 본. 칸 31 의 열 문단에
//!   `hp:lineseg` 가 그대로 있다.
//! - `issue6639-reference-input-160.hwpx` — 위에서 그 열 문단의 `hp:linesegarray`
//!   **만** 지운 본. rhwp 가 직접 줄을 잡는다. (한/글로 출력하면 원본 PDF 와 96dpi
//!   래스터가 픽셀 단위로 같다 — `samples/issue6639/README.md`)
//!
//! 그러므로 **한/글이 적어 둔 `horzpos`/`horzsize` 가 이 칸의 독립 기대값**이다.
//! 숫자를 박지 않고 형제 파일에서 읽어 비교한다.
//!
//! ```text
//!   한/글 저장    horzpos=800  horzsize=39208
//!   수정 전 rhwp  column_start=0    segment_width=40808   (+1600 = 문단 좌우 여백)
//!   수정 후 rhwp  column_start=800  segment_width=39206
//! ```
//!
//! # 이 검사가 말하지 않는 것
//!
//! 여기서 잠그는 것은 **줄 상자의 좌단과 폭**뿐이다. 그 상자 안에서 줄이 어느 글자에서
//! 갈리는지는 `issue_7407_long_token_break_uses_measured_char_width.rs` 가 한/글이 적어
//! 둔 줄별 `textpos` 로 잠근다. 두 검사는 함께 읽는다 — 상자가 맞아야 그 기대값이
//! 성립하고, 상자만 맞아서는 줄이 맞지 않는다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

/// 한/글 저장본과 줄 캐시만 지운 본 — 내용은 같다.
const HANCOM: &str = "samples/issue6639/issue6639-hancom-160.hwpx";
const NO_CACHE: &str = "samples/issue6639/issue6639-reference-input-160.hwpx";
/// 대상 칸 — 열 문단이 `paraPr 18`(좌우 여백 있음)을 함께 쓴다.
const CELL: usize = 31;

/// 칸의 첫 줄 상자 `(column_start, segment_width)`.
fn first_line_box(rel: &str) -> (i32, i32) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{rel} 읽기: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let Some(Control::Table(table)) = core.document().sections[0].paragraphs[0].controls.get(2)
    else {
        panic!("표를 찾지 못했다 — 시험 설정 오류");
    };
    let seg = table
        .cells
        .get(CELL)
        .and_then(|c| c.paragraphs.first())
        .and_then(|p| p.line_segs.first())
        .expect("칸 첫 문단의 첫 줄");
    (seg.column_start, seg.segment_width)
}

/// rhwp 가 직접 잡은 줄 상자가 한/글이 같은 칸에 적어 둔 상자와 같다.
#[test]
fn a_recomposed_cell_line_box_matches_the_one_hancom_stored() {
    let (hancom_start, hancom_width) = first_line_box(HANCOM);
    let (rhwp_start, rhwp_width) = first_line_box(NO_CACHE);

    assert!(
        hancom_start > 0,
        "정답지 전제가 깨졌다 — 한/글 저장본의 줄 원점이 0 이면 이 칸은 여백이 없는 \
         문단이라 이 검사가 아무것도 잠그지 않는다. horzpos={hancom_start}"
    );
    assert_eq!(
        rhwp_start, hancom_start,
        "줄 원점이 한/글과 다르다 — 칸 문단 상자가 문단 좌여백만큼 들어가지 않았다."
    );
    // px 왕복에서 ±2 HWPUNIT 이 남는다. 문단 여백(1600)의 100분의 1 미만이다.
    let gap = (rhwp_width - hancom_width).abs();
    assert!(
        gap <= 4,
        "줄 폭이 한/글과 {gap} HWPUNIT 다르다 — 한/글 {hancom_width} / rhwp {rhwp_width}. \
         문단 좌우 여백이 가용 너비에서 빠지지 않으면 여기서 1600 이 남는다."
    );
}
