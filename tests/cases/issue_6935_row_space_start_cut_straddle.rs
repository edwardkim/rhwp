//! [Issue #6935] 한 조각의 시작 컷과 끝 컷이 서로 다른 인덱스 공간에서 올 수 있다.
//!
//! ## 형상 (18179365 2쪽 실측)
//!
//! ```text
//!   p1  rows 0..5    startCut []         endCut [5,9]         ← 행 공간(행 4 의 row_span==1 칸 둘)
//!   p2  rows 4..12   startCut [5,9]      endCut [4,16,3,3]    ← 시작=행 공간 / 끝=블록 공간
//!   p3  rows 9..13   startCut [4,16,3,3]
//! ```
//!
//! `(2,3)` 칸이 5행(2..7)을 걸친다. 행 공간 컷은 **그 행의 `row_span == 1` 칸만** 담으므로
//! 걸친 칸에는 적을 자리가 없다.
//!
//! ## 종전 결함
//!
//! `PageItem::PartialTable` 의 `is_block_split` 하나가 두 사실을 OR 로 합쳤다.
//!
//! ```rust,ignore
//! is_block_split: split_block_start.is_some() || start_cut_is_block,
//! //              ^^^ 이번 조각의 끝 컷        ^^^ 들어온 시작 컷
//! ```
//!
//! 렌더러는 그 하나로 양쪽을 다 읽어, 시작이 행 공간인데 끝이 블록 공간인 조각에서 **시작
//! 쪽까지 블록 서수로** 조회했다. 걸친 칸의 블록 서수는 행 공간 컷 벡터 범위 밖이라
//! `su = 0` 으로 떨어지고, 동시에 그 칸이 "컷 소관"으로 잡혀 `#1748` 의 높이 기반
//! 구제(`rowbreak_straddle_cut_units`)까지 막혔다 — 앞 조각이 그린 내용을 **처음부터 다시**
//! 그린다(2쪽 렌더 글자 1,412 → 정본 대비 +434, 글자 겹침 47건).
//!
//! ## 이 검사
//!
//! 같은 빌드 안에서 `start_cut_is_block` 만 뒤집어 두 동작을 직접 대조한다.
//! `true` 는 종전의 합쳐진 값이고, `false` 가 이 조각의 사실이다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::table::{Cell, Table, TablePageBreak};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const CELL_W: u32 = 20_000;
const ROW_H: u32 = 4_000;
/// 걸친 칸이 덮는 행 수 — 조각 경계가 이 안쪽에 떨어진다.
const SPAN: u16 = 4;
const ROWS: u16 = 6;

fn para(text: &str) -> Paragraph {
    Paragraph {
        text: text.to_string(),
        ..Default::default()
    }
}

