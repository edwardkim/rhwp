//! [#5382] M04-3: HWP5 편집 후 parse→serialize→reparse 왕복 property (IrDiff 0).
//!
//! 작은 픽스처에 **이미 있는** `rhwp run` step 4종(`fill_fields` · `replace_text` ·
//! `set_cell` · `set_checkbox`)만 적용한다. DocumentCore 편집 API 를 발명하지 않고,
//! 이 픽스처가 표현하지 못하는 step 은 skip 한다 (누름틀/표/□ 없음 등).
//!
//! 비교는 기존 [`diff_documents`] 이다. HWPX 왕복은 M04-2.
//!
//! CI 는 싼 기본값(`CI_CASES`). 전체 화력은 `PROPTEST_CASES` (proptest 표준).
#![cfg(not(target_arch = "wasm32"))]

use proptest::prelude::*;

use rhwp::document_core::queries::table_extract::extract_tables;
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::parser::parse_hwp;
use rhwp::serializer::hwpx::roundtrip::diff_documents;
use rhwp::serializer::serialize_hwp;

/// 표·누름틀 없는 일반 HWP5. `hwp5_roundtrip_baseline` A등급이라 무편집 왕복은 IrDiff 0.
const FIXTURE_TEXT: &[u8] = include_bytes!("../../samples/para-001.hwp");
/// 19×9 표, 중첩 없음. baseline 등급이라 무편집 왕복은 IrDiff 0.
const FIXTURE_TABLE: &[u8] = include_bytes!("../../samples/table-001.hwp");

/// CI 기본. `PROPTEST_CASES` 가 있으면 proptest 가 덮어쓴다.
const CI_CASES: u32 = 8;
const CI_MAX_SHRINK_ITERS: u32 = 32;

const FIND_NEEDLES: &[&str] = &["오호라", "乾坤", "구궁산"];
const TABLE_FIND_NEEDLES: &[&str] = &["품질", "5월", "평가"];
const REPLACE_TEXTS: &[&str] = &["", "한국", "X", "2026"];
const CELL_TEXTS: &[&str] = &["", "서울", "완료", "1"];
const FIELD_NAMES: &[&str] = &["이름", "myMsg01", "title"];

/// 기존 `rhwp run` step. 스키마 4종과 1:1 이고, 실행은 엔진이 이미 쓰는 함수만 호출한다.
#[derive(Clone, Debug)]
enum ExistingRunStep {
    FillFields {
        name: String,
        value: String,
    },
    ReplaceText {
        find: String,
        replace: String,
        occurrence: Option<u32>,
    },
    SetCell {
        table: u32,
        row: u16,
        col: u16,
        text: String,
    },
    SetCheckbox {
        occurrence: u32,
    },
}

fn prop_config() -> ProptestConfig {
    ProptestConfig {
        cases: CI_CASES,
        max_shrink_iters: CI_MAX_SHRINK_ITERS,
        ..ProptestConfig::default()
    }
}

fn arb_replace_text(needles: &'static [&'static str]) -> impl Strategy<Value = ExistingRunStep> {
    (
        prop::sample::select(needles),
        prop::sample::select(REPLACE_TEXTS),
        proptest::option::of(Just(0u32)),
    )
        .prop_map(|(find, replace, occurrence)| ExistingRunStep::ReplaceText {
            find: (*find).to_string(),
            replace: (*replace).to_string(),
            occurrence,
        })
}

fn arb_set_cell() -> impl Strategy<Value = ExistingRunStep> {
    (
        Just(0u32),
        0u16..3,
        0u16..3,
        prop::sample::select(CELL_TEXTS),
    )
        .prop_map(|(table, row, col, text)| ExistingRunStep::SetCell {
            table,
            row,
            col,
            text: (*text).to_string(),
        })
}

fn arb_fill_fields() -> impl Strategy<Value = ExistingRunStep> {
    (
        prop::sample::select(FIELD_NAMES),
        prop::sample::select(REPLACE_TEXTS),
    )
        .prop_map(|(name, value)| ExistingRunStep::FillFields {
            name: (*name).to_string(),
            value: (*value).to_string(),
        })
}

fn arb_set_checkbox() -> impl Strategy<Value = ExistingRunStep> {
    (0u32..2).prop_map(|occurrence| ExistingRunStep::SetCheckbox { occurrence })
}

/// 4종을 모두 생성하되, 이 픽스처에서 실제로 먹을 수 있는 replace_text 에 무게를 둔다.
fn arb_step_for_text_fixture() -> impl Strategy<Value = ExistingRunStep> {
    prop_oneof![
        6 => arb_replace_text(FIND_NEEDLES),
        1 => arb_set_cell(),
        1 => arb_fill_fields(),
        1 => arb_set_checkbox(),
    ]
}

