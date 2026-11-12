//! 관계형·고정형 — [#9035] 셀 안 자리차지 개체가 첫 줄을 민 뒤 문단의 글자처럼 취급 표.
//!
//! 주석 갱신: 2026-11-13 KST(+09:00), 새 파일.
//!
//! - 입력: 공개 서식 `samples/issue9035/field_practice_confirmation_signature.hwp`
//!   ([별지 제23호의8서식] 현장실습확인서, 장사 등에 관한 법률 시행규칙, rhwp-issue-docs#1087).
//!   1쪽 아래 서명란 셀의 뒤 문단은 글자처럼 취급 표와 자리차지 개체를 함께 가지며, 개체가
//!   문단 시작을 차지해 첫 저장 줄을 아래로 민다.
//! - 기준: `pdf/issue9035/field_practice_confirmation_signature-print-2022.pdf`
//!   (Hwp 2022 12.0.0.4547 Print, Hancom PDF 1.3.0.547) 1쪽. 「현장실습기관의 장」 줄이 위,
//!   「장례지도사교육기관의 장」 줄이 아래에 따로 놓인다.
//! - 수정 전 rhwp(devel)는 뒤 문단의 표 원점을 밀리기 전 문단 시작에서 재어 두 서명 줄이 같은
//!   y 에 완전히 겹쳤다(layout-anomaly textOverlap 2건, 겹침 높이 21.3px).
//!
//! 검사: 두 서명 글 줄이 위아래 순서로 놓이고 세로로 겹치지 않으며, 전체 쪽수 = 한컴 Print 1쪽.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue9035/field_practice_confirmation_signature.hwp";
/// 한컴 2022 Print 의 전체 쪽수.
const PRINT_PAGES: u32 = 1;

/// (위끝, 아래끝, 글) 묶음을 모은다.
fn runs(node: &RenderNode, out: &mut Vec<(f64, f64, String)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push((
            node.bbox.y,
            node.bbox.y + node.bbox.height,
            run.text.clone(),
        ));
    }
    for child in &node.children {
        runs(child, out);
    }
}

fn ink(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// `needle` 을 포함하는 글 줄의 세로 범위(같은 줄 조각을 합친다).
fn line_span(all: &[(f64, f64, String)], needle: &str) -> (f64, f64) {
    let hit = all
        .iter()
        .find(|(_, _, text)| ink(text).contains(needle))
        .unwrap_or_else(|| panic!("글 줄 {needle}"));
    (hit.0, hit.1)
}

#[test]
fn float_pushed_tac_table_line_does_not_overlap_previous_signature_line() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("open {SAMPLE}: {e:?}"));
    assert_eq!(core.page_count(), PRINT_PAGES, "한컴 Print 의 전체 쪽수");

    let tree = core.build_page_render_tree(0).expect("1쪽");
    let mut all = Vec::new();
    runs(&tree.root, &mut all);

    let field = line_span(&all, "현장실습기관의");
    let school = line_span(&all, "장례지도사교육기관의");
    assert!(
        field.1 <= school.0,
        "「현장실습기관의 장」 줄 {field:?} 아래에 「장례지도사교육기관의 장」 줄 {school:?} 이 겹치지 않고 놓인다"
    );
}
