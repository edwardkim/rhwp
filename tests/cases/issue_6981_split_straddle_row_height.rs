//! [#6981] 조각 경계가 rowspan 블록 안쪽이면 이어받는 조각의 걸친 셀이 제 높이를 받는다.
//!
//! # 무엇이 깨져 있었나
//!
//! 조각 경계가 rowspan 블록 안쪽에 떨어지면, 이어받는 조각의 걸친 셀은 `#1748` 의
//! 높이-컷으로 **남은 유닛 전부**를 받는다. 그런데 그 셀이 덮는 행들의 높이는 같은 행의
//! `row_span==1` 셀만 보고 정해진다. 어긋난 만큼 clip 이 글자를 지운다 — 렌더 트리에는
//! 정상 좌표로 있는데 화면에 한 자도 안 나간다.
//!
//! `samples/task2287/1342000_edu_curriculum_map.hwp` 377쪽, 셀 `(62,8) row_span=2`:
//!
//! ```text
//!   저장: 셀 3882 HU = 51.76px · 여백 141 HU = 1.88px · 문단 3개 vpos 0·1300·2600 HU
//!         내용 바닥 48.00px + 1.88 = 49.88 = 51.76 − 1.88   ← 온전하면 딱 맞는다
//!   그리드 행 62 = 30.65px · 행 63 = 21.11px
//!         앞 조각(30.65px)에 문단 1개, 이어받는 조각(21.11px)에 문단 2개(32.55px 필요)
//!         → `• 선언문 작성` 이 11.4px 밖 → 사라짐
//! ```
//!
//! # 양쪽 잠금
//!
//! 늘리면 안 되는 두 형상을 실문서로 함께 잠근다. 조판·렌더가 같은 출처
//! (`straddle_continuation_demand`)를 쓰지 않으면 아래 둘이 곧바로 깨진다.
//!
//! - `samples/issue6803/1376496-…​.hwp` — 셀 `(8,0) row_span=6`(문단 67개)은 남은
//!   내용이 이 조각에 다 안 들어간다. 컷이 소관이라 늘리면 안 된다(늘리면 남은
//!   1,140.8px 을 892.9px 행합에 요구해 표가 쪽 밖으로 나갔다).
//! - `samples/issue6795/1341000-…​.hwp` — 걸친 셀의 마지막 행이 조각 끝에서 **다시
//!   잘리는** 형상. 조판만 앞서 예약하면 셀 상자가 내용보다 작아진다(실측 2줄).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// 자기 칸 상자 아래로 내려간 글줄 — 이 결함이 화면에서 글자를 지우는 그 모양이다.
fn lines_outside_their_cell(node: &RenderNode, cell_bottom: Option<f64>, out: &mut Vec<String>) {
    if !node.visible || node.editor_only {
        return;
    }
    let mut bottom = cell_bottom;
    if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        bottom = Some(node.bbox.y + node.bbox.height);
    }
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        if let Some(limit) = bottom {
            let text = line_text(node);
            if !text.trim().is_empty() && node.bbox.y + node.bbox.height > limit + 0.5 {
                out.push(text);
            }
        }
    }
    for child in &node.children {
        lines_outside_their_cell(child, bottom, out);
    }
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

