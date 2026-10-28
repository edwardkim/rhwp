//! #7482: compare final page ownership and placement, rather than plan internals.
//! Independent Hancom 2020 PDFs for these exact inputs establish row membership,
//! centered ink, follower order and equal advances of the empty/space/image-host
//! paragraphs. See samples/issue7482/README.md for generation and visual evidence.

#![cfg(not(target_arch = "wasm32"))]

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned())
}

fn page(sample: &str) -> Value {
    let input = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue7482")
        .join(sample);
    rendered_page(&input, None)
}

/// Hancom 2020 p3 places the centered picture-host text and its successor
/// at a 12pt (16px) pitch. A past exclusion outside these rows must not
/// replace the legacy owner's cursor with an unrelated absolute plan.
#[test]
fn unrelated_exclusion_preserves_picture_host_successor_pitch() {
    let input = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/issue_6970/synth_no_ls_square_wrap.hwp");
    let tree = rendered_page(&input, Some(2));
    let all = nodes(&tree);
    let host = find(&all, "TextLine", 141);
    let follower = find(&all, "TextLine", 142);
    assert!(coord(follower, "y") >= bottom(host));
    assert!(
        (coord(follower, "y") - coord(host, "y") - 16.0).abs() < 0.1,
        "host={:?}, follower={:?}",
        host["bbox"],
        follower["bbox"]
    );
    let empty = find(&all, "TextLine", 143);
    assert!((coord(empty, "y") - coord(follower, "y") - 16.0).abs() < 0.1);
    assert_eq!(text_of(&all, 142), "펔퐝 @풦픯햸홁훊 흓갸곁 굊귓깜껥꽮.");
}

/// The independent Hancom PDF places the first plain paragraph in the free
/// left lane of the Square table, before that table's bottom. Subsequent
/// paragraphs retain their order; the larger TAC owns the next page.
#[test]
fn square_exclusion_survives_host_without_advancing_following_text_to_its_bottom() {
    let input = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue7482/tac-after-plain-paragraphs.hwpx");
    let tree = rendered_page(&input, Some(0));
    let all = nodes(&tree);
    let square = find(&all, "Table", 0);
    let follower = find(&all, "TextLine", 1);
    assert!(
        coord(follower, "y") < bottom(square),
        "{:?}",
        follower["bbox"]
    );
    assert!(
        coord(follower, "x") + coord(follower, "w") <= coord(square, "x"),
        "the plain line overlaps the Square object"
    );
    let mut previous_bottom = bottom(find(&all, "TextLine", 0));
    for pi in 1..=22 {
        let line = find(&all, "TextLine", pi);
        assert!(coord(line, "y") >= previous_bottom);
        assert_eq!(text_of(&all, pi), "공간 확인");
        previous_bottom = bottom(line);
    }
    assert_eq!(all.iter().filter(|n| n["type"] == "Table").count(), 1);
    let next = rendered_page(&input, Some(1));
    let next_nodes = nodes(&next);
    assert_eq!(
        next_nodes.iter().filter(|n| n["type"] == "Table").count(),
        1
    );
    assert!(coord(find(&next_nodes, "Table", 23), "h") > 800.0);
}

