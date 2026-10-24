//! [#6035] 원본 CELL 평가표의 내용 컷과 물리 쪽 프레임을 함께 보존한다.
//!
//! 독립 한컴2020 Print의 물 공급 설비 제목은35쪽, 세 하위 항목은36쪽이다.
//! 제목과 항목을 같은 쪽으로 강제하던 기존 검사는 이 정상 경계를 빈 쪽으로 오판했다.
//! 원본의 저장 줄·문단 소속과 실제 셀의 포함·순서·보존을 검사한다.
//! 현재7749d의 Native/fresh WASM 전48쪽: 최저47쪽93.64014%, 미달·누락0.
//! 현재 전쪽 RGBA도 같으며 증적은 통합 review의 진행185에 연결한다.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::{control::Control, table::Table};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6035/cgmp_evaluation_table.hwpx";
const HEADING: &str = "다. 물 공급 설비는 다음을 만족하는가";

fn load_doc() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&fs::read(path).expect("평가표 원본")).expect("원본 열기")
}

fn compact(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn source(core: &DocumentCore) -> (usize, usize, &Table) {
    core.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find_map(|(pi, paragraph)| {
            paragraph
                .controls
                .iter()
                .enumerate()
                .find_map(|(ci, control)| {
                    let Control::Table(table) = control else {
                        return None;
                    };
                    table
                        .cells
                        .iter()
                        .any(|cell| cell.paragraphs.iter().any(|p| p.text.contains(HEADING)))
                        .then_some((pi, ci, table.as_ref()))
                })
        })
        .expect("원본 평가표의 소유 문단·컨트롤")
}

fn source_row(table: &Table, text: &str) -> u16 {
    table
        .cells
        .iter()
        .find(|cell| {
            cell.paragraphs
                .iter()
                .any(|p| compact(&p.text).contains(&compact(text)))
        })
        .expect("원본 평가 항목")
        .row
}

#[derive(Debug)]
struct Fragment {
    page: u32,
    start: usize,
    end: usize,
    start_cut: Vec<usize>,
    end_cut: Vec<usize>,
}

fn fragments(core: &DocumentCore, pi: usize, ci: usize) -> Vec<Fragment> {
    let pages = core.dump_page_items_json(None);
    let mut result = Vec::new();
    for (page, entry) in pages.as_array().expect("쪽 목록").iter().enumerate() {
        for column in entry["columns"].as_array().into_iter().flatten() {
            for item in column["items"].as_array().into_iter().flatten() {
                if item["kind"] != "partialTable"
                    || item["paraIndex"] != pi
                    || item["controlIndex"] != ci
                {
                    continue;
                }
                let cut = |key: &str| {
                    item[key]
                        .as_array()
                        .expect("유닛 컷")
                        .iter()
                        .map(|v| v.as_u64().expect("컷 위치") as usize)
                        .collect()
                };
                result.push(Fragment {
                    page: page as u32,
                    start: item["startRow"].as_u64().expect("시작 행") as usize,
                    end: item["endRow"].as_u64().expect("끝 행") as usize,
                    start_cut: cut("startCut"),
                    end_cut: cut("endCut"),
                });
            }
        }
    }
    result
}

fn split_pair(fragments: &[Fragment], row: u16) -> (&Fragment, &Fragment) {
    let index = fragments
        .iter()
        .position(|f| f.end == usize::from(row) + 1 && !f.end_cut.is_empty())
        .expect("원본 행의 종료 컷");
    let before = &fragments[index];
    let after = &fragments[index + 1];
    assert_eq!(
        after.page,
        before.page + 1,
        "불필요한 빈 쪽을 만들지 않는다"
    );
    (before, after)
}

fn walk<'a>(node: &'a RenderNode, nodes: &mut Vec<&'a RenderNode>) {
    nodes.push(node);
    for child in &node.children {
        walk(child, nodes);
    }
}

fn row_cell(node: &RenderNode, pi: usize, ci: usize, row: u16) -> &RenderNode {
    let mut nodes = Vec::new();
    walk(node, &mut nodes);
    let table = nodes
        .into_iter()
        .find(|n| {
            matches!(&n.node_type, RenderNodeType::Table(t)
        if t.para_index == Some(pi) && t.control_index == Some(ci))
        })
        .expect("원본 표의 실제 그리기 소유자");
    table.children.iter().find(|child| {
        matches!(&child.node_type, RenderNodeType::TableCell(c) if c.row == row && c.col == 0)
    }).expect("원본 행의 내용 셀")
}

fn cell_text(cell: &RenderNode) -> String {
    let mut nodes = Vec::new();
    walk(cell, &mut nodes);
    compact(
        &nodes
            .into_iter()
            .filter_map(|node| {
                if let RenderNodeType::TextRun(run) = &node.node_type {
                    Some(run.text.as_str())
                } else {
                    None
                }
            })
            .collect::<String>(),
    )
}

