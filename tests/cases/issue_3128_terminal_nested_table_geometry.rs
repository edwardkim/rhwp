//! Issue #3128: native HWP5 RowBreak 종료 조각이 빈 host tail을 두 번 예약해
//! 34쪽 표와 뒤따르는 직접편익 표를 한컴 2024 PDF보다 아래로 미는 회귀 가드.
//!
//! 정답지는 `samples/issue1891/76076_regulatory_analysis-2024.pdf`의 34쪽이다.
//! 첫 표의 continuation frame은 y=77..463px, 다음 직접편익 표는 y=512px에서
//! 시작한다(96dpi raster). SVG render-tree bbox는 stroke/clip 외곽을 포함하므로
//! raster 선과 1∼4px 차이를 허용하되, 상·하단을 독립적으로 고정한다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/76076_regulatory_analysis.hwp";

fn find_table_with_owner_para(node: &RenderNode, para_index: usize) -> Option<&RenderNode> {
    if matches!(
        &node.node_type,
        RenderNodeType::Table(table) if table.para_index == Some(para_index)
    ) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_table_with_owner_para(child, para_index))
}

fn text_line_contains(node: &RenderNode, needles: &[&str]) -> bool {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        let text = node
            .children
            .iter()
            .filter_map(|child| match &child.node_type {
                RenderNodeType::TextRun(run) => Some(run.text.as_str()),
                _ => None,
            })
            .collect::<String>();
        if needles.iter().all(|needle| text.contains(needle)) {
            return true;
        }
    }
    node.children
        .iter()
        .any(|child| text_line_contains(child, needles))
}

#[test]
fn issue_3128_terminal_continuation_keeps_hancom_line_wrap() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(path).expect("read #3128 authority fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse #3128 authority fixture");
    let page = core
        .build_page_render_tree(33)
        .expect("render HWP 2024 PDF p34");
    let continuation =
        find_table_with_owner_para(&page.root, 325).expect("p34 activity-cost continuation");

    assert!(
        text_line_contains(continuation, &["연동시스템 등"]),
        "Hancom keeps `연동시스템 등` on the first indented continuation line"
    );
}
