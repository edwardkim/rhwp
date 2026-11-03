//! [#8532] 관계형 회귀 — 셀 단위 나눔(RowBreak) 표에서 여러 행에 걸친 셀이 있는 행 묶음이
//! 쪽 끝에 걸리고, 셀 저장 사다리가 그 자리에서 쪽을 되감으면(다음 줄 vpos 0) 그 되감김이
//! 한/글의 쪽 경계다. 앞 조각이 한 줄 높이여도 고아 방지 최소 높이로 묶음 전체를 다음 쪽으로
//! 넘기지 않는다.
//!
//! - 입력: `samples/issue8532/gangseo_exempt_certificates.hwp` — 공개 자치법규 별표
//!   「[별표 10] 전직시험이 면제되는 자격증 구분표(부산광역시 강서구의회 지방공무원 인사 규칙)」
//!   원본 그대로(72×5 RowBreak 표).
//! - 독립 기준: 같은 HWP 의 한/글 2022 Print `pdf/issue8532_gangseo_exempt_certificates-2020.pdf`
//!   — 6쪽. 아래 표의 낱말별 쪽 등장 횟수는 그 PDF 의 쪽별 텍스트에서 센 값이다.
//! - 수정 전 rhwp 는 3쪽 끝 「위생」 묶음을 4쪽으로 통째 넘겨 4~6쪽 행이 한 칸씩 밀렸다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn core(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|error| panic!("parse {sample}: {error:?}"))
}

fn collect(node: &RenderNode, text: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        text.extend(run.text.chars().filter(|c| !c.is_whitespace()));
    }
    for child in &node.children {
        collect(child, text);
    }
}

fn page_text(core: &DocumentCore, page: u32) -> String {
    let tree = core
        .build_page_render_tree(page)
        .unwrap_or_else(|error| panic!("{}쪽 렌더 트리: {error:?}", page + 1));
    let mut text = String::new();
    collect(&tree.root, &mut text);
    text
}

/// (낱말, 한/글 Print 1~6쪽 등장 횟수) — 문서 안에서 한 묶음에만 나오는 낱말.
const HANCOM_PAGE_COUNTS: &[(&str, [usize; 6])] = &[
    ("가축인공수정사", [0, 0, 1, 0, 0, 0]),
    ("항공공장정비사", [0, 0, 0, 2, 0, 0]),
    ("항공기관사", [0, 0, 0, 2, 0, 0]),
    ("조리기능장", [0, 0, 0, 0, 1, 0]),
    ("조리기능사", [0, 0, 0, 0, 1, 0]),
    ("제1종운전면허(대형)", [0, 0, 0, 0, 0, 1]),
    ("제2종운전면허(보통)", [0, 0, 0, 0, 0, 1]),
    ("기계정비,조경", [0, 0, 0, 0, 0, 1]),
];

#[test]
fn rowspan_block_splits_at_stored_cell_restart_even_for_one_line() {
    let core = core("samples/issue8532/gangseo_exempt_certificates.hwp");
    assert_eq!(core.page_count(), 6, "한/글 Print 쪽수");
    let pages: Vec<String> = (0..6).map(|page| page_text(&core, page)).collect();
    for (word, expected) in HANCOM_PAGE_COUNTS {
        let actual: Vec<usize> = pages
            .iter()
            .map(|text| text.matches(word).count())
            .collect();
        assert_eq!(
            actual.as_slice(),
            expected.as_slice(),
            "「{word}」 쪽별 등장 횟수(소속·누락·중복)는 한/글과 같다"
        );
    }
}
