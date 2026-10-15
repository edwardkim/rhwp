//! Exact-input Hancom Print puts the complete source tail paragraphs on page 8.
//! Verify their owning cell and page, as well as preservation across all pages.
//! No pixel coordinate or renderer-derived expected text is used.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "rhwp-7518-tail-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn rendered_pages(input: &Path) -> Vec<Value> {
    let dir = TempDir::new();
    let bin = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
    let result = Command::new(bin)
        .arg("export-render-tree")
        .arg(input)
        .args(["--compat", "2022", "-o"])
        .arg(&dir.0)
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
    let mut files: Vec<_> = std::fs::read_dir(&dir.0)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    files
        .iter()
        .map(|path| serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap())
        .collect()
}

fn nodes(root: &Value) -> Vec<&Value> {
    let mut result = vec![root];
    if let Some(children) = root["children"].as_array() {
        for child in children {
            result.extend(nodes(child));
        }
    }
    result
}

fn tail_text(page: &Value, para_index: usize) -> String {
    let outer = nodes(page)
        .into_iter()
        .find(|node| node["type"] == "Table" && node["pi"] == 3)
        .unwrap();
    outer["children"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|node| node["type"] == "Cell" && node["row"] == 7 && node["col"] == 1)
        .flat_map(nodes)
        .filter(|node| node["type"] == "TextRun" && node["pi"].as_u64() == Some(para_index as u64))
        .filter_map(|node| node["text"].as_str())
        .collect()
}

fn normalize(text: &str) -> String {
    text.split_whitespace().collect()
}

#[test]
fn complete_tail_paragraphs_keep_their_print_page_and_cell_owner() {
    for family in ["valid_orientation", "valid_generated"] {
        let input = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "tests/fixtures/pr7518_review_page_budget/{family}/nested-mixed-cell.hwp"
        ));
        let source = rhwp::parser::parse_hwp(&std::fs::read(&input).unwrap()).unwrap();
        let source_outer = source.sections[0].paragraphs[3]
            .controls
            .iter()
            .find_map(|control| match control {
                Control::Table(table) => Some(table.as_ref()),
                _ => None,
            })
            .unwrap();
        let source_tail = source_outer
            .cells
            .iter()
            .find(|cell| cell.row == 7 && cell.col == 1)
            .unwrap();
        assert_eq!(source_tail.paragraphs.len(), 14);
        assert!(source_tail.paragraphs[9..12]
            .iter()
            .all(|para| !para.text.trim().is_empty()));
        let pages = rendered_pages(&input);
        // Independent PDFs for both exact inputs have eight pages. Their final
        // page owns all text of source tail paragraphs 9, 10 and 11.
        assert_eq!(pages.len(), 8, "{family}: Print page count");
        for (index, para) in source_tail.paragraphs.iter().enumerate().skip(9) {
            let expected = normalize(&para.text);
            if expected.is_empty() {
                continue;
            }
            let complete = pages
                .iter()
                .map(|page| tail_text(page, index))
                .collect::<String>();
            assert_eq!(
                normalize(&complete),
                expected,
                "{family}: paragraph {index} preservation"
            );
            assert_eq!(
                normalize(&tail_text(&pages[7], index)),
                expected,
                "{family}: complete source paragraph {index} belongs to Print page 8"
            );
        }
    }
}