fn escaped_lines(path: &str, page: u32) -> Vec<String> {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("재현체 {path}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("로드 {path}: {e}"));
    let tree = core
        .build_page_render_tree(page)
        .unwrap_or_else(|e| panic!("렌더 {path} p{page}: {e}"));
    let mut out = Vec::new();
    lines_outside_their_cell(&tree.root, None, &mut out);
    out
}

fn escaped_lines_all_pages(path: &str) -> Vec<String> {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("재현체 {path}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("로드 {path}: {e}"));
    let mut out = Vec::new();
    for page in 0..core.page_count() {
        if let Ok(tree) = core.build_page_render_tree(page) {
            lines_outside_their_cell(&tree.root, None, &mut out);
        }
    }
    out
}

const TARGET: &str = "samples/task2287/1342000_edu_curriculum_map.hwp";
const NEEDLE: &str = "선언문 작성";

/// 이 수정은 쪽수를 바꾼다(413 → 414). 쪽 번호에 묶지 않고 전 쪽을 훑는다.
///
/// `선언문 작성` 은 이 문서에 **여러 번** 나온다(정본 273·287·378쪽). 첫 하나만 보면
/// 멀쩡한 쪽을 집어 판정이 뒤집히므로 **모든 출현**을 본다.
fn all_occurrences(path: &str, needle: &str) -> Vec<(u32, f64, f64)> {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("재현체 {path}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("로드 {path}: {e}"));
    let mut out = Vec::new();
    for page in 0..core.page_count() {
        let Ok(tree) = core.build_page_render_tree(page) else {
            continue;
        };
        let mut found = Vec::new();
        locate(&tree.root, None, needle, &mut found);
        for (line_bottom, cell_bottom) in found {
            out.push((page, line_bottom, cell_bottom));
        }
    }
    out
}

fn locate(node: &RenderNode, cell_bottom: Option<f64>, needle: &str, out: &mut Vec<(f64, f64)>) {
    if !node.visible || node.editor_only {
        return;
    }
    let mut bottom = cell_bottom;
    if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        bottom = Some(node.bbox.y + node.bbox.height);
    }
    if matches!(node.node_type, RenderNodeType::TextLine(_)) && line_text(node).contains(needle) {
        if let Some(limit) = bottom {
            out.push((node.bbox.y + node.bbox.height, limit));
            return;
        }
    }
    for child in &node.children {
        locate(child, bottom, needle, out);
    }
}

#[test]
fn no_occurrence_of_the_line_escapes_its_cell() {
    let found = all_occurrences(TARGET, NEEDLE);
    assert!(
        found.len() >= 3,
        "`{NEEDLE}` 출현이 {}건뿐입니다 — 검사 대상이 비었습니다",
        found.len()
    );
    let escaped: Vec<_> = found
        .iter()
        .filter(|(_, line_bottom, cell_bottom)| *line_bottom > *cell_bottom + 0.5)
        .collect();
    assert!(
        escaped.is_empty(),
        "`{NEEDLE}` 줄이 자기 칸 밖입니다(clip 이 지운다) — (쪽0based, 줄바닥, 칸바닥) {escaped:?}"
    );
}

/// 그 쪽 전체에도 칸 밖 글줄이 없어야 한다 — 한 줄만 밀어 넣고 다른 줄을 밀어내면 안 된다.
#[test]
fn the_pages_that_carry_it_have_no_escaped_line() {
    for (page, _, _) in all_occurrences(TARGET, NEEDLE) {
        let escaped = escaped_lines(TARGET, page);
        assert!(
            escaped.is_empty(),
            "{page}쪽(0-based)에 칸 밖으로 나간 글줄이 있습니다: {escaped:?}"
        );
    }
}

/// 반례 ① — 남은 내용이 조각에 다 안 들어가는 거대 걸침 셀은 늘리지 않는다.
#[test]
fn giant_straddle_cell_that_must_be_cut_is_not_grown() {
    let path = "samples/issue6803/1376496-neighborhood-facility-land-table.hwp";
    let escaped = escaped_lines_all_pages(path);
    assert!(
        escaped.is_empty(),
        "{path} 에 칸 밖 글줄이 생겼습니다(늘리면 안 되는 셀을 늘렸습니다): {escaped:?}"
    );
}

/// 반례 ② — 걸친 셀의 마지막 행이 조각 끝에서 **다시 잘리는** 형상.
///
/// 이 문서에는 이 변경과 무관한 칸 밖 글줄이 이미 3줄 있다(`성(정량, `·`P/F)`·`그램 ` —
/// 엔진의 `LAYOUT_OVERFLOW_CELL` 원장에는 안 잡히는 축이라 `overflow_cell_baseline`
/// 에도 항목이 없다). 지금 보정은 이 문서에서 **한 번도 발동하지 않으므로**(진단
/// `RHWP_DIAG_6981` 0건) 그 3줄은 건드리지 않는다.
///
/// 조판이 렌더보다 앞서 예약하던 중간 상태에서는 여기가 5줄이 됐다 — 그 비대칭을 잡는다.
const ISSUE_6795_PREEXISTING_ESCAPES: usize = 3;

#[test]
fn straddle_cell_whose_last_row_is_cut_again_is_not_grown() {
    let path = "samples/issue6795/1341000-201100013-cyber-university-application.hwp";
    let escaped = escaped_lines_all_pages(path);
    assert!(
        escaped.len() <= ISSUE_6795_PREEXISTING_ESCAPES,
        "{path} 에 칸 밖 글줄이 늘었습니다(조판만 앞서 예약했습니다): {escaped:?}"
    );
}

/// 원본의 section 28 / 표 문단을 보존한 내부 IR 계약 입력이다.
/// 별도로 한컴에서 저장하거나 PDF로 변환한 축소 문서가 아니다.
fn isolated_curriculum_table() -> rhwp::model::document::Document {
    let bytes = std::fs::read(TARGET).expect("committed curriculum fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse curriculum fixture");
    let mut doc = core.document().clone();
    let section = doc.sections[28].clone();
    let paragraph = section.paragraphs[2].clone();
    assert!(
        paragraph.controls.iter().any(|control| {
            matches!(control, rhwp::model::control::Control::Table(table)
            if table.row_count == 83 && table.cells.iter().any(|cell|
                cell.row == 62 && cell.col == 8 && cell.row_span == 2
                && cell.paragraphs.iter().any(|para| para.text.contains("선언문"))))
        }),
        "fixture source table contract changed"
    );
    doc.sections = vec![section];
    doc.sections[0].paragraphs = vec![paragraph];
    doc
}

fn assert_tables_inside_body(node: &RenderNode, body_bottom: Option<f64>) {
    if !node.visible || node.editor_only {
        return;
    }
    let body_bottom = if matches!(node.node_type, RenderNodeType::Body { .. }) {
        Some(node.bbox.y + node.bbox.height)
    } else {
        body_bottom
    };
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        if let Some(bottom) = body_bottom {
            assert!(
                node.bbox.y + node.bbox.height <= bottom + 0.5,
                "table bottom {} exceeds body bottom {bottom}",
                node.bbox.y + node.bbox.height
            );
        }
    }
    for child in &node.children {
        assert_tables_inside_body(child, body_bottom);
    }
}

fn assert_continuation_document(doc: rhwp::model::document::Document) -> u32 {
    let mut core = DocumentCore::new_empty();
    core.set_document(doc);
    let mut occurrences = Vec::new();
    for page in 0..core.page_count() {
        let tree = core
            .build_page_render_tree(page)
            .expect("render every fragment");
        assert_tables_inside_body(&tree.root, None);
        locate(&tree.root, None, NEEDLE, &mut occurrences);
    }
    assert_eq!(occurrences.len(), 1, "continuation text lost or duplicated");
    assert!(
        occurrences.iter().all(|(line, cell)| *line <= *cell + 0.5),
        "continuation line must remain inside its cell: {occurrences:?}"
    );
    core.page_count()
}

/// PR 원안은 시작 컷에서 이미 소비한 유닛을 재예약해 p3 본문을 4.213px 넘었다.
#[test]
fn consumed_start_cut_does_not_grow_the_fragment_past_the_body() {
    assert_eq!(assert_continuation_document(isolated_curriculum_table()), 4);
}

/// 본문 예산을 1px 간격으로 ±20px 바꾼다. 행·글자·rowspan은 그대로 유지하며,
/// 여러 걸침 셀이 끝나는 조각의 높이 누적과 실제 끝 컷/이월을 함께 검사한다.
#[test]
fn continuation_height_respects_varying_page_budgets() {
    let source = isolated_curriculum_table();
    let mut counts = std::collections::BTreeSet::new();
    for delta_hu in (-1500i32..=1500).step_by(75) {
        let mut doc = source.clone();
        let page = &mut doc.sections[0].section_def.page_def;
        page.margin_bottom = (page.margin_bottom as i32 + delta_hu) as u32;
        counts.insert(assert_continuation_document(doc));
    }
    assert!(
        counts.len() > 1,
        "budget variants must exercise a page break transition"
    );
}

/// 한컴 2022 기준 PDF p83에는 전남부터 제주까지의 조례가 함께 있다.
/// 앞 쪽에서 소비한 빈 행 밴드를 내용 컷만으로 재계산하면 이 행들이 밀려난다.
#[test]
fn physical_blank_band_is_not_reserved_again_on_continuation() {
    let bytes = std::fs::read(TARGET).expect("committed curriculum fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse curriculum fixture");
    let tree = core.build_page_render_tree(82).expect("render page 83");
    let text = line_text(&tree.root);
    assert!(
        text.contains("전라남도교육청"),
        "Jeonnam ordinance moved out of page 83"
    );
    assert!(
        text.contains("제주특별자치도교육청"),
        "Jeju ordinance moved out of page 83"
    );
    assert_tables_inside_body(&tree.root, None);
}

/// 최종 컷 뒤에 내용 없는 페이지를 할당하면 378쪽에는 쪽 번호만 남는다.
/// 마지막 표의 글자는 앞 쪽에 남고, 다음 구역 본문은 빈 쪽 없이 이어져야 한다.
#[test]
fn completed_terminal_cut_does_not_allocate_an_empty_page() {
    let bytes = std::fs::read(TARGET).expect("committed curriculum fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse curriculum fixture");
    let tree = core
        .build_page_render_tree(377)
        .expect("render successor page");
    fn body_has_text(node: &RenderNode) -> bool {
        if matches!(node.node_type, RenderNodeType::Body { .. }) {
            return !line_text(node).trim().is_empty();
        }
        node.children.iter().any(body_has_text)
    }
    assert!(
        body_has_text(&tree.root),
        "completed row cut left an empty successor page"
    );
    assert!(
        line_text(&tree.root).contains("노동인권"),
        "next section must follow the completed table"
    );
}
