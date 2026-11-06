//! Issue #1994: 용지 기준 글뒤 표의 바깥여백 원점을 한 번 소비한다.
//!
//! 검사 유형: 관계형. 원본 오프셋·바깥여백과 최종 표 원점의 관계를 검사한다.
//! 절대 PDF 픽셀 핀이 아니다. 독립 기준은 samples/issue1994/issue_1994.pdf 3쪽이다.
//! Native 전 4쪽 중 3쪽은 88.23109% → 91.80504%이며 fresh WASM 새 후보는
//! 아직 검증 중이다. 브랜치의 전쪽 98% 완료 기준을 통과했다는 의미가 아니다.
//! 주석 갱신: 2026-11-06 21:06 KST.

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn tables<'a>(node: &'a RenderNode, found: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        found.push(node);
    }
    for child in &node.children {
        tables(child, found);
    }
}

#[test]
fn paper_overlay_consumes_its_source_outer_origin_once() {
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/basic/issue1994_behindtext_table_20200830.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    core.set_dpi(96.0);
    let page = core.build_page_render_tree(2).unwrap();
    let mut placed = Vec::new();
    tables(&page.root, &mut placed);
    for owner in [33, 34] {
        let Control::Table(source) = &core.document().sections[0].paragraphs[owner].controls[0]
        else {
            panic!("원본 용지 기준 글뒤 표");
        };
        let table = placed
            .iter()
            .find(|node| matches!(&node.node_type, RenderNodeType::Table(meta)
                if meta.para_index == Some(owner) && meta.cell_context.is_none()))
            .unwrap();
        let expected = f64::from(source.common.horizontal_offset)
            + f64::from(source.outer_margin_left);
        let actual_hwp = table.bbox.x * 7200.0 / 96.0;
        assert!((actual_hwp - expected).abs() < 0.01,
            "표 원점은 원본의 바깥 상자 오프셋과 여백을 한 번 소비함: {actual_hwp} / {expected}");
    }
}
