//! Issue #4138: 셀 나누기 후 stale line_segs·vpos 사다리 붕괴로 인한
//! glyph truncation + 잘못된 페이지네이션 회귀 가드.
//!
//! 재현 문서: `samples/issue1949_giant_cell_nested_tables_perf.hwp`
//! (3×1 RowBreak 표가 PartialTable continuation 으로 115쪽에 걸침).
//!
//! 결함 2단:
//! 1. `Table::split_cell_into` 가 셀 폭을 절반으로 바꾸면서 저장 line_segs 를
//!    그대로 둔다 → 렌더러가 옛 폭(44508) 기준 줄을 새 셀(22395) 클립 경계에
//!    그대로 그려 glyph 가 잘린다. (수정 전 실측: 4,321 seg 전부 stale)
//! 2. per-para reflow 만 배선하면 각 문단의 vpos 원점이 보존된 채 줄 수만
//!    늘어나 사다리가 역행하고, 컷 기계가 이를 RowBreak hard break 로 오판해
//!    페이지를 과소 적재한다. (실측: 118→222쪽)
//!
//! 정정: `reflow_stale_cells_after_split`(table_ops.rs) — stale 셀 재래핑 후
//! `rebuild_table_cell_vpos_ladder_native` 로 사다리를 단조 재구축한다.

use std::fs;
use std::path::Path;

use rhwp::model::control::Control;
use rhwp::model::table::Table;

const SAMPLE: &str = "samples/issue1949_giant_cell_nested_tables_perf.hwp";
/// 대상 표: (section 0, 문단 0, control 2). 분할 대상 셀: (row 2, col 0).
const CTRL_IDX: usize = 2;
const TARGET_ROW: u16 = 2;

fn load() -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).expect("read sample");
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse")
}

fn target_table(doc: &rhwp::wasm_api::HwpDocument) -> &Table {
    match &doc.document().sections[0].paragraphs[0].controls[CTRL_IDX] {
        Control::Table(t) => t,
        other => panic!("controls[{CTRL_IDX}] 이 표가 아님: {other:?}"),
    }
}

/// 저장 seg 폭이 셀 폭을 넘는(=옛 폭 기준 stale) 문단 수.
///
/// 텍스트 없이 컨트롤만 호스팅하는 문단은 제외한다: `reflow_line_segs` 는 이
/// 문단에서 원본 seg 폭 template 을 보존하고, 중첩 표 자체를 분할 시 리스케일할지는
/// 한컴 오라클 미확인(#4138 미해결 항목 (a))이라 이 가드의 계약 밖이다.
fn stale_text_paras(table: &Table) -> usize {
    table
        .cells
        .iter()
        .flat_map(|cell| {
            cell.paragraphs
                .iter()
                .filter(|p| !(p.text.is_empty() && !p.controls.is_empty()))
                .map(move |p| (cell.width, p))
        })
        .filter(|(cell_width, p)| {
            p.line_segs
                .iter()
                .any(|ls| (ls.segment_width as i64) > (*cell_width as i64))
        })
        .count()
}

/// 분할 대상 행 셀들의 vpos 사다리 역행 수 (0 이어야 단조).
fn ladder_regressions(table: &Table, row: u16) -> usize {
    let mut regressions = 0usize;
    for cell in table.cells.iter().filter(|c| c.row == row) {
        let mut prev_end: i64 = i64::MIN;
        for p in &cell.paragraphs {
            if let (Some(first), Some(last)) = (p.line_segs.first(), p.line_segs.last()) {
                if (first.vertical_pos as i64) < prev_end {
                    regressions += 1;
                }
                prev_end = last.vertical_pos as i64;
            }
        }
    }
    regressions
}

/// 셀 나누기(1×2) 뒤: stale seg 0, 사다리 단조, 페이지 흐름 복원.
#[test]
fn split_cell_into_reflows_stale_segs_and_rebuilds_ladder() {
    let mut doc = load();
    assert_eq!(doc.page_count(), 115, "issue1949 기준 쪽수 핀");

    doc.split_table_cell_into_native(0, 0, CTRL_IDX, TARGET_ROW, 0, 1, 2, false, false)
        .expect("split_cell_into 1×2");

    let table = target_table(&doc);
    let stale = stale_text_paras(table);
    assert_eq!(
        stale, 0,
        "#4138 회귀: 분할 뒤 옛 폭 기준 stale line_segs 문단이 {stale}개 남음 \
         (수정 원복 시 실측 2,498문단/4,321seg). glyph 가 셀 클립 경계에서 잘린다."
    );

    let regressions = ladder_regressions(table, TARGET_ROW);
    assert_eq!(
        regressions, 0,
        "#4138 회귀: 재래핑된 셀의 vpos 사다리가 {regressions}회 역행. \
         컷 기계가 hard break 로 오판해 페이지를 과소 적재한다."
    );

    // 실측 거동 핀 (한컴 정답 쪽수는 미확인 — #4138 미해결 항목).
    // 수정 원복 시 118쪽(stale 줄로 과소 계산), 사다리 재구축 생략 시 222쪽(과소 적재).
    let pages = doc.page_count();
    assert_eq!(
        pages, 195,
        "#4138 회귀: 분할 뒤 쪽수 {pages} (기대 195). 118 이면 reflow 누락, \
         222 이면 vpos 사다리 재구축 누락."
    );
}

/// 범위 분할 경로(`split_table_cells_in_range_native`)도 같은 계약을 따른다.
#[test]
fn split_cells_in_range_reflows_stale_segs() {
    let mut doc = load();

    doc.split_table_cells_in_range_native(
        0, 0, CTRL_IDX, TARGET_ROW, 0, TARGET_ROW, 0, 1, 2, false,
    )
    .expect("split_cells_in_range 1×2");

    let table = target_table(&doc);
    assert_eq!(
        stale_text_paras(table),
        0,
        "#4138 회귀(범위 분할): stale line_segs 잔존"
    );
    assert_eq!(
        ladder_regressions(table, TARGET_ROW),
        0,
        "#4138 회귀(범위 분할): vpos 사다리 역행"
    );
    assert_eq!(doc.page_count(), 195, "#4138 회귀(범위 분할): 쪽수 불일치");
}
