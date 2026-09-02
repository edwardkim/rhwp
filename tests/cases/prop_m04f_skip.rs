//! [#5465] M04-f: skip 정직 표.
//!
//! 픽스처가 표현하지 못하는 기존 run step 은 skip 한다.
//! 새 DocumentCore mutation 으로 skip 을 메우지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use rhwp::document_core::queries::table_extract::extract_tables;
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/proptest_m04f");
const SAMPLES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/samples");

const SKIP_REASONS: [&str; 11] = [
    "empty_find",
    "no_hits",
    "occurrence_oob",
    "field_missing",
    "table_missing",
    "nested_table",
    "cell_missing",
    "cell_control_char",
    "checkbox_missing",
    "all_steps_skipped",
    "unclaimed_capability",
];

fn read_jsonl(rel: &str) -> Vec<serde_json::Value> {
    let path = Path::new(ROOT).join(rel);
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_str(line).unwrap_or_else(|e| panic!("{rel}: {e}")))
        .collect()
}

fn read_json(rel: &str) -> serde_json::Value {
    let path = Path::new(ROOT).join(rel);
    serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap()
}

fn sample(rel: &str) -> PathBuf {
    Path::new(SAMPLES).join(rel.strip_prefix("samples/").unwrap_or(rel))
}