fn assert_text_inside(cell: &RenderNode) {
    let mut nodes = Vec::new();
    walk(cell, &mut nodes);
    for node in nodes {
        if matches!(&node.node_type, RenderNodeType::TextRun(run) if !run.text.trim().is_empty()) {
            assert!(
                node.bbox.y >= cell.bbox.y - 1e-8
                    && node.bbox.y + node.bbox.height <= cell.bbox.y + cell.bbox.height + 1e-8,
                "실제 글줄은 소유 셀의 물리 높이 안에 있어야 한다: 글줄 {:?}, 셀 {:?}",
                node.bbox,
                cell.bbox
            );
        }
    }
}

#[test]
fn issue_6035_water_supply_subitems_follow_the_original_split_frame() {
    let core = load_doc();
    let (pi, ci, table) = source(&core);
    let row = source_row(table, HEADING);
    let original = table
        .cells
        .iter()
        .find(|c| c.row == row && c.col == 0)
        .unwrap();
    let fragments = fragments(&core, pi, ci);
    let (before, after) = split_pair(&fragments, row);
    assert_eq!(after.start, usize::from(row));
    assert_eq!(after.start_cut, before.end_cut, "같은 내용 컷을 이어받는다");
    let opening = core
        .build_page_render_tree(before.page)
        .expect("제목 쪽 조각");
    let continuation = core
        .build_page_render_tree(after.page)
        .expect("하위 항목 쪽 조각");
    let heading = row_cell(&opening.root, pi, ci, row);
    let items = row_cell(&continuation.root, pi, ci, row);
    assert_eq!(cell_text(heading), compact(&original.paragraphs[0].text));
    let expected: String = original
        .paragraphs
        .iter()
        .skip(1)
        .map(|p| compact(&p.text))
        .collect();
    assert_eq!(
        cell_text(items),
        expected,
        "세 항목의 순서·누락·중복을 함께 검사한다"
    );
    assert_text_inside(heading);
    assert_text_inside(items);
}

#[test]
fn issue_6035_heading_page_retains_preceding_evaluation_rows() {
    let core = load_doc();
    let (pi, ci, table) = source(&core);
    let row = source_row(table, HEADING);
    let fragments = fragments(&core, pi, ci);
    let (before, after) = split_pair(&fragments, row);
    let text = compact(
        &core
            .extract_page_text_native(before.page)
            .expect("제목 쪽의 본문"),
    );
    for preceding in ["제12조 출고관리", "제13조 보관관리", "제14조 물의 품질"] {
        assert!(
            text.contains(&compact(preceding)),
            "제목만 남긴 빈 쪽이 아니다: {preceding}"
        );
    }
    let mut cells: Vec<_> = table.cells.iter().filter(|cell| cell.row == row).collect();
    cells.sort_by_key(|cell| cell.col);
    let original_cut: Vec<_> = cells
        .iter()
        .map(|cell| cell.paragraphs[0].line_segs.len())
        .collect();
    assert_eq!(
        before.end_cut, original_cut,
        "저장 첫 문단의 완전한 글줄 컷"
    );
    assert_eq!(after.start_cut, before.end_cut, "다음 쪽의 같은 소유 컷");
    assert_eq!(core.page_count(), 48, "독립 Print 전쪽과 같은 소유");
}

#[test]
fn issue_6035_complete_row_does_not_emit_a_line_advance_only_tail() {
    let core = load_doc();
    let (pi, ci, table) = source(&core);
    let fragments = fragments(&core, pi, ci);
    // 독립 Print34/35쪽: 원자재 입고 행은 내용을 끝낸 뒤 빈 이어받기 셀을 만들지 않는다.
    let row = source_row(table, "나. 원자재 입고시");
    let index = fragments
        .iter()
        .position(|fragment| fragment.end == usize::from(row) + 1)
        .expect("완료 행을 포함하는 앞 조각");
    let before = &fragments[index];
    let after = &fragments[index + 1];
    assert_eq!(after.page, before.page + 1, "불필요한 빈 쪽이 없다");
    assert_eq!(
        after.start,
        usize::from(row) + 1,
        "완료 행의 빈 꼬리를 다음 쪽에 다시 만들지 않는다"
    );
    assert!(after.start_cut.is_empty());

    let opening = core
        .build_page_render_tree(before.page)
        .expect("완료 행의 앞쪽");
    let continuation = core
        .build_page_render_tree(after.page)
        .expect("다음 행의 이어받기 쪽");
    let original = table
        .cells
        .iter()
        .find(|cell| cell.row == row && cell.col == 0)
        .expect("완료 행의 원본 내용 셀");
    let expected: String = original
        .paragraphs
        .iter()
        .map(|paragraph| compact(&paragraph.text))
        .collect();
    let completed = row_cell(&opening.root, pi, ci, row);
    assert_eq!(
        cell_text(completed),
        expected,
        "원본 내용의 누락·중복이 없다"
    );
    assert_text_inside(completed);

    let mut next_nodes = Vec::new();
    walk(&continuation.root, &mut next_nodes);
    for node in next_nodes {
        if matches!(&node.node_type, RenderNodeType::Table(owner)
            if owner.para_index == Some(pi) && owner.control_index == Some(ci))
        {
            assert!(
                !node.children.iter().any(|child| {
                    matches!(&child.node_type, RenderNodeType::TableCell(cell) if cell.row == row)
                }),
                "완료 행의 어느 셀도 다음 쪽에 중복 표시하지 않는다"
            );
        }
    }
    let next_original = table
        .cells
        .iter()
        .find(|cell| cell.row == row + 1 && cell.col == 0)
        .expect("독립 Print에서 다음 쪽에 시작하는 원본 행");
    assert!(
        cell_text(row_cell(&continuation.root, pi, ci, row + 1))
            .starts_with(&compact(&next_original.paragraphs[0].text)),
        "다음 쪽은 다음 원본 행의 내용부터 이어진다"
    );
}

