//! [#7407] 한 줄에 안 들어가는 토큰은 **잰 폭**으로 자른다 — 상수 12.0px 이 아니라.
//!
//! # 무엇이 깨져 있었나
//!
//! 토큰 하나가 한 줄보다 길면 줄 나눔은 글자 단위 폴백으로 내려가 글자별 폭을 누적한다.
//! 그 글자별 폭 `base_char_widths` 를 토큰 생성기가 **inline control 이 있는 토큰에만**
//! 채웠다. 보통의 긴 한글 토큰은 빈 벡터를 받았고, 폴백은
//!
//! ```text
//! let char_w_px = if is_cjk_char(ch) { cursor.line_max_fs.max(12.0) } else { ... };
//! ```
//!
//! 라는 **측정과 무관한 상수**로 떨어졌다. 같은 토큰을 재는 자가 둘이 된 것이다 —
//! 합(`base_width`)은 `estimate_text_width_unrounded` 로 재고, 조각은 글자 크기로 쟀다.
//!
//! 대상 칸은 9pt(12.000px) 장평 95% 라 실제 전진폭이 **11.400px** 인데 폴백은
//! **12.000px** 로 셌다. 연속 줄 38208 HWPUNIT 에서 38208/900 = 42 자와
//! 38208/855 = 44 자의 차이다. 43 번째 글자는 오른쪽에 19px 을 남기고 들어가는데도
//! 거부됐다(수정 전 실측: 줄마다 42 자, x 194.25→673.05, 상자 끝 703.69).
//!
//! # 기대값의 출처 — 한/글이 스스로 적은 줄 시작 위치
//!
//! 두 입력은 **같은 문서**다. `issue6639-hancom-160.hwpx` 는 한/글 저장본이라 칸 31 의
//! 열 문단에 `hp:lineseg` 가 그대로 있고, `issue6639-reference-input-160.hwpx` 는 그
//! `hp:linesegarray` **만** 지운 본이라 rhwp 가 직접 줄을 잡는다. 후자를 한/글로 출력하면
//! 원본 PDF 와 96dpi 래스터가 픽셀 단위로 같다(`samples/issue6639/README.md`).
//!
//! 그러므로 한/글이 적어 둔 줄별 `textpos` 가 이 칸의 독립 기대값이다. 줄 수만이 아니라
//! **어느 글자에서 줄이 갈렸는지**까지 잠근다.
//!
//! ```text
//!   한/글 저장   문단3 [0, 5, 49, 93, 137, 181]   (연속 줄마다 44 자)
//!   수정 전 rhwp 문단3 [0, 5, 47, 89, 131, 173, 215]  (42 자 · 줄 하나 더)
//! ```
//!
//! 정본 PDF(`pdf/issue6639/issue6639-reference-160-2020.pdf`)의 판독도 같은 값이다 —
//! 문단별 `[1, 4, 6, 5, 3, 1, 1, 1, 1, 2]`, 합 25 줄.
//!
//! # 이 검사가 말하지 않는 것
//!
//! 줄 상자의 좌단·폭은 `issue_7407_cell_line_box_uses_paragraph_margins.rs` 가 잠근다.
//! 그 상자가 맞아야 이 검사의 기대값이 성립하므로 두 검사는 함께 읽는다. 표 높이·가로선
//! 차이(#7407 의 B)는 여기서 다루지 않는다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

/// 한/글 저장본과 줄 캐시만 지운 본 — 내용은 같다.
const HANCOM: &str = "samples/issue6639/issue6639-hancom-160.hwpx";
const NO_CACHE: &str = "samples/issue6639/issue6639-reference-input-160.hwpx";
/// 대상 칸 — 열 문단이 `paraPr 18`(긴 한글 연속 + 내어쓰기)을 함께 쓴다.
const CELL: usize = 31;

/// 칸의 문단별 줄 시작 위치(UTF-16 offset).
fn line_starts(rel: &str) -> Vec<Vec<u32>> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{rel} 읽기: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let Some(Control::Table(table)) = core.document().sections[0].paragraphs[0].controls.get(2)
    else {
        panic!("표를 찾지 못했다 — 시험 설정 오류");
    };
    table
        .cells
        .get(CELL)
        .expect("대상 칸")
        .paragraphs
        .iter()
        .map(|p| p.line_segs.iter().map(|seg| seg.text_start).collect())
        .collect()
}

/// rhwp 가 직접 잡은 줄이 한/글이 같은 칸에 적어 둔 줄과 **글자 단위로** 같다.
#[test]
fn a_recomposed_line_starts_match_the_ones_hancom_stored() {
    let hancom = line_starts(HANCOM);
    let rhwp = line_starts(NO_CACHE);

    assert_eq!(
        hancom.len(),
        rhwp.len(),
        "문단 수가 다르면 두 입력이 같은 문서가 아니다 — 시험 설정 오류.          한/글={hancom:?} rhwp={rhwp:?}"
    );
    // 전제 확인: 정답지에 실제로 토큰 안에서 갈린 문단이 없으면 이 검사는 아무것도
    // 잠그지 않는다(키 오타·빈 칸으로 통과하는 것을 막는다).
    let broken = hancom.iter().filter(|starts| starts.len() >= 4).count();
    assert!(
        broken >= 3,
        "정답지 전제가 깨졌다 — 여러 줄로 갈린 문단이 {broken} 개뿐이다.          이 칸은 긴 토큰 폴백을 타지 않으므로 검사를 다시 설계해야 한다.          한/글={hancom:?}"
    );
    assert_eq!(
        rhwp, hancom,
        "줄이 한/글과 다른 글자에서 갈렸다 — 긴 토큰을 자를 때 잰 폭이 아니라 글자          크기 상수로 세면 연속 줄이 44 자가 아니라 42 자가 된다."
    );
}
