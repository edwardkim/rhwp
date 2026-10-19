//! Independent creation/Print and PDF scope controls: pdf/issue7998/local-regions.md.
#![cfg(feature = "pdf-import")]
use rhwp::{
    model::{control::Control, paragraph::Paragraph, style::BorderLineType, table::Table},
    parser::pdf::PdfImportOptions,
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};
type Address = (u16, u16, u16, u16);
fn load(key: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("pdf/issue7998/local-regions")
        .join(format!("{key}.pdf"));
    DocumentCore::from_pdf_bytes(&std::fs::read(path).unwrap(), &PdfImportOptions::default())
        .unwrap()
}
fn root(c: &DocumentCore) -> (usize, &Table) {
    c.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .enumerate()
        .find_map(|(i, v)| {
            if let Control::Table(t) = v {
                Some((i, t.as_ref()))
            } else {
                None
            }
        })
        .expect("editable Table required")
}
fn text(paras: &[Paragraph], out: &mut String) {
    for p in paras {
        out.extend(p.text.chars().filter(|c| !c.is_whitespace()));
        for control in &p.controls {
            match control {
                Control::Table(t) => {
                    for cell in &t.cells {
                        text(&cell.paragraphs, out);
                    }
                }
                Control::Shape(s) => {
                    if let Some(boxed) = s.drawing().and_then(|d| d.text_box.as_ref()) {
                        text(&boxed.paragraphs, out);
                    }
                }
                _ => {}
            }
        }
    }
}
fn contents(c: &DocumentCore) -> String {
    let mut out = String::new();
    text(&c.document().sections[0].paragraphs, &mut out);
    out
}
fn addresses(t: &Table) -> Vec<Address> {
    t.cells
        .iter()
        .map(|c| (c.row, c.col, c.row_span, c.col_span))
        .collect()
}
fn geometry(n: &RenderNode, cell: Option<&RenderNode>) -> usize {
    if let Some(cell) = cell {
        if matches!(n.node_type, RenderNodeType::TextRun(_)) {
            let (a, b) = (n.bbox, cell.bbox);
            assert!(
                a.x >= b.x - 0.5
                    && a.y >= b.y - 0.5
                    && a.x + a.width <= b.x + b.width + 0.5
                    && a.y + a.height <= b.y + b.height + 0.5,
                "text must remain inside its actual owning cell"
            );
        }
    }
    let owner = if matches!(n.node_type, RenderNodeType::TableCell(_)) {
        Some(n)
    } else {
        cell
    };
    usize::from(matches!(n.node_type, RenderNodeType::Table(_)))
        + n.children.iter().map(|n| geometry(n, owner)).sum::<usize>()
}
fn expected(key: &str) -> (&'static str, Vec<Address>) {
    match key {
        "merged-blank" | "space-cell" => (
            "MERGEDHEADERLOWERRIGHT",
            vec![(0, 0, 1, 2), (1, 0, 1, 1), (1, 1, 1, 1)],
        ),
        "linear-columns" => ("CELL11CELL12", vec![(0, 0, 1, 1), (0, 1, 1, 1)]),
        "linear-rows" => ("CELL11CELL21", vec![(0, 0, 1, 1), (1, 0, 1, 1)]),
        _ => (
            "CELL11CELL12CELL21CELL22",
            vec![(0, 0, 1, 1), (0, 1, 1, 1), (1, 0, 1, 1), (1, 1, 1, 1)],
        ),
    }
}
fn verify(c: &DocumentCore, want: &str, addrs: &[Address]) {
    assert_eq!(c.page_count(), 1);
    assert_eq!(
        contents(c),
        want,
        "source order and complete unique ownership"
    );
    assert_eq!(
        addresses(root(c).1),
        addrs,
        "cell regions and physical spans"
    );
    assert_eq!(
        geometry(&c.build_page_render_tree(0).unwrap().root, None),
        1
    );
}
fn roundtrip(c: &DocumentCore, want: &str, addrs: &[Address]) {
    verify(c, want, addrs);
    for bytes in [
        c.export_hwp_with_adapter().unwrap(),
        c.export_hwpx_native().unwrap(),
    ] {
        verify(&DocumentCore::from_bytes(&bytes).unwrap(), want, addrs);
    }
}
#[test]
fn active_content_axes_recover_separate_cells_without_inventing_ink() {
    for key in [
        "local-columns",
        "local-outer-only",
        "local-l-shape",
        "local-form",
        "linear-columns",
        "linear-rows",
        "merged-blank",
    ] {
        let c = load(key);
        let (want, addrs) = expected(key);
        roundtrip(&c, want, &addrs);
        // Hidden internal ink is not restored as a solid native border.
        if key == "local-outer-only" {
            let t = root(&c).1;
            for (i, side) in [
                (0, 1),
                (0, 3),
                (1, 0),
                (1, 3),
                (2, 1),
                (2, 2),
                (3, 0),
                (3, 2),
            ] {
                let id = usize::from(t.cells[i].border_fill_id) - 1;
                assert_eq!(
                    c.document().doc_info.border_fills[id].borders[side].line_type,
                    BorderLineType::None
                );
            }
        }
    }
}
#[test]
fn absent_blank_boundaries_and_dead_or_partial_scopes_preserve_positional_content() {
    for (key, want) in [
        ("empty-merged-row", "CELL11CELL12CELL31CELL32"),
        ("hidden-blank", "CELL11CELL21CELL22"),
        ("local-blank", "CELL11CELL12CELL22"),
        ("local-missing-clip", "CELL11CELL12CELL21CELL22"),
        ("local-restored-clip", "CELL11CELL12CELL21CELL22"),
        ("local-partial-clip", "CELL11CELL12CELL21CELL22"),
        ("local-open-exterior", "CELL11CELL12CELL21CELL22"),
    ] {
        let c = load(key);
        let original = contents(&c);
        for token in want.as_bytes().chunks(6) {
            let token = std::str::from_utf8(token).unwrap();
            assert_eq!(
                original.matches(token).count(),
                1,
                "{key}: each source label is preserved exactly once"
            );
        }
        let check = |c: &DocumentCore| {
            assert_eq!(c.page_count(), 1);
            assert_eq!(
                contents(c),
                original,
                "{key}: preserve the source's positional content order"
            );
            assert_eq!(
                geometry(&c.build_page_render_tree(0).unwrap().root, None),
                0,
                "{key}: uncertain topology is not emitted"
            );
        };
        check(&c);
        for bytes in [
            c.export_hwp_with_adapter().unwrap(),
            c.export_hwpx_native().unwrap(),
        ] {
            check(&DocumentCore::from_bytes(&bytes).unwrap());
        }
    }
}
#[test]
fn positive_blank_enclosure_and_explicit_space_are_different_paragraph_content() {
    for key in ["merged-blank", "space-cell"] {
        let mut c = load(key);
        let (want, addrs) = expected(key);
        let space = if key == "space-cell" { " " } else { "" };
        let check = |c: &DocumentCore, expected: &str| {
            assert_eq!(
                root(c).1.cells[1]
                    .paragraphs
                    .iter()
                    .map(|p| p.text.as_str())
                    .collect::<String>(),
                expected,
                "intentional space is content, not an absent cell"
            );
        };
        roundtrip(&c, want, &addrs);
        check(&c, space);
        for bytes in [
            c.export_hwp_with_adapter().unwrap(),
            c.export_hwpx_native().unwrap(),
        ] {
            check(&DocumentCore::from_bytes(&bytes).unwrap(), space);
        }
        let ci = root(&c).0;
        c.insert_text_in_cell_native(0, 0, ci, 1, 0, 0, "X")
            .unwrap();
        for bytes in [
            c.export_hwp_with_adapter().unwrap(),
            c.export_hwpx_native().unwrap(),
        ] {
            let n = DocumentCore::from_bytes(&bytes).unwrap();
            check(&n, &format!("X{space}"));
            assert_eq!(addresses(root(&n).1), addrs);
        }
        c.delete_text_in_cell_native(0, 0, ci, 1, 0, 0, 1).unwrap();
        check(&c, space);
        roundtrip(&c, want, &addrs);
    }
}
#[test]
fn character_and_axis_edits_keep_local_cell_owners_and_order_in_both_formats() {
    for key in [
        "local-outer-only",
        "local-l-shape",
        "merged-blank",
        "linear-columns",
        "linear-rows",
    ] {
        let mut c = load(key);
        let (want, addrs) = expected(key);
        let ci = root(&c).0;
        c.insert_text_in_cell_native(0, 0, ci, 0, 0, 0, "X")
            .unwrap();
        roundtrip(&c, &format!("X{want}"), &addrs);
        c.delete_text_in_cell_native(0, 0, ci, 0, 0, 0, 1).unwrap();
        roundtrip(&c, want, &addrs);
        for row in [true, false] {
            let mut c = load(key);
            let ci = root(&c).0;
            let (rows, cols) = (root(&c).1.row_count, root(&c).1.col_count);
            let before_hwp = c.export_hwp_native().unwrap();
            let before_hwpx = c.export_hwpx_native().unwrap();
            let before_tree = serde_json::to_value(c.build_page_render_tree(0).unwrap()).unwrap();
            let result = if row {
                c.insert_table_row_native(0, 0, ci, 0, true)
            } else {
                c.insert_table_column_native(0, 0, ci, 0, true)
            };
            // Content/clip evidence proves these cells, but an incomplete
            // source separator does not authorize inventing structural bands.
            if key == "local-l-shape" {
                assert!(result.unwrap_err().to_string().contains("동등성"));
                assert_eq!(c.export_hwp_native().unwrap(), before_hwp);
                assert_eq!(c.export_hwpx_native().unwrap(), before_hwpx);
                assert_eq!(
                    serde_json::to_value(c.build_page_render_tree(0).unwrap()).unwrap(),
                    before_tree
                );
                roundtrip(&c, want, &addrs);
                continue;
            }
            result.unwrap();
            assert_eq!(
                (root(&c).1.row_count, root(&c).1.col_count),
                if row {
                    (rows + 1, cols)
                } else {
                    (rows, cols + 1)
                }
            );
            let changed = addresses(root(&c).1);
            roundtrip(&c, want, &changed);
            if row {
                c.delete_table_row_native(0, 0, ci, 1).unwrap();
            } else {
                c.delete_table_column_native(0, 0, ci, 1).unwrap();
            }
            roundtrip(&c, want, &addrs);
        }
    }
}
#[test]
fn a_failed_paint_family_does_not_cancel_an_independent_editable_table() {
    let origin: serde_json::Value = serde_json::from_slice(
        &std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("pdf/issue7998/local-regions/independent-roots-origin.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let labels = origin["expectedLabels"].as_array().unwrap();
    let check = |c: &DocumentCore, edited: bool| {
        assert_eq!(c.page_count(), 1);
        let all = contents(c);
        let t = root(c).1;
        assert_eq!(
            t.cells.len(),
            18,
            "the independent upper table must survive a failed sibling"
        );
        for (i, v) in labels
            .iter()
            .filter(|v| v["label"].as_str().unwrap().starts_with('C'))
            .enumerate()
        {
            let mut s = String::new();
            text(&t.cells[i].paragraphs, &mut s);
            let label = v["label"].as_str().unwrap();
            assert_eq!(
                s,
                if edited && i == 0 {
                    format!("X{label}")
                } else {
                    label.to_owned()
                }
            );
            assert_eq!(
                (t.cells[i].row, t.cells[i].col),
                (
                    v["row"].as_u64().unwrap() as u16,
                    v["column"].as_u64().unwrap() as u16
                )
            );
        }
        for v in labels {
            assert_eq!(
                all.matches(v["label"].as_str().unwrap()).count(),
                1,
                "unique complete source ownership"
            );
        }
        assert_eq!(
            all.chars().filter(|c| *c == 'X').count(),
            usize::from(edited)
        );
        geometry(&c.build_page_render_tree(0).unwrap().root, None);
    };
    let mut c = load("independent-roots");
    check(&c, false);
    for bytes in [
        c.export_hwp_with_adapter().unwrap(),
        c.export_hwpx_native().unwrap(),
    ] {
        check(&DocumentCore::from_bytes(&bytes).unwrap(), false);
    }
    let ci = root(&c).0;
    c.insert_text_in_cell_native(0, 0, ci, 0, 0, 0, "X")
        .unwrap();
    check(&c, true);
    for bytes in [
        c.export_hwp_with_adapter().unwrap(),
        c.export_hwpx_native().unwrap(),
    ] {
        check(&DocumentCore::from_bytes(&bytes).unwrap(), true);
    }
    c.delete_text_in_cell_native(0, 0, ci, 0, 0, 0, 1).unwrap();
    check(&c, false);
}
