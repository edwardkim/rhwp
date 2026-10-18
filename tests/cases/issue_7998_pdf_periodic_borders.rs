//! Independent source operators and Hancom Print: pdf/issue7998/periodic-borders.md.
#![cfg(feature = "pdf-import")]
use rhwp::{
    model::{control::Control, paragraph::Paragraph, style::BorderLineType, table::Table},
    parser::pdf::PdfImportOptions,
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};
fn input(key: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("pdf/issue7998/periodic-borders")
        .join(key)
}
fn source(key: &str) -> DocumentCore {
    DocumentCore::from_pdf_bytes(
        &std::fs::read(input(&format!("{key}.pdf"))).unwrap(),
        &PdfImportOptions::default(),
    )
    .unwrap()
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
fn text(paras: &[Paragraph], out: &mut String) {
    for p in paras {
        out.extend(p.text.chars().filter(|c| !c.is_whitespace()));
        for c in &p.controls {
            match c {
                Control::Table(t) => {
                    for cell in &t.cells {
                        text(&cell.paragraphs, out)
                    }
                }
                Control::Shape(s) => {
                    if let Some(t) = s.drawing().and_then(|d| d.text_box.as_ref()) {
                        text(&t.paragraphs, out)
                    }
                }
                _ => {}
            }
        }
    }
}
fn contents(c: &DocumentCore) -> String {
    let mut s = String::new();
    text(&c.document().sections[0].paragraphs, &mut s);
    s
}
fn dot_sides(c: &DocumentCore, t: &Table) -> usize {
    t.cells
        .iter()
        .flat_map(|cell| {
            &c.document().doc_info.border_fills[usize::from(cell.border_fill_id) - 1].borders
        })
        .filter(|s| s.line_type == BorderLineType::Dot)
        .count()
}
fn owners(n: &RenderNode, cell: Option<&RenderNode>, tables: &mut usize) {
    if matches!(n.node_type, RenderNodeType::Table(_)) {
        *tables += 1;
    }
    if let Some(cell) = cell {
        if matches!(
            n.node_type,
            RenderNodeType::Table(_) | RenderNodeType::TextRun(_)
        ) {
            let a = n.bbox;
            let b = cell.bbox;
            assert!(
                a.x >= b.x - 0.5
                    && a.y >= b.y - 0.5
                    && a.x + a.width <= b.x + b.width + 0.5
                    && a.y + a.height <= b.y + b.height + 0.5,
                "content must remain in its owning cell"
            );
        }
    }
    let cell = if matches!(n.node_type, RenderNodeType::TableCell(_)) {
        Some(n)
    } else {
        cell
    };
    for child in &n.children {
        owners(child, cell, tables)
    }
}
fn verify(c: &DocumentCore, want: &str, rows: u16, cols: u16) {
    assert_eq!(c.page_count(), 1);
    assert_eq!(
        contents(c),
        want,
        "reading order, omissions and duplication"
    );
    assert_eq!((root(c).1.row_count, root(c).1.col_count), (rows, cols));
    let mut tables = 0;
    owners(
        &c.build_page_render_tree(0).unwrap().root,
        None,
        &mut tables,
    );
    assert_eq!(
        tables, 2,
        "the parent and child remain actual editable tables"
    );
    let child = root(c)
        .1
        .cells
        .iter()
        .flat_map(|cell| &cell.paragraphs)
        .flat_map(|p| &p.controls)
        .find_map(|ctrl| match ctrl {
            Control::Table(t) => Some(t.as_ref()),
            _ => None,
        })
        .unwrap();
    assert!(
        child
            .cells
            .iter()
            .all(
                |cell| c.document().doc_info.border_fills[usize::from(cell.border_fill_id) - 1]
                    .borders
                    .iter()
                    .all(|side| side.line_type == BorderLineType::Solid)
            ),
        "editing the parent must preserve the child's independent border properties"
    );
}
fn roundtrip(c: &DocumentCore, rows: u16, cols: u16) {
    for bytes in [
        c.export_hwp_native().unwrap(),
        c.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        verify(&reopened, &contents(c), rows, cols);
        assert_eq!(
            dot_sides(&reopened, root(&reopened).1),
            dot_sides(c, root(c).1)
        );
    }
}
#[test]
fn compatible_periodic_cells_become_native_and_preserve_nested_owners() {
    let c = source("compatible-inner-dot");
    let (_, t) = root(&c);
    assert_eq!(
        dot_sides(&c, t),
        8,
        "four cells share two periodic internal axes"
    );
    let want = contents(&c);
    for s in [
        "OUTSIDEABOVE",
        "BEFORECHILD",
        "AFTERCHILD",
        "UPPERRIGHT",
        "LOWERLEFT",
        "LOWERRIGHT",
        "OUTSIDEAFTER",
    ] {
        assert_eq!(want.matches(s).count(), 1);
    }
    assert!(want.find("BEFORECHILD").unwrap() < want.find("AFTERCHILD").unwrap());
    verify(&c, &want, 2, 2);
    roundtrip(&c, 2, 2);
}
#[test]
fn periodic_style_and_row_column_edits_survive_both_storage_formats() {
    for action in ["row", "column", "style"] {
        let mut c = source("compatible-inner-dot");
        let ci = root(&c).0;
        let want = contents(&c);
        match action {
            "row" => {
                c.insert_table_row_native(0, 0, ci, 0, true).unwrap();
            }
            "column" => {
                c.insert_table_column_native(0, 0, ci, 0, true).unwrap();
            }
            _ => {
                c.set_cell_properties_native(0,0,ci,0,r##"{"borderLeft":{"type":1,"width":5,"color":"#d00000"},"borderRight":{"type":1,"width":5,"color":"#d00000"},"borderTop":{"type":1,"width":5,"color":"#d00000"},"borderBottom":{"type":1,"width":5,"color":"#d00000"}}"##).unwrap();
                let cell = &root(&c).1.cells[0];
                assert!(
                    c.document().doc_info.border_fills[usize::from(cell.border_fill_id) - 1]
                        .borders
                        .iter()
                        .all(|s| s.line_type == BorderLineType::Solid && s.color == 0xd0)
                );
            }
        }
        let (r, col) = match action {
            "row" => (3, 2),
            "column" => (2, 3),
            _ => (2, 2),
        };
        verify(&c, &want, r, col);
        roundtrip(&c, r, col);
        if action != "style" {
            if action == "row" {
                c.delete_table_row_native(0, 0, ci, 1).unwrap();
            } else {
                c.delete_table_column_native(0, 0, ci, 1).unwrap();
            }
            verify(&c, &want, 2, 2);
            assert_eq!(dot_sides(&c, root(&c).1), 8);
            roundtrip(&c, 2, 2);
        }
    }
}
#[test]
fn phase_missing_ink_and_wrong_pen_do_not_authenticate_native_dots() {
    for key in [
        "phase-shifted-dot",
        "underreach-dot",
        "wrong-width-dot",
        "occluded-inner-dot",
    ] {
        let c = source(key);
        assert_eq!(
            dot_sides(&c, root(&c).1),
            0,
            "{key} must retain non-equivalent source paint"
        );
        let want = contents(&c);
        for bytes in [
            c.export_hwp_native().unwrap(),
            c.export_hwpx_native().unwrap(),
        ] {
            let reopened = DocumentCore::from_bytes(&bytes).unwrap();
            assert_eq!(contents(&reopened), want);
            assert_eq!(dot_sides(&reopened, root(&reopened).1), 0);
            assert_eq!(reopened.page_count(), 1);
        }
    }
}
fn patterns(v: &serde_json::Value, out: &mut Vec<Vec<f64>>) {
    match v {
        serde_json::Value::Object(v) => {
            if let Some(v) = v.get("dashPattern") {
                out.push(
                    v.as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_f64().unwrap())
                        .collect(),
                );
            }
            for v in v.values() {
                patterns(v, out)
            }
        }
        serde_json::Value::Array(v) => {
            for v in v {
                patterns(v, out)
            }
        }
        _ => {}
    }
}
#[test]
fn four_print_qualified_widths_reach_the_public_paint_transport() {
    let c =
        DocumentCore::from_bytes(&std::fs::read(input("native-dot-widths.hwpx")).unwrap()).unwrap();
    for bytes in [
        c.export_hwp_native().unwrap(),
        c.export_hwpx_native().unwrap(),
    ] {
        let c = DocumentCore::from_bytes(&bytes).unwrap();
        let v: serde_json::Value =
            serde_json::from_str(&c.get_page_layer_tree_native(0).unwrap()).unwrap();
        let mut got = vec![];
        patterns(&v, &mut got);
        for expected in [[1.12, 1.6], [1.6, 2.4], [2.72, 4.0], [8.32, 12.48]] {
            assert!(
                got.iter().any(|p| p.len() == 2
                    && (p[0] - expected[0]).abs() < 1e-6
                    && (p[1] - expected[1]).abs() < 1e-6),
                "independent 600dpi Print repeat missing: {expected:?}"
            );
        }
    }
}
