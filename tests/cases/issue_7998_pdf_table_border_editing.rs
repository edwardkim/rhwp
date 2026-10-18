//! Independent inputs/Print qualification: pdf/issue7998/native-border-sources.md.
#![cfg(feature = "pdf-import")]
use rhwp::{
    model::{control::Control, paragraph::Paragraph, style::BorderLineType, table::Table},
    parser::pdf::PdfImportOptions,
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};
fn base() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("pdf/issue7998/native-borders")
}
fn root(c: &DocumentCore) -> (usize, &Table) {
    c.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .enumerate()
        .find_map(|(i, c)| {
            if let Control::Table(t) = c {
                Some((i, t.as_ref()))
            } else {
                None
            }
        })
        .unwrap()
}
fn child(t: &Table) -> &Table {
    t.cells
        .iter()
        .flat_map(|c| &c.paragraphs)
        .flat_map(|p| &p.controls)
        .find_map(|c| {
            if let Control::Table(t) = c {
                Some(t.as_ref())
            } else {
                None
            }
        })
        .unwrap()
}
fn full(c: &DocumentCore, t: &Table) -> bool {
    t.cells.iter().all(|cell| {
        c.document().doc_info.border_fills[usize::from(cell.border_fill_id) - 1]
            .borders
            .iter()
            .all(|b| b.line_type == BorderLineType::Solid)
    })
}
fn load(key: &str) -> DocumentCore {
    DocumentCore::from_pdf_bytes(
        &std::fs::read(base().join(format!("{key}.pdf"))).unwrap(),
        &PdfImportOptions::default(),
    )
    .unwrap()
}
fn text(paras: &[Paragraph], out: &mut Vec<char>) {
    for p in paras {
        out.extend(p.text.chars());
        for c in &p.controls {
            if let Control::Table(t) = c {
                for c in &t.cells {
                    text(&c.paragraphs, out)
                }
            }
        }
    }
}
fn geometry(n: &RenderNode, cell: Option<(f64, f64, f64, f64)>) {
    let b = n.bbox;
    let own = if matches!(n.node_type, RenderNodeType::TableCell(_)) {
        Some((b.x, b.y, b.width, b.height))
    } else {
        cell
    };
    if matches!(
        n.node_type,
        RenderNodeType::Table(_) | RenderNodeType::TextRun(_) | RenderNodeType::Image(_)
    ) {
        if let Some((x, y, w, h)) = cell {
            assert!(
                b.x >= x - 0.5
                    && b.y >= y - 0.5
                    && b.x + b.width <= x + w + 0.5
                    && b.y + b.height <= y + h + 0.5,
                "content must remain in physical owner: {b:?} vs {cell:?}"
            )
        }
    }
    for c in &n.children {
        geometry(c, own)
    }
}
fn verify(c: &DocumentCore, want: &[char], rows: u16, cols: u16) {
    assert_eq!(c.page_count(), 1);
    let (_, t) = root(c);
    assert_eq!((t.row_count, t.col_count), (rows, cols));
    assert!(full(c, t));
    let mut got = vec![];
    text(&c.document().sections[0].paragraphs, &mut got);
    assert_eq!(got, want, "preserve source text and reading order");
    assert_eq!((child(t).row_count, child(t).col_count), (2, 2));
    geometry(&c.build_page_render_tree(0).unwrap().root, None)
}
#[test]
fn continuous_sides_are_borders_while_child_gaps_retain_source_paint() {
    for key in ["solid-parent", "mixed-parent"] {
        let c = load(key);
        let saved = [
            c.export_hwp_native().unwrap(),
            c.export_hwpx_native().unwrap(),
        ];
        for c in
            std::iter::once(c).chain(saved.iter().map(|v| DocumentCore::from_bytes(v).unwrap()))
        {
            let (_, t) = root(&c);
            assert!(full(&c, t));
            let child = child(t);
            let border =
                &c.document().doc_info.border_fills[child.cells[0].border_fill_id as usize - 1];
            assert_eq!(border.borders[0].line_type, BorderLineType::Solid);
            assert_eq!(border.borders[2].line_type, BorderLineType::Solid);
            assert_eq!(border.borders[1].line_type, BorderLineType::None);
            assert_eq!(border.borders[3].line_type, BorderLineType::None);
            let mut fragments = 0;
            for cell in &child.cells {
                let content = i64::from(cell.height)
                    - i64::from(cell.padding.top)
                    - i64::from(cell.padding.bottom);
                for p in &cell.paragraphs {
                    for ctrl in &p.controls {
                        if let Control::Shape(s) = ctrl {
                            fragments += 1;
                            assert!(
                                s.common().flow_with_text,
                                "cell paint follows its paragraph"
                            );
                            assert!(i64::from(s.common().height) <= content, "continuous full-height border must not become a cell content requirement");
                        }
                    }
                }
            }
            assert!(fragments > 0, "real gaps retain source paint");
            geometry(&c.build_page_render_tree(0).unwrap().root, None);
        }
    }
}
#[test]
fn native_parent_row_column_and_border_edits_keep_nested_content() {
    for key in ["solid-parent", "mixed-parent"] {
        for action in ["row", "column", "border"] {
            let mut c = load(key);
            let mut input = c.document().clone();
            for fonts in &mut input.doc_info.font_faces {
                for font in fonts {
                    font.name = "Arial".into()
                }
            }
            c.set_document(input);
            let mut want = vec![];
            text(&c.document().sections[0].paragraphs, &mut want);
            let ci = root(&c).0;
            match action {
                "row" => {
                    c.insert_table_row_native(0, 0, ci, 0, true).unwrap();
                }
                "column" => {
                    c.insert_table_column_native(0, 0, ci, 0, true).unwrap();
                }
                _ => {
                    c.set_cell_properties_native(0,0,ci,0,r##"{"borderLeft":{"type":1,"width":5,"color":"#d00000"},"borderRight":{"type":1,"width":5,"color":"#d00000"},"borderTop":{"type":1,"width":5,"color":"#d00000"},"borderBottom":{"type":1,"width":5,"color":"#d00000"}}"##).unwrap();
                }
            }
            let (rows, cols) = if action == "row" {
                (3, 2)
            } else if action == "column" {
                (2, 3)
            } else {
                (2, 2)
            };
            verify(&c, &want, rows, cols);
            if action == "border" {
                let b = &c.document().doc_info.border_fills
                    [root(&c).1.cells[0].border_fill_id as usize - 1];
                assert!(b.borders.iter().all(|b| b.color == 0xd0 && b.width == 5));
            }
            for bytes in [
                c.export_hwp_native().unwrap(),
                c.export_hwpx_native().unwrap(),
            ] {
                verify(
                    &DocumentCore::from_bytes(&bytes).unwrap(),
                    &want,
                    rows,
                    cols,
                )
            }
            if action != "border" {
                if action == "row" {
                    c.delete_table_row_native(0, 0, ci, 1).unwrap();
                } else {
                    c.delete_table_column_native(0, 0, ci, 1).unwrap();
                }
                verify(&c, &want, 2, 2);
                for bytes in [
                    c.export_hwp_native().unwrap(),
                    c.export_hwpx_native().unwrap(),
                ] {
                    verify(&DocumentCore::from_bytes(&bytes).unwrap(), &want, 2, 2)
                }
            }
        }
    }
}
fn first_table(n: &RenderNode) -> Option<&RenderNode> {
    if matches!(n.node_type, RenderNodeType::Table(_)) {
        return Some(n);
    }
    n.children.iter().find_map(first_table)
}
fn edges(n: &RenderNode, out: &mut Vec<(f64, f64, f64, f64, u32)>) {
    if let RenderNodeType::Line(l) = &n.node_type {
        if !n.editor_only {
            out.push((l.x1, l.y1, l.x2, l.y2, l.style.color))
        }
    }
    for c in &n.children {
        edges(c, out)
    }
}
fn check_ties(c: &DocumentCore) {
    let tree = c.build_page_render_tree(0).unwrap();
    let root = first_table(&tree.root).unwrap();
    let b = root.bbox;
    let x = b.x + b.width / 2.;
    let y = b.y + b.height / 2.;
    let mut lines = vec![];
    edges(root, &mut lines);
    for (horizontal, at, lo, hi, color) in [
        (false, x, b.y, y, 0xd00000),
        (false, x, y, b.y + b.height, 0xd0),
        (true, y, b.x, x, 0xd00000),
        (true, y, x, b.x + b.width, 0xd0),
    ] {
        let mut length = 0.;
        for &(x1, y1, x2, y2, c) in &lines {
            let (on, a, z) = if horizontal {
                (
                    (y1 - at).abs() < 0.5 && (y2 - at).abs() < 0.5,
                    x1.min(x2),
                    x1.max(x2),
                )
            } else {
                (
                    (x1 - at).abs() < 0.5 && (x2 - at).abs() < 0.5,
                    y1.min(y2),
                    y1.max(y2),
                )
            };
            let span = z.min(hi) - a.max(lo);
            if on && span > 0.5 {
                assert_eq!(c, color, "right/lower cell owns equal-style shared edge");
                length += span;
            }
        }
        assert!(length >= hi - lo - 0.5, "shared edge is not omitted")
    }
}
#[test]
fn equal_styles_use_geometric_owners_independent_of_cell_vector_order() {
    let mut c =
        DocumentCore::from_bytes(&std::fs::read(base().join("border-tie.hwpx")).unwrap()).unwrap();
    check_ties(&c);
    let mut input = c.document().clone();
    let ci = root(&c).0;
    if let Control::Table(t) = &mut input.sections[0].paragraphs[0].controls[ci] {
        t.cells.reverse();
        t.rebuild_grid();
    }
    c.set_document(input);
    check_ties(&c);
    for bytes in [
        c.export_hwp_native().unwrap(),
        c.export_hwpx_native().unwrap(),
    ] {
        check_ties(&DocumentCore::from_bytes(&bytes).unwrap());
    }
}
