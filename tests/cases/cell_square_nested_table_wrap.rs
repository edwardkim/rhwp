//! 독립 한컴 2020 Print와 Native255/fresh WASM073 전 1쪽 raw 100%·직접 판독을 확인했다.
//! 원본 소유·순서·크기 관계를 유지하며 중요한 글자 원점만 독립 PDF의 ±2px로 보호한다.
//! 입력/PDF 해시·TSV·판독과 수정 전 FAIL/수정 후 PASS는
//! mydocs/working/task_semanticist21_20261026_inline_wrap_contracts.md에 연결한다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SOURCE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/pr8045_square_wrap/square-hancom2020.hwpx"
));

fn collect<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        collect(child, out);
    }
}

fn contents(node: &RenderNode) -> String {
    let mut text = match &node.node_type {
        RenderNodeType::TextRun(run) => run.display_or_text().to_owned(),
        _ => String::new(),
    };
    for child in &node.children {
        text.push_str(&contents(child));
    }
    text
}

#[test]
fn verified_square_saved_lane_and_parent_frame_share_source_ownership() {
    let core = DocumentCore::from_bytes(SOURCE).unwrap();
    assert_eq!(core.page_count(), 1, "독립 Print 전체 1쪽");
    let source = core.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|paragraph| &paragraph.controls)
        .find_map(|control| {
            if let Control::Table(table) = control {
                Some(table)
            } else {
                None
            }
        })
        .unwrap();
    let cell = &source.cells[0];
    let memo_source = cell.paragraphs.iter().find(|p| p.text == "메모").unwrap();
    let row = &memo_source.line_segs[0];
    let tree = core.build_page_render_tree(0).unwrap();
    let mut nodes = Vec::new();
    collect(&tree.root, &mut nodes);
    let select = |cols| {
        let selected: Vec<_> = nodes.iter().copied().filter(|node| {
            matches!(&node.node_type, RenderNodeType::Table(t) if t.row_count == 1 && t.col_count == cols)
        }).collect();
        assert_eq!(selected.len(), 1, "원본 표의 단일 소유");
        selected[0]
    };
    let outer = select(1);
    let nested = select(5);
    let memo: Vec<_> = nodes.iter().copied().filter(|node| {
        matches!(&node.node_type, RenderNodeType::TextRun(run) if run.display_or_text() == "메모")
    }).collect();
    assert_eq!(memo.len(), 1, "내용 누락·중복 없음");
    let memo = memo[0];
    let unit = 96.0 / 7200.0;
    for (actual, expected) in [
        (
            (memo.bbox.x - outer.bbox.x) / unit,
            f64::from(row.column_start),
        ),
        (
            (memo.bbox.y - outer.bbox.y) / unit,
            f64::from(row.vertical_pos),
        ),
        (outer.bbox.height / unit, f64::from(cell.height)),
    ] {
        assert!(
            (actual - expected).abs() <= 1.0,
            "원본 저장 위치·행 높이의 단일 소유: {actual} / {expected}"
        );
    }
    assert!(
        memo.bbox.x + unit >= nested.bbox.x + nested.bbox.width,
        "어울림 표 옆 가로 배제"
    );
    assert!(
        memo.bbox.y >= outer.bbox.y
            && memo.bbox.y + memo.bbox.height <= outer.bbox.y + outer.bbox.height,
        "메모는 부모 프레임 내부"
    );
    assert_eq!(
        contents(outer).matches("메모").count(),
        1,
        "부모에 속한 내용 보존"
    );
    // 독립 Print 메모 원점: PDF pt → 96dpi. 양쪽 raw 100%·직접 판독을 확인했다.
    let svg = core.render_page_svg_native(0).unwrap();
    let xml = roxmltree::Document::parse(&svg).unwrap();
    let text = xml
        .descendants()
        .find(|n| {
            n.is_element()
                && n.tag_name().name() == "text"
                && n.text().is_some_and(|text| text.starts_with('메'))
        })
        .expect("메모 첫 글자의 SVG 원점");
    for (name, pt) in [("x", 257.50042724609375), ("y", 111.23843383789062)] {
        let actual = text.attribute(name).unwrap().parse::<f64>().unwrap();
        let expected = pt * 96.0 / 72.0;
        assert!(
            (actual - expected).abs() <= 2.0,
            "독립 Print 메모 {name}: {actual} / {expected}"
        );
    }
}