fn rendered_page(input: &Path, page: Option<u32>) -> Value {
    let dir = std::env::temp_dir().join(format!(
        "rhwp-7482-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let mut command = Command::new(rhwp_bin());
    command
        .arg("export-render-tree")
        .arg(input)
        .arg("-o")
        .arg(&dir);
    if let Some(page) = page {
        command.arg("-p").arg(page.to_string());
    }
    let output = command.output().expect("run actual renderer");
    assert!(output.status.success(), "{output:?}");
    let paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    assert_eq!(paths.len(), 1, "one requested page: {input:?}");
    let tree = serde_json::from_str(&std::fs::read_to_string(&paths[0]).unwrap()).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    tree
}

/// Hancom 2020 p38 has a horizontal rule at 389pt = 518.67px.
/// A new whole-paragraph owner must not put the standalone RowBreak table
/// into the preceding paragraph's last line (the failed candidate used 478.4px).
#[test]
fn standalone_rowbreak_table_preserves_preceding_text_and_following_flow() {
    for sample in [
        "samples/76076_regulatory_analysis.hwp",
        "samples/issue1891/76076_regulatory_analysis.hwpx",
    ] {
        let input = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
        let tree = rendered_page(&input, Some(37));
        let all = nodes(&tree);
        let table = find(&all, "Table", 358);
        assert!(
            (coord(table, "y") - 518.67).abs() < 1.0,
            "{sample}: {:?}",
            table["bbox"]
        );
        assert!(
            all.iter()
                .filter(|n| n["type"] == "TextRun" && n["pi"] == 357)
                .all(|n| bottom(n) <= coord(table, "y")),
            "preceding text overlaps the table: {sample}"
        );
        assert_eq!(
            all.iter()
                .filter(|n| n["type"] == "Table" && n["pi"] == 358)
                .count(),
            1
        );
        assert!(
            coord(find(&all, "TextLine", 360), "y") >= bottom(table),
            "following heading overlaps the table: {sample}"
        );
    }
}

fn body_nodes<'a>(node: &'a Value, in_table: bool, out: &mut Vec<&'a Value>) {
    if !in_table {
        out.push(node);
    }
    let inside = in_table || node["type"] == "Table";
    for child in node["children"].as_array().into_iter().flatten() {
        body_nodes(child, inside, out);
    }
}

fn nodes(tree: &Value) -> Vec<&Value> {
    let mut out = Vec::new();
    body_nodes(tree, false, &mut out);
    out
}

fn coord(node: &Value, name: &str) -> f64 {
    node["bbox"][name].as_f64().unwrap()
}

fn bottom(node: &Value) -> f64 {
    coord(node, "y") + coord(node, "h")
}

fn find<'a>(all: &[&'a Value], kind: &str, pi: u64) -> &'a Value {
    all.iter()
        .copied()
        .find(|n| n["type"] == kind && n["pi"].as_u64() == Some(pi))
        .unwrap_or_else(|| panic!("missing {kind} owned by paragraph {pi}"))
}

fn text_of(all: &[&Value], pi: u64) -> String {
    all.iter()
        .filter(|n| n["type"] == "TextRun" && n["pi"].as_u64() == Some(pi))
        .filter_map(|n| n["text"].as_str())
        .collect()
}

fn tac_pair(sample: &str, same_row: bool) {
    let tree = page(sample);
    let all = nodes(&tree);
    let tables: Vec<_> = all
        .iter()
        .copied()
        .filter(|n| n["type"] == "Table")
        .collect();
    assert_eq!(tables.len(), 2, "both TACs must belong to the single page");
    assert_eq!(tables[0]["pi"], 0);
    assert_eq!(tables[1]["pi"], 0);
    assert_eq!(tables[0]["ci"], 2);
    assert_eq!(tables[1]["ci"], 3);
    if same_row {
        assert!((coord(tables[0], "y") - coord(tables[1], "y")).abs() < 1.0);
        assert!(coord(tables[1], "x") >= coord(tables[0], "x") + coord(tables[0], "w"));
    } else {
        assert!(
            coord(tables[1], "y") + 0.5 >= bottom(tables[0]),
            "second TAC must own the next row"
        );
        let column = all.iter().find(|n| n["type"] == "Column").unwrap();
        for table in &tables {
            assert!(
                (coord(table, "x") + coord(table, "w") / 2.0
                    - coord(column, "x")
                    - coord(column, "w") / 2.0)
                    .abs()
                    < 0.5,
                "consumed separator spaces must not displace centered table ink"
            );
        }
    }
    let follower = find(&all, "TextLine", 1);
    assert!(tables
        .iter()
        .all(|n| coord(follower, "y") + 0.5 >= bottom(n)));
    assert_eq!(text_of(&all, 1).trim(), "TAC 대조 문단");
}

#[test]
fn fitting_tacs_share_one_row() {
    tac_pair("tac-one-row.hwpx", true);
}

#[test]
fn insufficient_width_moves_second_tac_to_centered_next_row() {
    tac_pair("tac-two-rows.hwpx", false);
}

#[test]
fn authored_break_moves_second_tac_even_when_both_fit() {
    tac_pair("tac-explicit-break.hwpx", false);
}

