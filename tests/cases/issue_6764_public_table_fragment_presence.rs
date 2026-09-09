//! Public-fixture guard: the repaired table must remain visible, not disappear.

use serde_json::Value;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::Command;

struct RenderOutput(PathBuf);

impl Drop for RenderOutput {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn target_tables<'a>(node: &'a Value, found: &mut Vec<&'a Value>) {
    if node["type"] == "Table"
        && node["rows"] == 46
        && node["cols"] == 3
        && node["pi"] == 4
        && node["ci"] == 0
    {
        found.push(node);
    }
    if let Some(children) = node["children"].as_array() {
        for child in children {
            target_tables(child, found);
        }
    }
}

fn visible_text(node: &Value, text: &mut String) {
    if let Some(value) = node["text"].as_str() {
        text.push_str(value);
    }
    if let Some(children) = node["children"].as_array() {
        for child in children {
            visible_text(child, text);
        }
    }
}

#[test]
fn repaired_public_table_keeps_its_leading_rows_inside_the_paper() {
    let sample = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6764/1613000-202200037-air-traffic-controller-cbta.hwp");
    assert!(
        sample.is_file(),
        "the public regression fixture is required"
    );
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let output = RenderOutput(std::env::temp_dir().join(format!(
        "rhwp-issue6764-presence-{}-{nonce}",
        std::process::id()
    )));
    let rhwp_bin =
        std::env::var_os("CARGO_BIN_EXE_rhwp").unwrap_or_else(|| env!("CARGO_BIN_EXE_rhwp").into());
    let result = Command::new(rhwp_bin)
        .arg("export-render-tree")
        .arg(&sample)
        // CLI page indexes are zero-based; this is physical page 183.
        .args(["--page", "182", "--output"])
        .arg(&output.0)
        .output()
        .expect("run the current candidate CLI");
    assert!(
        result.status.success(),
        "render-tree export failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let tree: Value = serde_json::from_slice(
        &std::fs::read(output.0.join("render_tree_183.json"))
            .expect("physical page 183 must exist"),
    )
    .expect("valid render tree");
    let mut tables = Vec::new();
    target_tables(&tree, &mut tables);
    assert_eq!(tables.len(), 1, "the original 46-row table must be present");
    let table = tables[0];
    let rows: BTreeSet<u64> = table["children"]
        .as_array()
        .expect("visible cells")
        .iter()
        .filter(|child| child["type"] == "Cell")
        .map(|child| child["row"].as_u64().expect("source row index"))
        .collect();
    assert!(
        (0..23).all(|row| rows.contains(&row)),
        "the first 23 source rows must not disappear: {rows:?}"
    );
    let mut text = String::new();
    visible_text(table, &mut text);
    for anchor in ["1. 학습 및 동기부여", "2. 팀(Team) 안에서의 상호 작용"] {
        assert!(
            text.contains(anchor),
            "missing visible table content: {anchor}"
        );
    }
    let bbox = &table["bbox"];
    let bottom = bbox["y"].as_f64().expect("table y") + bbox["h"].as_f64().expect("table height");
    let paper_bottom = tree["bbox"]["h"].as_f64().expect("paper height");
    assert!(
        bottom <= paper_bottom + 1.0,
        "the repaired table still leaves the paper: {bottom} > {paper_bottom}"
    );
}
