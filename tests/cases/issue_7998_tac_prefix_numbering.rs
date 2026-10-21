//! Actual source ownership and complete prefix-row occupancy of tac-img-02.
//! The original HWP and independent 2020 Print are qualified on all 66 pages.
//! Per-page Native/fresh WASM evidence: task_m100_7998_stage38.md.
#![cfg(not(target_arch = "wasm32"))]

use serde_json::Value;
use std::{path::Path, process::Command};

fn pages() -> Vec<Value> {
    struct Directory(std::path::PathBuf);
    impl Drop for Directory {
        fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); }
    }
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let dir = Directory(std::env::temp_dir().join(format!("rhwp-7998-prefix-{}-{stamp}", std::process::id())));
    std::fs::create_dir(&dir.0).unwrap();
    let binary = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
    let result = Command::new(binary)
        .arg("export-render-tree")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/tac-img-02.hwp"))
        .args(["--compat", "2022", "-o"]).arg(&dir.0).output().unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let mut files: Vec<_> = std::fs::read_dir(&dir.0).unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|file| file.extension().is_some_and(|e| e == "json")).collect();
    files.sort();
    assert_eq!(files.len(), 66, "independent Print: 66 pages, including empty numbered page26");
    files.iter().map(|file| serde_json::from_slice(&std::fs::read(file).unwrap()).unwrap()).collect()
}

fn walk<'a>(node: &'a Value, out: &mut Vec<&'a Value>) {
    out.push(node);
    if let Some(children) = node["children"].as_array() {
        for child in children { walk(child, out); }
    }
}
fn owned<'a>(page: &'a Value, kind: &str, para: u64) -> &'a Value {
    let mut nodes = Vec::new(); walk(page, &mut nodes);
    let found: Vec<_> = nodes.into_iter().filter(|node| node["type"] == kind && node["pi"] == para).collect();
    assert_eq!(found.len(), 1, "one {kind} owned by source paragraph {para}");
    found[0]
}
fn top(node: &Value) -> f64 { node["bbox"]["y"].as_f64().unwrap() }
fn bottom(node: &Value) -> f64 { top(node) + node["bbox"]["h"].as_f64().unwrap() }
fn number(page: &Value) -> String {
    let footer = page["children"].as_array().unwrap().iter().find(|node| node["type"] == "Footer").unwrap();
    let mut nodes = Vec::new(); walk(footer, &mut nodes);
    nodes.into_iter().filter(|node| node["type"] == "TextRun")
        .filter_map(|node| node["text"].as_str()).collect::<String>().split_whitespace().collect()
}

#[test]
fn a_later_number_restart_does_not_hide_an_active_source_position() {
    let pages = pages();
    assert!(number(&pages[0]).is_empty(), "source cover PageHide remains effective");
    // Page26 is the 100%/100% qualified page. Its independent Print's Gulim
    // baseline is 798pt; the CLI render-tree uses 96dpi and a baseline-ending
    // automatic-number box. This tests compiled source metrics without fonts
    // installed on the CI host.
    let expected_baseline = 798.0 * 96.0 / 72.0;
    assert!((bottom(owned(&pages[25], "TextRun", 0)) - expected_baseline).abs() <= 2.0);
    for (page, expected) in [(25, "-26-"), (44, "-45-"), (52, "-53-"), (60, "-1-"), (61, "-2-")] {
        assert_eq!(number(&pages[page]), expected, "physical page{}", page+1);
    }
}

#[test]
fn stored_prefix_rows_are_not_reclaimed_from_the_next_table() {
    let pages = pages();
    let page = &pages[44];
    let prefix = owned(page, "TextLine", 730);
    let first = owned(page, "Table", 730);
    let spacer = owned(page, "TextLine", 731);
    let next = owned(page, "Table", 732);
    assert!(bottom(prefix) < top(first), "prefix owns its own row");
    assert!(bottom(first) < top(spacer) && bottom(spacer) < top(next),
        "first table, empty successor and next table preserve independent Print order");
    let page = &pages[52];
    let first = owned(page, "Table", 837);
    let heading = owned(page, "TextLine", 839);
    let next = owned(page, "Table", 841);
    let last = owned(page, "Table", 845);
    assert!(bottom(first) < top(heading) && bottom(heading) < top(next),
        "next table may not cover its preceding heading");
    assert!(bottom(next) < top(last), "following table remains after the preceding table");
}