#[test]
fn overlay_anchor_belongs_to_spacer_row_after_tac() {
    let tree = page("original-no-ls-overlay-tac.hwp");
    let all = nodes(&tree);
    let tables: Vec<_> = all
        .iter()
        .copied()
        .filter(|n| n["type"] == "Table" && n["pi"] == 1)
        .collect();
    assert_eq!(tables.len(), 2);
    assert_eq!(tables[0]["ci"], 0);
    assert_eq!(tables[1]["ci"], 1);
    assert!(
        coord(tables[1], "y") >= bottom(tables[0]),
        "overlay belongs to spacer row after TAC"
    );
}

#[test]
fn empty_picture_host_reserves_its_own_line() {
    let tree = page("original-no-ls-overlay-tac.hwp");
    let all = nodes(&tree);
    let empty = find(&all, "TextLine", 2);
    let spaces = find(&all, "TextLine", 3);
    let image = find(&all, "Image", 4);
    let footer = find(&all, "TextLine", 5);
    let advance = coord(spaces, "y") - coord(empty, "y");
    assert!(advance > coord(empty, "h"));
    assert!(
        (coord(footer, "y") - coord(spaces, "y") - 2.0 * advance).abs() < 0.5,
        "equal-style empty, spaces and InFront image-host paragraphs each reserve a line"
    );
    assert!(coord(image, "y") > coord(spaces, "y"));
    assert_eq!(text_of(&all, 5).trim(), "썬콤 뀀쑵템픽꼇콕놘됨");
}

#[test]
fn tac_bottom_margin_advances_followers_once() {
    let normal = page("margin284.hwpx");
    let wider = page("margin1034.hwpx");
    let normal_nodes = nodes(&normal);
    let wider_nodes = nodes(&wider);
    let t0 = find(&normal_nodes, "Table", 1);
    let t1 = find(&wider_nodes, "Table", 1);
    assert_eq!(t0["bbox"], t1["bbox"], "only bottom outer margin changed");
    for pi in [2, 3, 5] {
        let before = find(&normal_nodes, "TextLine", pi);
        let after = find(&wider_nodes, "TextLine", pi);
        // Independent PDFs and the input's +750 HWPUNIT at 96dpi establish +10px.
        assert!((coord(after, "y") - coord(before, "y") - 10.0).abs() < 0.2);
    }
}

#[test]
fn square_host_rows_restore_full_width() {
    let tree = page("long-boundary-8.hwpx");
    let all = nodes(&tree);
    let square = find(&all, "Table", 0);
    let lines: Vec<_> = all
        .iter()
        .copied()
        .filter(|line| {
            line["type"] == "TextLine"
                && line["pi"] == 0
                && all.iter().any(|run| {
                    run["type"] == "TextRun"
                        && run["pi"] == 0
                        && run["text"].as_str().is_some_and(|t| !t.trim().is_empty())
                        && (coord(run, "y") - coord(line, "y")).abs() < 0.1
                        && coord(run, "x") >= coord(line, "x") - 0.1
                        && coord(run, "x") < coord(line, "x") + coord(line, "w")
                })
        })
        .collect();
    assert_eq!(
        lines.len(),
        3,
        "independent PDF has three visible host rows"
    );
    for line in &lines[..2] {
        assert!(coord(line, "x") + coord(line, "w") <= coord(square, "x"));
    }
    assert!(coord(lines[2], "y") >= bottom(square));
    assert!(coord(lines[2], "w") > coord(lines[0], "w"));
    assert_eq!(text_of(&all, 0).matches("문단 배치 검증").count(), 8);
}

#[test]
fn long_square_host_advances_following_paragraph() {
    let tree = page("long-boundary-8.hwpx");
    let all = nodes(&tree);
    let follower = find(&all, "TextLine", 1);
    for run in all.iter().filter(|run| {
        run["type"] == "TextRun"
            && run["pi"] == 0
            && run["text"].as_str().is_some_and(|t| !t.trim().is_empty())
    }) {
        assert!(
            coord(follower, "y") >= bottom(run),
            "follower must follow all host content"
        );
    }
    assert_eq!(text_of(&all, 1).trim(), "후속 문단 경계");
}

