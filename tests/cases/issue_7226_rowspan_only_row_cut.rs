//! [#7226] 칸이 전부 걸침 칸인 행이 두 조각에 **이중 소유**돼 글자가 포개진다.
//!
//! # 무엇이 깨져 있었나
//!
//! `RowCut`(= `start_cut`/`end_cut`)은 그 행의 `row_span == 1` 칸만 col 순서로 색인한다
//! (`table_partial::single_row_cut_index`). 그래서 **칸이 전부 걸침 칸인 행**은 앞 조각이
//! 그 행의 물리 밴드를 얼마나 소비했든 컷에 적을 자리가 없다 — 이어받는 조각은 빈
//! `start_cut` 을 받아 같은 칸을 **첫 유닛부터 다시 칠한다.**
//!
//! `samples/task2287/1342000_edu_curriculum_map.hwp` 행 29 의 칸은 셋 다 걸침 칸이다
//! (`(29,2) (29,3) (29,4)` 모두 `row_span=2`).
//!
//! ```text
//!   32쪽(0-based 31) 조각  rows 24..30  endCut=[]     ← 행 29 를 빈 꼬리 밴드로 소비
//!                          (실제로는 `(29,3)` 의 14줄을 칠한다 — 렌더 트리 실측)
//!   33쪽(0-based 32) 조각  rows 29..39  startCut=[]   ← 행 29 를 처음부터 다시 그린다
//!   결과                   같은 칸의 글줄이 3.4px 간격으로 포개짐 — 33쪽 textOverlap 21건
//! ```
//!
//! # 고친 것
//!
//! 앞 조각이 남긴 잔여 밴드(`start_row_height_override`)는 **그 행에서 시작하는 걸침
//! 칸**에도 같은 뜻이다 — 소비 높이 = 선언 행 높이 − 잔여 밴드. 그 높이로 유닛 컷의
//! 시작(`su`)을 이어 준다(`table_partial::resumes_inside_own_start_row`). 컷 부기가
//! 성립하는 행(= `row_span == 1` 칸이 하나라도 있는 행)은 종전대로 `start_cut` 이
//! 소관하므로 건드리지 않는다.
//!
//! # 독립 기대값 — 한/글 2024 출력(415쪽, `oracle_page_count_baseline.tsv:546`)
//!
//! ```text
//!   오라클 32쪽  서울 칸 16줄 (안전교육 51 … 가정폭력 예방 및 방지를)
//!   오라클 33쪽  같은 칸 1줄  (위한 교육 1)            ← 행 **내부** 분할
//! ```
//!
//! 한/글은 이 행을 통째로 다음 쪽으로 밀지 않고 행 안에서 끊는다. 이 검사는 그 성질
//! (내부 분할 · 중복 없음 · 내용 보존)을 잠그되, 조각 경계의 정확한 줄 수(우리 14 대
//! 오라클 16)는 별개 축이라 잠그지 않는다.
//!
//! # 쪽을 늘리는 방향은 오라클과 어긋난다
//!
//! 이 문서의 쪽 정렬은 1..144쪽에서 오프셋 0 이다(`export-text` ↔ 정본 PDF 순차 정렬,
//! `#7226` 코멘트). 겹침을 "행을 다음 쪽으로 밀어" 없애는 접근(기각된 가설 4·6)은
//! 쪽수를 413 → 414 로 늘려 그 뒤를 전부 어긋나게 한다 — 아래 쪽수 검사가 그것을 막는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::diagnostics::layout_anomaly::{scan_page, AnomalyOptions};
use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const TARGET: &str = "samples/task2287/1342000_edu_curriculum_map.hwp";
/// 걸침 전용 행(모든 칸 `row_span=2`)의 시수 칸.
const ROW: u16 = 29;
const COL: u16 = 3;
/// 이 칸의 마지막 줄 — 정본도 마지막 조각에 이 한 줄을 둔다.
const LAST_LINE: &str = "위한 교육 1";
/// 앞 조각을 집는 닻. 같은 `(29,3)` 좌표의 칸은 이 문서의 다른 표에도 있으므로
/// 좌표가 아니라 **내용**으로 대상 조각을 고른다.
const FIRST_FRAGMENT_ANCHOR: &str = "감염병 및 약물 오남용";

