//! Issue #1052: 글상자 안 각주 본문이 페이지 하단 각주 영역에 누락되는 결함 회귀 가드.
//!
//! 재현 문서: `samples/hwpx/footnote-tbox-01.hwpx` + `samples/footnote-tbox-01.hwp`.
//!
//! 한컴 PDF 정답지 (`pdf-large/hwpx/footnote-tbox-01.pdf`):
//! ```
//! 글상자 내부에 각주가 있는 경우
//!  여기에 각주1)가 들어있는 경우
//! 와우
//! 사람2)들은
//!
//! 1) 글상자 내부 각주
//! 2) 일반 문단내 각주
//! ```
//!
//! 결함 본질: typeset.rs (main paginator, default) 가 Shape (글상자) 내부
//! paragraphs.controls 의 footNote 컨트롤을 traverse 하지 않아 페이지 하단
//! 각주 영역 (FootnoteRef) 에 추가되지 못함.
//! engine.rs (legacy, env opt-in) 는 이미 처리. `feedback_image_renderer_paths_separate`
//! 의 두 경로 동기화 누락 사례.

use std::fs;
use std::path::Path;

fn load_doc(rel: &str) -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse")
}

fn page_svg(doc: &rhwp::wasm_api::HwpDocument, page: u32) -> String {
    doc.render_page_svg_native(page).expect("render_page_svg")
}

/// SVG 의 모든 `<text>...</text>` 내용을 순서대로 이어붙인 문자열 반환.
/// 한 글자씩 분리된 text element 순서 sequence 가 sub-string 으로 등장하는지
/// 확인하기 위한 헬퍼.
fn svg_text_sequence(svg: &str) -> String {
    let mut out = String::new();
    let mut rest = svg;
    while let Some(open) = rest.find("<text") {
        let after_open = &rest[open..];
        if let Some(gt) = after_open.find('>') {
            let after_tag = &after_open[gt + 1..];
            if let Some(close) = after_tag.find("</text>") {
                out.push_str(&after_tag[..close]);
                rest = &after_tag[close + "</text>".len()..];
                continue;
            }
        }
        break;
    }
    out
}

/// HWPX: 글상자 안 각주 본문 "글상자 내부 각주" 가 페이지 하단 각주 영역에 표시.
#[test]
fn issue_1052_textbox_footnote_appears_in_footer_area_hwpx() {
    let doc = load_doc("samples/hwpx/footnote-tbox-01.hwpx");
    let svg = page_svg(&doc, 0);
    let seq = svg_text_sequence(&svg);
    assert!(
        seq.contains("글상자내부각주"),
        "글상자 안 각주 본문 '글상자 내부 각주' 가 페이지 하단 각주 영역에 표시되어야 함 \
         (Task #1052 본질). text sequence={:?}",
        seq
    );
}

/// HWP: 동일 결함 + 정합 확인 (variant 회귀 부재).
#[test]
fn issue_1052_textbox_footnote_appears_in_footer_area_hwp() {
    let doc = load_doc("samples/footnote-tbox-01.hwp");
    let svg = page_svg(&doc, 0);
    let seq = svg_text_sequence(&svg);
    assert!(
        seq.contains("글상자내부각주"),
        "HWP variant 도 글상자 안 각주 본문 '글상자 내부 각주' 가 페이지 하단 표시. \
         text sequence={:?}",
        seq
    );
}

/// 본문 직속 각주 (기존 동작) 회귀 부재 — "일반 문단내 각주" 유지.
#[test]
fn issue_1052_body_footnote_no_regression_hwpx() {
    let doc = load_doc("samples/hwpx/footnote-tbox-01.hwpx");
    let svg = page_svg(&doc, 0);
    let seq = svg_text_sequence(&svg);
    assert!(
        seq.contains("일반문단내각주"),
        "본문 직속 각주 '일반 문단내 각주' 가 회귀 없이 표시되어야 함. text sequence={:?}",
        seq
    );
}

/// 글상자 안 각주 마크 "1)" 기존 위치 유지 (본 회귀 가드는 텍스트 부재 검사).
#[test]
fn issue_1052_textbox_footnote_marker_present() {
    let doc = load_doc("samples/hwpx/footnote-tbox-01.hwpx");
    let svg = page_svg(&doc, 0);
    // 글상자 안 각주 번호 마크 (suffix=")")
    let one_paren = svg.matches("1)").count();
    let two_paren = svg.matches("2)").count();
    // 각주 마크 (본문 위치) + 각주 본문 영역 prefix (1)/2)) 양쪽 모두 표시
    assert!(
        one_paren >= 1,
        "글상자 안 각주 마크 '1)' 가 표시되어야 함 (occurrences={})",
        one_paren
    );
    assert!(
        two_paren >= 1,
        "본문 각주 마크 '2)' 가 표시되어야 함 (occurrences={})",
        two_paren
    );
}
