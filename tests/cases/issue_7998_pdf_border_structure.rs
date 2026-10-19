#![cfg(feature = "pdf-import")]
use rhwp::{
    model::{border_paint, control::Control, style::BorderLineType, table::Table},
    parser::pdf::PdfImportOptions,
    DocumentCore,
};
fn load() -> DocumentCore {
    DocumentCore::from_pdf_bytes(
        &std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("pdf/issue7998/structure-borders/native-pen.pdf"),
        )
        .unwrap(),
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
fn all_borders(c: &DocumentCore) {
    let t = root(c).1;
    assert!(t.cells.iter().all(|cell| c.document().doc_info.border_fills
        [cell.border_fill_id as usize - 1]
        .borders
        .iter()
        .all(|b| b.line_type == BorderLineType::Solid)));
    assert!(border_paint::encode(c.document()).unwrap().is_none());
}
fn text(ps: &[rhwp::model::paragraph::Paragraph]) -> String {
    let mut out = String::new();
    for p in ps {
        out.push_str(&p.text);
        for ctrl in &p.controls {
            if let Control::Table(t) = ctrl {
                for cell in &t.cells {
                    out.push_str(&text(&cell.paragraphs));
                }
            }
            if let Control::Shape(shape) = ctrl {
                if let Some(tb) = shape.drawing().and_then(|d| d.text_box.as_ref()) {
                    out.push_str(&text(&tb.paragraphs));
                }
            }
        }
    }
    out
}
fn child(c: &DocumentCore) -> serde_json::Value {
    root(c)
        .1
        .cells
        .iter()
        .flat_map(|c| &c.paragraphs)
        .flat_map(|p| &p.controls)
        .find_map(|c| {
            if let Control::Table(t) = c {
                Some(serde_json::to_value(t).unwrap())
            } else {
                None
            }
        })
        .unwrap()
}
fn apply(c: &mut DocumentCore, mode: usize) -> Result<String, rhwp::error::HwpError> {
    let ci = root(c).0;
    match mode {
        0 => c.insert_table_row_native(0, 0, ci, 0, false),
        1 => c.insert_table_row_native(0, 0, ci, 0, true),
        2 => c.insert_table_column_native(0, 0, ci, 0, false),
        3 => c.insert_table_column_native(0, 0, ci, 0, true),
        4 => c.delete_table_row_native(0, 0, ci, 1),
        _ => c.delete_table_column_native(0, 0, ci, 1),
    }
}
#[test]
fn inserted_units_inherit_real_borders_without_cloning_text_or_child() {
    for mode in 0..4 {
        let mut c = load();
        let want = text(&c.document().sections[0].paragraphs);
        let nested = child(&c);
        apply(&mut c, mode).unwrap();
        all_borders(&c);
        assert_eq!(text(&c.document().sections[0].paragraphs), want);
        assert_eq!(child(&c), nested);
        let t = root(&c).1;
        assert_eq!(
            (t.row_count, t.col_count),
            if mode < 2 { (3, 2) } else { (2, 3) }
        );
        assert_eq!(t.cells.len(), 6);
        for cell in &t.cells {
            if (mode < 2 && cell.row == if mode == 0 { 0 } else { 1 })
                || (mode >= 2 && cell.col == if mode == 2 { 0 } else { 1 })
            {
                assert!(
                    cell.paragraphs
                        .iter()
                        .all(|p| p.text.is_empty() && p.controls.is_empty()),
                    "only style belongs in a new cell"
                );
            }
        }
    }
}
#[test]
fn deletion_keeps_surviving_content_child_and_complete_edges() {
    for mode in 4..6 {
        let mut c = load();
        let nested = child(&c);
        apply(&mut c, mode).unwrap();
        all_borders(&c);
        let got = text(&c.document().sections[0].paragraphs);
        assert!(
            got.contains("BEFORE CHILD")
                && got.contains("AFTER CHILD")
                && got.contains("OUTSIDE AFTER")
        );
        assert!(!got.contains("LOWER RIGHT"));
        if mode == 4 {
            assert!(!got.contains("LOWER LEFT"));
            assert!(got.contains("UPPER RIGHT"));
        } else {
            assert!(!got.contains("UPPER RIGHT"));
            assert!(got.contains("LOWER LEFT"));
        }
        assert_eq!(child(&c), nested);
        assert_eq!(root(&c).1.cells.len(), 2);
    }
}
#[test]
fn deleting_inserted_units_restores_original_grid_and_content() {
    for mode in 0..4 {
        let mut c = load();
        let want = text(&c.document().sections[0].paragraphs);
        apply(&mut c, mode).unwrap();
        let ci = root(&c).0;
        if mode < 2 {
            c.delete_table_row_native(0, 0, ci, if mode == 0 { 0 } else { 1 })
                .unwrap();
        } else {
            c.delete_table_column_native(0, 0, ci, if mode == 2 { 0 } else { 1 })
                .unwrap();
        }
        all_borders(&c);
        assert_eq!((root(&c).1.row_count, root(&c).1.col_count), (2, 2));
        assert_eq!(text(&c.document().sections[0].paragraphs), want);
    }
}
#[test]
fn both_formats_keep_native_inheritance_and_later_border_editing() {
    for mode in 0..6 {
        let mut c = load();
        apply(&mut c, mode).unwrap();
        let want = text(&c.document().sections[0].paragraphs);
        for bytes in [
            c.export_hwp_with_adapter_snapshot().unwrap(),
            c.export_hwpx_native().unwrap(),
        ] {
            let mut n = DocumentCore::from_bytes(&bytes).unwrap();
            all_borders(&n);
            assert_eq!(text(&n.document().sections[0].paragraphs), want);
            let ci = root(&n).0;
            n.set_cell_properties_native(
                0,
                0,
                ci,
                0,
                r##"{"borderLeft":{"type":1,"width":5,"color":"#d00000"}}"##,
            )
            .unwrap();
            let id = root(&n).1.cells[0].border_fill_id;
            assert_eq!(
                n.document().doc_info.border_fills[id as usize - 1].borders[0].color,
                0xd0
            );
        }
    }
}
#[test]
fn snapshot_undo_redo_restores_exact_source_after_text_inverse() {
    for mode in 0..6 {
        let mut c = load();
        let before = c.save_snapshot_native();
        let original = c.export_hwpx_native().unwrap();
        apply(&mut c, mode).unwrap();
        let after = c.save_snapshot_native();
        let edited = c.export_hwpx_native().unwrap();
        let ci = root(&c).0;
        c.insert_text_in_cell_native(0, 0, ci, 0, 0, 0, "X")
            .unwrap();
        c.delete_text_in_cell_native(0, 0, ci, 0, 0, 0, 1).unwrap();
        c.restore_snapshot_native(before).unwrap();
        assert_eq!(c.export_hwpx_native().unwrap(), original);
        assert!(border_paint::encode(c.document()).unwrap().is_some());
        c.restore_snapshot_native(after).unwrap();
        all_borders(&c);
        assert_eq!(c.export_hwpx_native().unwrap(), edited);
    }
}
#[test]
fn unsupported_pen_phase_and_alpha_reject_without_mutating_document() {
    for key in ["compound-solid", "compound-dash", "compound-alpha"] {
        for mode in 0..6 {
            let bytes = std::fs::read(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join(format!("pdf/issue7998/compound-borders/{key}.pdf")),
            )
            .unwrap();
            let mut c = DocumentCore::from_pdf_bytes(&bytes, &PdfImportOptions::default()).unwrap();
            let before = c.export_hwpx_native().unwrap();
            assert!(apply(&mut c, mode).is_err());
            assert_eq!(c.export_hwpx_native().unwrap(), before);
        }
    }
}
#[test]
fn invalid_model_request_does_not_commit_prepared_border_styles() {
    let mut c = load();
    let before = c.export_hwpx_native().unwrap();
    let ci = root(&c).0;
    assert!(c.insert_table_row_native(0, 0, ci, 99, false).is_err());
    assert!(c.delete_table_column_native(0, 0, ci, 99).is_err());
    assert_eq!(c.export_hwpx_native().unwrap(), before);
}

#[test]
fn native_border_colors_preserve_the_source_rgb_contract() {
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("pdf/issue7998/structure-borders/native-pen-blue.pdf"),
    )
    .unwrap();
    let mut c = DocumentCore::from_pdf_bytes(&bytes, &PdfImportOptions::default()).unwrap();
    apply(&mut c, 1).unwrap();
    for bytes in [
        c.export_hwp_with_adapter_snapshot().unwrap(),
        c.export_hwpx_native().unwrap(),
    ] {
        let n = DocumentCore::from_bytes(&bytes).unwrap();
        for cell in &root(&n).1.cells {
            assert!(
                n.document().doc_info.border_fills[cell.border_fill_id as usize - 1]
                    .borders
                    .iter()
                    .all(|b| b.color == 0x996633)
            );
        }
    }
}

