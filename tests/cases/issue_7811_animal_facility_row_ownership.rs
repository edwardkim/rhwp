//! 관계형·고정형 — #7811 자리차지에 참여하는 글 앞으로 표의 행 소유와 후속 본문 쪽을 보호한다.
//! 주석 갱신 2026-11-28T07:25:26+09:00. 직전 파일 커밋 9eb669d2052f45362122549813188a35d8029b24 2026-11-28T04:50:18+09:00.
//! Native Skia·fresh WASM Print를 독립 한컴 PDF에 96 DPI·PDFium·2px raw 실루엣으로 대조했다.
//! 측정 코드 48ce72ba80f6ca43a2914ec60093348b064df4d2.
//! animal-facility · 기준 pdf/issue5792/2700727_animal_facility_standards-hwpx-2020.pdf · 전체 6쪽.
//! 전쪽 최저 Native 99.99% / fresh WASM 100.00%.
//! 2쪽 Native 100.00% / fresh WASM 100.00%.
//! 3쪽 Native 100.00% / fresh WASM 100.00%.
//! 4쪽 Native 100.00% / fresh WASM 100.00%.
//! 전쪽 TSV와 대표 PNG는 mydocs/pr/assets/issues7789_7812/animal-facility/에 있다.
//! 2·3쪽에 42행이 나뉘고 4쪽에서 곤충이용 본문이 시작한다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeSet;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue5792/2700727_animal_facility_standards.hwpx";
const TABLE_PARA: usize = 9;

fn columns(node: &RenderNode) -> impl Iterator<Item = &RenderNode> {
    node.children
        .iter()
        .filter(|child| matches!(child.node_type, RenderNodeType::Body { .. }))
        .flat_map(|body| &body.children)
        .filter(|child| matches!(child.node_type, RenderNodeType::Column(_)))
}

fn collect_text(node: &RenderNode, text: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        text.extend(run.text.chars().filter(|ch| !ch.is_whitespace()));
    }
    for child in &node.children {
        collect_text(child, text);
    }
}

#[test]
fn animal_facility_preserves_split_rows_and_following_body() {
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
        .expect("동물이용 취급시설 기준 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("HWPX 파싱");
    let Control::Table(table) = &core.document().sections[0].paragraphs[TABLE_PARA].controls[0]
    else {
        panic!("문단 {TABLE_PARA}의 원본 표");
    };
    assert!(table.common.flow_with_text);
    assert_eq!(core.page_count(), 6, "독립 한컴 Print 쪽수");

    let mut owned = Vec::new();
    let mut fragment_pages = Vec::new();
    let mut following_heading_pages = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("쪽 렌더 트리");
        for column in columns(&tree.root) {
            for frame in &column.children {
                let RenderNodeType::Table(rendered) = &frame.node_type else {
                    continue;
                };
                if rendered.para_index != Some(TABLE_PARA) || rendered.control_index != Some(0) {
                    continue;
                }
                fragment_pages.push(page + 1);
                let rows: BTreeSet<_> = frame
                    .children
                    .iter()
                    .filter_map(|node| match &node.node_type {
                        RenderNodeType::TableCell(cell) => Some(cell.row),
                        _ => None,
                    })
                    .collect();
                owned.extend(rows);
                for line in column
                    .children
                    .iter()
                    .filter(|n| matches!(n.node_type, RenderNodeType::TextLine(_)))
                {
                    let mut text = String::new();
                    collect_text(line, &mut text);
                    if text.is_empty() {
                        continue;
                    }
                    let overlap = (line.bbox.y + line.bbox.height)
                        .min(frame.bbox.y + frame.bbox.height)
                        - line.bbox.y.max(frame.bbox.y);
                    assert!(
                        overlap <= 0.5,
                        "{}쪽 표 조각이 본문 줄과 {overlap:.2}px 겹침",
                        page + 1,
                    );
                }
            }
            for line in &column.children {
                if matches!(line.node_type, RenderNodeType::TextLine(_)) {
                    let mut text = String::new();
                    collect_text(line, &mut text);
                    if text.contains("곤충이용") {
                        following_heading_pages.push(page + 1);
                    }
                }
            }
        }
    }
    assert_eq!(
        owned,
        (0..table.row_count).collect::<Vec<_>>(),
        "42행의 순서와 단일 소유"
    );
    assert_eq!(fragment_pages, [2, 3], "독립 Print의 표 조각");
    assert_eq!(following_heading_pages, [4], "독립 Print의 후속 본문");
}
