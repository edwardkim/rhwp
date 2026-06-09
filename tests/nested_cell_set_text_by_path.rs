//! Nested table-cell REPLACE via `set_cell_text_by_path_native`.
//!
//! Bug: editing a cell that lives INSIDE another cell (table-in-cell-in-table)
//! was impossible because the edit API addressed only one cell level. This test
//! proves the new path-based REPLACE rewrites the innermost cell content while
//! leaving the surrounding structure intact.
//!
//! Fixture: `samples/exam_social.hwp` — paragraph 1 holds a 1x1 wrapper table
//! (control 0); inside its cell 0 / paragraph 0 sits a NESTED table (control 1).
//! The path `[(0,0,0),(1,1,0)]` descends body→outer table→outer cell→outer cell
//! paragraph→nested table→nested cell 1→nested cell paragraph 0. (Same path the
//! issue_717 hit-test test uses for `insert_text_in_cell_by_path`.)

use std::path::Path;

use rhwp::wasm_api::HwpDocument;

fn load(rel: &str) -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    HwpDocument::from_bytes(&bytes).expect("parse fixture")
}

/// Count non-overlapping occurrences of `needle` in `hay` (char-based).
fn count_occurrences(hay: &str, needle: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }
    let mut n = 0;
    let mut rest = hay;
    while let Some(i) = rest.find(needle) {
        n += 1;
        rest = &rest[i + needle.len()..];
    }
    n
}

#[test]
fn set_cell_text_by_path_replaces_innermost_nested_cell() {
    let mut doc = load("samples/exam_social.hwp");

    // Body anchor + nested path to the innermost cell.
    let section = 0usize;
    let parent_para = 1usize;
    let path = [(0usize, 0usize, 0usize), (1usize, 1usize, 0usize)];

    // ── Baseline: capture original innermost-cell text + the OUTER cell text. ──
    let original_inner = doc
        .get_text_in_cell_by_path(section, parent_para, &path, 0, usize::MAX)
        .expect("read original inner text");
    assert!(
        !original_inner.trim().is_empty(),
        "fixture precondition: innermost cell must start non-empty (got {:?})",
        original_inner
    );

    // Outer wrapper cell content (path of length 1 = the outer cell's own
    // paragraph, which HOLDS the nested table). Used to prove the outer
    // structure survives the inner replace.
    let outer_path = [(0usize, 0usize, 0usize)];
    let outer_before = doc
        .get_text_in_cell_by_path(section, parent_para, &outer_path, 0, usize::MAX)
        .expect("read outer cell text before");

    // ── Act: REPLACE the innermost cell with two new paragraphs. ──
    let new_lines = vec![
        "NESTED_REPLACED_LINE_ONE".to_string(),
        "NESTED_REPLACED_LINE_TWO".to_string(),
    ];
    let res = doc
        .set_cell_text_by_path_native(section, parent_para, &path, &new_lines)
        .expect("set_cell_text_by_path_native");
    assert!(
        res.contains("\"cellParaCount\":2"),
        "result must report 2 cell paragraphs, got {res}"
    );

    // ── Assert (a): innermost cell text is REPLACED — old gone, new present. ──
    // Paragraph 0 now holds line one.
    let line0 = doc
        .get_text_in_cell_by_path(section, parent_para, &path, 0, usize::MAX)
        .expect("read new inner line 0");
    assert_eq!(line0, "NESTED_REPLACED_LINE_ONE", "inner cell para 0 text");

    // Paragraph 1 now holds line two (the second descent ends at cell_para 1).
    let path_p1 = [(0usize, 0usize, 0usize), (1usize, 1usize, 1usize)];
    let line1 = doc
        .get_text_in_cell_by_path(section, parent_para, &path_p1, 0, usize::MAX)
        .expect("read new inner line 1");
    assert_eq!(line1, "NESTED_REPLACED_LINE_TWO", "inner cell para 1 text");

    // No overlap / duplication: each new line appears EXACTLY ONCE in its own
    // cell paragraph (the precise no-overlap guarantee — `set_cell_text` rewrote
    // the whole cell, it did not append/interleave). `get_text_in_cell_by_path`
    // reads only the innermost resolved paragraph, so a count of 1 per line
    // proves the old content was cleared and the new content was not doubled.
    assert_eq!(
        count_occurrences(&line0, "NESTED_REPLACED_LINE_ONE"),
        1,
        "new line one must appear exactly once in inner cell para 0 (no overlap/dup): {line0:?}"
    );
    assert_eq!(
        count_occurrences(&line1, "NESTED_REPLACED_LINE_TWO"),
        1,
        "new line two must appear exactly once in inner cell para 1 (no overlap/dup): {line1:?}"
    );
    // The innermost cell must have EXACTLY 2 paragraphs now (old extra paras, if
    // any, were collapsed; we did not leave stale paragraphs behind). A read of
    // a hypothetical 3rd paragraph (cellParaIndex 2) must therefore FAIL.
    let path_p2 = [(0usize, 0usize, 0usize), (1usize, 1usize, 2usize)];
    assert!(
        doc.get_text_in_cell_by_path(section, parent_para, &path_p2, 0, usize::MAX)
            .is_err(),
        "inner cell must hold exactly 2 replacement paragraphs (no stale 3rd para)"
    );
    // The old inner text must be gone from both replaced cell paragraphs.
    let trimmed_old = original_inner.trim();
    if !trimmed_old.is_empty() {
        assert_eq!(
            count_occurrences(&line0, trimmed_old) + count_occurrences(&line1, trimmed_old),
            0,
            "old inner text {:?} must be gone from replaced cell (paras {:?} / {:?})",
            trimmed_old,
            line0,
            line1
        );
    }

    // ── Assert (b): the OUTER structure is intact. ──
    // The wrapper cell still hosts the nested table and its own paragraph text
    // is unchanged (only the INNER cell was rewritten).
    let outer_after = doc
        .get_text_in_cell_by_path(section, parent_para, &outer_path, 0, usize::MAX)
        .expect("read outer cell text after");
    assert_eq!(
        outer_after, outer_before,
        "outer wrapper cell's own paragraph text must be unchanged by inner replace"
    );

    // The enumerator still finds the outer + nested tables and the nested cell
    // at the same path, now carrying the new text.
    let full = doc.enumerate_elements();
    assert!(
        full.contains("\"type\":\"table\""),
        "outer/nested tables must still be enumerable after replace"
    );
    assert!(
        full.contains("NESTED_REPLACED_LINE_ONE"),
        "replaced nested-cell text must be discoverable via enumerate_elements"
    );
    assert!(
        full.contains("\"type\":\"cell\""),
        "cells must still be enumerable after replace"
    );
}

