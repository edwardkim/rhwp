//! #2097 원본 행 소유와 수동으로 짧게 만든 마지막 셀의 물리 예산을 구분한다.
//! 독립 한컴 PDF의 쪽별 내용과 원본 본문 여백이 기대값의 근거다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::collections::BTreeSet;
use std::path::Path;

fn load(sample: &str) -> DocumentCore {
    let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(sample))
        .expect("커밋된 독립 기준 입력");
    DocumentCore::from_bytes(&bytes).expect("원본 로드")
}

fn table(node: &RenderNode, para: usize) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(t) if t.para_index == Some(para)) {
        return Some(node);
    }
    node.children.iter().find_map(|child| table(child, para))
}

fn text(node: &RenderNode) -> String {
    let mut result = match &node.node_type {
        RenderNodeType::TextRun(run) => run.text.clone(),
        _ => String::new(),
    };
    for child in &node.children {
        result.push_str(&text(child));
    }
    result
}

fn rows(node: &RenderNode) -> BTreeSet<u16> {
    node.children
        .iter()
        .filter_map(|child| match &child.node_type {
            RenderNodeType::TableCell(cell) => Some(cell.row),
            _ => None,
        })
        .collect()
}

fn assert_terminal_row_ownership(sample: &str, host: usize, has_head: bool) {
    let core = load(sample);
    assert_eq!(core.page_count(), 2, "독립 한컴2020 PDF2쪽");
    let first = core.build_page_render_tree(0).expect("첫쪽");
    let next = core.build_page_render_tree(1).expect("이어받기쪽");
    let first_table = table(&first.root, host).expect("첫 표 조각");
    let next_table = table(&next.root, host).expect("마지막 행 조각");
    assert_eq!(
        rows(first_table),
        BTreeSet::from([0, 1]),
        "PDF1쪽 큰 행/중간 행"
    );
    assert_eq!(
        rows(next_table),
        BTreeSet::from([2]),
        "PDF2쪽 마지막 행 단일 소유"
    );
    // 원본 용지 본문: 위7085HU, 높이70018HU. 표 가시 끝이 이 경계를 지켜야 한다.
    let body_bottom = (7085.0 + 70018.0) / 75.0;
    assert!(
        first_table.bbox.y + first_table.bbox.height <= body_bottom + 0.5,
        "마지막 행을 강제 소비해 첫쪽 paint가 본문 밖으로 넘치면 안 됨"
    );
    if has_head {
        assert_eq!(
            text(&first.root).matches("HEAD LINE BEFORE TABLE").count(),
            1
        );
        assert!(!text(&next.root).contains("HEAD LINE BEFORE TABLE"));
    }
    for (needle, page) in [
        ("BIG ROW", 0),
        ("MID ROW", 0),
        ("TAIL ROW EXPANDING", 1),
        ("AFTER TABLE", 1),
    ] {
        let all = [text(&first.root), text(&next.root)];
        assert_eq!(all[page].matches(needle).count(), 1, "{needle} 소유/누락");
        assert_eq!(all[1 - page].matches(needle).count(), 0, "{needle} 중복");
    }
    fn line<'a>(node: &'a RenderNode, needle: &str) -> Option<&'a RenderNode> {
        if matches!(node.node_type, RenderNodeType::TextLine(_)) && text(node).contains(needle) {
            return Some(node);
        }
        node.children.iter().find_map(|child| line(child, needle))
    }
    let after = line(&next.root, "AFTER TABLE").expect("뒤 문단의 실제 글줄");
    assert!(
        after.bbox.y + 0.5 >= next_table.bbox.y + next_table.bbox.height,
        "뒤 글줄은 실제 마지막 조각의 물리 끝 뒤에 배치"
    );
}

#[test]
fn valid_midpage_rowbreak_keeps_all_original_rows() {
    let core = load("samples/task2097/3080901_pii_ledger.hwp");
    assert_eq!(core.page_count(), 1, "독립 한컴2022/2020 실문서1쪽");
    let page = core.build_page_render_tree(0).expect("원본 첫쪽");
    let original = table(&page.root, 2).expect("원본 단일 표");
    assert_eq!(
        rows(original),
        (0..17).collect(),
        "17개 원본 행의 단일 쪽 소유"
    );
    assert!(
        text(&page.root).contains("공문 발송 시 명기 문구"),
        "실제 마지막 행 내용 보존"
    );
    // 원본 본문132.266667px와 높이876.853333px는 저장 용지/여백에서 정한다.
    assert!(
        original.bbox.y + original.bbox.height <= 1009.12 + 0.5,
        "실문서 통째 표도 원본 본문 경계 안에 배치"
    );
}

/// 원본 수동 입력을 한컴2020 HWP로 저장한 뒤 HWPX로 독립 재생성했다.
/// 셀 줄은1200→1000HU, vpos141→0으로 바뀌지만 원본을 덮어쓰지 않는다.
#[test]
fn independently_resaved_terminal_row_keeps_page_ownership() {
    let core = load("samples/issue2097/rowbreak-midpage-resaved-2020.hwpx");
    assert_eq!(core.page_count(), 2, "재생성한 한컴 입력도2쪽");
    for (page_index, expected_rows) in [(0, BTreeSet::from([0, 1])), (1, BTreeSet::from([2]))] {
        let page = core
            .build_page_render_tree(page_index)
            .expect("재생성 입력 쪽");
        let part = table(&page.root, 1).expect("재생성 입력 표 조각");
        assert_eq!(
            rows(part),
            expected_rows,
            "각 실제 행은 해당 쪽에 한 번만 출력"
        );
    }
}

#[test]
fn manually_short_terminal_cell_preserves_rows_and_physical_budget() {
    assert_terminal_row_ownership(
        "samples/task2097/rowbreak_midpage_declared_fits.hwpx",
        1,
        true,
    );
}

/// 쪽 시작도 가시 줄이 남은 물리 공간보다 크면 마지막 행을 이월한다.
#[test]
fn manually_short_fragment_start_cell_preserves_rows_and_physical_budget() {
    assert_terminal_row_ownership(
        "samples/task2105/rowbreak_table_declared_fits.hwpx",
        0,
        false,
    );
}

/// 원본과 구분한 독립 한컴 재저장 입력에서도 같은 실제 소유 계약을 확인한다.
#[test]
fn independently_resaved_fragment_start_keeps_rows_and_physical_budget() {
    assert_terminal_row_ownership(
        "samples/issue2097/rowbreak-fragment-start-resaved-2020.hwpx",
        0,
        false,
    );
}
