//! Original 36308670 and independent Hancom Print PDF (two pages).
//! Native/fresh WASM evidence and before/after controls: pr_7628_review.md.
//! Assert source units, physical ownership, containment and order, without
//! freezing absolute output coordinates or an SVG hash.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{document_core::DocumentCore, model::control::Control};
use serde_json::Value;
use std::{path::Path, process::Command, sync::OnceLock};

const INPUT: &str = "samples/issue7344/original/36308670_seoul_approval.hwpx";
// export-render-tree serializes dimensions with one decimal place. This is
// serialization uncertainty, not a tolerance for shifting a source frame.
const ROUNDING: f64 = 0.05;
const HU_PER_OUTPUT_UNIT: f64 = 7200.0 / 96.0;

struct TreeDirectory(std::path::PathBuf);
impl Drop for TreeDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn source() -> DocumentCore {
    DocumentCore::from_bytes(
        &std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(INPUT)).unwrap(),
    )
    .unwrap()
}
fn pages() -> &'static Vec<Value> {
    static PAGES: OnceLock<Vec<Value>> = OnceLock::new();
    PAGES.get_or_init(|| {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let out = TreeDirectory(
            std::env::temp_dir().join(format!("rhwp-approval-tree-{}-{stamp}", std::process::id())),
        );
        std::fs::create_dir(&out.0).unwrap();
        let bin = std::env::var("CARGO_BIN_EXE_rhwp")
            .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
        let result = Command::new(bin)
            .arg("export-render-tree")
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join(INPUT))
            .arg("-o")
            .arg(&out.0)
            .output()
            .expect("render original approval");
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let mut files: Vec<_> = std::fs::read_dir(&out.0)
            .unwrap()
            .map(|f| f.unwrap().path())
            .filter(|p| {
                p.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("render_tree_")
            })
            .collect();
        files.sort();
        files
            .into_iter()
            .map(|p| serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap())
            .collect()
    })
}
fn walk<'a>(
    node: &'a Value,
    owner: Option<&'a Value>,
    out: &mut Vec<(&'a Value, Option<&'a Value>)>,
) {
    let owner = if node["type"] == "Cell" {
        Some(node)
    } else {
        owner
    };
    out.push((node, owner));
    if let Some(children) = node["children"].as_array() {
        for child in children {
            walk(child, owner, out);
        }
    }
}
fn nodes(node: &Value) -> Vec<(&Value, Option<&Value>)> {
    let mut out = Vec::new();
    walk(node, None, &mut out);
    out
}
fn report(page: usize) -> &'static Value {
    let found: Vec<_> = nodes(&pages()[page])
        .into_iter()
        .filter(|(n, _)| n["type"] == "Table" && n["pi"] == 5 && n["ci"] == 0)
        .collect();
    assert_eq!(found.len(), 1, "one report fragment per PDF page");
    found[0].0
}
fn cell(page: usize, row: u16, col: u16) -> &'static Value {
    let found: Vec<_> = report(page)["children"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["type"] == "Cell" && n["row"] == row && n["col"] == col)
        .collect();
    assert_eq!(found.len(), 1, "one physical cell owner");
    found[0]
}
fn normalized(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}
fn text(node: &Value) -> String {
    let mut value = node["text"].as_str().unwrap_or("").to_owned();
    if let Some(children) = node["children"].as_array() {
        for child in children {
            value.push_str(&text(child));
        }
    }
    normalized(&value)
}
fn top(node: &Value) -> f64 {
    node["bbox"]["y"].as_f64().unwrap()
}
fn height(node: &Value) -> f64 {
    node["bbox"]["h"].as_f64().unwrap()
}
fn bottom(node: &Value) -> f64 {
    top(node) + height(node)
}

