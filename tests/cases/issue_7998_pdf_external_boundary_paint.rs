//! Source and independent expectations: pdf/issue7998/paint-ownership.md.
#![cfg(feature = "pdf-import")]
use rhwp::{
    model::{control::Control, paragraph::Paragraph, table::Table},
    parser::pdf::PdfImportOptions,
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};
use std::{fs, path::Path};
fn tables(c: &DocumentCore) -> Vec<(usize, &Table)> {
    c.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            if let Control::Table(t) = c {
                Some((i, t.as_ref()))
            } else {
                None
            }
        })
        .collect()
}
fn text(p: &[Paragraph]) -> String {
    let mut s = String::new();
    for p in p {
        s.extend(p.text.chars().filter(|c| !c.is_whitespace()));
        for c in &p.controls {
            match c {
                Control::Table(t) => {
                    for c in &t.cells {
                        s += &text(&c.paragraphs)
                    }
                }
                Control::Shape(s2) => {
                    if let Some(t) = s2.drawing().and_then(|d| d.text_box.as_ref()) {
                        s += &text(&t.paragraphs)
                    }
                }
                _ => {}
            }
        }
    }
    s
}
fn geometry(n: &RenderNode, owner: Option<&RenderNode>) {
    if let Some(o) = owner {
        if matches!(n.node_type, RenderNodeType::TextRun(_)) {
            let (a, b) = (n.bbox, o.bbox);
            assert!(
                a.x >= b.x - 0.5
                    && a.y >= b.y - 0.5
                    && a.x + a.width <= b.x + b.width + 0.5
                    && a.y + a.height <= b.y + b.height + 0.5,
                "text escaped its actual cell"
            );
        }
    }
    let owner = if matches!(n.node_type, RenderNodeType::TableCell(_)) {
        Some(n)
    } else {
        owner
    };
    for ch in &n.children {
        geometry(ch, owner)
    }
}
fn red(
    n: &RenderNode,
    inside: bool,
    out: &mut Vec<(rhwp::renderer::render_tree::BoundingBox, bool)>,
) {
    if matches!(&n.node_type,RenderNodeType::Rectangle(r)if r.style.fill_color==Some(255)) {
        out.push((n.bbox, inside));
    }
    for c in &n.children {
        red(
            c,
            inside || matches!(n.node_type, RenderNodeType::TableCell(_)),
            out,
        )
    }
}
fn paint(c: &DocumentCore, key: &str, root: &Path) {
    if key == "independent-roots" {
        return;
    }
    let origins: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("paint-ownership/origin.json")).unwrap())
            .unwrap();
    let r = origins
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["key"] == key)
        .unwrap();
    let a = r["rectanglePdfPt"].as_array().unwrap();
    let (w, h) = (a[2].as_f64().unwrap(), a[3].as_f64().unwrap());
    let tree = c.build_page_render_tree(0).unwrap();
    let mut parts = Vec::new();
    red(&tree.root, false, &mut parts);
    assert!(
        !parts.is_empty(),
        "the original red boundary must remain visible"
    );
    if key == "crossing-right" {
        assert!(parts.iter().any(|(_, cell)| *cell));
        assert!(parts.iter().any(|(_, cell)| !*cell));
    } else {
        assert!(
            parts.iter().all(|(_, cell)| !*cell),
            "fully external paint keeps its original page owner"
        );
    }
    let area = parts.iter().map(|(r, _)| r.width * r.height).sum::<f64>();
    let scale = 96.0 / 72.0;
    let error = (0.02 * (w + h) + 0.0004) * scale * scale;
    assert!(
        (area - w * h * scale * scale).abs() <= error,
        "physical paint must be conserved exactly once: {key} {area} expected {}",
        w * h * scale * scale
    );
    for (i, (a, _)) in parts.iter().enumerate() {
        for (b, _) in &parts[i + 1..] {
            let x = (a.x + a.width).min(b.x + b.width) - a.x.max(b.x);
            let y = (a.y + a.height).min(b.y + b.height) - a.y.max(b.y);
            assert!(x <= 0.02 || y <= 0.02, "red source pieces must not overlap");
        }
    }
}
fn check(c: &DocumentCore, key: &str, root: &Path, edited: bool) {
    assert_eq!(c.page_count(), 1);
    let t = tables(c);
    assert_eq!(
        t.len(),
        if key == "independent-roots" { 2 } else { 1 },
        "every justified editable root must be emitted"
    );
    if key == "independent-roots" {
        let v: serde_json::Value = serde_json::from_slice(
            &fs::read(root.join("local-regions/independent-roots-origin.json")).unwrap(),
        )
        .unwrap();
        let labels = v["expectedLabels"].as_array().unwrap();
        assert_eq!(t[0].1.cells.len(), 18);
        assert_eq!(t[1].1.cells.len(), 19);
        let all = text(&c.document().sections[0].paragraphs);
        for v in labels {
            let label = v["label"].as_str().unwrap();
            assert_eq!(all.matches(label).count(), 1);
            let table = if label.starts_with('C') {
                t[0].1
            } else {
                t[1].1
            };
            let cell = table
                .cells
                .iter()
                .find(|c| {
                    c.row == v["row"].as_u64().unwrap() as u16
                        && c.col == v["column"].as_u64().unwrap() as u16
                })
                .unwrap();
            assert_eq!(
                (cell.row_span, cell.col_span),
                (
                    v["rows"].as_u64().unwrap() as u16,
                    v["columns"].as_u64().unwrap() as u16
                )
            );
            let s = text(&cell.paragraphs);
            assert_eq!(
                s,
                if edited && label == "L11111" {
                    format!("X{label}")
                } else {
                    label.into()
                }
            );
        }
    } else {
        assert_eq!(t[0].1.cells.len(), 4);
        for (i, c) in t[0].1.cells.iter().enumerate() {
            assert_eq!(
                (c.row, c.col, c.row_span, c.col_span),
                ((i / 2) as u16, (i % 2) as u16, 1, 1)
            );
            let want = format!(
                "{}CELL{}{}",
                if edited && i == 0 { "X" } else { "" },
                i / 2 + 1,
                i % 2 + 1
            );
            assert_eq!(text(&c.paragraphs), want);
        }
    }
    geometry(&c.build_page_render_tree(0).unwrap().root, None);
    paint(c, key, root)
}

