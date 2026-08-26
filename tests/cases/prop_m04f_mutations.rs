//! [#5465] M04-f: 기존 step 시퀀스 변형.
//!
//! 시퀀스는 fill_fields · replace_text · set_cell · set_checkbox 만 이어 붙인다.
//! DocumentCore 편집 함수를 새로 만들지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::parser::hwpx::parse_hwpx;
use rhwp::parser::parse_hwp;
use rhwp::serializer::hwpx::roundtrip::diff_documents;
use rhwp::serializer::hwpx::serialize_hwpx;
use rhwp::serializer::serialize_hwp;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/proptest_m04f");
const SAMPLES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/samples");
const ACTIONS: [&str; 4] = ["fill_fields", "replace_text", "set_cell", "set_checkbox"];

fn read_jsonl(rel: &str) -> Vec<serde_json::Value> {
    let path = Path::new(ROOT).join(rel);
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_str(line).unwrap_or_else(|e| panic!("{rel}: {e}")))
        .collect()
}

#[test]
fn mutation_sequences_use_only_existing_actions() {
    let rows = read_jsonl("catalogs/mutation_sequences.jsonl");
    assert!(rows.len() >= 40);
    let mut families = BTreeSet::new();
    for row in &rows {
        families.insert(row["family"].as_str().unwrap().to_string());
        let steps = row["steps"].as_array().unwrap();
        assert!(!steps.is_empty(), "{}", row["id"]);
        for step in steps {
            let action = step["action"].as_str().unwrap();
            assert!(ACTIONS.contains(&action), "{action}");
        }
    }
    for family in [
        "replace_then_replace_same",
        "set_cell_then_same_cell",
        "fill_always_skip_on_claimed_no_fields",
        "checkbox_always_skip_on_claimed_none",
        "schema_action_pair",
        "mixed_unexpressible",
    ] {
        assert!(families.contains(family), "{family}");
    }
}

#[test]
fn schema_action_pairs_cover_four_by_four() {
    let rows = read_jsonl("catalogs/mutation_sequences.jsonl");
    let mut pairs = BTreeSet::new();
    for row in &rows {
        if row["family"] != "schema_action_pair" {
            continue;
        }
        let steps = row["steps"].as_array().unwrap();
        assert_eq!(steps.len(), 2);
        pairs.insert((
            steps[0]["action"].as_str().unwrap().to_string(),
            steps[1]["action"].as_str().unwrap().to_string(),
        ));
        assert_eq!(row["expected"], "schema_ok");
        assert_eq!(row["fixture"], "plan_only");
    }
    assert_eq!(pairs.len(), 16, "4×4 쌍 {pairs:?}");
}

#[test]
fn unclaimed_sequences_do_not_guess_apply() {
    let rows = read_jsonl("catalogs/mutation_sequences.jsonl");
    let mut n = 0;
    for row in &rows {
        if row["fixture"] != "ref_mixed_hwpx" {
            continue;
        }
        n += 1;
        assert_eq!(row["expected"], "skip_all");
        assert!(row["note"].as_str().unwrap().contains("추측"));
    }
    assert!(n >= 1);
}

#[test]
fn text_fixture_cell_is_skip_not_invented() {
    let rows = read_jsonl("catalogs/mutation_sequences.jsonl");
    let mut n = 0;
    for row in &rows {
        if row["family"] != "text_fixture_cell_is_skip" {
            continue;
        }
        n += 1;
        assert_eq!(row["expected"], "apply_then_skip");
        assert_eq!(row["steps"][1]["action"], "set_cell");
    }
    assert!(n >= 1);
}

#[test]
fn handwritten_hwpx_replace_sequence_roundtrips() {
    let bytes = fs::read(Path::new(SAMPLES).join("hwpx/ref/ref_text.hwpx")).unwrap();
    let mut core = DocumentCore::from_bytes(&bytes).unwrap();
    assert!(!core.grep("Hello", true, None).is_empty());
    core.replace_all_native("Hello", "한국", true).unwrap();
    assert!(core.grep("Hello", true, None).is_empty());
    assert!(!core.grep("한국", true, None).is_empty());
    let edited = core.document().clone();
    let out = serialize_hwpx(&edited).unwrap();
    let reparsed = parse_hwpx(&out).unwrap();
    let diff = diff_documents(&edited, &reparsed);
    assert!(diff.is_empty(), "{}", diff.differences.len());
}

#[test]
fn handwritten_hwp5_cell_then_same_cell_roundtrips() {
    let bytes = fs::read(Path::new(SAMPLES).join("table-001.hwp")).unwrap();
    let mut core = DocumentCore::from_bytes(&bytes).unwrap();
    // 기존 set_cell 경로: 비우고 쓰기. 두 번 적용해도 새 API 아님.
    let first = apply_set_cell_like_engine(&mut core, 0, 2, 1, "서울").unwrap();
    assert!(first);
    let second = apply_set_cell_like_engine(&mut core, 0, 2, 1, "완료").unwrap();
    assert!(second);
    let edited = core.document().clone();
    let out = serialize_hwp(&edited).unwrap();
    let reparsed = parse_hwp(&out).unwrap();
    let diff = diff_documents(&edited, &reparsed);
    assert!(diff.is_empty(), "{}", diff.differences.len());
}

fn apply_set_cell_like_engine(
    core: &mut DocumentCore,
    table: usize,
    row: u16,
    col: u16,
    text: &str,
) -> Result<bool, String> {
    use rhwp::document_core::queries::table_extract::extract_tables;
    use rhwp::model::control::Control;

    let grids = extract_tables(core.document());
    let Some(grid) = grids.get(table) else {
        return Ok(false);
    };
    if !grid.container_path.is_empty() {
        return Ok(false);
    }
    let Control::Table(tbl) =
        &core.document().sections[grid.section].paragraphs[grid.paragraph].controls[grid.control]
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
    let section = grid.section;
    let paragraph = grid.paragraph;
    let control = grid.control;
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

#[test]
fn fill_and_checkbox_sequences_on_text_fixture_are_skip_all() {
    let rows = read_jsonl("catalogs/mutation_sequences.jsonl");
    for row in &rows {
        if row["fixture"] != "ref_text_hwpx" {
            continue;
        }
        if row["family"] == "fill_always_skip_on_claimed_no_fields"
            || row["family"] == "checkbox_always_skip_on_claimed_none"
        {
            assert_eq!(row["expected"], "skip_all");
        }
        if row["family"] == "mixed_unexpressible" {
            assert_eq!(row["expected"], "reject");
        }
    }
}