/// `(1,1)` 이 4행(1..5)을 걸치는 6행×2열 표.
fn straddle_table() -> Table {
    let mut cells = Vec::new();
    for row in 0..ROWS {
        for col in 0..2u16 {
            if col == 1 && (1..1 + SPAN).contains(&row) {
                continue; // 걸침 칸이 덮는다
            }
            cells.push(Cell {
                row,
                col,
                row_span: 1,
                col_span: 1,
                width: CELL_W,
                height: ROW_H,
                paragraphs: vec![para(&format!("r{row}c{col}"))],
                ..Default::default()
            });
        }
    }
    cells.push(Cell {
        row: 1,
        col: 1,
        row_span: SPAN,
        col_span: 1,
        width: CELL_W,
        height: ROW_H * u32::from(SPAN),
        // 조각을 넘기려면 유닛이 여럿이어야 한다.
        paragraphs: (0..12).map(|i| para(&format!("span{i:02}"))).collect(),
        ..Default::default()
    });
    Table {
        row_count: ROWS,
        col_count: 2,
        cells,
        page_break: TablePageBreak::RowBreak,
        ..Default::default()
    }
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

/// 걸친 칸이 이 조각에서 그리는 첫 글자.
fn straddling_cell_first_text(root: &RenderNode) -> Option<String> {
    let mut nodes = Vec::new();
    walk(root, &mut nodes);
    let cell = nodes.iter().find(|node| match &node.node_type {
        RenderNodeType::TableCell(cell) => cell.row == 1 && cell.col == 1 && cell.row_span == SPAN,
        _ => false,
    })?;
    let mut inner = Vec::new();
    walk(cell, &mut inner);
    inner.iter().find_map(|node| match &node.node_type {
        RenderNodeType::TextRun(run) if !run.text.trim().is_empty() => Some(run.text.clone()),
        _ => None,
    })
}

/// 시작 컷이 행 공간인 조각을 렌더해 걸친 칸의 첫 글자를 돌려준다.
fn first_text_with(start_cut_is_block: bool) -> Option<String> {
    use rhwp::model::page::{ColumnDef, PageDef};
    use rhwp::renderer::{
        composer::compose_paragraph,
        height_measurer::HeightMeasurer,
        layout::LayoutEngine,
        pagination::{PageItem, Paginator},
        style_resolver::resolve_styles,
    };

    let doc_info = rhwp::model::document::DocInfo::default();
    let paragraphs = vec![Paragraph {
        controls: vec![Control::Table(Box::new(straddle_table()))],
        ..Default::default()
    }];
    let styles = resolve_styles(&doc_info, 96.0);
    let composed = paragraphs.iter().map(compose_paragraph).collect::<Vec<_>>();
    let measured = HeightMeasurer::new(96.0).measure_section(&paragraphs, &composed, &styles, None);
    let mut pages = Paginator::new(96.0).paginate_with_measured(
        &paragraphs,
        &measured,
        &PageDef::default(),
        &ColumnDef::default(),
        0,
        &styles.para_styles,
    );

    let page = &mut pages.pages[0];
    // 18179365 2쪽의 형상: 시작은 행 3 의 행 공간 컷, 끝은 블록 공간 컷.
    page.column_contents[0].items = vec![PageItem::PartialTable {
        para_index: 0,
        control_index: 0,
        start_row: 3,
        end_row: ROWS as usize,
        is_continuation: true,
        start_cut: vec![1],
        end_cut: vec![1, 1, 1],
        is_block_split: true,
        start_cut_is_block,
        row_cursor_is_nested: false,
        end_row_height_override: None,
        start_row_height_override: None,
    }];

    let tree = LayoutEngine::new(96.0).build_render_tree(
        page,
        &paragraphs,
        &[],
        &[],
        &composed,
        &styles,
        &Default::default(),
        &[],
        None,
        &measured.tables,
        None,
        0,
        &[],
    );
    straddling_cell_first_text(&tree.root)
}

/// 시작 컷이 행 공간이면 걸친 칸은 **이어받는다** — 앞 조각을 다시 그리지 않는다.
///
/// 같은 빌드에서 `start_cut_is_block` 만 뒤집어 종전 동작(두 사실을 합친 값)과 대조한다.
#[test]
fn a_row_space_start_cut_does_not_restart_a_straddling_cell() {
    let conflated = first_text_with(true);
    let separated = first_text_with(false);

    let conflated = conflated.expect("종전 동작에서도 걸친 칸은 그려진다");
    let separated = separated.expect("수정 동작에서도 걸친 칸은 그려진다");

    assert_eq!(
        conflated, "span00",
        "시작 컷까지 블록 서수로 읽으면 걸친 칸이 첫 유닛부터 다시 그려진다 \
         — 이것이 #6935 의 중복 조판이다"
    );
    assert_ne!(
        separated, "span00",
        "시작 컷이 행 공간이면 걸친 칸에는 적을 자리가 없다. 컷 소관으로 잡아 \
         su = 0 으로 떨어뜨리지 말고 높이 기반 구제가 이어받아야 한다"
    );
}

/// 한컴 engine 2020의 같은 원본 3쪽에는 비공백 글자가 총 1958개다.
/// 실제 로드→pagination→paint의 글자 총량을 검사한다. 글자별 누락·추가는 별도 PDF 대조로 확인한다.
#[test]
fn real_collision_document_preserves_text_across_fragments() {
    let core = collision_core();
    assert_eq!(core.page_count(), 3, "한컴 기준 PDF와 같은 3쪽");
    let mut count = 0;
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("실물 페이지");
        let mut nodes = Vec::new();
        walk(&tree.root, &mut nodes);
        for node in nodes {
            if let RenderNodeType::TextRun(run) = &node.node_type {
                count += run.text.chars().filter(|c| !c.is_whitespace()).count();
            }
        }
    }
    assert_eq!(count, 1958, "기준 PDF의 문서 전체 글자 총량 보존");
}

fn collision_core() -> rhwp::document_core::DocumentCore {
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/issue6935/18179365_high_voltage_collision.hwp");
    rhwp::document_core::DocumentCore::from_bytes(&std::fs::read(file).expect("실물 HWP fixture"))
        .expect("실물 HWP 로드")
}

/// 기준 PDF의 세 쪽 모두 본문과 표가 용지 안에 있다. 작은 컷 불변식의
/// 통과만으로 표/글자 bottom의 130px 넘침을 놓치지 않는다.
#[test]
fn real_collision_document_fragment_stays_on_paper() {
    let core = collision_core();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("실물 페이지");
        let paper_bottom = tree.root.bbox.y + tree.root.bbox.height;
        let mut nodes = Vec::new();
        walk(&tree.root, &mut nodes);
        for node in nodes {
            if matches!(
                node.node_type,
                RenderNodeType::Table(_) | RenderNodeType::TextRun(_)
            ) {
                let bottom = node.bbox.y + node.bbox.height;
                assert!(
                    bottom <= paper_bottom + 1.0,
                    "page {} {:?}: bottom {bottom} > paper {paper_bottom}",
                    page + 1,
                    node.node_type
                );
            }
        }
    }
}
