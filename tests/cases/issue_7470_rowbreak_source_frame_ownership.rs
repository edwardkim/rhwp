//! Original RowBreak source-frame ownership, independently checked against
//! pdf/rowbreak-problem-pages-hwp-2024.pdf (18 pages, Native/fresh WASM).
//! Assertions preserve content owners and source padding/line geometry rather
//! than pinning the provisional SVG or absolute document pixel positions.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{document_core::DocumentCore, model::control::Control};
use serde_json::Value;
use std::{path::Path, process::Command, sync::OnceLock};

const SAMPLE: &str = "samples/rowbreak-problem-pages.hwp";

struct TreeDirectory(std::path::PathBuf);

impl Drop for TreeDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn pages() -> &'static Vec<Value> {
    static PAGES: OnceLock<Vec<Value>> = OnceLock::new();
    PAGES.get_or_init(|| {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let out = TreeDirectory(
            std::env::temp_dir().join(format!("rhwp-rowbreak-tree-{}-{stamp}", std::process::id())),
        );
        std::fs::create_dir(&out.0).expect("temporary render tree directory");
        let bin = std::env::var("CARGO_BIN_EXE_rhwp")
            .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
        let result = Command::new(bin)
            .arg("export-render-tree")
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
            .arg("-o")
            .arg(&out.0)
            .output()
            .expect("render original sample");
        assert!(
            result.status.success(),
            "render failed: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        let mut files: Vec<_> = std::fs::read_dir(&out.0)
            .unwrap()
            .map(|f| f.unwrap().path())
            .filter(|p| p.extension().is_some_and(|s| s == "json"))
            .collect();
        files.sort();
        files
            .into_iter()
            .map(|p| serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap())
            .collect()
    })
}

fn walk<'a>(n: &'a Value, cell: Option<&'a Value>, out: &mut Vec<(&'a Value, Option<&'a Value>)>) {
    let cell = if n["type"] == "Cell" { Some(n) } else { cell };
    out.push((n, cell));
    if let Some(children) = n["children"].as_array() {
        for child in children {
            walk(child, cell, out);
        }
    }
}

fn nodes(page: &Value) -> Vec<(&Value, Option<&Value>)> {
    let mut out = Vec::new();
    walk(page, None, &mut out);
    out
}

fn top(n: &Value) -> f64 {
    n["bbox"]["y"].as_f64().unwrap()
}
fn bottom(n: &Value) -> f64 {
    top(n) + n["bbox"]["h"].as_f64().unwrap()
}
fn line_text(n: &Value) -> String {
    n["children"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|c| c["text"].as_str())
        .collect()
}
fn compact(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}
fn occurrences(needle: &str) -> Vec<(usize, &Value, Option<&Value>)> {
    pages()
        .iter()
        .enumerate()
        .flat_map(|(page, p)| {
            nodes(p)
                .into_iter()
                .filter(|(n, _)| n["type"] == "TextLine")
                .filter(|(n, _)| compact(&line_text(n)).contains(needle))
                .map(move |(n, c)| (page, n, c))
        })
        .collect()
}

#[test]
fn original_page_count_and_cut_labels_keep_their_owners() {
    assert_eq!(pages().len(), 18, "same original PDF has 18 pages");
    for (label, owner) in [
        ("4.2공통표준도메인관리항목", 10),
        ("4.3공통표준도메인정의기준", 11),
    ] {
        let found = occurrences(label);
        assert_eq!(
            found.len(),
            1,
            "{label} must occur once, without losing or replaying a unit"
        );
        assert_eq!(found[0].0, owner, "{label} must keep its PDF page owner");
    }
}

#[test]
fn nested_terminal_line_is_owned_once_inside_the_continuation_cell() {
    let found = occurrences("하여정보를제공하거나정보의제공을매개하는자를말한다.");
    assert_eq!(
        found.len(),
        1,
        "last nested unit must neither disappear nor repeat"
    );
    let (page, line, cell) = found[0];
    assert_eq!(page, 7, "nested terminal sentence belongs to PDF page 8");
    let cell = cell.expect("nested line has a physical cell owner");
    assert!(
        top(line) >= top(cell) - 0.05 && bottom(line) <= bottom(cell) + 0.05,
        "terminal line escapes its source fragment: line {:?}, cell {:?}",
        line["bbox"],
        cell["bbox"]
    );
}

