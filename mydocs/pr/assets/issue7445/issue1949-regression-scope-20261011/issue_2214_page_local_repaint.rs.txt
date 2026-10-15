//! Issue #2214 Stage 3 contracts.
//!
//! cell-flow mutation result를 고정한다. 고정 px 배치 검사는 #7813에 이관했다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::wasm_api::HwpDocument;
use serde::Deserialize;

const HWP_SAMPLE: &str = "samples/issue1949_giant_cell_nested_tables_perf.hwp";
const HWPX_SAMPLE: &str = "samples/issue1949_giant_cell_nested_tables_perf.hwpx";

const SECTION: usize = 0;
const PARENT_PARAGRAPH: usize = 0;
const TABLE_CONTROL: usize = 2;
const CELL: usize = 2;
const TARGET_PARAGRAPH: usize = 5;
const INSERT_OFFSET: usize = 130;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CellEditResult {
    char_offset: usize,
    cell_flow_changed: bool,
}

fn load_sample(relative_path: &str) -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {relative_path}: {e}"));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {relative_path}: {e}"))
}

fn target_paragraph(core: &DocumentCore) -> &Paragraph {
    match &core.document().sections[SECTION].paragraphs[PARENT_PARAGRAPH].controls[TABLE_CONTROL] {
        Control::Table(table) => &table.cells[CELL].paragraphs[TARGET_PARAGRAPH],
        other => panic!("target control is not a table: {other:?}"),
    }
}

fn next_paragraph_vpos(core: &DocumentCore) -> i32 {
    match &core.document().sections[SECTION].paragraphs[PARENT_PARAGRAPH].controls[TABLE_CONTROL] {
        Control::Table(table) => {
            table.cells[CELL].paragraphs[TARGET_PARAGRAPH + 1]
                .line_segs
                .first()
                .expect("target next paragraph line")
                .vertical_pos
        }
        other => panic!("target control is not a table: {other:?}"),
    }
}

fn line_starts(doc: &HwpDocument) -> Vec<usize> {
    target_paragraph(doc)
        .line_segs
        .iter()
        .map(|seg| seg.text_start as usize)
        .collect()
}

fn relative_flow_advance(para: &Paragraph) -> i64 {
    let first = para.line_segs.first().expect("target first line");
    let last = para.line_segs.last().expect("target last line");
    i64::from(last.vertical_pos) + i64::from(last.line_height) + i64::from(last.line_spacing)
        - i64::from(first.vertical_pos)
}

fn parse_cell_edit_result(raw: String) -> CellEditResult {
    serde_json::from_str(&raw).expect("cell edit result json")
}

fn insert_batch(doc: &mut HwpDocument, count: usize) -> CellEditResult {
    let raw = doc
        .insert_text_in_cell_native_deferred_pagination(
            SECTION,
            PARENT_PARAGRAPH,
            TABLE_CONTROL,
            CELL,
            TARGET_PARAGRAPH,
            INSERT_OFFSET,
            &"1".repeat(count),
        )
        .expect("batch deferred insert");
    parse_cell_edit_result(raw)
}

/// fifth line 이 생기는 입력 수 — HWP·HWPX 모두 56 이다.
///
/// [#7418] 종전 HWPX 61 은 한/글 2020 이 한양신명조를 함초롬바탕으로 대체한 환경의 출력에서
/// 정한 값이었다. 한양신명조가 있는 환경에서 한/글 2024·2020 이 직접 나누면 두 형식 모두
/// 55 번째에서 `[0, 44, 84, 122, 128]` 로 전환한다. rhwp 는 56 번째에 `[.., 129]` 로
/// 전환해 넷째 줄에 한 글자를 더 담는다(남은 차이, `src/wasm_api/tests.rs` 와 같은 근거).
fn flow_boundary_insert_count(label: &str) -> usize {
    match label {
        "hwp" => 56,
        "hwpx" => 56,
        other => panic!("unknown #2214 fixture label: {other}"),
    }
}