fn arb_step_for_table_fixture() -> impl Strategy<Value = ExistingRunStep> {
    prop_oneof![
        6 => arb_set_cell(),
        1 => arb_replace_text(TABLE_FIND_NEEDLES),
        1 => arb_fill_fields(),
        1 => arb_set_checkbox(),
    ]
}

fn format_ir_diff(diff: &rhwp::serializer::hwpx::roundtrip::IrDiff) -> String {
    diff.differences
        .iter()
        .map(|d| d.to_string())
        .collect::<Vec<_>>()
        .join("; ")
}

/// 기존 엔진이 이 step 을 이 문서에 표현할 수 있으면 적용, 아니면 skip.
/// 새 mutation API 를 만들지 않는다.
fn apply_existing_step(core: &mut DocumentCore, step: &ExistingRunStep) -> Result<bool, String> {
    match step {
        ExistingRunStep::FillFields { name, value } => {
            match core.set_field_value_by_name_at(name, 0, value) {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            }
        }
        ExistingRunStep::ReplaceText {
            find,
            replace,
            occurrence,
        } => {
            if find.is_empty() {
                return Ok(false);
            }
            let hits = core.grep(find, true, None);
            if hits.is_empty() {
                return Ok(false);
            }
            let result = match occurrence {
                Some(n) => {
                    if (*n as usize) >= hits.len() {
                        return Ok(false);
                    }
                    core.replace_nth_native(find, replace, true, *n as usize)
                }
                None => core.replace_all_native(find, replace, true),
            };
            result.map(|_| true).map_err(|e| e.to_string())
        }
        ExistingRunStep::SetCell {
            table,
            row,
            col,
            text,
        } => apply_set_cell(core, *table, *row, *col, text),
        ExistingRunStep::SetCheckbox { occurrence } => {
            let hits = core.grep("□", true, None);
            if (*occurrence as usize) >= hits.len() {
                return Ok(false);
            }
            core.replace_nth_native("□", "☑", true, *occurrence as usize)
                .map(|_| true)
                .map_err(|e| e.to_string())
        }
    }
}

/// `run_plan_engine` 의 set_cell 과 같은 기존 셀 비우기→쓰기 경로.
fn apply_set_cell(
    core: &mut DocumentCore,
    table: u32,
    row: u16,
    col: u16,
    text: &str,
) -> Result<bool, String> {
    if text.chars().any(|ch| matches!(ch, '\r' | '\n' | '\t')) {
        return Ok(false);
    }
    let grids = extract_tables(core.document());
    let Some(grid) = grids.get(table as usize) else {
        return Ok(false);
    };
    if !grid.container_path.is_empty() {
        return Ok(false);
    }
    let section = grid.section;
    let paragraph = grid.paragraph;
    let control = grid.control;
    let Control::Table(tbl) =
        &core.document().sections[section].paragraphs[paragraph].controls[control]
    else {
        return Ok(false);
    };
    let Some(cell_idx) = tbl
        .cells
        .iter()
        .position(|cell| cell.row == row && cell.col == col)
    else {
        return Ok(false);
    };
    let para_lens: Vec<usize> = tbl.cells[cell_idx]
        .paragraphs
        .iter()
        .map(|para| para.text.chars().count())
        .collect();

    for (pi, len) in para_lens.iter().enumerate() {
        if *len == 0 {
            continue;
        }
        core.delete_text_in_cell_native(section, paragraph, control, cell_idx, pi, 0, *len)
            .map_err(|e| e.to_string())?;
    }
    if !text.is_empty() {
        core.insert_text_in_cell_native(section, paragraph, control, cell_idx, 0, 0, text)
            .map_err(|e| e.to_string())?;
    }
    Ok(true)
}

fn assert_edit_serialize_reparse(bytes: &[u8], steps: &[ExistingRunStep]) -> Result<(), String> {
    let mut core = DocumentCore::from_bytes(bytes).map_err(|e| format!("parse: {e}"))?;
    let mut applied = 0usize;
    for step in steps {
        if apply_existing_step(&mut core, step)? {
            applied += 1;
        }
    }
    if !steps.is_empty() && applied == 0 {
        return Err("skip".into());
    }

    let edited = core.document().clone();
    let out = serialize_hwp(&edited).map_err(|e| format!("serialize: {e}"))?;
    let reparsed = parse_hwp(&out).map_err(|e| format!("reparse: {e}"))?;
    let diff = diff_documents(&edited, &reparsed);
    if !diff.is_empty() {
        return Err(format!(
            "IrDiff {}건 (applied {applied}/{ }): {}",
            diff.differences.len(),
            steps.len(),
            format_ir_diff(&diff)
        ));
    }
    Ok(())
}

