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
                .join("pdf/issue7998/retained-border-structure/retained-pen.pdf"),
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
fn load_other(name: &str) -> DocumentCore {
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "pdf/issue7998/retained-border-structure/{name}.pdf"
        )),
    )
    .unwrap();
    DocumentCore::from_pdf_bytes(&bytes, &PdfImportOptions::default()).unwrap()
}
fn tagged(c: &DocumentCore) -> usize {
    let bytes = border_paint::encode(c.document()).unwrap().unwrap();
    u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize
}
fn owned_bands(c: &DocumentCore) {
    let t = root(c).1;
    for cell in &t.cells {
        let fill = &c.document().doc_info.border_fills[cell.border_fill_id as usize - 1];
        assert!(
            fill.borders[..4]
                .iter()
                .all(|b| b.line_type == BorderLineType::None),
            "nonstandard ink must not be rounded to a native BorderFill bucket"
        );
        assert_eq!(cell.paragraphs.iter().flat_map(|p| &p.controls).filter(|ctrl|
            matches!(ctrl, Control::Shape(s) if s.common().cell_border_paint.is_some())).count(), 4);
    }
    assert_eq!(
        tagged(c),
        4 * t.cells.len() + 2 * usize::from(t.row_count + t.col_count),
        "every cell half and the physical outside half keeps its current owner"
    );
}

#[test]
fn inserted_units_inherit_nonstandard_ink_without_cloning_content() {
    for mode in 0..4 {
        let mut c = load();
        let before = text(&c.document().sections[0].paragraphs);
        let nested = child(&c);
        apply(&mut c, mode).unwrap();
        owned_bands(&c);
        assert_eq!(child(&c), nested);
        assert_eq!(text(&c.document().sections[0].paragraphs), before);
        let t = root(&c).1;
        assert_eq!(
            (t.row_count, t.col_count),
            if mode < 2 { (3, 2) } else { (2, 3) }
        );
        for cell in &t.cells {
            if (mode < 2 && cell.row == if mode == 0 { 0 } else { 1 })
                || (mode >= 2 && cell.col == if mode == 2 { 0 } else { 1 })
            {
                assert!(
                    cell.paragraphs.iter().all(|p| {
                        p.text.is_empty() && p.controls.iter().all(|ctrl|
                    matches!(ctrl,Control::Shape(s) if s.common().cell_border_paint.is_some()))
                    }),
                    "new cells inherit border ink, not text or the nested table"
                );
            }
        }
    }
}

#[test]
fn deletion_preserves_survivors_and_inserting_then_deleting_preserves_topology() {
    for mode in 0..6 {
        let mut c = load();
        let before = text(&c.document().sections[0].paragraphs);
        let nested = child(&c);
        apply(&mut c, mode).unwrap();
        if mode < 4 {
            let ci = root(&c).0;
            if mode < 2 {
                c.delete_table_row_native(0, 0, ci, if mode == 0 { 0 } else { 1 })
                    .unwrap();
            } else {
                c.delete_table_column_native(0, 0, ci, if mode == 2 { 0 } else { 1 })
                    .unwrap();
            }
            assert_eq!((root(&c).1.row_count, root(&c).1.col_count), (2, 2));
            assert_eq!(text(&c.document().sections[0].paragraphs), before);
        } else {
            let got = text(&c.document().sections[0].paragraphs);
            assert!(
                got.contains("BEFORE CHILD")
                    && got.contains("AFTER CHILD")
                    && got.contains("OUTSIDE AFTER")
            );
            assert!(!got.contains("LOWER RIGHT"));
            assert_eq!(got.contains("UPPER RIGHT"), mode == 4);
            assert_eq!(got.contains("LOWER LEFT"), mode == 5);
            assert_eq!(root(&c).1.cells.len(), 2);
        }
        assert_eq!(child(&c), nested);
        owned_bands(&c);
    }
}