#[test]
fn original_approval_keeps_both_page_frames_and_every_body_paragraph() {
    assert_eq!(pages().len(), 2, "independent Print PDF has two pages");
    let original_document = source();
    let Control::Table(table) = &original_document.document().sections[0].paragraphs[5].controls[0]
    else {
        panic!("original report table");
    };
    let body = table
        .cells
        .iter()
        .find(|c| c.row == 4 && c.col == 1)
        .unwrap();
    assert_eq!(body.paragraphs.len(), 19, "original body paragraphs");
    // The PDF page cut is before original paragraph 16, not a measured
    // height inferred from the implementation under test.
    for (page, range) in [(0, 0..16), (1, 16..19)] {
        let expected: String = body.paragraphs[range]
            .iter()
            .map(|p| normalized(&p.text))
            .collect();
        assert_eq!(
            text(cell(page, 4, 1)),
            expected,
            "page owner, order, loss and duplication"
        );
    }
    for col in [0, 1] {
        let original = table
            .cells
            .iter()
            .find(|c| c.row == 4 && c.col == col)
            .unwrap();
        let observed_hu = (height(cell(0, 4, col)) + height(cell(1, 4, col))) * HU_PER_OUTPUT_UNIT;
        let serialization_error_hu = 2.0 * ROUNDING * HU_PER_OUTPUT_UNIT;
        assert!(
            (observed_hu - f64::from(original.height)).abs() <= serialization_error_hu,
            "source row's occupied space must survive the continuation cut"
        );
    }
    for page in [0, 1] {
        let body_cell = cell(page, 4, 1);
        for (line, _) in nodes(body_cell)
            .into_iter()
            .filter(|(n, _)| n["type"] == "TextLine")
        {
            assert!(
                top(line) >= top(body_cell) - 2.0 * ROUNDING
                    && bottom(line) <= bottom(body_cell) + 4.0 * ROUNDING,
                "body line must remain inside its physical fragment"
            );
        }
    }
}

#[test]
fn original_approval_pictures_stay_inside_their_cells_before_captions() {
    assert_eq!(pages().len(), 2);
    assert!(
        !nodes(report(0)).iter().any(|(n, _)| n["type"] == "Image"),
        "PDF photographs start on page two"
    );
    let images: Vec<_> = nodes(report(1))
        .into_iter()
        .filter(|(n, _)| n["type"] == "Image")
        .collect();
    assert_eq!(images.len(), 5, "all five independent PDF photographs");
    for ((image, owner), (row, col)) in images.iter().zip([(5, 1), (5, 2), (5, 3), (5, 4), (6, 1)])
    {
        let owner = owner.expect("picture's actual cell owner");
        assert_eq!(owner["row"], row);
        assert_eq!(owner["col"], col);
        assert!(
            top(image) >= top(owner) - 2.0 * ROUNDING
                && bottom(image) <= bottom(owner) + 4.0 * ROUNDING,
            "photograph escapes its actual physical cell"
        );
    }
    assert!(
        bottom(images[0].0) <= top(images[4].0) + 3.0 * ROUNDING,
        "the two sign photographs retain vertical order"
    );
    for (col, image_indices) in [(1, vec![0, 4]), (2, vec![1]), (3, vec![2, 3])] {
        let caption = cell(1, 7, col);
        assert!(
            !text(caption).is_empty(),
            "original caption remains visible"
        );
        let first_line = nodes(caption)
            .into_iter()
            .filter(|(n, _)| n["type"] == "TextLine")
            .map(|(n, _)| top(n))
            .fold(f64::INFINITY, f64::min);
        assert!(first_line.is_finite(), "caption has an actual text line");
        for index in image_indices {
            assert!(
                bottom(images[index].0) <= first_line + 3.0 * ROUNDING,
                "picture overlaps the following caption"
            );
        }
    }
    let ending: Vec<_> = nodes(&pages()[1])
        .into_iter()
        .filter(|(n, _)| n["type"] == "TextLine" && text(n) == "끝.")
        .collect();
    assert_eq!(
        ending.len(),
        1,
        "final body line is neither lost nor duplicated"
    );
    assert!(
        top(ending[0].0) >= bottom(report(1)) - 3.0 * ROUNDING,
        "final body follows the report"
    );
}