#[test]
fn issue_6035_original_reset_frames_keep_remaining_lines_inside_the_cell() {
    let core = load_doc();
    let (pi, ci, table) = source(&core);
    let fragments = fragments(&core, pi, ci);
    // 원본 문단 내부0→0 및 문단 간0→0 경계 모두 실제 남은 글줄을 셀 안에 둔다.
    for text in ["가. 제조공정 단계별로", "제4장 품질 보증"] {
        let row = source_row(table, text);
        let original = table
            .cells
            .iter()
            .find(|cell| cell.row == row && cell.col == 0)
            .unwrap();
        let (before, after) = split_pair(&fragments, row);
        assert_eq!(after.start, usize::from(row));
        assert_eq!(after.start_cut, before.end_cut);
        let opening = core
            .build_page_render_tree(before.page)
            .expect("경계 앞 조각");
        let continuation = core
            .build_page_render_tree(after.page)
            .expect("경계 뒤 조각");
        let first = row_cell(&opening.root, pi, ci, row);
        let next = row_cell(&continuation.root, pi, ci, row);
        let expected: String = original
            .paragraphs
            .iter()
            .map(|p| compact(&p.text))
            .collect();
        assert_eq!(
            cell_text(first) + &cell_text(next),
            expected,
            "원본 내용 완전 보존"
        );
        assert_text_inside(first);
        assert_text_inside(next);
        let declared = rhwp::renderer::hwpunit_to_px(original.height as i32, 96.0);
        assert!(
            first.bbox.height + next.bbox.height >= declared - 1e-8,
            "원본 행의 최소 물리 높이를 내용 컷의 줄 전진으로 대체하지 않는다"
        );
    }
}

#[test]
fn issue_6035_single_line_row_retains_its_independent_blank_band() {
    let core = load_doc();
    let (pi, ci, table) = source(&core);
    let fragments = fragments(&core, pi, ci);
    // 독립 Print41/42쪽: 문서관리 제목의 선언 높이에 속한 진짜 빈 밴드는 이어받는다.
    let row = source_row(table, "제29조 문서관리");
    let original = table
        .cells
        .iter()
        .find(|cell| cell.row == row && cell.col == 0)
        .unwrap();
    let (before, after) = split_pair(&fragments, row);
    assert_eq!(after.start, usize::from(row));
    assert_eq!(after.start_cut, before.end_cut);
    let opening = core
        .build_page_render_tree(before.page)
        .expect("빈 밴드 앞 조각");
    let continuation = core
        .build_page_render_tree(after.page)
        .expect("빈 밴드 뒤 조각");
    let heading = row_cell(&opening.root, pi, ci, row);
    let band = row_cell(&continuation.root, pi, ci, row);
    let following = row_cell(&continuation.root, pi, ci, row + 1);
    assert_eq!(cell_text(heading), compact(&original.paragraphs[0].text));
    assert!(cell_text(band).is_empty(), "제목 글줄 중복 금지");
    assert!(band.bbox.height > 0.0, "독립 물리 빈 밴드 보존");
    assert!(following.bbox.y >= band.bbox.y + band.bbox.height - 1e-8);
    let declared = rhwp::renderer::hwpunit_to_px(original.height as i32, 96.0);
    assert!(
        (heading.bbox.height + band.bbox.height - declared).abs() < 1e-8,
        "두 물리 조각은 원본 최소 행 높이를 한 번만 소비한다"
    );
    assert_text_inside(following);
}