#[test]
fn both_saves_keep_owned_ink_and_hwp_top_level_shape_header_contract() {
    for mode in 0..6 {
        let mut c = load();
        apply(&mut c, mode).unwrap();
        let want = text(&c.document().sections[0].paragraphs);
        for (hwp, bytes) in [
            (true, c.export_hwp_with_adapter_snapshot().unwrap()),
            (false, c.export_hwpx_native().unwrap()),
        ] {
            let mut n = DocumentCore::from_bytes(&bytes).unwrap();
            owned_bands(&n);
            assert_eq!(text(&n.document().sections[0].paragraphs), want);
            if hwp {
                // Independent HWP record contract, confirmed by the maintainer's
                // corruption/normal-open pair, not the permissive rhwp parser.
                let parent = &n.document().sections[0].paragraphs[0];
                let controls = parent.controls.iter().chain(
                    root(&n)
                        .1
                        .cells
                        .iter()
                        .flat_map(|c| &c.paragraphs)
                        .flat_map(|p| &p.controls),
                );
                let mut count = 0;
                for ctrl in controls {
                    if let Control::Shape(s) = ctrl {
                        if s.common().cell_border_paint.is_some()
                            || s.common().outside_cell_border_paint.is_some()
                        {
                            assert!(
                                s.drawing().unwrap().shape_attr.is_two_ctrl_id,
                                "new top-level shapes require two IDs in SHAPE_COMPONENT"
                            );
                            count += 1;
                        }
                    }
                }
                assert_eq!(count, tagged(&n));
            }
            let ci = root(&n).0;
            n.insert_table_row_native(0, 0, ci, 0, true).unwrap();
            n.delete_table_row_native(0, 0, ci, 1).unwrap();
            owned_bands(&n);
        }
    }
}

#[test]
fn source_units_color_and_repeat_edits_do_not_accumulate_pen_drift() {
    // PDF declarations 0.50pt/0.51pt = 50/51HU; independent integer crop
    // projection permits one HU, not rounding to the nearest native style.
    for (name, pen, color) in [("retained-pen", 50i64, 0u32), ("odd-blue", 51, 0x996633)] {
        let mut c = load_other(name);
        apply(&mut c, 1).unwrap();
        for bytes in [
            c.export_hwp_with_adapter_snapshot().unwrap(),
            c.export_hwpx_native().unwrap(),
        ] {
            let mut n = DocumentCore::from_bytes(&bytes).unwrap();
            let nested = child(&n);
            let want = text(&n.document().sections[0].paragraphs);
            let mut first = None;
            for _ in 0..3 {
                for cell in &root(&n).1.cells {
                    let mut widths = [0; 4];
                    for ctrl in cell.paragraphs.iter().flat_map(|p| &p.controls) {
                        if let Control::Shape(s) = ctrl {
                            if let Some(tag) = s.common().cell_border_paint {
                                use border_paint::CellSide::*;
                                let i = match tag.side {
                                    Left => 0,
                                    Right => 1,
                                    Top => 2,
                                    Bottom => 3,
                                };
                                widths[i] = if i < 2 {
                                    s.common().width
                                } else {
                                    s.common().height
                                };
                                assert_eq!(
                                    s.drawing().unwrap().fill.solid.unwrap().background_color,
                                    color
                                );
                            }
                        }
                    }
                    let observed = [widths[0] + widths[1], widths[2] + widths[3]];
                    assert!(observed.iter().all(|&w| (i64::from(w) - pen).abs() <= 1));
                    assert_eq!(*first.get_or_insert(observed), observed);
                }
                let ci = root(&n).0;
                n.insert_table_column_native(0, 0, ci, 0, true).unwrap();
                n.delete_table_column_native(0, 0, ci, 1).unwrap();
            }
            assert_eq!(child(&n), nested);
            assert_eq!(text(&n.document().sections[0].paragraphs), want);
        }
    }
}

#[test]
fn explicit_shared_side_edit_removes_only_its_two_halves_after_both_saves() {
    let mut c = load();
    apply(&mut c, 1).unwrap();
    for bytes in [
        c.export_hwp_with_adapter_snapshot().unwrap(),
        c.export_hwpx_native().unwrap(),
    ] {
        let mut n = DocumentCore::from_bytes(&bytes).unwrap();
        let before = tagged(&n);
        let nested = child(&n);
        let want = text(&n.document().sections[0].paragraphs);
        let ci = root(&n).0;
        n.set_cell_properties_native(
            0,
            0,
            ci,
            0,
            r##"{"borderRight":{"type":1,"width":5,"color":"#d00000"}}"##,
        )
        .unwrap();
        assert_eq!(tagged(&n), before - 2);
        assert_eq!(child(&n), nested);
        assert_eq!(text(&n.document().sections[0].paragraphs), want);
        let f =
            &n.document().doc_info.border_fills[root(&n).1.cells[0].border_fill_id as usize - 1];
        assert_eq!(f.borders[1].line_type, BorderLineType::Solid);
        assert_eq!(f.borders[1].color, 0xd0);
    }
}

