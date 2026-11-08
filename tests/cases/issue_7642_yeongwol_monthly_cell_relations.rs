//! 영월 서식의 월별·누계 셀 소유, 두 줄 순서와 셀 내부 포함 계약.
//! 독립 한컴 2020 Print와 Native/fresh WASM 전 4쪽 최저 99.29%를 확인했다.
//! 절대 화면 좌표로 셀을 선택하거나 과거의 한 줄 출력을 고정하지 않는다.
//! 동일 함수는 수정 전 fe5ad4db9에서 누계의 셀 포함 위반으로 FAIL했다.
//! 수정 후 PASS와 전체 쪽 시각 선행 증거는 task_m100_7642_stage405.md에 있다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn collect<'a>(
    node: &'a RenderNode,
    accept: fn(&RenderNode) -> bool,
    found: &mut Vec<&'a RenderNode>,
) {
    if accept(node) {
        found.push(node);
    }
    for child in &node.children {
        collect(child, accept, found);
    }
}

fn text(node: &RenderNode) -> String {
    let mut runs = Vec::new();
    collect(
        node,
        |n| matches!(n.node_type, RenderNodeType::TextRun(_)),
        &mut runs,
    );
    runs.iter()
        .map(|n| match &n.node_type {
            RenderNodeType::TextRun(run) => run.display_or_text(),
            _ => unreachable!(),
        })
        .collect::<String>()
}

#[test]
fn monthly_and_cumulative_labels_keep_source_cell_and_two_contained_lines() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue5699/20099369_yeongwol_forms.hwp");
    let core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("원본 읽기")).expect("원본 로드");
    assert_eq!(
        core.page_count(),
        4,
        "독립 Print 전체 쪽수와 내용 소유를 함께 확인한다"
    );
    let mut owners = Vec::new();
    for page_index in 0..core.page_count() {
        let page = core
            .build_page_render_tree(page_index)
            .expect("실제 쪽 출력");
        let mut tables = Vec::new();
        collect(
            &page.root,
            |n| matches!(n.node_type, RenderNodeType::Table(_)),
            &mut tables,
        );
        for table in tables {
            let mut cells = Vec::new();
            collect(
                table,
                |n| matches!(n.node_type, RenderNodeType::TableCell(_)),
                &mut cells,
            );
            for cell in cells {
                if text(cell) != "월별누계" {
                    continue;
                }
                let RenderNodeType::Table(table_owner) = &table.node_type else {
                    unreachable!()
                };
                let RenderNodeType::TableCell(cell_owner) = &cell.node_type else {
                    unreachable!()
                };
                let section = table_owner.section_index.expect("원본 구역 소유");
                let paragraph = table_owner.para_index.expect("원본 문단 소유");
                let control = table_owner.control_index.expect("원본 표 소유");
                let Control::Table(source_table) =
                    &core.document().sections[section].paragraphs[paragraph].controls[control]
                else {
                    panic!("실제 표의 원본 주소")
                };
                let source_cell = &source_table.cells
                    [cell_owner.model_cell_index.expect("원본 셀 인덱스") as usize];
                assert_eq!(
                    (cell_owner.row, cell_owner.col),
                    (source_cell.row, source_cell.col)
                );
                let source_text: String = source_cell
                    .paragraphs
                    .iter()
                    .flat_map(|p| p.text.chars())
                    .filter(|c| !c.is_whitespace())
                    .collect();
                assert_eq!(source_text, "월별누계", "출력과 원본의 셀 내용 대응");
                let mut lines = Vec::new();
                collect(
                    cell,
                    |n| matches!(n.node_type, RenderNodeType::TextLine(_)),
                    &mut lines,
                );
                assert_eq!(
                    lines.len(),
                    2,
                    "독립 Print의 두 줄을 한 줄로 축소하지 않는다"
                );
                assert_eq!(
                    (text(lines[0]), text(lines[1])),
                    ("월별".into(), "누계".into())
                );
                assert!(
                    lines[0].bbox.y + lines[0].bbox.height <= lines[1].bbox.y,
                    "월별 뒤에 누계를 겹치지 않고 표시한다"
                );
                for line in &lines {
                    assert!(
                        line.bbox.x >= cell.bbox.x
                            && line.bbox.x + line.bbox.width <= cell.bbox.x + cell.bbox.width
                            && line.bbox.y >= cell.bbox.y
                            && line.bbox.y + line.bbox.height <= cell.bbox.y + cell.bbox.height,
                        "각 글줄의 점유 범위가 소유 셀 내부에 있다"
                    );
                }
                owners.push((
                    page_index,
                    section,
                    paragraph,
                    control,
                    cell_owner.row,
                    cell_owner.col,
                ));
            }
        }
    }
    assert_eq!(owners.len(), 1, "월별·누계 소유 셀의 누락·중복을 검출한다");
    assert_eq!(
        owners[0].0, 2,
        "독립 Print의 세 번째 쪽에 해당 내용이 보존된다"
    );
}