/// A 1-element path REPLACE must match the single-level `set_cell_text_native`
/// (regression guard for the shared format-preserving logic).
#[test]
fn set_cell_text_by_path_single_level_matches_legacy() {
    let mut a = load("samples/table-001.hwp");
    let mut b = load("samples/table-001.hwp");

    let lines = vec!["ALPHA".to_string(), "BETA".to_string()];

    // Locate the first table via enumerate to pick a valid (para, control, cell).
    // table-001.hwp: paragraph holding the table is small; use (para=0..) probe.
    // Single-level API on (sec0, para?, ctrl0, cell0):
    // Find a paragraph that has a table by trying enumerate ref of first cell.
    let json = a.enumerate_elements();
    // crude parse: first cell ref carries parentParaIndex/controlIndex/cellIndex.
    let cell_marker = "\"cell\":{\"parentParaIndex\":";
    let idx = json
        .find(cell_marker)
        .expect("table-001 must have at least one cell");
    let tail = &json[idx + cell_marker.len()..];
    let nums: Vec<usize> = tail
        .split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .take(4)
        .map(|s| s.parse().unwrap())
        .collect();
    let (ppi, ctrl, cell) = (nums[0], nums[1], nums[2]);

    let legacy = a
        .set_cell_text_native(0, ppi, ctrl, cell, &lines)
        .expect("legacy single-level set");
    let path = [(ctrl, cell, 0usize)];
    let by_path = b
        .set_cell_text_by_path_native(0, ppi, &path, &lines)
        .expect("by-path single-level set");

    assert_eq!(
        legacy, by_path,
        "1-element path REPLACE must report the same result as legacy"
    );
    // Both documents' cell now reads ALPHA / BETA identically.
    let a_line0 = a
        .get_text_in_cell_by_path(0, ppi, &path, 0, usize::MAX)
        .unwrap();
    let b_line0 = b
        .get_text_in_cell_by_path(0, ppi, &path, 0, usize::MAX)
        .unwrap();
    assert_eq!(a_line0, "ALPHA");
    assert_eq!(b_line0, "ALPHA");
    assert_eq!(a_line0, b_line0);
}