#[test]
fn snapshot_restore_keeps_original_paint_and_captured_edit_after_text_inverse() {
    for mode in 0..6 {
        let mut c = load();
        let original = c.export_hwpx_native().unwrap();
        let before = c.save_snapshot_native();
        apply(&mut c, mode).unwrap();
        let after = c.save_snapshot_native();
        let edited = c.export_hwpx_native().unwrap();
        let ci = root(&c).0;
        c.insert_text_in_cell_native(0, 0, ci, 0, 0, 0, "X")
            .unwrap();
        c.delete_text_in_cell_native(0, 0, ci, 0, 0, 0, 1).unwrap();
        c.restore_snapshot_native(before).unwrap();
        assert_eq!(c.export_hwpx_native().unwrap(), original);
        c.restore_snapshot_native(after).unwrap();
        assert_eq!(c.export_hwpx_native().unwrap(), edited);
        owned_bands(&c);
    }
}

#[test]
fn nonuniform_axis_pen_and_invalid_indices_reject_atomically() {
    for mode in 0..6 {
        let mut c = load_other("nonuniform-axis");
        let before = c.export_hwpx_native().unwrap();
        assert!(apply(&mut c, mode).is_err());
        assert_eq!(c.export_hwpx_native().unwrap(), before);
    }
    let mut c = load();
    let before = c.export_hwpx_native().unwrap();
    let ci = root(&c).0;
    assert!(c.insert_table_row_native(0, 0, ci, 99, false).is_err());
    assert!(c.delete_table_column_native(0, 0, ci, 99).is_err());
    assert_eq!(c.export_hwpx_native().unwrap(), before);
}

#[test]
fn non_cell_frame_paint_does_not_authorize_cell_frame_rewriting() {
    use rhwp::model::{
        shape::{HorzAlign, VertAlign},
        table::VerticalAlign,
    };
    for mode in 0..5 {
        let mut c = load();
        let mut d = c.document().clone();
        let ci = root(&c).0;
        let Control::Table(t) = &mut d.sections[0].paragraphs[0].controls[ci] else {
            unreachable!()
        };
        match mode {
            0 => t.cells[0].vertical_align = VerticalAlign::Center,
            1 => t.cells[0].vertical_align = VerticalAlign::Bottom,
            4 => t.cells[0]
                .paragraphs
                .insert(0, rhwp::model::paragraph::Paragraph::new_empty()),
            _ => {
                let p = t.cells[0]
                    .paragraphs
                    .iter_mut()
                    .flat_map(|p| &mut p.controls)
                    .find_map(|ctrl| {
                        if let Control::Picture(p) = ctrl {
                            p.common.cell_border_paint.map(|_| p)
                        } else {
                            None
                        }
                    })
                    .unwrap();
                if mode == 2 {
                    p.common.horz_align = HorzAlign::Center;
                } else {
                    p.common.vert_align = VertAlign::Bottom;
                }
            }
        }
        c.set_document(d);
        let before = c.export_hwpx_native().unwrap();
        assert!(c.insert_table_row_native(0, 0, ci, 0, true).is_err());
        assert_eq!(c.export_hwpx_native().unwrap(), before);
    }
}

fn nested_table(c: &DocumentCore) -> &Table {
    root(c)
        .1
        .cells
        .iter()
        .flat_map(|c| &c.paragraphs)
        .flat_map(|p| &p.controls)
        .find_map(|c| match c {
            Control::Table(t) => Some(t.as_ref()),
            _ => None,
        })
        .unwrap()
}

