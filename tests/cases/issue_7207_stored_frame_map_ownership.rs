//! Original source frames checked against pdf/18095317_eogu_geumji-2020.pdf.
//! Native/fresh WASM visual evidence is recorded in task_m100_7207_stage5.md.
//! Page ownership, cell containment and content order are independent of the
//! implementation's declared frame heights or a provisional SVG hash.
#![cfg(not(target_arch = "wasm32"))]

use serde_json::Value;
use std::{path::Path, process::Command, sync::OnceLock};

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
            std::env::temp_dir().join(format!("rhwp-eogu-tree-{}-{stamp}", std::process::id())),
        );
        std::fs::create_dir(&out.0).expect("temporary tree directory");
        let bin = std::env::var("CARGO_BIN_EXE_rhwp")
            .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
        let result = Command::new(bin)
            .arg("export-render-tree")
            .arg(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("samples/task2097/18095317_eogu_geumji.hwp"),
            )
            .arg("-o")
            .arg(&out.0)
            .output()
            .expect("render original source");
        assert!(
            result.status.success(),
            "render failed: {}",
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
    cell: Option<&'a Value>,
    out: &mut Vec<(&'a Value, Option<&'a Value>)>,
) {
    let cell = if node["type"] == "Cell" {
        Some(node)
    } else {
        cell
    };
    out.push((node, cell));
    if let Some(children) = node["children"].as_array() {
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
fn text(node: &Value) -> String {
    let mut text = node["text"].as_str().unwrap_or("").to_owned();
    if let Some(children) = node["children"].as_array() {
        for child in children {
            text.push_str(&self::text(child));
        }
    }
    text.chars().filter(|c| !c.is_whitespace()).collect()
}
fn top(n: &Value) -> f64 {
    n["bbox"]["y"].as_f64().unwrap()
}
fn bottom(n: &Value) -> f64 {
    top(n) + n["bbox"]["h"].as_f64().unwrap()
}
fn unique_line(needle: &str, owner: usize) -> (&'static Value, &'static Value) {
    let found: Vec<_> = pages()
        .iter()
        .enumerate()
        .flat_map(|(page, p)| {
            nodes(p)
                .into_iter()
                .filter(move |(n, _)| n["type"] == "TextLine" && text(n).contains(needle))
                .map(move |(n, c)| (page, n, c))
        })
        .collect();
    assert_eq!(
        found.len(),
        1,
        "{needle}: content must neither disappear nor repeat"
    );
    assert_eq!(found[0].0, owner, "{needle}: independent PDF page owner");
    (found[0].1, found[0].2.expect("physical cell owner"))
}
#[test]
fn original_cut_labels_keep_their_pdf_pages_and_cell_owners() {
    assert_eq!(pages().len(), 21, "same original Hancom PDF has 21 pages");
    // This original declared opening frame is corroborated by the PDF's
    // first-page table frame. It includes blank physical space after the cut;
    // its height must not collapse to the consumed text's ink extent.
    let source = rhwp::document_core::DocumentCore::from_bytes(
        &std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/task2097/18095317_eogu_geumji.hwp"),
        )
        .unwrap(),
    )
    .unwrap();
    let original = source.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .find_map(|c| {
            if let rhwp::model::control::Control::Table(t) = c {
                Some(t)
            } else {
                None
            }
        })
        .expect("original opening table");
    let declared = f64::from(original.common.height) * 96.0 / 7200.0;
    let first_frame = nodes(&pages()[0])
        .into_iter()
        .find(|(n, _)| n["type"] == "Table")
        .unwrap()
        .0;
    assert!(
        (first_frame["bbox"]["h"].as_f64().unwrap() - declared).abs() <= 0.5,
        "original first-frame space must survive the text cut"
    );
    for (label, owner) in [
        ("2)소형선망어업", 1),
        ("다.근해채낚기어업", 2),
        ("마.근해안강망어업", 3),
    ] {
        let (line, cell) = unique_line(label, owner);
        assert!(
            top(line) >= top(cell) - 0.1 && bottom(line) <= bottom(cell) + 0.1,
            "label escapes continuation cell"
        );
    }
}
#[test]
fn notes_follow_the_terminal_fishing_row_on_page_ten() {
    let (last, last_cell) = unique_line("마.모든어업", 9);
    let (note, note_cell) = unique_line("주1)동해구외끌이중형저인망어업", 9);
    assert!(
        top(note) >= bottom(last_cell) - 0.1,
        "notes must follow the actual terminal row"
    );
    assert!(
        bottom(last) <= bottom(last_cell) + 0.1 && top(note) >= top(note_cell) - 0.1,
        "text must retain its cell owner"
    );
    for (n, _) in nodes(&pages()[9]) {
        if n["type"] == "TextLine" && !text(n).is_empty() {
            assert!(
                top(n) >= bottom(last_cell) - 0.1 || bottom(n) <= bottom(last_cell) + 0.1,
                "a note must not straddle the row boundary"
            );
        }
    }
}
#[test]
fn map_continuations_keep_all_six_images_and_caption_order() {
    let source = rhwp::document_core::DocumentCore::from_bytes(
        &std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/task2097/18095317_eogu_geumji.hwp"),
        )
        .unwrap(),
    )
    .unwrap();
    let map_cell = source
        .document()
        .sections
        .iter()
        .flat_map(|section| &section.paragraphs)
        .flat_map(|paragraph| &paragraph.controls)
        .filter_map(|control| match control {
            rhwp::model::control::Control::Table(table) => Some(table),
            _ => None,
        })
        .flat_map(|table| &table.cells)
        .find(|cell| {
            cell.paragraphs.iter().any(|paragraph| {
                paragraph
                    .text
                    .chars()
                    .filter(|ch| !ch.is_whitespace())
                    .collect::<String>()
                    .contains("[부도1]")
            })
        })
        .expect("original map cell");
    assert_eq!(
        map_cell.vertical_align,
        rhwp::model::table::VerticalAlign::Center,
        "the original stored frame owns center alignment"
    );
    let (first_caption, frame) = unique_line("[부도1]", 16);
    // The independent PDF's opening map fragment has a blank band larger
    // than one caption line. A Top fallback collapses that source-owned band.
    assert!(
        top(first_caption) - top(frame) > first_caption["bbox"]["h"].as_f64().unwrap(),
        "the centered opening map fragment must preserve its blank band"
    );
    let mut total = 0;
    for (page, count) in [(16, 1), (17, 2), (18, 1), (19, 1), (20, 1)] {
        let images: Vec<_> = nodes(&pages()[page])
            .into_iter()
            .filter(|(n, _)| n["type"] == "Image")
            .collect();
        assert_eq!(
            images.len(),
            count,
            "independent PDF map image ownership on page {}",
            page + 1
        );
        total += images.len();
        for (image, cell) in &images {
            let cell = cell.expect("map has source cell owner");
            assert!(
                top(image) >= top(cell) - 0.1 && bottom(image) <= bottom(cell) + 0.1,
                "map escapes its continuation cell"
            );
        }
        for pair in images.windows(2) {
            assert!(
                bottom(pair[0].0) <= top(pair[1].0) + 0.1,
                "maps overlap or reorder"
            );
        }
    }
    assert_eq!(total, 6, "all original images must be retained");
    for (caption, page) in [
        ("[부도1]", 16),
        ("[부도2]", 17),
        ("[부도3]", 17),
        ("[부도4]", 18),
        ("[부도5]", 19),
    ] {
        let (line, cell) = unique_line(caption, page);
        assert!(
            top(line) >= top(cell) - 0.1 && bottom(line) <= bottom(cell) + 0.1,
            "caption escapes source cell"
        );
    }
    for page in [17, 18, 19] {
        let last = nodes(&pages()[page])
            .into_iter()
            .filter(|(n, _)| n["type"] == "Image")
            .map(|(n, _)| bottom(n))
            .fold(f64::NEG_INFINITY, f64::max);
        let caption = match page {
            17 => "[부도3]",
            18 => "[부도4]",
            _ => "[부도5]",
        };
        assert!(
            top(unique_line(caption, page).0) >= last - 0.1,
            "following caption must not overlap the map"
        );
    }
}