/// 기존 `apply_existing_step` 과 같은 판정. 새 API 없음.
fn apply_existing_step(core: &mut DocumentCore, step: &serde_json::Value) -> Result<bool, String> {
    match step["action"].as_str() {
        Some("fill_fields") => {
            let data = step["data"].as_object().ok_or("data")?;
            let (name, value) = data.iter().next().ok_or("empty data")?;
            let text = match value {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            match core.set_field_value_by_name_at(name, 0, &text) {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            }
        }
        Some("replace_text") => {
            let find = step["find"].as_str().unwrap_or("");
            if find.is_empty() {
                return Ok(false);
            }
            let replace = step["replace"].as_str().unwrap_or("");
            let hits = core.grep(find, true, None);
            if hits.is_empty() {
                return Ok(false);
            }
            if let Some(n) = step.get("occurrence").and_then(|v| v.as_u64()) {
                if (n as usize) >= hits.len() {
                    return Ok(false);
                }
                core.replace_nth_native(find, replace, true, n as usize)
                    .map(|_| true)
                    .map_err(|e| e.to_string())
            } else {
                core.replace_all_native(find, replace, true)
                    .map(|_| true)
                    .map_err(|e| e.to_string())
            }
        }
        Some("set_cell") => {
            let text = step["text"].as_str().unwrap_or("");
            if text.chars().any(|ch| matches!(ch, '\r' | '\n' | '\t')) {
                return Ok(false);
            }
            let table = step["table"].as_u64().unwrap_or(0) as usize;
            let row = step["row"].as_u64().unwrap_or(0) as u16;
            let col = step["col"].as_u64().unwrap_or(0) as u16;
            let grids = extract_tables(core.document());
            let Some(grid) = grids.get(table) else {
                return Ok(false);
            };
            if !grid.container_path.is_empty() {
                return Ok(false);
            }
            let Control::Table(tbl) = &core.document().sections[grid.section].paragraphs
                [grid.paragraph]
                .controls[grid.control]
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
        Some("set_checkbox") => {
            let occurrence = step["occurrence"].as_u64().unwrap_or(0) as usize;
            let hits = core.grep("□", true, None);
            if occurrence >= hits.len() {
                return Ok(false);
            }
            core.replace_nth_native("□", "☑", true, occurrence)
                .map(|_| true)
                .map_err(|e| e.to_string())
        }
        _ => Ok(false),
    }
}

#[test]
fn skip_reason_taxonomy_is_closed() {
    let reasons = read_json("catalogs/skip_reasons.json");
    let arr = reasons.as_array().unwrap();
    let mut codes = BTreeSet::new();
    for row in arr {
        let code = row["code"].as_str().unwrap();
        assert!(SKIP_REASONS.contains(&code), "모르는 reason {code}");
        assert!(codes.insert(code.to_string()));
        assert!(row["engine"].as_str().unwrap().len() > 3);
        assert!(row["honest"].as_str().unwrap().contains("않"));
    }
    assert_eq!(codes.len(), SKIP_REASONS.len());
}

#[test]
fn skip_catalog_rows_are_honest() {
    let rows = read_jsonl("catalogs/skip_catalog.jsonl");
    assert!(rows.len() >= 200);
    let mut ids = BTreeSet::new();
    for row in &rows {
        let id = row["id"].as_str().unwrap();
        assert!(ids.insert(id.to_string()), "중복 {id}");
        let reason = row["reason"].as_str().unwrap();
        assert!(SKIP_REASONS.contains(&reason), "{id} {reason}");
        let expected = row["expected"].as_str().unwrap();
        assert!(
            expected == "skip" || expected == "reject" || expected == "skip_if_nested",
            "{id} {expected}"
        );
        if reason == "unclaimed_capability" {
            assert_eq!(row["claimed"], false);
            assert_eq!(expected, "skip");
        }
        if reason == "all_steps_skipped" {
            assert_eq!(expected, "reject");
        }
    }
}

#[test]
fn mixed_fixture_is_never_guessed() {
    let rows = read_jsonl("catalogs/skip_catalog.jsonl");
    let mixed: Vec<_> = rows
        .iter()
        .filter(|row| row["fixture"] == "ref_mixed_hwpx")
        .collect();
    assert!(!mixed.is_empty());
    for row in mixed {
        assert_eq!(row["reason"], "unclaimed_capability");
        assert_eq!(row["expected"], "skip");
    }
}

#[test]
fn claimed_text_fixtures_skip_missing_surfaces() {
    let rows = read_jsonl("catalogs/skip_catalog.jsonl");
    for fixture in ["ref_text_hwpx", "para001_hwp5", "ref_empty_hwpx"] {
        let reasons: BTreeSet<_> = rows
            .iter()
            .filter(|row| row["fixture"] == fixture)
            .map(|row| row["reason"].as_str().unwrap())
            .collect();
        assert!(reasons.contains("field_missing"), "{fixture}");
        assert!(reasons.contains("table_missing"), "{fixture}");
        assert!(reasons.contains("checkbox_missing"), "{fixture}");
        assert!(reasons.contains("empty_find"), "{fixture}");
    }
}

#[test]
fn sample_skip_rows_match_engine_on_real_fixtures() {
    let rows = read_jsonl("catalogs/skip_catalog.jsonl");
    let fixtures = read_json("catalogs/fixtures.json");
    let mut checked = 0usize;
    for row in &rows {
        if row["expected"] != "skip" {
            continue;
        }
        let reason = row["reason"].as_str().unwrap();
        if matches!(
            reason,
            "unclaimed_capability" | "all_steps_skipped" | "nested_table"
        ) {
            continue;
        }
        if !row["step"].is_object() {
            continue;
        }
        let fx_id = row["fixture"].as_str().unwrap();
        let Some(fx) = fixtures
            .as_array()
            .unwrap()
            .iter()
            .find(|fx| fx["id"] == fx_id)
        else {
            continue;
        };
        if !fx["claimed"].as_bool().unwrap() {
            continue;
        }
        // 대표만: 각 (fixture, reason) 첫 행.
        checked += 1;
        if checked > 48 {
            break;
        }
        let rel = fx["path"].as_str().unwrap();
        let bytes = fs::read(sample(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"));
        let mut core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("{rel}: {e}"));
        let applied = apply_existing_step(&mut core, &row["step"])
            .unwrap_or_else(|e| panic!("{}: {e}", row["id"]));
        assert!(
            !applied,
            "{} 는 {reason} 인데 apply 되었다",
            row["id"].as_str().unwrap()
        );
    }
    assert!(checked >= 16, "skip 실측 {checked}");
}

#[test]
fn applyable_replace_needles_are_not_in_skip_as_no_hits() {
    let fixtures = read_json("catalogs/fixtures.json");
    let rows = read_jsonl("catalogs/skip_catalog.jsonl");
    for fx in fixtures.as_array().unwrap() {
        if !fx["claimed"].as_bool().unwrap() {
            continue;
        }
        for needle in fx["needles"].as_array().unwrap() {
            let needle = needle.as_str().unwrap();
            let bad = rows.iter().any(|row| {
                row["fixture"] == fx["id"]
                    && row["reason"] == "no_hits"
                    && row["step"]["find"] == needle
                    && row["step"].get("occurrence").is_none()
            });
            assert!(
                !bad,
                "{} 의 주장 needle {needle} 가 no_hits 로 적혀 있다",
                fx["id"]
            );
        }
    }
}

#[test]
fn honesty_report_mentions_every_claimed_fixture() {
    let md = fs::read_to_string(Path::new(ROOT).join("reports/skip_honesty.md")).unwrap();
    for id in [
        "ref_text_hwpx",
        "ref_table_hwpx",
        "para001_hwp5",
        "table001_hwp5",
        "ref_empty_hwpx",
        "ref_mixed_hwpx",
    ] {
        assert!(md.contains(id), "{id}");
    }
}