#[test]
fn inherited_borders_are_painted_on_real_cell_frames() {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn first(n: &RenderNode) -> Option<&RenderNode> {
        if matches!(n.node_type, RenderNodeType::Table(_)) {
            return Some(n);
        }
        n.children.iter().find_map(first)
    }
    fn lines(n: &RenderNode, out: &mut Vec<[f64; 4]>) {
        if let RenderNodeType::Line(l) = &n.node_type {
            if !n.editor_only {
                out.push([l.x1, l.y1, l.x2, l.y2]);
            }
        }
        for c in &n.children {
            lines(c, out)
        }
    }
    for mode in 0..6 {
        let mut c = load();
        apply(&mut c, mode).unwrap();
        let tree = c.build_page_render_tree(0).unwrap();
        let table = first(&tree.root).unwrap();
        let cells: Vec<_> = table
            .children
            .iter()
            .filter(|n| matches!(n.node_type, RenderNodeType::TableCell(_)))
            .collect();
        assert_eq!(cells.len(), root(&c).1.cells.len());
        let mut ink = vec![];
        lines(table, &mut ink);
        for cell in cells {
            let b = cell.bbox;
            for (h, at, lo, hi) in [
                (false, b.x, b.y, b.y + b.height),
                (false, b.x + b.width, b.y, b.y + b.height),
                (true, b.y, b.x, b.x + b.width),
                (true, b.y + b.height, b.x, b.x + b.width),
            ] {
                let mut spans = vec![];
                for &[x, y, a, z] in &ink {
                    let (on, l, r) = if h {
                        (
                            (y - at).abs() < 0.5 && (z - at).abs() < 0.5,
                            x.min(a),
                            x.max(a),
                        )
                    } else {
                        (
                            (x - at).abs() < 0.5 && (a - at).abs() < 0.5,
                            y.min(z),
                            y.max(z),
                        )
                    };
                    if on && r > lo && l < hi {
                        spans.push((l.max(lo), r.min(hi)));
                    }
                }
                spans.sort_by(|a, b| a.0.total_cmp(&b.0));
                let mut end = lo;
                for (a, b) in spans {
                    assert!(a <= end + 0.5, "visible border gap on real cell");
                    end = end.max(b)
                }
                assert!(end >= hi - 0.5, "inherited style was not painted");
            }
        }
    }
}
#[test]
fn stale_crop_and_noncanonical_mask_reject_before_structural_changes() {
    for crop in [true, false] {
        let mut c = load();
        let mut doc = c.document().clone();
        let Control::Table(t) = doc.sections[0].paragraphs[0]
            .controls
            .iter_mut()
            .find(|c| matches!(c, Control::Table(_)))
            .unwrap()
        else {
            unreachable!()
        };
        let p = t
            .cells
            .iter_mut()
            .flat_map(|c| &mut c.paragraphs)
            .flat_map(|p| &mut p.controls)
            .find_map(|c| {
                if let Control::Picture(p) = c {
                    p.common.cell_border_paint.map(|_| p)
                } else {
                    None
                }
            })
            .unwrap();
        if crop {
            p.crop.right -= 10;
        } else {
            p.common.cell_border_paint.as_mut().unwrap().sides = 16;
        }
        c.set_document(doc);
        let before = c.export_hwpx_native().unwrap();
        assert!(apply(&mut c, 1).is_err());
        assert_eq!(c.export_hwpx_native().unwrap(), before);
    }
}