fn expected_line_starts(label: &str, inserted: usize) -> &'static [usize] {
    if inserted >= flow_boundary_insert_count(label) {
        match label {
            // 두 형식이 같은 문서라 줄도 같다 — 종전 HWPX 값은 대체 글꼴 환경의 줄이었다.
            "hwp" | "hwpx" => &[0, 44, 84, 122, 129],
            other => panic!("unknown #2214 fixture label: {other}"),
        }
    } else {
        &[0, 44, 84, 122]
    }
}

/// Production 신호의 입력 권위값: line-count 자체가 아니라 상대 flow advance 변화다.
#[test]
fn issue_2214_cell_flow_transition_baseline() {
    for (label, sample) in [("hwp", HWP_SAMPLE), ("hwpx", HWPX_SAMPLE)] {
        let mut doc28 = load_sample(sample);
        let original28 = target_paragraph(&doc28).text.clone();
        let initial_advance = relative_flow_advance(target_paragraph(&doc28));
        let initial_next_vpos = next_paragraph_vpos(&doc28);
        let batch_result = insert_batch(&mut doc28, 28);
        assert_eq!(batch_result.char_offset, INSERT_OFFSET + 28);
        assert!(
            !batch_result.cell_flow_changed,
            "{label}: 28-char batch must report stable cell flow"
        );
        assert_eq!(line_starts(&doc28), vec![0, 44, 84, 122]);
        assert_eq!(
            relative_flow_advance(target_paragraph(&doc28)),
            initial_advance,
            "{label}: 28-char edit must not change cell flow"
        );
        assert_eq!(
            next_paragraph_vpos(&doc28),
            initial_next_vpos,
            "{label}: 28-char edit must preserve next paragraph vpos"
        );
        assert_eq!(
            target_paragraph(&doc28).text,
            format!("{original28}{}", "1".repeat(28)),
            "{label}: 28-char batch text"
        );

        let mut doc50 = load_sample(sample);
        let original50 = target_paragraph(&doc50).text.clone();
        let mut changed_inputs = Vec::new();
        for inserted in 0..62 {
            let before = relative_flow_advance(target_paragraph(&doc50));
            let next_vpos_before = next_paragraph_vpos(&doc50);
            let result = doc50
                .insert_text_in_cell_native_deferred_pagination(
                    SECTION,
                    PARENT_PARAGRAPH,
                    TABLE_CONTROL,
                    CELL,
                    TARGET_PARAGRAPH,
                    INSERT_OFFSET + inserted,
                    "1",
                )
                .expect("per-key deferred insert");
            let result = parse_cell_edit_result(result);
            let delta = relative_flow_advance(target_paragraph(&doc50)) - before;
            let expected = if inserted + 1 == flow_boundary_insert_count(label) {
                1920
            } else {
                0
            };
            assert_eq!(
                result.char_offset,
                INSERT_OFFSET + inserted + 1,
                "{label}: input {} result offset",
                inserted + 1
            );
            assert_eq!(
                result.cell_flow_changed,
                expected != 0,
                "{label}: input {} flow result",
                inserted + 1
            );
            assert_eq!(
                delta,
                expected,
                "{label}: input {} flow delta",
                inserted + 1
            );
            assert_eq!(
                i64::from(next_paragraph_vpos(&doc50) - next_vpos_before),
                expected,
                "{label}: input {} next paragraph vpos delta",
                inserted + 1
            );
            if delta != 0 {
                changed_inputs.push(inserted + 1);
            }
        }
        assert_eq!(
            changed_inputs,
            vec![flow_boundary_insert_count(label)],
            "{label}: exactly one flow boundary"
        );
        assert_eq!(line_starts(&doc50), expected_line_starts(label, 62));
        assert_eq!(
            target_paragraph(&doc50).text,
            format!("{original50}{}", "1".repeat(62)),
            "{label}: 62-char sequential text"
        );
        assert_eq!(doc28.page_count(), 115, "{label}: 28-char page count");
        assert_eq!(doc50.page_count(), 115, "{label}: 50-char page count");
    }
}
