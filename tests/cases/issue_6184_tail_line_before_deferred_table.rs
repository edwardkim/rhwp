//! #6184: 원본 축약 파일 자체의 독립 Print에 맞춰 호스트·표·꼬리의 소유를 검사한다.
//!
//! 과거 검사는 전체 원문10쪽의 위치를 축약본 첫 쪽에 고정했다. 같은 축약 HWP의
//! 실제 한컴2020 Print는2쪽이며, 두 번째 쪽에 첫 주석 줄→표→마지막 주석 줄이 있다.
//! 절대900/200px 영역이나 과거 문서의 쪽 번호를 새 축약본의 기대값으로 쓰지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue6184/156489124_tail_line_before_deferred_table.hwp";
const HOST_PARA: usize = 38;

fn compact(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace() && !c.is_control())
        .collect()
}

fn text(node: &RenderNode) -> String {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        return run.text.clone();
    }
    node.children.iter().map(text).collect()
}

fn collect(
    node: &RenderNode,
    page: usize,
    tables: &mut Vec<(usize, BoundingBox)>,
    lines: &mut Vec<(usize, String, BoundingBox)>,
) {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(HOST_PARA) && table.cell_context.is_none() {
            tables.push((page, node.bbox));
        }
        return; // 셀의 지역 문단 번호를 본문 호스트로 오인하지 않는다.
    }
    if matches!(&node.node_type, RenderNodeType::TextLine(line) if line.para_index == Some(HOST_PARA))
    {
        let content = compact(&text(node));
        if !content.is_empty() {
            lines.push((page, content, node.bbox));
        }
    }
    for child in &node.children {
        collect(child, page, tables, lines);
    }
}

#[test]
fn issue_6184_cut_input_preserves_print_host_table_and_tail_ownership() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("축약 원본")).expect("열기");
    assert_eq!(
        core.page_count(),
        2,
        "같은 축약 원본의 독립 Print 전체 쪽수"
    );
    let mut tables = Vec::new();
    let mut lines = Vec::new();
    for page in 0..core.page_count() {
        collect(
            &core.build_page_render_tree(page).unwrap().root,
            page as usize,
            &mut tables,
            &mut lines,
        );
    }
    assert_eq!(tables.len(), 1, "호스트 표의 누락·중복");
    assert_eq!(lines.len(), 2, "독립 Print의 선행·후행 주석 줄");
    assert_eq!(tables[0].0, 1, "Print 두 번째 쪽의 표 소유");
    assert!(
        lines.iter().all(|line| line.0 == tables[0].0),
        "호스트와 표의 같은 쪽 소유"
    );
    assert!(lines[0].1.starts_with("※국민적관심이큰"));
    assert_eq!(lines[1].1, "선대책발표");
    assert_eq!(
        lines.iter().map(|line| line.1.as_str()).collect::<String>(),
        compact(&core.document().sections[0].paragraphs[HOST_PARA].text),
        "원문 순서·누락·중복"
    );
    let table = tables[0].1;
    assert!(lines[0].2.y + lines[0].2.height <= table.y, "첫 줄은 표 위");
    assert!(
        lines[1].2.y >= table.y + table.height,
        "마지막 줄은 표 아래"
    );
}