fn core() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(TARGET);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("재현체 {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

fn line_text(node: &RenderNode) -> String {
    let mut s = String::new();
    if let RenderNodeType::TextRun(run) = &node.node_type {
        s.push_str(run.display_or_text());
    }
    for c in &node.children {
        s.push_str(&line_text(c));
    }
    s
}

/// `(row, col)` 칸이 이 쪽에서 칠한 글줄(빈 줄 제외, 위에서 아래 순).
fn cell_lines(root: &RenderNode, row: u16, col: u16) -> Vec<String> {
    fn walk(node: &RenderNode, row: u16, col: u16, inside: bool, out: &mut Vec<(f64, String)>) {
        if !node.visible || node.editor_only {
            return;
        }
        let mut inside = inside;
        if let RenderNodeType::TableCell(c) = &node.node_type {
            inside = c.row == row && c.col == col;
        }
        if inside && matches!(node.node_type, RenderNodeType::TextLine(_)) {
            let text = line_text(node);
            if !text.trim().is_empty() {
                out.push((node.bbox.y, text.trim().to_string()));
            }
        }
        for child in &node.children {
            walk(child, row, col, inside, out);
        }
    }
    let mut found = Vec::new();
    walk(root, row, col, false, &mut found);
    found.sort_by(|a, b| a.0.total_cmp(&b.0));
    found.into_iter().map(|(_, t)| t).collect()
}

/// 대상 걸침 전용 행의 두 조각 — `(앞 조각 쪽, 글줄)`, `(뒤 조각 쪽, 글줄)`.
///
/// 앞 조각은 닻으로 고르고, 뒤 조각은 **바로 다음 쪽**의 같은 칸이다. 수정 전에는
/// 그 다음 쪽이 같은 칸을 첫 유닛부터 다시 칠했다.
fn target_fragments(core: &DocumentCore) -> ((u32, Vec<String>), (u32, Vec<String>)) {
    let mut anchored: Vec<(u32, Vec<String>)> = Vec::new();
    for page in 0..core.page_count() {
        let Ok(tree) = core.build_page_render_tree(page) else {
            continue;
        };
        let lines = cell_lines(&tree.root, ROW, COL);
        if lines.iter().any(|l| l.contains(FIRST_FRAGMENT_ANCHOR)) {
            anchored.push((page, lines));
        }
    }
    assert!(
        !anchored.is_empty(),
        "앞 조각(`{FIRST_FRAGMENT_ANCHOR}`)을 못 찾았습니다 — 검사 대상이 비었습니다"
    );
    let first = anchored.remove(0);
    let next_page = first.0 + 1;
    let tree = core
        .build_page_render_tree(next_page)
        .expect("이어받는 쪽 렌더 트리");
    let next = (next_page, cell_lines(&tree.root, ROW, COL));
    (first, next)
}

/// 걸침 전용 행은 **행 안에서** 나뉘고 두 조각이 같은 줄을 나눠 갖지 않는다.
#[test]
fn the_rowspan_only_row_is_split_inside_and_never_repainted() {
    let core = core();
    let ((first_page, first), (next_page, next)) = target_fragments(&core);

    // 닻 줄은 한 쪽에만 있어야 한다 — 수정 전에는 31·32쪽 **양쪽**에 있었다(이중 소유).
    let anchored: Vec<u32> = (0..core.page_count())
        .filter(|page| {
            core.build_page_render_tree(*page)
                .map(|tree| {
                    cell_lines(&tree.root, ROW, COL)
                        .iter()
                        .any(|l| l.contains(FIRST_FRAGMENT_ANCHOR))
                })
                .unwrap_or(false)
        })
        .collect();
    assert_eq!(
        anchored.len(),
        1,
        "`{FIRST_FRAGMENT_ANCHOR}` 줄이 여러 쪽에 그려졌습니다(이중 소유) — 쪽 {anchored:?}"
    );

    // 행을 통째로 다음 쪽으로 민 것이 아니라 **행 안에서** 끊었다(정본 16/1 분할).
    assert!(
        first.len() >= 10 && !next.is_empty(),
        "행 내부 분할이어야 한다 — 앞 조각 {}줄 / 뒤 조각 {}줄",
        first.len(),
        next.len()
    );

    // 이중 소유(= 겹침의 실체)가 없다. 수정 전에는 뒤 조각이 앞 조각의 14줄을
    // 첫 유닛부터 다시 칠했다.
    let repainted: Vec<&String> = next.iter().filter(|line| first.contains(line)).collect();
    assert!(
        repainted.is_empty(),
        "뒤 조각이 앞 조각의 글줄을 다시 칠했습니다({}쪽→{}쪽): {repainted:?}",
        first_page,
        next_page
    );

    // 내용 보존 — 이어붙인 결과의 끝이 이 칸의 마지막 줄이다.
    let tail = next.last().expect("뒤 조각 글줄");
    assert!(
        tail.contains(LAST_LINE),
        "칸의 마지막 줄(`{LAST_LINE}`)이 보존되어야 한다 — 실제 마지막 줄 {tail:?}"
    );
}

/// 이어받는 쪽에 글자 겹침이 없다 — 이 이슈가 보고한 신호 그 자체.
#[test]
fn the_continuation_page_has_no_text_overlap() {
    let core = core();
    let (_, (page, _)) = target_fragments(&core);
    let tree = core.build_page_render_tree(page).expect("렌더 트리");
    let anomalies = scan_page(
        page,
        &tree.root,
        core.page_count(),
        &AnomalyOptions::default(),
    );
    let overlaps: Vec<String> = anomalies
        .text_overlap
        .iter()
        .map(|o| format!("{} x {} ({:.1}px²)", o.path_a, o.path_b, o.overlap_area()))
        .collect();
    assert!(
        overlaps.is_empty(),
        "{page}쪽(0-based)에 글자 겹침이 남았습니다 {}건: {overlaps:?}",
        overlaps.len()
    );
}

/// 겹침을 쪽을 늘려 없애면 안 된다 — 이 구간의 쪽 번호는 이미 정본과 같다.
#[test]
fn the_fix_does_not_add_a_page() {
    assert_eq!(
        core().page_count(),
        413,
        "쪽수가 변했습니다 — 이 문서의 1..144쪽은 정본(415쪽)과 오프셋 0 으로 정렬한다"
    );
}

/// Presence in the tree is not visibility: the entire final line must fit
/// inside its owning physical cell and the body, after reservation and paint.
#[test]
fn the_last_owned_line_is_inside_the_reserved_cell_and_body() {
    fn collect(
        node: &RenderNode,
        cell: Option<f64>,
        body: Option<f64>,
        out: &mut Vec<(f64, f64, f64)>,
    ) {
        let cell = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
            Some(node.bbox.y + node.bbox.height)
        } else {
            cell
        };
        let body = if matches!(node.node_type, RenderNodeType::Body { .. }) {
            Some(node.bbox.y + node.bbox.height)
        } else {
            body
        };
        if matches!(node.node_type, RenderNodeType::Table(_)) && line_text(node).contains(LAST_LINE)
        {
            let target = node.children.iter().find(|child| {
                matches!(&child.node_type, RenderNodeType::TableCell(c) if c.row == ROW && c.col == COL)
                    && line_text(child).contains(LAST_LINE)
            });
            if let Some(target) = target {
                let target_bottom = target.bbox.y + target.bbox.height;
                let next_top = node
                    .children
                    .iter()
                    .filter_map(|child| match &child.node_type {
                        RenderNodeType::TableCell(c) if c.col == COL && c.row >= ROW + 2 => {
                            Some(child.bbox.y)
                        }
                        _ => None,
                    })
                    .reduce(f64::min)
                    .expect("following row stays in the fragment");
                assert!(
                    target_bottom <= next_top + 0.5,
                    "reserved tail overlaps next row"
                );
                assert!(
                    node.bbox.y + node.bbox.height <= body.expect("body") + 0.5,
                    "physical table exceeds reserved body"
                );
            }
        }
        if matches!(node.node_type, RenderNodeType::TextLine(_))
            && line_text(node).contains(LAST_LINE)
        {
            out.push((
                node.bbox.y + node.bbox.height,
                cell.expect("owning cell"),
                body.expect("body"),
            ));
        }
        for child in &node.children {
            collect(child, cell, body, out);
        }
    }
    let core = core();
    let (_, (page, _)) = target_fragments(&core);
    let tree = core.build_page_render_tree(page).unwrap();
    let mut found = Vec::new();
    collect(&tree.root, None, None, &mut found);
    assert_eq!(found.len(), 1, "one terminal line must be owned: {found:?}");
    for (bottom, cell, body) in found {
        assert!(
            bottom <= cell + 0.5,
            "last line bottom {bottom} exceeds physical cell {cell}"
        );
        assert!(
            bottom <= body + 0.5,
            "last line bottom {bottom} exceeds body {body}"
        );
    }
}

