//! Independent creation/Print and PDF scope controls: pdf/issue7998/cell-regions.md.
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
        .join("pdf/issue7998/cell-regions")
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
        "unwitnessed-cell" => (
            "MERGEDHEADERLOWERRIGHT",
            vec![(0, 0, 1, 2), (1, 0, 1, 1), (1, 1, 1, 1)],
        ),
        "rowspan" => (
            "MERGEDLEFTUPPERRIGHTLOWERRIGHT",
            vec![(0, 0, 2, 1), (0, 1, 1, 1), (1, 1, 1, 1)],
        ),
        "hidden-separate" => (
            "UPPERLEFTUPPERRIGHTLOWERLEFTLOWERRIGHT",
            vec![(0, 0, 1, 1), (0, 1, 1, 1), (1, 0, 1, 1), (1, 1, 1, 1)],
        ),
        _ => (
            "MERGEDHEADERLOWERLEFTLOWERRIGHT",
            vec![(0, 0, 1, 2), (1, 0, 1, 1), (1, 1, 1, 1)],
        ),
    }
}
fn verify(c: &DocumentCore, want: &str, addrs: &[Address]) {
    assert_eq!(c.page_count(), 1);
    assert_eq!(
        contents(c),
        want,
        "content order, missing or duplicate characters"
    );
    assert_eq!(
        addresses(root(c).1),
        addrs,
        "cell ownership and physical spans"
    );
    assert_eq!(
        geometry(&c.build_page_render_tree(0).unwrap().root, None),
        1
    );
}
fn roundtrip(c: &DocumentCore, want: &str, addrs: &[Address]) {
    verify(c, want, addrs);
    for bytes in [
        c.export_hwp_native().unwrap(),
        c.export_hwpx_native().unwrap(),
    ] {
        verify(&DocumentCore::from_bytes(&bytes).unwrap(), want, addrs);
    }
}
#[test]
fn merged_cells_follow_live_content_regions_through_both_save_formats() {
    for key in [
        "colspan",
        "rowspan",
        "form-transformed",
        "form-shadowed-font",
        "unwitnessed-cell",
    ] {
        let c = load(key);
        let (want, addrs) = expected(key);
        roundtrip(&c, want, &addrs);
    }
}
#[test]
fn missing_shared_ink_does_not_merge_or_repaint_separate_cells() {
    fn cell_at(n: &RenderNode, row: u16, col: u16) -> Option<&RenderNode> {
        if matches!(&n.node_type, RenderNodeType::TableCell(cell) if cell.row==row && cell.col==col)
        {
            return Some(n);
        }
        n.children.iter().find_map(|n| cell_at(n, row, col))
    }
    fn ink_at(n: &RenderNode, cell: &RenderNode, x: f64, full: bool) -> bool {
        let a = n.bbox;
        let b = cell.bbox;
        let ink = matches!(&n.node_type,RenderNodeType::Rectangle(r) if r.style.fill_color==Some(0) && r.style.opacity>0.)
            && a.width < a.height
            && a.x <= x + 0.5
            && a.x + a.width >= x - 0.5
            && if full {
                a.y <= b.y + 0.5 && a.y + a.height >= b.y + b.height - 0.5
            } else {
                a.y < b.y + b.height - 0.5 && a.y + a.height > b.y + 0.5
            };
        ink || n.children.iter().any(|n| ink_at(n, cell, x, full))
    }
    let verify_ink = |c: &DocumentCore| {
        let t = root(c).1;
        let border = |cell: usize, side: usize| {
            c.document().doc_info.border_fills[usize::from(t.cells[cell].border_fill_id) - 1]
                .borders[side]
                .line_type
        };
        assert_eq!(border(0, 1), BorderLineType::None);
        assert_eq!(border(1, 0), BorderLineType::None);
        let tree = c.build_page_render_tree(0).unwrap();
        let upper_left = cell_at(&tree.root, 0, 0).unwrap();
        let upper_right = cell_at(&tree.root, 0, 1).unwrap();
        let lower_left = cell_at(&tree.root, 1, 0).unwrap();
        let lower_right = cell_at(&tree.root, 1, 1).unwrap();
        let x = upper_right.bbox.x;
        assert!(
            !ink_at(upper_left, upper_left, x, false)
                && !ink_at(upper_right, upper_right, x, false),
            "an absent separator must stay unpainted"
        );
        // A painted separator may be native BorderFill or cell-owned original
        // paint. The independent Print establishes ink, not its IR storage kind.
        assert!(
            border(2, 1) == BorderLineType::Solid
                || border(3, 0) == BorderLineType::Solid
                || ink_at(lower_left, lower_left, x, true)
                || ink_at(lower_right, lower_right, x, true),
            "the lower separator must span its owning row"
        );
    };
    let c = load("hidden-separate");
    let (want, addrs) = expected("hidden-separate");
    roundtrip(&c, want, &addrs);
    verify_ink(&c);
    for bytes in [
        c.export_hwp_native().unwrap(),
        c.export_hwpx_native().unwrap(),
    ] {
        verify_ink(&DocumentCore::from_bytes(&bytes).unwrap());
    }
}
#[test]
fn missing_restored_partial_conflicting_or_empty_scopes_remain_positional() {
    for key in [
        "missing-clip",
        "restored-clip",
        "partial-clip",
        "nonrectangular-clip",
        "competing-scopes",
        "painted-separator-conflict",
    ] {
        let c = load(key);
        assert_eq!(
            geometry(&c.build_page_render_tree(0).unwrap().root, None),
            0,
            "{key}"
        );
        let want = "MERGEDHEADERLOWERLEFTLOWERRIGHT";
        assert_eq!(
            contents(&c),
            want,
            "positional fallback must preserve all source content"
        );
    }
}
#[test]
fn character_and_axis_edits_preserve_merged_content_and_owners() {
    for key in ["colspan", "rowspan", "hidden-separate"] {
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
            if row {
                c.insert_table_row_native(0, 0, ci, 0, true).unwrap();
            } else {
                c.insert_table_column_native(0, 0, ci, 0, true).unwrap();
            }
            let t = root(&c).1;
            assert_eq!(
                (t.row_count, t.col_count),
                if row { (3, 2) } else { (2, 3) }
            );
            let edited = addresses(t);
            roundtrip(&c, want, &edited);
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
fn form_aliases_and_postscript_names_keep_the_real_font_family() {
    for key in ["colspan", "form-transformed", "form-shadowed-font"] {
        let c = load(key);
        let fonts = &c.document().doc_info.font_faces[0];
        assert!(
            fonts.iter().any(|f| f.name == "Arial"),
            "OpenType nameID 1 family must reach IR"
        );
        assert!(
            !fonts.iter().any(|f| f.name == "F1" || f.name == "ArialMT"),
            "a resource/PostScript identifier is not a family"
        );
    }
}