#[test]
fn parent_source_padding_does_not_become_selectable_nested_cell_pictures() {
    // Independent PDF: parent stroke axes lie outside the four child cells;
    // the child contains A/B/C/D and its own borders, no image XObject.
    // The original failure produced two fully transparent pictures per cell.
    let mut c = load_other("mixed-pen");
    for saved in 0..3 {
        let child = nested_table(&c);
        assert_eq!(child.cells.len(), 4);
        assert_eq!(
            text(
                &child
                    .cells
                    .iter()
                    .flat_map(|c| c.paragraphs.clone())
                    .collect::<Vec<_>>()
            ),
            "ABCD"
        );
        assert!(
            child
                .cells
                .iter()
                .flat_map(|c| &c.paragraphs)
                .flat_map(|p| &p.controls)
                .all(|c| !matches!(c, Control::Picture(_))),
            "empty parent raster-frame crops must not intercept child-cell text clicks"
        );
        for cell in &child.cells {
            let fill = &c.document().doc_info.border_fills[cell.border_fill_id as usize - 1];
            assert!(fill.borders[..4]
                .iter()
                .all(|b| b.line_type == BorderLineType::Solid));
        }
        if saved == 0 {
            c = DocumentCore::from_bytes(&c.export_hwp_with_adapter_snapshot().unwrap()).unwrap();
        } else if saved == 1 {
            c = DocumentCore::from_bytes(&c.export_hwpx_native().unwrap()).unwrap();
        }
    }
}

#[test]
fn axis_specific_ink_is_inherited_without_cloning_child_content() {
    for mode in 0..6 {
        let mut c = load_other("mixed-pen");
        let child_before = child(&c);
        let before = text(&c.document().sections[0].paragraphs);
        apply(&mut c, mode).unwrap();
        owned_bands(&c);
        assert_eq!(child(&c), child_before);
        let t = root(&c).1;
        if mode < 4 {
            assert_eq!(text(&c.document().sections[0].paragraphs), before);
            for cell in &t.cells {
                let new = if mode < 2 {
                    cell.row == if mode == 0 { 0 } else { 1 }
                } else {
                    cell.col == if mode == 2 { 0 } else { 1 }
                };
                if new {
                    assert!(cell.paragraphs.iter().all(|p| {
                        p.text.is_empty() && p.controls.iter().all(
                        |c| matches!(c,Control::Shape(s)if s.common().cell_border_paint.is_some()))
                    }));
                }
            }
        } else {
            let got = text(&c.document().sections[0].paragraphs);
            assert!(
                got.contains("BEFORE CHILD")
                    && got.contains("AFTER CHILD")
                    && got.contains("OUTSIDE AFTER")
            );
            assert!(!got.contains("LOWER RIGHT"));
            assert_eq!(got.contains("UPPER RIGHT"), mode == 4);
            assert_eq!(got.contains("LOWER LEFT"), mode == 5);
        }
    }
}

#[test]
fn axis_specific_source_pens_survive_both_formats_and_repeated_edits() {
    let mut c = load_other("mixed-pen");
    apply(&mut c, 1).unwrap();
    for bytes in [
        c.export_hwp_with_adapter_snapshot().unwrap(),
        c.export_hwpx_native().unwrap(),
    ] {
        let mut n = DocumentCore::from_bytes(&bytes).unwrap();
        let nested = child(&n);
        let want = text(&n.document().sections[0].paragraphs);
        let mut first = None;
        for _ in 0..3 {
            for cell in &root(&n).1.cells {
                let mut halves = [0u32; 4];
                for ctrl in cell.paragraphs.iter().flat_map(|p| &p.controls) {
                    if let Control::Shape(s) = ctrl {
                        if let Some(tag) = s.common().cell_border_paint {
                            use border_paint::CellSide::*;
                            let side = match tag.side {
                                Left => 0,
                                Right => 1,
                                Top => 2,
                                Bottom => 3,
                            };
                            halves[side] = if side < 2 {
                                s.common().width
                            } else {
                                s.common().height
                            };
                            // Independent PDF operator order: horizontal paths first,
                            // then vertical paths, with opaque black commutative ink.
                            assert_eq!(s.common().z_order, if side < 2 { 2 } else { 1 });
                        }
                    }
                }
                let pens = [halves[0] + halves[1], halves[2] + halves[3]];
                // PDF declarations 0.70/0.50pt = 70/50HU, not native width buckets.
                assert!((i64::from(pens[0]) - 70).abs() <= 1);
                assert!((i64::from(pens[1]) - 50).abs() <= 1);
                assert_eq!(*first.get_or_insert(pens), pens);
            }
            let ci = root(&n).0;
            n.insert_table_column_native(0, 0, ci, 0, true).unwrap();
            n.delete_table_column_native(0, 0, ci, 1).unwrap();
        }
        assert_eq!(child(&n), nested);
        assert_eq!(text(&n.document().sections[0].paragraphs), want);
    }
}