#[test]
fn fixture_identity_ir_diff_is_zero() {
    for (label, bytes) in [("para-001", FIXTURE_TEXT), ("table-001", FIXTURE_TABLE)] {
        assert_edit_serialize_reparse(bytes, &[]).unwrap_or_else(|e| panic!("{label}: {e}"));
    }
}

#[test]
fn handwritten_replace_text_roundtrips() {
    let steps = [ExistingRunStep::ReplaceText {
        find: "오호라".into(),
        replace: "한국".into(),
        occurrence: None,
    }];
    assert_edit_serialize_reparse(FIXTURE_TEXT, &steps).expect("replace_text 왕복");
}

#[test]
fn handwritten_set_cell_roundtrips() {
    let steps = [ExistingRunStep::SetCell {
        table: 0,
        row: 2,
        col: 1,
        text: "서울".into(),
    }];
    assert_edit_serialize_reparse(FIXTURE_TABLE, &steps).expect("set_cell 왕복");
}

#[test]
fn handwritten_replace_variants_roundtrip() {
    for (find, replace, occurrence) in [
        ("오호라", "한국", None),
        ("乾坤", "X", Some(0)),
        ("구궁산", "", None),
    ] {
        let steps = [ExistingRunStep::ReplaceText {
            find: find.into(),
            replace: replace.into(),
            occurrence,
        }];
        assert_edit_serialize_reparse(FIXTURE_TEXT, &steps)
            .unwrap_or_else(|e| panic!("{find}->{replace:?}: {e}"));
    }
}

#[test]
fn handwritten_set_cell_grid_roundtrips() {
    for (row, col, text) in [
        (0u16, 0u16, "서울"),
        (0, 1, ""),
        (2, 1, "완료"),
        (2, 2, "1"),
    ] {
        let steps = [ExistingRunStep::SetCell {
            table: 0,
            row,
            col,
            text: text.into(),
        }];
        assert_edit_serialize_reparse(FIXTURE_TABLE, &steps)
            .unwrap_or_else(|e| panic!("cell({row},{col}): {e}"));
    }
}

#[test]
fn table_needles_roundtrip_via_replace_text() {
    for find in ["품질", "5월", "평가"] {
        let steps = [ExistingRunStep::ReplaceText {
            find: find.into(),
            replace: "한국".into(),
            occurrence: None,
        }];
        assert_edit_serialize_reparse(FIXTURE_TABLE, &steps)
            .unwrap_or_else(|e| panic!("{find}: {e}"));
    }
}

#[test]
fn inexpressible_steps_are_skipped_not_invented() {
    let mut core = DocumentCore::from_bytes(FIXTURE_TEXT).expect("parse");
    assert!(
        !apply_existing_step(
            &mut core,
            &ExistingRunStep::FillFields {
                name: "이름".into(),
                value: "홍길동".into(),
            },
        )
        .expect("fill_fields"),
        "para-001 은 누름틀이 없어 fill_fields 를 skip 해야 한다"
    );
    assert!(
        !apply_existing_step(
            &mut core,
            &ExistingRunStep::SetCell {
                table: 0,
                row: 0,
                col: 0,
                text: "x".into(),
            },
        )
        .expect("set_cell"),
        "para-001 은 표가 없어 set_cell 을 skip 해야 한다"
    );
    assert!(
        !apply_existing_step(&mut core, &ExistingRunStep::SetCheckbox { occurrence: 0 },)
            .expect("set_checkbox"),
        "para-001 은 □ 가 없어 set_checkbox 를 skip 해야 한다"
    );
}

proptest! {
    #![proptest_config(prop_config())]

    #[test]
    fn edited_para001_hwp5_roundtrips_with_zero_ir_diff(
        steps in proptest::collection::vec(arb_step_for_text_fixture(), 0..3)
    ) {
        match assert_edit_serialize_reparse(FIXTURE_TEXT, &steps) {
            Ok(()) => {}
            Err(err) if err == "skip" => {
                return Err(TestCaseError::reject(
                    "이 픽스처가 표현할 수 없는 step 만 나옴",
                ));
            }
            Err(err) => return Err(TestCaseError::fail(err)),
        }
    }

    #[test]
    fn edited_table001_hwp5_roundtrips_with_zero_ir_diff(
        steps in proptest::collection::vec(arb_step_for_table_fixture(), 0..3)
    ) {
        match assert_edit_serialize_reparse(FIXTURE_TABLE, &steps) {
            Ok(()) => {}
            Err(err) if err == "skip" => {
                return Err(TestCaseError::reject(
                    "이 픽스처가 표현할 수 없는 step 만 나옴",
                ));
            }
            Err(err) => return Err(TestCaseError::fail(err)),
        }
    }
}
