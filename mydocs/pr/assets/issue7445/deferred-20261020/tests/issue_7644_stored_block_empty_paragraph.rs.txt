//! Original Hancom-saved 56345 HWP and its independent Hancom 2020 PDF:
//! p20 owns the first three nested-cell lines; p21 owns the last line and
//! the following real empty paragraph. Empty text does not erase its line box.
//! Native/fresh Docker WASM p20/p21: 92.69159% / 99.85289% before adding this test.
#![cfg(not(target_arch = "wasm32"))]

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

struct OutputDir(PathBuf);
impl OutputDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "rhwp-7644-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for OutputDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn nodes(node: &Value) -> Vec<&Value> {
    let mut result = vec![node];
    if let Some(children) = node["children"].as_array() {
        for child in children {
            result.extend(nodes(child));
        }
    }
    result
}

fn table(page: &Value, paragraph: u64) -> &Value {
    nodes(page)
        .into_iter()
        .find(|n| n["type"] == "Table" && n["pi"] == paragraph)
        .expect("source table must be present on its owning page")
}

fn last_cell(table: &Value) -> &Value {
    table["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["type"] == "Cell" && n["row"] == 5 && n["col"] == 1)
        .unwrap()
}

fn y(node: &Value) -> f64 {
    node["bbox"]["y"].as_f64().unwrap()
}

fn bottom(node: &Value) -> f64 {
    y(node) + node["bbox"]["h"].as_f64().unwrap()
}

#[test]
fn stored_block_tail_preserves_child_cut_padding_and_the_empty_line_owner() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let input = root.join("samples/issue6111/56345_regulatory_impact_analysis.hwp");
    let source = rhwp::parse_document(&std::fs::read(&input).unwrap()).unwrap();
    let rhwp::model::control::Control::Table(source_table) =
        &source.sections[0].paragraphs[359].controls[0]
    else {
        panic!("original outer table");
    };
    let blank_source = &source_table.cells[11].paragraphs[1];
    assert!(blank_source.text.is_empty() && blank_source.controls.is_empty());
    let expected_line_height = f64::from(blank_source.line_segs[0].line_height) / 75.0;

    let output = OutputDir::new();
    let binary = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
    let status = Command::new(binary)
        .arg("export-render-tree")
        .arg(&input)
        .arg("-o")
        .arg(output.0.as_path())
        .status()
        .unwrap();
    assert!(status.success());
    let read = |page| -> Value {
        serde_json::from_slice(
            &std::fs::read(
                output
                    .0
                    .as_path()
                    .join(format!("render_tree_{page:03}.json")),
            )
            .unwrap(),
        )
        .unwrap()
    };
    let page20 = read(20);
    let page21 = read(21);
    let first = last_cell(table(&page20, 359));
    let continued_table = table(&page21, 359);
    let continued = last_cell(continued_table);
    let child = continued["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["type"] == "Table")
        .unwrap();
    let blank = continued["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["type"] == "TextLine" && n["pi"] == 1)
        .expect("the continuation must own the real empty paragraph");
    // JSON rounds to one decimal place. These relations preserve source owners,
    // physical padding and containment without freezing absolute coordinates.
    assert!(
        y(blank) >= bottom(child) - 0.2,
        "blank precedes child frame end"
    );
    assert!(bottom(blank) <= bottom(continued) + 0.2);
    assert!((blank["bbox"]["h"].as_f64().unwrap() - expected_line_height).abs() < 0.2);
    assert_eq!(
        nodes(first)
            .iter()
            .filter(|n| n["type"] == "TextLine")
            .count(),
        3
    );
    assert_eq!(
        nodes(child)
            .iter()
            .filter(|n| n["type"] == "TextLine")
            .count(),
        1
    );
    assert!(!first["children"]
        .as_array()
        .unwrap()
        .iter()
        .any(|n| n["type"] == "TextLine" && n["pi"] == 1));
    let rhwp::model::control::Control::Table(source_child) =
        &source_table.cells[11].paragraphs[0].controls[1]
    else {
        panic!("source nested block table");
    };
    let actual_text: String = nodes(first)
        .into_iter()
        .chain(nodes(child))
        .filter(|n| n["type"] == "TextRun")
        .map(|n| n["text"].as_str().unwrap())
        .collect();
    let compact = |text: &str| {
        text.chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
    };
    assert_eq!(
        compact(&actual_text),
        compact(&source_child.cells[0].paragraphs[0].text),
        "nested source text must remain ordered and complete across the cut"
    );
    let tail_count = |page: &Value| {
        nodes(page)
            .into_iter()
            .filter(|n| n["type"] == "TextRun" && n["text"] == "수는 없음")
            .count()
    };
    assert_eq!(tail_count(&page20), 0);
    assert_eq!(tail_count(&page21), 1);
    for line in nodes(child).into_iter().filter(|n| n["type"] == "TextLine") {
        assert!(y(line) >= y(child) - 0.2 && bottom(line) <= bottom(child) + 0.2);
    }
    let following = nodes(&page21)
        .into_iter()
        .find(|n| n["type"] == "TextLine" && n["pi"] == 360)
        .unwrap();
    assert!(y(following) >= bottom(continued_table) - 0.2);
    // This saved parent ladder already absorbs its child: it must retain the
    // ordinary empty line inside its original cell, without double reservation.
    let control = last_cell(table(&page20, 347));
    let control_blank = control["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["type"] == "TextLine" && n["pi"] == 1)
        .unwrap();
    assert!(bottom(control_blank) <= bottom(control) + 0.2);
}