#[test]
fn visible_lines_stay_inside_both_sides_of_the_source_cut() {
    for page in [6, 7, 10, 11] {
        for (line, cell) in nodes(&pages()[page]) {
            if line["type"] != "TextLine" || compact(&line_text(line)).is_empty() {
                continue;
            }
            let Some(cell) = cell else {
                continue;
            };
            assert!(
                top(line) >= top(cell) - 0.05 && bottom(line) <= bottom(cell) + 0.05,
                "page {}, text {:?}: line {:?} escapes cell {:?}",
                page + 1,
                line_text(line),
                line["bbox"],
                cell["bbox"]
            );
        }
    }
}

#[test]
fn first_fragment_keeps_its_blank_band_below_the_last_owned_line() {
    let source = DocumentCore::from_bytes(
        &std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).unwrap(),
    )
    .unwrap();
    let Control::Table(table) = &source.document().sections[1].paragraphs[5].controls[0] else {
        panic!("source table");
    };
    let original = table
        .cells
        .iter()
        .find(|c| c.row == 6 && c.col == 2)
        .unwrap();
    let padding = original.effective_padding(&table.padding);
    let frame = nodes(&pages()[10])
        .into_iter()
        .find(|(n, _)| n["type"] == "Table" && n["pi"] == 5 && n["ci"] == 0)
        .expect("original first physical table frame")
        .0;
    // The original object declares its first physical frame, independently
    // corroborated by the PDF border. Content cuts must not erase its blank band.
    assert!(
        bottom(frame) - top(frame) + 0.1 >= f64::from(table.common.height) / 75.0,
        "first frame lost source-owned physical space: {:?}",
        frame["bbox"]
    );
    let found = occurrences("4.2공통표준도메인관리항목");
    assert_eq!(found.len(), 1);
    let (_, line, owner) = found[0];
    let cell = owner.unwrap();
    assert!(bottom(line) + f64::from(padding.bottom) / 75.0 <= bottom(cell) + 0.05,
        "physical first frame must retain source bottom padding and blank band after 4.2; line {:?}, cell {:?}", line["bbox"], cell["bbox"]);
    let next = occurrences("4.3공통표준도메인정의기준");
    assert_eq!(next.len(), 1);
    assert_eq!(next[0].0, 11, "reserving blank space cannot consume 4.3");
}

#[test]
fn visible_successor_uses_the_original_line_origin_without_readding_lead() {
    let source = DocumentCore::from_bytes(
        &std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).unwrap(),
    )
    .unwrap();
    let original = &source.document().sections[0].paragraphs[12];
    let body = nodes(&pages()[4])
        .into_iter()
        .find(|(n, _)| n["type"] == "Body")
        .unwrap()
        .0;
    let line = nodes(&pages()[4])
        .into_iter()
        .find(|(n, c)| c.is_none() && n["type"] == "TextLine" && n["pi"] == 12)
        .unwrap()
        .0;
    let expected = f64::from(original.line_segs[0].vertical_pos) / 75.0;
    // Stored line origin and normalized PDF baseline agree; no glyph-ink bounds are compared.
    assert!((top(line) - top(body) - expected).abs() <= 0.1,
        "visible successor shares the stored table-end boundary; line {:?}, body {:?}, saved={expected}", line["bbox"], body["bbox"]);
}

#[test]
fn coanchored_tac_does_not_skip_or_overlap_its_float_sibling() {
    let all = nodes(&pages()[15]);
    let tables: Vec<_> = all
        .iter()
        .filter(|(n, _)| n["type"] == "Table" && n["pi"] == 28)
        .map(|(n, _)| *n)
        .collect();
    assert_eq!(
        tables.len(),
        2,
        "TAC commitment must leave the float sibling to the normal control loop"
    );
    let tac = tables
        .iter()
        .find(|n| n["ci"] == 1 && n["rows"] == 3)
        .unwrap();
    let float = tables
        .iter()
        .find(|n| n["ci"] == 0 && n["rows"] == 1)
        .unwrap();
    assert!(
        bottom(tac) <= top(float),
        "source TAC must precede the separate float frame: {:?}, {:?}",
        tac["bbox"],
        float["bbox"]
    );
}
