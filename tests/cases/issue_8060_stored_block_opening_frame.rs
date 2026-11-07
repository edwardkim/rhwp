//! #8060 저장 쪽 경계에서 닫힌 병합 블록의 첫 상자와 이어받기.
//! 주석 갱신: 2026-11-07 14:54 KST.
//! 고정형: opening_and_continuing_titles_match_independent_print는 독립 Print의
//! 2·3쪽 문자 기준선을 96 DPI 쪽 좌표계에서 ±2px로 보호한다. 전체 48쪽도 확인한다.
//! 관계형: 원본 선언 프레임과 실제 종료선, 선택한 내용의 가운데 정렬,
//! 소진한 블록 앞 행의 비재생성, 앞뒤 쪽의 제목 단일 소유·순서를 확인한다.
//! Native/fresh WASM 전 48쪽 최저 99.22127%/99.24%, 2쪽
//! 99.99666%/99.99678%, 3쪽 양쪽 100%에서 review·overlay를 직접 판독했다.
//! 2024 호환, 원본 HWP SHA-256:
//! 4e9b240082ba91b99669325292cae2b62e88765854d1e7bbbc6d44146dc14d55.
//! 독립 기준: pdf/issue8060/71352_lpg_regulatory_impact-2024.pdf,
//! SHA-256: c2e1b4beea546d02436e85f9b491ab64093c6327bf6ef227f02f4f075608f501.
//! 증적: mydocs/pr/assets/issue8060/stored-block-opening-20261107/.
//! 실제 배치를 생산하는 공통 render tree를 사용하며 사설 글꼴 설치에 의존하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::table::VerticalAlign;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue8060/71352_lpg_regulatory_impact.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).expect("원본 LPG 문서"))
        .expect("원본 HWP 읽기");
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 48, "독립 Print 전체 쪽수");
    core
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if node.visible && !node.editor_only {
        out.push(node);
        for child in &node.children {
            walk(child, out);
        }
    }
}

fn source_table(root: &RenderNode) -> &RenderNode {
    let mut nodes = Vec::new();
    walk(root, &mut nodes);
    let selected: Vec<_> = nodes
        .into_iter()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::Table(table)
                if table.section_index == Some(0)
                    && table.para_index == Some(13) && table.control_index == Some(0))
        })
        .collect();
    assert_eq!(selected.len(), 1, "원본 표 조각의 단일 소유");
    selected[0]
}

fn closing_cell(table: &RenderNode, col: u16) -> &RenderNode {
    table
        .children
        .iter()
        .find(|node| {
            matches!(&node.node_type, RenderNodeType::TableCell(cell)
            if cell.row == 10 && cell.col == col)
        })
        .expect("원본 마지막 부분 행의 칸")
}

fn text(root: &RenderNode) -> String {
    let mut nodes = Vec::new();
    walk(root, &mut nodes);
    nodes
        .into_iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run) => Some(run.text.as_str()),
            _ => None,
        })
        .collect()
}

fn compact(value: &str) -> String {
    value
        .chars()
        .filter(|c| !c.is_whitespace() && !c.is_control())
        .collect()
}

#[test]
fn closing_title_centers_inside_the_selected_first_frame() {
    let core = load();
    let Control::Table(source) = &core.document().sections[0].paragraphs[13].controls[0] else {
        panic!("원본 표");
    };
    let source_cell = source
        .cells
        .iter()
        .find(|cell| cell.row == 10 && cell.col == 1)
        .unwrap();
    assert_eq!(source_cell.vertical_align, VerticalAlign::Center);
    let tree = core.build_page_render_tree(1).unwrap();
    let cell = closing_cell(source_table(&tree.root), 1);
    let line = cell
        .children
        .iter()
        .find(|node| {
            matches!(node.node_type, RenderNodeType::TextLine(_)) && text(node).contains("비용편익")
        })
        .expect("첫 쪽이 실제 소유한 제목 줄");
    let line_center = line.bbox.y + line.bbox.height / 2.0;
    let cell_center = cell.bbox.y + cell.bbox.height / 2.0;
    assert!(
        (line_center - cell_center).abs() <= 1.0 / 75.0,
        "선택한 첫 줄만 원본 Center에 정렬: 줄={line_center}, 칸={cell_center}"
    );
}

#[test]
fn first_fragment_paints_the_same_source_frame_it_reserves() {
    let core = load();
    let Control::Table(source) = &core.document().sections[0].paragraphs[13].controls[0] else {
        panic!("원본 표");
    };
    let declared = f64::from(source.common.height) / 75.0;
    let tree = core.build_page_render_tree(1).unwrap();
    let table = source_table(&tree.root);
    for col in [1, 2] {
        let cell = closing_cell(table, col);
        let physical = cell.bbox.y + cell.bbox.height - table.bbox.y;
        assert!(
            (physical - declared).abs() <= 1.0 / 75.0,
            "원본 개체 상자의 종료선: 칸={col}, 실제={physical}, 선언={declared}"
        );
    }
}

#[test]
fn completed_block_prefix_does_not_open_a_blank_continuation_row() {
    let core = load();
    let tree = core.build_page_render_tree(2).unwrap();
    let table = source_table(&tree.root);
    let cell = closing_cell(table, 1);
    assert!(
        (cell.bbox.y - table.bbox.y).abs() <= 1.0 / 75.0,
        "이미 소진한 블록 앞 행을 빈 띠로 다시 열지 않는다"
    );
    assert!(compact(&text(cell)).starts_with("분석"));
}

#[test]
fn source_closing_and_continuing_title_text_is_owned_once() {
    let core = load();
    let mut pages = Vec::new();
    for page in [1, 2] {
        let tree = core.build_page_render_tree(page).unwrap();
        pages.push(compact(&text(closing_cell(source_table(&tree.root), 1))));
    }
    assert_eq!(pages, ["11.비용편익", "분석(정성분석)"]);
}

#[test]
fn opening_and_continuing_titles_match_independent_print() {
    let core = load();
    // 독립 PDF 문자 원점(pt)을 96/72로 환산했다. 후보 출력에서 복사하지 않는다.
    for (page, expected) in [
        (1, vec![("11.비용편익", 1024.9598795572917)]),
        (
            2,
            vec![
                ("분석", 103.40699259440105),
                ("(정성분석)", 131.21660359700522),
            ],
        ),
    ] {
        let tree = core.build_page_render_tree(page).unwrap();
        let cell = closing_cell(source_table(&tree.root), 1);
        let mut nodes = Vec::new();
        walk(cell, &mut nodes);
        let mut found = Vec::new();
        for node in nodes {
            let RenderNodeType::TextLine(line) = &node.node_type else {
                continue;
            };
            let title = compact(&text(node));
            let &(_, baseline) = expected
                .iter()
                .find(|(value, _)| *value == title)
                .expect("독립 Print의 해당 쪽 소유 제목 줄");
            let actual = node.bbox.y + line.baseline;
            assert!(
                (actual - baseline).abs() <= 2.0,
                "{}쪽 {title}: 실제 기준선={actual}, 독립 Print={baseline}, 허용 ±2px",
                page + 1
            );
            found.push(title);
        }
        let owners: Vec<_> = expected
            .iter()
            .map(|(value, _)| value.to_string())
            .collect();
        assert_eq!(found, owners, "줄 소속·순서·누락·중복 보존");
    }
}
