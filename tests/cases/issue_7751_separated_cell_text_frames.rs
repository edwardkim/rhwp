//! #7751: 셀 사이·바깥 간격을 문단의 폭으로 배분하지 않는다.
//! 원본 편람 155쪽의 줄 소속·셀 내부 포함과 독립 Print의 기준선을 보호한다.
//! 두 형식 전 383쪽의 Native/fresh WASM 90%와 155쪽 양쪽 98%·직접 판독을 확인했다.
//! 증거: mydocs/pr/assets/issue_7751/separated_cell_text_frames/README.md
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::{model::control::Control, DocumentCore};

fn compact(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}
fn text(node: &RenderNode) -> String {
    let mut s = match &node.node_type {
        RenderNodeType::TextRun(r) => r.display_or_text().to_owned(),
        _ => String::new(),
    };
    for child in &node.children {
        s.push_str(&text(child));
    }
    s
}
fn descendants(node: &RenderNode) -> Vec<&RenderNode> {
    let mut result = vec![node];
    for child in &node.children {
        result.extend(descendants(child));
    }
    result
}
fn assert_separated_text_frame(extension: &str) {
    let input = format!("samples/2025 행정업무운영 편람(최종).{extension}");
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(&input);
    let mut core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("커밋된 편람 원본")).unwrap();
    core.set_hangul2024_compat(true);
    let page = core.build_page_render_tree(154).expect("독립 Print 155쪽");
    let tables: Vec<_> = descendants(&page.root)
        .into_iter()
        .filter(|n| {
            matches!(&n.node_type, RenderNodeType::Table(t)
            if t.section_index == Some(4) && t.para_index == Some(47)
                && t.cell_context.is_none())
        })
        .collect();
    assert_eq!(tables.len(), 1, "{extension}: 원본 표 소유·중복");
    let cells: Vec<_> = tables[0]
        .children
        .iter()
        .filter(
            |n| matches!(&n.node_type, RenderNodeType::TableCell(c) if c.row == 8 && c.col == 1),
        )
        .collect();
    assert_eq!(cells.len(), 1, "{extension}: 글자 기준 셀 소유·중복");
    let cell = cells[0];
    let lines: Vec<_> = descendants(cell).into_iter().filter(|n| {
        matches!(&n.node_type, RenderNodeType::TextLine(line) if line.para_index == Some(0))
    }).collect();
    assert_eq!(
        lines.len(),
        1,
        "{extension}: 독립 Print의 첫 기준 문단은 한 줄"
    );
    let source = core.document().sections[4].paragraphs[47]
        .controls
        .iter()
        .find_map(|c| {
            if let Control::Table(t) = c {
                Some(t.as_ref())
            } else {
                None
            }
        })
        .unwrap();
    let source_cell = source
        .cells
        .iter()
        .find(|c| c.row == 8 && c.col == 1)
        .unwrap();
    assert_eq!(
        compact(&text(lines[0])),
        compact(&source_cell.paragraphs[0].text),
        "{extension}: 첫 문단 내용·순서·누락·중복"
    );
    let leads: Vec<_> = descendants(lines[0])
        .into_iter()
        .filter(|n| {
            matches!(&n.node_type, RenderNodeType::TextRun(r)
            if compact(r.display_or_text()).starts_with("가.글자는줄또는칸의왼쪽부터"))
        })
        .collect();
    assert_eq!(leads.len(), 1, "{extension}: 실제 첫 글줄");
    let lead = leads[0];
    let RenderNodeType::TextRun(run) = &lead.node_type else {
        unreachable!()
    };
    assert!(
        lead.bbox.x >= cell.bbox.x,
        "{extension}: 첫 문단의 왼쪽은 소유 셀 내부"
    );
    assert!(
        lead.bbox.x + lead.bbox.width <= cell.bbox.x + cell.bbox.width,
        "{extension}: 첫 문단의 전진 끝은 셀 clip 밖으로 나가지 않는다"
    );
    // 두 독립 한컴 2024 Print가 모두 같은 실제 기준선을 준다.
    // PDF span origin (147pt, 209.1599884033203pt)을 96dpi로 환산했다.
    for (actual, expected, label) in [
        (lead.bbox.x, 196.0, "첫 글자 x"),
        (
            lead.bbox.y + run.baseline,
            278.87998453776044,
            "첫 글줄 기준선",
        ),
    ] {
        assert!(
            (actual - expected).abs() <= 2.0,
            "{extension}: {label}, 독립 Print {expected:.6}px, 실제 {actual:.6}px"
        );
    }
}
#[test]
fn handbook_hwp_separated_cell_owns_its_text_frame() {
    assert_separated_text_frame("hwp");
}
#[test]
fn handbook_hwpx_separated_cell_owns_its_text_frame() {
    assert_separated_text_frame("hwpx");
}
