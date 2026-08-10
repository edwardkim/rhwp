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

/// 편집한 native HWP의 제품 경로는 저장 뒤 다시 여는 것이다. 메모리 `page_count()`는
/// serializer가 정규화하는 line segment/문단 상태를 거치지 않으므로, 한컴 PDF와의
/// 쪽수 oracle에는 저장본을 재파싱한 값만 사용한다.
fn saved_hwp(doc: &rhwp::wasm_api::HwpDocument) -> rhwp::wasm_api::HwpDocument {
    let bytes = doc.export_hwp_native().expect("분할 HWP 저장");
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("분할 HWP 재파싱")
}

fn paragraph_instance_id(para: &rhwp::model::paragraph::Paragraph) -> Option<u32> {
    para.raw_header_extra
        .get(6..10)
        .map(|bytes| u32::from_le_bytes(bytes.try_into().expect("instanceId 4바이트")))
}

/// `Cell::new_from_template`가 만든 오른쪽 빈 peer의 zeroed instanceId는 HWP5
/// 저장·재파싱 뒤에도 남는다. 이 값과 clone metadata가 renderer의 strict-cut
/// provenance이므로 실제 제품 저장 경계에서 직접 고정한다.
fn saved_hwp_page_count_and_split_provenance(doc: &rhwp::wasm_api::HwpDocument) -> u32 {
    let reparsed = saved_hwp(doc);
    let table = target_table(&reparsed);
    let source = table
        .cells
        .iter()
        .find(|cell| cell.row == TARGET_ROW && cell.col == 0)
        .expect("분할 원문 셀");
    let peer = table
        .cells
        .iter()
        .find(|cell| cell.row == TARGET_ROW && cell.col == 1)
        .expect("분할 빈 peer");
    let source_instance_id = paragraph_instance_id(&source.paragraphs[0])
        .expect("원문 셀 첫 문단은 instanceId 4바이트를 보존해야 한다");
    assert_ne!(
        source_instance_id, 0,
        "원문 셀 첫 문단은 0이 아닌 기존 instanceId를 보존해야 한다"
    );
    assert_eq!(
        paragraph_instance_id(&peer.paragraphs[0]),
        Some(0),
        "new_from_template 빈 peer는 저장 뒤에도 zeroed instanceId여야 한다"
    );
    reparsed.page_count()
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

/// 좁아진 셀에서 본문 뒤에 남은 폭을 넘는 inline control은 독립 줄을 가져야 한다.
///
/// 한컴 2020이 분할 HWP를 HWPX로 재저장한 오라클에서 p288(중간 nested table),
/// p322/p2001(nested table), p2286(trailing picture)는 모두 text prefix와 control
/// source line이 나뉜다. 이 경계가 합쳐지면 객체가 셀 우측 경계에서 clip되고 이후
/// RowBreak page owner도 달라진다.
fn split_inline_control_line_regressions(table: &Table, row: u16) -> usize {
    let Some(cell) = table
        .cells
        .iter()
        .find(|cell| cell.row == row && cell.col == 0)
    else {
        return 1;
    };
    [288usize, 322, 2001, 2286]
        .into_iter()
        .filter(|para_idx| {
            cell.paragraphs
                .get(*para_idx)
                .is_none_or(|para| para.line_segs.len() != 2)
        })
        .count()
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
    assert_eq!(
        split_inline_control_line_regressions(table, TARGET_ROW),
        0,
        "#4138 회귀: text + inline control source line이 다시 합쳐짐"
    );

    // 한컴 2020은 같은 1×2 분할 저장본을 197쪽 PDF로 출력한다. 제품 경로인 native
    // HWP 저장→재파싱에서도 정확히 같은 쪽수를 유지해야 한다.
    let pages = saved_hwp_page_count_and_split_provenance(&doc);
    assert_eq!(
        pages, 197,
        "#4138 회귀: 저장 뒤 재파싱 쪽수 {pages} (한컴 2020 PDF=197)"
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
    assert_eq!(
        split_inline_control_line_regressions(table, TARGET_ROW),
        0,
        "#4138 회귀(범위 분할): text + inline control source line 재결합"
    );
    assert_eq!(
        saved_hwp_page_count_and_split_provenance(&doc),
        197,
        "#4138 회귀(범위 분할): 저장 뒤 재파싱 쪽수 불일치"
    );
}