#[test]
fn full_width_square_preserves_empty_decoration_host_and_outer_margins() {
    let tree = page("../issue7481/synth_square_host_full_width_table_no_ls.hwp");
    let all = nodes(&tree);
    let column = all.iter().find(|node| node["type"] == "Column").unwrap();
    let empty = find(&all, "TextLine", 1);
    let square = find(&all, "Table", 2);
    let host = find(&all, "TextLine", 2);
    let follower = find(&all, "Table", 3);
    // Independent saved Hancom rows: the empty decoration host and the next
    // empty paragraph each own a 2000HU line at 160% spacing. The Square
    // host's visible line is below the table's 283HU bottom outer margin.
    assert!(coord(empty, "y") - coord(column, "y") >= coord(empty, "h"));
    assert!(coord(square, "y") >= bottom(empty));
    assert!(coord(host, "y") - bottom(square) > 0.5);
    assert!(coord(follower, "y") >= bottom(host));
    assert_eq!(
        text_of(&all, 2)
            .chars()
            .filter(|c| !c.is_whitespace())
            .count(),
        4
    );
}

fn mixed_tac_rows(sample: &str, same_row: bool) {
    let tree = page(sample);
    let all = nodes(&tree);
    let tables: Vec<_> = all
        .iter()
        .copied()
        .filter(|node| node["type"] == "Table")
        .collect();
    assert_eq!(tables.len(), 2);
    if same_row {
        assert!((coord(tables[0], "y") - coord(tables[1], "y")).abs() < 1.0);
    } else {
        assert!(coord(tables[1], "y") >= bottom(tables[0]));
    }
    let letter = |name: &str| {
        *all.iter()
            .find(|node| {
                node["type"] == "TextRun"
                    && node["pi"] == 0
                    && node["text"]
                        .as_str()
                        .is_some_and(|text| text.trim() == name)
            })
            .unwrap()
    };
    let before = letter("B");
    let between = letter("A");
    let after = letter("C");
    assert!(coord(before, "x") + coord(before, "w") <= coord(tables[0], "x") + 0.5);
    assert!(coord(between, "x") >= coord(tables[0], "x") + coord(tables[0], "w") - 0.5);
    assert!(
        coord(after, "x") >= coord(tables[1], "x") + coord(tables[1], "w") - 0.5,
        "text after the second TAC must not overlap its table"
    );
    assert_eq!(
        text_of(&all, 0)
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .collect::<String>(),
        "BAC"
    );
    assert!(coord(find(&all, "TextLine", 1), "y") >= bottom(tables[1]));
}

#[test]
fn mixed_text_and_two_tacs_preserve_one_row_and_content_order() {
    mixed_tac_rows("tac-mixed-one-row.hwpx", true);
}

#[test]
fn mixed_text_authored_break_moves_second_tac_to_next_row() {
    mixed_tac_rows("tac-mixed-explicit-break.hwpx", false);
}

#[test]
fn word_after_fitting_tac_pair_moves_whole_to_next_row() {
    let tree = page("tac-word-after-pair.hwpx");
    let all = nodes(&tree);
    let tables: Vec<_> = all
        .iter()
        .copied()
        .filter(|node| node["type"] == "Table")
        .collect();
    assert_eq!(tables.len(), 2);
    assert!((coord(tables[0], "y") - coord(tables[1], "y")).abs() < 1.0);
    let letters: Vec<_> = all
        .iter()
        .copied()
        .filter(|node| {
            node["type"] == "TextRun"
                && node["pi"] == 0
                && node["text"]
                    .as_str()
                    .is_some_and(|text| !text.trim().is_empty())
        })
        .collect();
    assert_eq!(
        letters
            .iter()
            .filter_map(|node| node["text"].as_str())
            .collect::<String>(),
        "validation"
    );
    assert!(letters
        .iter()
        .all(|node| coord(node, "y") >= bottom(tables[0]).max(bottom(tables[1]))));
    assert!(letters
        .iter()
        .all(|node| (coord(node, "y") - coord(letters[0], "y")).abs() < 0.1));
    assert!(coord(find(&all, "TextLine", 1), "y") >= bottom(letters[0]));
}