#[test]
fn unrelated_png_is_not_consumed_by_an_ink_order_proof() {
    for effect in 0..3 {
        let mut c = load_other("mixed-pen");
        let ci = root(&c).0;
        let mut d = c.document().clone();
        let Control::Table(t) = &d.sections[0].paragraphs[0].controls[ci] else {
            unreachable!()
        };
        let mut p = t
            .cells
            .iter()
            .flat_map(|c| &c.paragraphs)
            .flat_map(|p| &p.controls)
            .find_map(|c| match c {
                Control::Picture(p) if p.common.cell_border_paint.is_some() => Some(p.clone()),
                _ => None,
            })
            .unwrap();
        p.common.cell_border_paint = None;
        p.common.outside_cell_border_paint = None;
        p.common.z_order = 2;
        p.common.instance_id = 500001;
        if effect == 1 {
            p.image_attr.brightness = 1;
        }
        if effect == 2 {
            p.border_attr.width = 1;
        }
        let independent = serde_json::to_value(&p).unwrap();
        d.sections[0].paragraphs[0]
            .controls
            .push(Control::Picture(p));
        c.set_document(d);
        let before = c.export_hwpx_native().unwrap();
        let result = apply(&mut c, 1);
        if effect == 0 {
            result.unwrap();
            let pictures: Vec<_> = c.document().sections[0].paragraphs[0]
                .controls
                .iter()
                .filter_map(|c| match c {
                    Control::Picture(p) if p.common.instance_id == 500001 => Some(p),
                    _ => None,
                })
                .collect();
            assert_eq!(pictures.len(), 1);
            assert_eq!(serde_json::to_value(pictures[0]).unwrap(), independent);
        } else {
            assert!(
                result.is_err(),
                "sharing an image resource is not permission to move modified ink"
            );
            assert_eq!(c.export_hwpx_native().unwrap(), before);
        }
    }
}

#[test]
fn actual_cell_image_data_is_preserved_instead_of_dropping_all_cell_pictures() {
    fn images(ps: &[rhwp::model::paragraph::Paragraph], c: &DocumentCore, out: &mut Vec<Vec<u8>>) {
        for p in ps {
            for ctrl in &p.controls {
                match ctrl {
                    Control::Table(t) => {
                        for cell in &t.cells {
                            images(&cell.paragraphs, c, out);
                        }
                    }
                    Control::Picture(p) => {
                        let bytes = c
                            .document()
                            .bin_data_content
                            .iter()
                            .find(|b| b.id == p.image_attr.bin_data_id)
                            .unwrap()
                            .data
                            .load_limited(4 * 1024 * 1024)
                            .unwrap();
                        if bytes.starts_with(b"\x89PNG")
                            && bytes[16..24] == [0, 0, 0, 2, 0, 0, 0, 2]
                        {
                            out.push(bytes);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    for key in ["nested-dash-image", "nested-periodic-image"] {
        let bytes = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("pdf/issue7998/{key}.pdf")),
        )
        .unwrap();
        let c = DocumentCore::from_pdf_bytes(&bytes, &PdfImportOptions::default()).unwrap();
        let mut expected = Vec::new();
        images(&c.document().sections[0].paragraphs, &c, &mut expected);
        assert_eq!(
            expected.len(),
            1,
            "independent source declares one genuine 2x2 image"
        );
        for bytes in [
            c.export_hwp_with_adapter_snapshot().unwrap(),
            c.export_hwpx_native().unwrap(),
        ] {
            let n = DocumentCore::from_bytes(&bytes).unwrap();
            let mut actual = Vec::new();
            images(&n.document().sections[0].paragraphs, &n, &mut actual);
            assert_eq!(actual, expected);
        }
    }
}