/// Keep the original table IR and vary only the body's physical budget. This
/// is a contract test, not a separately Hancom-saved fixture or PDF oracle.
#[test]
fn same_row_reservation_survives_neighboring_page_budgets() {
    use rhwp::model::control::Control;
    let source = core();
    let (section, paragraph) = source.document().sections.iter().find_map(|section| {
        section.paragraphs.iter().find(|para| para.controls.iter().any(|control| {
            matches!(control, Control::Table(table) if table.cells.iter().any(|cell|
                cell.row == ROW && cell.col == COL && cell.paragraphs.iter().any(|p| p.text.contains(FIRST_FRAGMENT_ANCHOR))))
        })).map(|para| (section.clone(), para.clone()))
    }).expect("original rowspan-only table");
    for delta in [-75i32, 0, 75] {
        let mut document = source.document().clone();
        document.sections = vec![section.clone()];
        document.sections[0].paragraphs = vec![paragraph.clone()];
        let page = &mut document.sections[0].section_def.page_def;
        page.margin_bottom = (page.margin_bottom as i32 + delta) as u32;
        let mut candidate = DocumentCore::new_empty();
        candidate.set_document(document);
        fn check(node: &RenderNode, cell: Option<f64>, body: Option<f64>, count: &mut usize) {
            let cell = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
                Some(node.bbox.y + node.bbox.height)
            } else {
                cell
            };
            let body = if matches!(node.node_type, RenderNodeType::Body { .. }) {
                Some(node.bbox.y + node.bbox.height)
            } else {
                body
            };
            if matches!(node.node_type, RenderNodeType::Table(_))
                && line_text(node).contains(LAST_LINE)
            {
                assert!(
                    node.bbox.y + node.bbox.height <= body.expect("body") + 0.5,
                    "table must fit the physical page budget: table={:?}, body={body:?}",
                    node.bbox
                );
            }
            if matches!(node.node_type, RenderNodeType::TextLine(_))
                && line_text(node).contains(LAST_LINE)
            {
                *count += 1;
                assert!(
                    node.bbox.y + node.bbox.height <= cell.expect("cell") + 0.5,
                    "owned tail must fit its reserved cell: line={:?}, cell={cell:?}",
                    node.bbox
                );
            }
            for child in &node.children {
                check(child, cell, body, count);
            }
        }
        let mut count = 0;
        for page in 0..candidate.page_count() {
            check(
                &candidate.build_page_render_tree(page).unwrap().root,
                None,
                None,
                &mut count,
            );
        }
        assert_eq!(
            count, 1,
            "last line lost or duplicated for budget delta {delta}"
        );
    }
}
