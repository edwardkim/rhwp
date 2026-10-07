//! Issue #2279 / PR #2470: 마스킹 원본 두 종류의 한컴 2022 정합 회귀.
//!
//! 독립 PDF와 Native/fresh WASM 전쪽 비교에서 두 문서의 최저값이90% 이상인
//! 보정 결과를 기준으로 한다. 과거9쪽 중간값이나 절대 픽셀 원점을 고정하지
//! 않고2/8쪽 소속과 실제 글줄·셀 여백·후속 내용의 관계를 검사한다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn load(rel_path: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel_path);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {rel_path}: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {rel_path}: {e:?}"))
}

fn find_node<'a>(
    node: &'a RenderNode,
    predicate: &impl Fn(&RenderNode) -> bool,
) -> Option<&'a RenderNode> {
    if predicate(node) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_node(child, predicate))
}

fn compact_text(node: &RenderNode) -> String {
    fn collect(node: &RenderNode, top: f64, bottom: f64, text: &mut String) {
        let (top, bottom) = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
            (
                top.max(node.bbox.y),
                bottom.min(node.bbox.y + node.bbox.height),
            )
        } else {
            (top, bottom)
        };
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if node.bbox.y >= top - 0.5 && node.bbox.y + node.bbox.height <= bottom + 0.5 {
                text.extend(run.text.chars().filter(|ch| !ch.is_whitespace()));
            }
        }
        for child in &node.children {
            collect(child, top, bottom, text);
        }
    }
    let mut text = String::new();
    collect(node, node.bbox.y, node.bbox.y + node.bbox.height, &mut text);
    text
}

/// 사진 제거 뒤 셀 내용이 소유하는 실제 행과 후속 계획의2쪽 소속을 유지한다.
#[test]
fn issue_2470_stale_lh_table_36382471_matches_hangul_two_pages() {
    let doc = load("samples/issue2470/36382471_masked.hwpx");
    assert_eq!(doc.page_count(), 2, "한컴2022 독립 PDF는2쪽이다");
    let tree = doc.build_page_render_tree(1).expect("2쪽 렌더 트리");
    let table = find_node(&tree.root, &|node| {
        matches!(&node.node_type,
        RenderNodeType::Table(t) if t.para_index == Some(10) && t.control_index == Some(0))
    })
    .expect("2쪽 현황사진 표의 원본 소속");
    let Control::Table(source) = &doc.document().sections[0].paragraphs[10].controls[0] else {
        panic!("현황사진 표의 원본 소속이 바뀌었다");
    };
    let cells: Vec<_> = table
        .children
        .iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::TableCell(_)))
        .collect();
    assert_eq!(cells.len(), source.cells.len(), "모든 원본 셀을 보존한다");
    for cell in cells {
        let RenderNodeType::TableCell(owner) = &cell.node_type else {
            unreachable!()
        };
        let original = source
            .cells
            .iter()
            .find(|c| c.row == owner.row && c.col == owner.col)
            .expect("원본 셀 소속");
        let line = find_node(cell, &|node| {
            matches!(node.node_type, RenderNodeType::TextLine(_))
        })
        .expect("마스킹 글줄");
        let pad = original.effective_padding(&source.padding);
        let padding_height = f64::from(pad.top + pad.bottom) * 96.0 / 7200.0;
        assert!(
            (cell.bbox.height - line.bbox.height - padding_height).abs() <= 0.5,
            "사진 제거 뒤 행은 글줄과 안 여백을 소유하며 낡은 개체 높이로 재확장하지 않는다"
        );
        assert_eq!(compact_text(cell), "*", "셀의 마스킹 글자를 보존한다");
        assert!(
            line.bbox.y >= cell.bbox.y - 0.5
                && line.bbox.y + line.bbox.height <= cell.bbox.y + cell.bbox.height + 0.5,
            "글줄은 자신의 셀 안에 있어야 한다"
        );
    }
    let following = find_node(&tree.root, &|node| {
        matches!(&node.node_type,
        RenderNodeType::TextLine(line) if line.para_index == Some(11))
    })
    .expect("같은2쪽의 후속 추진계획");
    assert!(
        following.bbox.y >= table.bbox.y + table.bbox.height,
        "추진계획은 현황사진 표 뒤에 놓인다"
    );
    let text = compact_text(&tree.root);
    assert!(
        text.contains("추진계획") && text.contains("향후계획") && text.contains("견적서및타견적서"),
        "2쪽 후속 내용이 보존된다"
    );
}

/// 독립8쪽 기준과6~8쪽의 도해·자격 문단·향후 계획 소속을 유지한다.
#[test]
fn issue_2470_masked_rewrap_36341511_matches_hangul_eight_pages() {
    let doc = load("samples/issue2470/36341511_masked.hwpx");
    assert_eq!(
        doc.page_count(),
        8,
        "한컴2022 독립 PDF는8쪽이며 과거9쪽 중간값을 고정하지 않는다"
    );
    for (page, required) in [
        (
            5,
            vec!["추진절차", "서남센터2처리장조목스크린및인양기철거공사"],
        ),
        (
            6,
            vec!["면허를등록한업체", "서울특별시로등록한업체", "감시제어공사"],
        ),
        (7, vec!["향후계획", "사업이행및준공"]),
    ] {
        let tree = doc
            .build_page_render_tree(page)
            .expect("원본 문단 소속 쪽 렌더 트리");
        let text = compact_text(&tree.root);
        for expected in required {
            assert!(
                text.contains(expected),
                "{}쪽은 독립 PDF의 {expected} 내용을 소유한다",
                page + 1
            );
        }
    }
}
