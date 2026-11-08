//! [#8105] 행을 닫는 표 조각은 칸 끝 빈 문단의 줄 상자까지 차지한다.
//!
//! 86712 규제영향분석서 21쪽(정본 `pdf/86712_regulatory_analysis-hwp-2024.pdf`): 1×1 표 `pi=101` 의
//! 끝 조각은 칸의 마지막 빈 문단까지 그린다. 정본은 그 빈 줄까지 상자에 담고 뒤 문단 «3» 을 상자 아래에
//! 둔다. 수정 전 rhwp 는 조각 높이를 잴 때만 끝 빈 문단을 빼(330.2px < 그린 상자 344.3px) «3» 이
//! 표 상자 안에 겹쳤고 21쪽 나머지가 14px 씩 위로 갔다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE_86712: &str = "samples/86712_regulatory_analysis.hwp";

fn core() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE_86712);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).expect("parse 86712")
}

fn line_text(node: &RenderNode) -> String {
    fn text(node: &RenderNode, out: &mut String) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            out.extend(run.text.chars().filter(|c| !c.is_whitespace()));
        }
        for child in &node.children {
            text(child, out);
        }
    }
    let mut out = String::new();
    text(node, &mut out);
    out
}

/// 쪽의 표 `(구역, 문단, 개체)` 상자 아래와, 표 밖 본문 줄 `(위, 글)`.
fn table_bottom_and_body_lines(
    core: &DocumentCore,
    page: u32,
    table: (usize, usize, usize),
) -> (Option<f64>, Vec<(f64, String)>) {
    fn walk(
        node: &RenderNode,
        table: (usize, usize, usize),
        bottom: &mut Option<f64>,
        lines: &mut Vec<(f64, String)>,
    ) {
        match &node.node_type {
            RenderNodeType::Table(t) => {
                if (t.section_index, t.para_index, t.control_index)
                    == (Some(table.0), Some(table.1), Some(table.2))
                {
                    *bottom = Some(node.bbox.y + node.bbox.height);
                }
                return;
            }
            RenderNodeType::TextLine(_) => {
                lines.push((node.bbox.y, line_text(node)));
                return;
            }
            _ => {}
        }
        for child in &node.children {
            walk(child, table, bottom, lines);
        }
    }
    let tree = core
        .build_page_render_tree(page)
        .unwrap_or_else(|error| panic!("{}쪽 렌더 트리: {error:?}", page + 1));
    let (mut bottom, mut lines) = (None, Vec::new());
    walk(&tree.root, table, &mut bottom, &mut lines);
    (bottom, lines)
}

#[test]
fn closing_fragment_keeps_the_cell_trailing_empty_line_box() {
    let core = core();
    let (bottom, lines) = table_bottom_and_body_lines(&core, 20, (0, 101, 0));
    let bottom = bottom.expect("21쪽에 표 pi=101 의 끝 조각이 있어야 한다");
    let (top, _) = lines
        .iter()
        .find(|(_, text)| text == "3")
        .unwrap_or_else(|| panic!("21쪽에 문단 «3» 이 있어야 한다: {lines:?}"));
    assert!(
        *top >= bottom - 0.5,
        "문단 «3» 은 표 끝 조각 상자 아래에서 시작한다(정본): 줄 위 {top:.2} / 표 아래 {bottom:.2}"
    );
}
