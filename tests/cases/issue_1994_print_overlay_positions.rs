//! 고정형: #1994 독립 Print의 용지 기준 표 두 개의 원점을 96DPI ±2px로 보호한다.
//! 주석 갱신: 2026-11-06T22:13:05+09:00.
//! 2026-11-06 raw(96DPI, PDFium, 2px): 3쪽 Native 98.22544% / fresh WASM 98.19505%.
//! 전4쪽 최저 98.22544% / 98.14207%; source fa0436f495, 기준 PDF 글꼴 환경에서 측정했다.
//! 회귀 실행은 시스템 글꼴 설치에 의존하지 않으며 바깥여백 관계 검사는 별도 파일로 유지한다.
//! 근거: mydocs/pr/assets/issue1994/reference-fonts-20261106/README.md.

use rhwp::renderer::font_environment::FontEnvironment;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn find_table(node: &RenderNode, owner: usize) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(meta)
        if meta.para_index == Some(owner) && meta.cell_context.is_none())
    {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_table(child, owner))
}

#[test]
fn paper_overlay_origins_match_the_independent_print() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let bytes =
        std::fs::read(root.join("samples/basic/issue1994_behindtext_table_20200830.hwp")).unwrap();
    let mut core = DocumentCore::from_bytes(&bytes).unwrap();
    let declaration = std::fs::read_to_string(root.join(
        "mydocs/pr/assets/issue1994/reference-fonts-20261106/reference-font-environment.json",
    ))
    .unwrap();
    core.set_font_environment(Some(FontEnvironment::from_json(&declaration).unwrap()))
        .unwrap();
    let page = core.build_page_render_tree(2).unwrap();
    // samples/issue1994/issue_1994.pdf 3쪽의 괘선 원점(pt).
    for (owner, pdf_x, pdf_y) in [(33, 40.755, 46.655_029_3), (34, 40.875, 388.351_013_2)] {
        let table = find_table(&page.root, owner).expect("용지 기준 원본 표");
        let expected_x = pdf_x * (96.0 / 72.0);
        let expected_y = pdf_y * (96.0 / 72.0);
        assert!(
            (table.bbox.x - expected_x).abs() <= 2.0,
            "독립 Print의 x 원점에서 ±2px: actual={}, expected={expected_x}",
            table.bbox.x
        );
        assert!(
            (table.bbox.y - expected_y).abs() <= 2.0,
            "독립 Print의 y 원점에서 ±2px: actual={}, expected={expected_y}",
            table.bbox.y
        );
    }
}