fn fixture_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("pdf/issue7998")
}
fn load(key: &str) -> DocumentCore {
    let parent = if key == "independent-roots" {
        "local-regions"
    } else {
        "paint-ownership"
    };
    DocumentCore::from_pdf_bytes(
        &fs::read(fixture_root().join(parent).join(format!("{key}.pdf"))).unwrap(),
        &PdfImportOptions::default(),
    )
    .unwrap()
}
fn roundtrip_and_edit(key: &str) {
    let root = fixture_root();
    let mut c = load(key);
    check(&c, key, &root, false);
    for bytes in [
        c.export_hwp_with_adapter().unwrap(),
        c.export_hwpx_native().unwrap(),
    ] {
        check(
            &DocumentCore::from_bytes(&bytes).unwrap(),
            key,
            &root,
            false,
        );
    }
    let ci = tables(&c)[usize::from(key == "independent-roots")].0;
    c.insert_text_in_cell_native(0, 0, ci, 0, 0, 0, "X")
        .unwrap();
    check(&c, key, &root, true);
    for bytes in [
        c.export_hwp_with_adapter().unwrap(),
        c.export_hwpx_native().unwrap(),
    ] {
        check(&DocumentCore::from_bytes(&bytes).unwrap(), key, &root, true);
    }
    c.delete_text_in_cell_native(0, 0, ci, 0, 0, 0, 1).unwrap();
    check(&c, key, &root, false);
}
#[test]
fn required_external_boundary_keeps_page_owner_and_does_not_cancel_cells() {
    for key in ["outside-right", "outside-bottom"] {
        roundtrip_and_edit(key);
    }
}
#[test]
fn crossing_paint_is_partitioned_once_and_unrelated_paint_stays_external() {
    for key in ["crossing-right", "unrelated-outside"] {
        roundtrip_and_edit(key);
    }
}
#[test]
fn independent_roots_keep_all_cell_spans_content_and_editable_ownership() {
    roundtrip_and_edit("independent-roots");
}
