//! 검사 유형: 관계형 — 문단 오프셋과 바깥 위 여백을 각각 한 번만 소비한다.
//! 주석 갱신: 2026-11-18 02:21 KST.
//! 화학 원본 95쪽 Native 99.12075% (59e989c098와 같은 제품 소스), 최신 fresh WASM은 미검증이다.
//! 96 DPI·2px raw·독립 한컴 2020 Hancom PDF Print, 전 103쪽 Native 최저 98.46202%다.
//! 근거: mydocs/pr/assets/issues7733_7738_20261117/font-chain-inline-justify/cause.md.
//! 실제 PDF 좌표를 고정하지 않는다. 같은 원본의 세로 오프셋과 여백 변경량을 비교한다.
//! 원래 #9083의 앵커 소유 보정을 유지하며 양수 오프셋을 여백 미소비로 오인하지 않는다.
//! 개인 글꼴 설치에 의존하지 않는다.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::{model::control::Control, DocumentCore};

fn table_top(offset: u32, margin_delta: i16) -> f64 {
    let mut core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue6782/1480000-201900042-chemical-labeling-standards.hwp"
    ))
    .unwrap();
    core.set_hangul2024_compat(false);
    let mut source = core.document().clone();
    let Control::Table(table) = &mut source.sections[4].paragraphs[262].controls[0] else {
        panic!("원본 빈 문단의 1×1 자리차지 표");
    };
    assert!(!table.common.treat_as_char);
    assert_eq!((table.row_count, table.col_count), (1, 1));
    table.common.vertical_offset = offset;
    table.outer_margin_top += margin_delta;
    core.set_document(source);
    let tree = core.build_page_render_tree(94).unwrap();
    fn find(node: &RenderNode) -> Option<f64> {
        if matches!(&node.node_type, RenderNodeType::Table(meta)
            if meta.para_index == Some(262) && meta.control_index == Some(0))
        {
            return Some(node.bbox.y);
        }
        node.children.iter().find_map(find)
    }
    find(&tree.root).expect("같은 원본 첫 조각")
}

#[test]
fn positive_paragraph_offset_does_not_reopen_an_already_owned_top_margin() {
    let delta = table_top(525, 0) - table_top(0, 0);
    let expected = 525.0 * 96.0 / 7200.0;
    assert!(
        (delta - expected).abs() <= 96.0 / 7200.0,
        "문단 오프셋 변화만 소비한다: {delta} vs {expected}"
    );
}

#[test]
fn changing_the_top_margin_of_an_offset_fragment_advances_it_only_once() {
    let delta = table_top(525, 75) - table_top(525, 0);
    let expected = 75.0 * 96.0 / 7200.0;
    assert!(
        (delta - expected).abs() <= 96.0 / 7200.0,
        "양수 오프셋이 있는 조각의 여백도 한 번만 소비한다: {delta} vs {expected}"
    );
}
