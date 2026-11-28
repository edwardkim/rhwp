//! 관계형·고정형 — #7807 개요 번호의 실제 전진 뒤 제목이 이어지고 원본 쪽수를 보존한다.
//! 주석 갱신 2026-11-28T10:42:02+09:00. 직전 파일 커밋 be48607ad9444d8232df4f699c73832d9a34885d 2026-11-28T07:25:36+09:00.
//! Native Skia·fresh WASM Print를 독립 한컴 PDF에 96 DPI·PDFium·2px raw 실루엣으로 대조했다.
//! 측정 코드 be48607ad9444d8232df4f699c73832d9a34885d.
//! so-sueop-hwp3 · 기준 pdf/SO-SUEOP-hwp-2020.pdf · 전체 46쪽.
//! 전쪽 최저 Native 98.88% / fresh WASM 98.49%.
//! 17쪽 Native 99.87% / fresh WASM 98.59%.
//! 42쪽 Native 99.89% / fresh WASM 99.94%.
//! 전쪽 TSV와 대표 PNG는 mydocs/pr/assets/issues7789_7812/so-sueop-hwp3/에 있다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn outline_runs<'tree>(
    node: &'tree RenderNode,
    paragraph: usize,
    output: &mut Vec<&'tree RenderNode>,
) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.section_index == Some(0) && run.para_index == Some(paragraph) {
            output.push(node);
        }
    }
    for child in &node.children {
        outline_runs(child, paragraph, output);
    }
}

fn assert_outline_heading(core: &DocumentCore, page: u32, label: &str, title: &str) -> (f64, f64) {
    let paragraph = core.document().sections[0]
        .paragraphs
        .iter()
        .position(|paragraph| paragraph.text.starts_with(title))
        .expect("장 제목을 가진 원본 문단");
    let tree = core
        .build_page_render_tree(page)
        .expect("장 제목이 있는 쪽");
    let mut owned = Vec::new();
    outline_runs(&tree.root, paragraph, &mut owned);
    let [marker, body, ..] = owned.as_slice() else {
        panic!("번호와 본문을 같은 원본 문단에 순서대로 그려야 한다");
    };
    let (RenderNodeType::TextRun(marker_run), RenderNodeType::TextRun(body_run)) =
        (&marker.node_type, &body.node_type)
    else {
        panic!("장 번호와 제목의 렌더 런");
    };
    assert_eq!(marker_run.text, label);
    assert!(body_run.text.starts_with(title));
    assert!(
        marker_run.char_start.is_none(),
        "개요 번호는 원문 본문과 별개다"
    );
    assert_eq!(body_run.char_start, Some(0));
    assert_eq!(marker.bbox.y, body.bbox.y);
    assert!(
        body.bbox.x + 0.02 >= marker.bbox.x + marker.bbox.width,
        "장 번호의 실제 폭 뒤에서 제목이 시작해야 한다"
    );
    (marker.bbox.x, body.bbox.x)
}

#[test]
fn hwp3_outline_title_starts_after_the_current_ordinal_width() {
    let bytes = std::fs::read("samples/SO-SUEOP.hwp").expect("회귀 원본");
    let mut core = DocumentCore::from_bytes(&bytes).expect("HWP3 원본 읽기");
    core.set_hangul2024_compat(false);
    assert_eq!(core.page_count(), 46, "독립 한컴 Print와 같은 쪽수");

    let (first_marker_x, first_body_x) =
        assert_outline_heading(&core, 16, "9)", "사하촌          김정한");
    let (later_marker_x, later_body_x) =
        assert_outline_heading(&core, 41, "24)", "삼포가는길   황석영");
    let source_rounding = rhwp::renderer::hwpunit_to_px(1, rhwp::renderer::DEFAULT_DPI);
    assert!(
        (first_marker_x - later_marker_x).abs() <= source_rounding,
        "같은 들여쓰기의 번호 시작선"
    );
    assert!(
        later_body_x > first_body_x,
        "한컴 Print 17·42쪽처럼 두 자리 번호 뒤 제목이 한 자리보다 오른쪽에서 시작한다"
    );
}
