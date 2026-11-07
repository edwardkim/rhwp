//! 관계형: #1994 원본 오프셋·바깥여백과 최종 표 원점의 소비 관계를 검사한다.
//! 주석 갱신: 2026-11-06T22:13:05+09:00; 직전 파일 커밋: 2026-11-06T21:09:45+09:00 (fa0436f495).
//! 2026-11-06 raw(96DPI, PDFium, 2px): 3쪽 Native 98.22544% / fresh WASM 98.19505%.
//! 전4쪽 최저 98.22544% / 98.14207%; 기준 PDF가 사용한 글꼴 환경의 시각 결과다.
//! 실행 가능한 원점 관계 검사는 시스템 글꼴 설치를 요구하지 않는다. 절대 px 핀은 별도 파일이다.
//! 근거: mydocs/pr/assets/issue1994/reference-fonts-20261106/README.md.

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
            .find(|node| {
                matches!(&node.node_type, RenderNodeType::Table(meta)
                if meta.para_index == Some(owner) && meta.cell_context.is_none())
            })
            .unwrap();
        let expected =
            f64::from(source.common.horizontal_offset) + f64::from(source.outer_margin_left);
        let actual_hwp = table.bbox.x * 7200.0 / 96.0;
        assert!(
            (actual_hwp - expected).abs() < 0.01,
            "표 원점은 원본의 바깥 상자 오프셋과 여백을 한 번 소비함: {actual_hwp} / {expected}"
        );
    }
}
