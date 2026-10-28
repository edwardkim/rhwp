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
fn invalid_structural_indices_reject_atomically() {
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
        p.common.cell_frame_paint = false;
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

fn parallel_source_halves(c: &DocumentCore, vertical: bool) {
    for cell in &root(c).1.cells {
        let mut seen = [false; 4];
        for control in cell.paragraphs.iter().flat_map(|p| &p.controls) {
            let Control::Shape(shape) = control else {
                continue;
            };
            let Some(tag) = shape.common().cell_border_paint else {
                continue;
            };
            use border_paint::CellSide::*;
            let side = match tag.side {
                Left => 0,
                Right => 1,
                Top => 2,
                Bottom => 3,
            };
            // Independent PDF declarations: V=0.90/0.70pt, H=0.50pt;
            // or V=0.70pt, H=0.90/0.50pt. 1pt is 100 HWPUNITs.
            let pen = if side < 2 {
                if vertical && side == 0 && cell.col == 0 {
                    90
                } else {
                    70
                }
            } else if !vertical && side == 2 && cell.row == 0 {
                90
            } else {
                50
            };
            let expected = if side == 0 || side == 2 {
                (pen + 1) / 2
            } else {
                pen / 2
            };
            let actual = if side < 2 {
                shape.common().width
            } else {
                shape.common().height
            };
            assert!(
                (i64::from(actual) - i64::from(expected)).abs() <= 1,
                "source pen {pen}HU, side {side}: observed half {actual}"
            );
            assert!(!seen[side], "one owned physical half per side");
            seen[side] = true;
        }
        assert!(seen.into_iter().all(|value| value));
    }
}

#[test]
fn parallel_line_pens_survive_structural_edits_and_both_saves() {
    for (name, vertical, modes) in [
        ("nonuniform-axis", true, [0, 1, 4]),
        ("nonuniform-horizontal", false, [2, 3, 5]),
    ] {
        for mode in modes {
            let mut c = load_other(name);
            let nested = child(&c);
            let geometry = |c: &DocumentCore| {
                nested_table(c)
                    .cells
                    .iter()
                    .map(|cell| {
                        (
                            cell.row,
                            cell.col,
                            cell.row_span,
                            cell.col_span,
                            cell.width,
                            cell.height,
                            text(&cell.paragraphs),
                            cell.paragraphs
                                .iter()
                                .map(|p| serde_json::to_value(&p.line_segs).unwrap())
                                .collect::<Vec<_>>(),
                        )
                    })
                    .collect::<Vec<_>>()
            };
            let expected_child_geometry = geometry(&c);
            let source = c.export_hwpx_native().unwrap();
            let snapshot = c.save_snapshot_native();
            apply(&mut c, mode).unwrap();
            owned_bands(&c);
            parallel_source_halves(&c, vertical);
            assert_eq!(child(&c), nested);
            let want = text(&c.document().sections[0].paragraphs);
            assert!(
                want.contains("BEFORE CHILD")
                    && want.contains("AFTER CHILD")
                    && want.contains("OUTSIDE AFTER")
            );
            if mode < 4 {
                for cell in &root(&c).1.cells {
                    let new = match mode {
                        0 => cell.row == 0,
                        1 => cell.row == 1,
                        2 => cell.col == 0,
                        _ => cell.col == 1,
                    };
                    if new {
                        assert!(cell.paragraphs.iter().all(|p| p.text.is_empty()
                            && p.controls.iter().all(|control|
                                matches!(control, Control::Shape(s) if s.common().cell_border_paint.is_some()))));
                    }
                }
            } else {
                assert!(!want.contains("LOWER RIGHT"));
                assert_eq!(want.contains("UPPER RIGHT"), vertical);
                assert_eq!(want.contains("LOWER LEFT"), !vertical);
            }
            for bytes in [
                c.export_hwp_with_adapter_snapshot().unwrap(),
                c.export_hwpx_native().unwrap(),
            ] {
                let mut n = DocumentCore::from_bytes(&bytes).unwrap();
                // HWP legitimately synthesizes raw header/cache bytes. Check
                // source content and geometry across formats, then protect the
                // complete reopened subtree during subsequent parent edits.
                assert_eq!(geometry(&n), expected_child_geometry);
                let reopened_child = child(&n);
                for _ in 0..3 {
                    owned_bands(&n);
                    parallel_source_halves(&n, vertical);
                    let ci = root(&n).0;
                    if vertical {
                        n.insert_table_row_native(0, 0, ci, 0, true).unwrap();
                        parallel_source_halves(&n, vertical);
                        n.delete_table_row_native(0, 0, ci, 1).unwrap();
                    } else {
                        n.insert_table_column_native(0, 0, ci, 0, true).unwrap();
                        parallel_source_halves(&n, vertical);
                        n.delete_table_column_native(0, 0, ci, 1).unwrap();
                    }
                    assert_eq!(text(&n.document().sections[0].paragraphs), want);
                    assert_eq!(child(&n), reopened_child);
                }
            }
            c.restore_snapshot_native(snapshot).unwrap();
            assert_eq!(c.export_hwpx_native().unwrap(), source);
        }
    }
}

#[test]
fn within_cell_side_pen_changes_reject_atomically() {
    for source in ["within-cell-vertical", "within-cell-horizontal"] {
        for mode in 0..6 {
            let mut c = load_other(source);
            let before = c.export_hwpx_native().unwrap();
            assert!(apply(&mut c, mode).is_err());
            assert_eq!(c.export_hwpx_native().unwrap(), before);
        }
    }
}

fn conflict_physical_pens(c: &DocumentCore, vertical: bool, lines: &[u32]) {
    owned_bands(c);
    for cell in &root(c).1.cells {
        let mut seen = [false; 4];
        for control in cell.paragraphs.iter().flat_map(|p| &p.controls) {
            let Control::Shape(shape) = control else {
                continue;
            };
            let Some(tag) = shape.common().cell_border_paint else {
                continue;
            };
            use border_paint::CellSide::*;
            let side = match tag.side {
                Left => 0,
                Right => 1,
                Top => 2,
                Bottom => 3,
            };
            let pen = if vertical {
                if side < 2 {
                    lines[usize::from(cell.col) + side]
                } else {
                    50
                }
            } else if side >= 2 {
                lines[usize::from(cell.row) + side - 2]
            } else {
                70
            };
            let want = if side == 0 || side == 2 {
                pen.div_ceil(2)
            } else {
                pen / 2
            };
            let got = if side < 2 {
                shape.common().width
            } else {
                shape.common().height
            };
            // PDF operator widths, subject only to the existing one-HU crop
            // projection uncertainty. This is not an output pixel coordinate.
            assert!((i64::from(got) - i64::from(want)).abs() <= 1);
            assert!(!seen[side]);
            seen[side] = true;
        }
        assert!(seen.into_iter().all(|v| v));
    }
}

#[test]
fn conflicting_axis_inherits_actual_template_and_preserves_losing_style_in_both_saves() {
    // PDF declares .90/.70pt vertical pens or .90/.50pt horizontal pens.
    // Independent native BorderFill .4/.2mm Hancom Print confirms the wider
    // shared edge, then the thinner edge after deleting the original neighbour.
    for vertical in [true, false] {
        for mode in 0..6 {
            let mut c = load_other(if vertical {
                "nonuniform-axis"
            } else {
                "nonuniform-horizontal"
            });
            let ci = root(&c).0;
            let original = c.export_hwpx_native().unwrap();
            let snapshot = c.save_snapshot_native();
            let child_before = child(&c);
            let source_text: Vec<_> = root(&c)
                .1
                .cells
                .iter()
                .map(|c| text(&c.paragraphs))
                .collect();
            let low = if vertical { 70 } else { 50 };
            let mut lines = if mode < 2 || mode == 4 || mode == 5 {
                if vertical {
                    c.insert_table_column_native(
                        0,
                        0,
                        ci,
                        u16::from(mode == 5),
                        mode == 1 || mode == 5,
                    )
                } else {
                    c.insert_table_row_native(
                        0,
                        0,
                        ci,
                        u16::from(mode == 5),
                        mode == 1 || mode == 5,
                    )
                }
                .unwrap();
                if mode == 1 || mode == 5 {
                    vec![90, low, low, low]
                } else {
                    vec![90, 90, low, low]
                }
            } else {
                let index = u16::from(mode == 3);
                if vertical {
                    c.delete_table_column_native(0, 0, ci, index)
                } else {
                    c.delete_table_row_native(0, 0, ci, index)
                }
                .unwrap();
                if mode == 2 {
                    vec![low, low]
                } else {
                    vec![90, low]
                }
            };
            if mode == 4 {
                if vertical {
                    c.delete_table_column_native(0, 0, ci, 1)
                } else {
                    c.delete_table_row_native(0, 0, ci, 1)
                }
                .unwrap();
                lines = vec![90, low, low];
            }
            let mut expected = std::collections::BTreeMap::new();
            for r in 0..2u16 {
                for col in 0..2u16 {
                    let index = if vertical { col } else { r };
                    let at = match mode {
                        0 => Some(index + 1),
                        1 => Some(index + u16::from(index >= 1)),
                        2 => (index > 0).then_some(index.saturating_sub(1)),
                        3 => (index == 0).then_some(0),
                        5 => Some(index),
                        _ => (index > 0).then_some(1),
                    };
                    if let Some(at) = at {
                        expected.insert(
                            if vertical { (r, at) } else { (at, col) },
                            source_text[usize::from(r) * 2 + usize::from(col)].clone(),
                        );
                    }
                }
            }
            conflict_physical_pens(&c, vertical, &lines);
            if mode < 2 {
                assert_eq!(child(&c), child_before);
                let new_index = u16::from(mode == 1);
                for cell in &root(&c).1.cells {
                    if (if vertical { cell.col } else { cell.row }) == new_index {
                        assert!(
                            text(&cell.paragraphs).is_empty(),
                            "new cells never clone source content"
                        );
                        assert!(!cell
                            .paragraphs
                            .iter()
                            .flat_map(|p| &p.controls)
                            .any(|c| matches!(c, Control::Table(_))));
                    }
                }
            }
            for bytes in [
                c.export_hwp_with_adapter_snapshot().unwrap(),
                c.export_hwpx_native().unwrap(),
            ] {
                let mut reopened = DocumentCore::from_bytes(&bytes).unwrap();
                conflict_physical_pens(&reopened, vertical, &lines);
                for cell in &root(&reopened).1.cells {
                    assert_eq!(
                        text(&cell.paragraphs),
                        expected
                            .get(&(cell.row, cell.col))
                            .cloned()
                            .unwrap_or_default()
                    );
                }
                if mode == 0 {
                    let meta = border_paint::encode(reopened.document()).unwrap().unwrap();
                    assert_eq!(&meta[..8], b"RBPO\0\0\0\x06");
                    let first = root(&reopened)
                        .1
                        .cells
                        .iter()
                        .find(|c| c.row == 0 && c.col == 0)
                        .unwrap();
                    let losing = first
                        .paragraphs
                        .iter()
                        .flat_map(|p| &p.controls)
                        .find_map(|c| match c {
                            Control::Shape(s) => {
                                s.common().cell_border_paint.and_then(|t| t.cell_pen)
                            }
                            _ => None,
                        })
                        .expect("the new cell retains its own thinner right/bottom pen");
                    assert!((i64::from(losing) - i64::from(low)).abs() <= 1);
                    let rci = root(&reopened).0;
                    if vertical {
                        reopened.delete_table_column_native(0, 0, rci, 1)
                    } else {
                        reopened.delete_table_row_native(0, 0, rci, 1)
                    }
                    .unwrap();
                    conflict_physical_pens(&reopened, vertical, &[90, low, low]);
                    for cell in &root(&reopened).1.cells {
                        let at = if vertical { cell.col } else { cell.row };
                        let content = text(&cell.paragraphs);
                        if at == 0 {
                            assert!(content.is_empty());
                        } else {
                            let index = if vertical {
                                usize::from(cell.row) * 2 + 1
                            } else {
                                2 + usize::from(cell.col)
                            };
                            assert_eq!(
                                content, source_text[index],
                                "only the explicitly deleted source cells disappear"
                            );
                        }
                    }
                }
            }
            c.restore_snapshot_native(snapshot).unwrap();
            assert_eq!(c.export_hwpx_native().unwrap(), original);
        }
    }
}

#[test]
fn invalid_losing_pen_sidecar_fails_closed_without_changing_visible_geometry() {
    let mut c = load_other("nonuniform-axis");
    let ci = root(&c).0;
    c.insert_table_column_native(0, 0, ci, 0, false).unwrap();
    let bytes = border_paint::encode(c.document()).unwrap().unwrap();
    let mut at = 12;
    let mut pen_at = None;
    for _ in 0..u32::from_le_bytes(bytes[8..12].try_into().unwrap()) {
        let extra = if bytes[at + 28] & 0x80 != 0 { 32 } else { 0 };
        let end = at + 65 + extra;
        if u32::from_le_bytes(bytes[end..end + 4].try_into().unwrap()) != 0 {
            pen_at = Some(end);
        }
        at = end + 4;
    }
    assert_eq!(at, bytes.len());
    for invalid in [1, u32::MAX] {
        let mut data = bytes.clone();
        let pos = pen_at.unwrap();
        data[pos..pos + 4].copy_from_slice(&invalid.to_le_bytes());
        let mut doc = c.document().clone();
        fn clear(ps: &mut [rhwp::model::paragraph::Paragraph]) {
            for p in ps {
                for control in &mut p.controls {
                    match control {
                        Control::Shape(s) => {
                            s.common_mut().cell_border_paint = None;
                            s.common_mut().cell_frame_paint = false;
                            s.common_mut().outside_cell_border_paint = None;
                        }
                        Control::Picture(p) => {
                            p.common.cell_border_paint = None;
                            p.common.cell_frame_paint = false;
                            p.common.outside_cell_border_paint = None;
                        }
                        Control::Table(t) => {
                            for c in &mut t.cells {
                                clear(&mut c.paragraphs);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        clear(&mut doc.sections[0].paragraphs);
        let geometry_before = serde_json::to_value(&doc.sections[0].paragraphs).unwrap();
        border_paint::decode(&mut doc, &data);
        assert!(border_paint::encode(&doc).unwrap().is_none());
        assert_eq!(
            serde_json::to_value(&doc.sections[0].paragraphs).unwrap(),
            geometry_before
        );
    }
}

type IntervalPens = std::collections::BTreeMap<(u16, u16), [u32; 4]>;
fn interval_oracle(kind: &str) -> IntervalPens {
    let mut p = IntervalPens::new();
    for r in 0..2 {
        for c in 0..2 {
            p.insert((r, c), [70, 70, 50, 50]);
        }
    }
    match kind {
        "v-outer" => p.get_mut(&(1, 0)).unwrap()[0] = 90,
        "h-outer" => p.get_mut(&(0, 0)).unwrap()[2] = 90,
        "v-shared" => {
            p.get_mut(&(0, 0)).unwrap()[1] = 90;
            p.get_mut(&(0, 1)).unwrap()[0] = 90;
        }
        _ => {
            p.get_mut(&(0, 1)).unwrap()[3] = 90;
            p.get_mut(&(1, 1)).unwrap()[2] = 90;
        }
    }
    p
}
fn interval_expected(p: &IntervalPens, op: &str) -> IntervalPens {
    let mut q = IntervalPens::new();
    for (&(r, c), &a) in p {
        match op {
            "row-before" => {
                q.insert((r + 1, c), a);
                if r == 0 {
                    q.insert((0, c), a);
                }
            }
            "col-before" => {
                q.insert((r, c + 1), a);
                if c == 0 {
                    q.insert((r, 0), a);
                }
            }
            "delete-row" => {
                if r > 0 {
                    q.insert((r - 1, c), a);
                }
            }
            "delete-col" => {
                if c > 0 {
                    q.insert((r, c - 1), a);
                }
            }
            "append-row" => {
                q.insert((r, c), a);
                if r == 1 {
                    q.insert((2, c), a);
                }
            }
            _ => {
                q.insert((r, c), a);
                if c == 1 {
                    q.insert((r, 2), a);
                }
            }
        }
    }
    q
}
fn interval_apply(c: &mut DocumentCore, op: &str) {
    let ci = root(c).0;
    match op {
        "row-before" => c.insert_table_row_native(0, 0, ci, 0, false),
        "col-before" => c.insert_table_column_native(0, 0, ci, 0, false),
        "delete-row" => c.delete_table_row_native(0, 0, ci, 0),
        "delete-col" => c.delete_table_column_native(0, 0, ci, 0),
        "append-row" => c.insert_table_row_native(0, 0, ci, 1, true),
        _ => c.insert_table_column_native(0, 0, ci, 1, true),
    }
    .unwrap();
}
fn interval_check(c: &DocumentCore, p: &IntervalPens) {
    owned_bands(c);
    let t = root(c).1;
    assert_eq!(t.cells.len(), p.len());
    let mut hidden = 0;
    for cell in &t.cells {
        let a = p[&(cell.row, cell.col)];
        let mut want = a;
        if cell.col > 0 {
            want[0] = want[0].max(p[&(cell.row, cell.col - 1)][1]);
        }
        if cell.col + 1 < t.col_count {
            want[1] = want[1].max(p[&(cell.row, cell.col + 1)][0]);
        }
        if cell.row > 0 {
            want[2] = want[2].max(p[&(cell.row - 1, cell.col)][3]);
        }
        if cell.row + 1 < t.row_count {
            want[3] = want[3].max(p[&(cell.row + 1, cell.col)][2]);
        }
        let mut seen = [false; 4];
        for control in cell.paragraphs.iter().flat_map(|p| &p.controls) {
            if let Control::Shape(s) = control {
                if let Some(tag) = s.common().cell_border_paint {
                    use border_paint::CellSide::*;
                    let side = match tag.side {
                        Left => 0,
                        Right => 1,
                        Top => 2,
                        Bottom => 3,
                    };
                    let want_half = if side == 0 || side == 2 {
                        want[side].div_ceil(2)
                    } else {
                        want[side] / 2
                    };
                    let got = if side < 2 {
                        s.common().width
                    } else {
                        s.common().height
                    };
                    assert!(
                        (got as i64 - want_half as i64).abs() <= 1,
                        "cell {:?} side {side}: {got} vs {want_half}",
                        (cell.row, cell.col)
                    );
                    assert!(!seen[side]);
                    seen[side] = true;
                    if a[side] < want[side] {
                        hidden += 1;
                        let own = tag.cell_pen.expect("losing cell pen must survive");
                        assert!((own as i64 - a[side] as i64).abs() <= 1);
                    }
                }
            }
        }
        assert!(seen.into_iter().all(|x| x));
    }
    let meta = border_paint::encode(c.document()).unwrap().unwrap();
    assert_eq!(
        u32::from_le_bytes(meta[8..12].try_into().unwrap()) as usize,
        4 * t.cells.len() + 2 * usize::from(t.row_count + t.col_count)
    );
    if hidden > 0 {
        assert_eq!(&meta[..8], b"RBPO\0\0\0\x06");
    }
}

fn interval_expected_texts(
    c: &DocumentCore,
    op: &str,
) -> std::collections::BTreeMap<(u16, u16), String> {
    let mut out = std::collections::BTreeMap::new();
    for cell in &root(c).1.cells {
        let (r, col) = (cell.row, cell.col);
        let at = match op {
            "row-before" => Some((r + 1, col)),
            "col-before" => Some((r, col + 1)),
            "delete-row" => {
                if r == 0 {
                    None
                } else {
                    Some((r - 1, col))
                }
            }
            "delete-col" => {
                if col == 0 {
                    None
                } else {
                    Some((r, col - 1))
                }
            }
            _ => Some((r, col)),
        };
        if let Some(at) = at {
            out.insert(at, text(&cell.paragraphs));
        }
    }
    out
}
fn interval_check_texts(
    c: &DocumentCore,
    expected: &std::collections::BTreeMap<(u16, u16), String>,
) {
    for cell in &root(c).1.cells {
        assert_eq!(
            text(&cell.paragraphs),
            expected
                .get(&(cell.row, cell.col))
                .cloned()
                .unwrap_or_default()
        );
    }
    let all = text(&c.document().sections[0].paragraphs);
    assert_eq!(all.matches("OUTSIDE ABOVE").count(), 1);
    assert_eq!(all.matches("OUTSIDE AFTER").count(), 1);
    // Four zero margins exist in the HWP LIST_HEADER. Keep that presence
    // when crossing HWP -> IR -> HWPX; independent Hancom Print otherwise
    // loses the above-table text even though rhwp still paints it.
    let mut outside = 0;
    for p in &c.document().sections[0].paragraphs {
        for ctrl in &p.controls {
            if let Control::Shape(shape) = ctrl {
                if let Some(tb) = shape.drawing().and_then(|d| d.text_box.as_ref()) {
                    if tb.paragraphs.iter().any(|p| {
                        p.text.contains("OUTSIDE ABOVE") || p.text.contains("OUTSIDE AFTER")
                    }) {
                        assert!(tb.text_margin_explicit);
                        assert_eq!(
                            [
                                tb.margin_left,
                                tb.margin_right,
                                tb.margin_top,
                                tb.margin_bottom
                            ],
                            [0; 4]
                        );
                        outside += 1;
                    }
                }
            }
        }
    }
    assert_eq!(outside, 2);
}

#[test]
fn cell_interval_pens_survive_insertion_deletion_saves_and_repeated_edits() {
    // Independent PDF operators declare each of the four original cells' sides.
    // Wider same-colour shared ink follows the separately printed native
    // BorderFill contract; inherited cell content is never copied into new cells.
    for (kind, source, ops) in [
        (
            "v-outer",
            "nonuniform-interval",
            ["row-before", "col-before", "delete-row", "append-col"],
        ),
        (
            "h-outer",
            "horizontal-interval",
            ["row-before", "col-before", "delete-col", "append-row"],
        ),
        (
            "v-shared",
            "shared-vertical-interval",
            ["row-before", "col-before", "delete-row", "delete-col"],
        ),
        (
            "h-shared",
            "shared-horizontal-interval",
            ["row-before", "col-before", "delete-row", "delete-col"],
        ),
    ] {
        for op in ops {
            let mut c = load_other(source);
            let source_child = child(&c);
            let expected_text = interval_expected_texts(&c, op);
            let source_hwpx = c.export_hwpx_native().unwrap();
            let snapshot = c.save_snapshot_native();
            let expected = interval_expected(&interval_oracle(kind), op);
            interval_apply(&mut c, op);
            interval_check(&c, &expected);
            interval_check_texts(&c, &expected_text);
            if !op.starts_with("delete-") {
                assert_eq!(child(&c), source_child);
            }
            for bytes in [
                c.export_hwp_with_adapter_snapshot().unwrap(),
                c.export_hwpx_native().unwrap(),
            ] {
                let mut reopened = DocumentCore::from_bytes(&bytes).unwrap();
                interval_check(&reopened, &expected);
                interval_check_texts(&reopened, &expected_text);
                let child_before = if op.starts_with("delete-") {
                    None
                } else {
                    Some(child(&reopened))
                };
                let ci = root(&reopened).0;
                for _ in 0..2 {
                    reopened
                        .insert_table_column_native(0, 0, ci, 0, false)
                        .unwrap();
                    reopened.delete_table_column_native(0, 0, ci, 0).unwrap();
                    interval_check(&reopened, &expected);
                    reopened
                        .insert_table_row_native(0, 0, ci, 0, false)
                        .unwrap();
                    reopened.delete_table_row_native(0, 0, ci, 0).unwrap();
                    interval_check(&reopened, &expected);
                    interval_check_texts(&reopened, &expected_text);
                    if let Some(before) = &child_before {
                        assert_eq!(child(&reopened), *before);
                    }
                }
            }
            c.restore_snapshot_native(snapshot).unwrap();
            assert_eq!(c.export_hwpx_native().unwrap(), source_hwpx);
        }
    }
}

#[test]
fn saved_interval_conflict_recovers_the_original_cells_own_outer_pen() {
    for (kind, source, op, column) in [
        ("v-shared", "shared-vertical-interval", "col-before", true),
        (
            "h-shared",
            "shared-horizontal-interval",
            "row-before",
            false,
        ),
    ] {
        let mut c = load_other(source);
        let original_text = interval_expected_texts(&c, "unchanged");
        let own = interval_oracle(kind);
        interval_apply(&mut c, op);
        for bytes in [
            c.export_hwp_with_adapter_snapshot().unwrap(),
            c.export_hwpx_native().unwrap(),
        ] {
            let mut n = DocumentCore::from_bytes(&bytes).unwrap();
            let before = n.save_snapshot_native();
            let edited = n.export_hwpx_native().unwrap();
            let ci = root(&n).0;
            if column {
                n.delete_table_column_native(0, 0, ci, 0)
            } else {
                n.delete_table_row_native(0, 0, ci, 0)
            }
            .unwrap();
            interval_check(&n, &own);
            interval_check_texts(&n, &original_text);
            assert_eq!(nested_table(&n).cells.len(), 4);
            n.restore_snapshot_native(before).unwrap();
            assert_eq!(n.export_hwpx_native().unwrap(), edited);
        }
    }
}

fn topology_edit(c: &mut DocumentCore, mode: usize) -> Result<String, rhwp::error::HwpError> {
    let ci = root(c).0;
    match mode {
        0 => c.merge_table_cells_native(0, 0, ci, 0, 0, 0, 1),
        1 => c.merge_table_cells_native(0, 0, ci, 0, 0, 1, 0),
        2 => c.split_table_cell_native(0, 0, ci, 0, 0),
        3 => c.split_table_cell_into_native(0, 0, ci, 0, 0, 2, 2, false, true),
        _ => c.split_table_cells_in_range_native(0, 0, ci, 0, 0, 1, 1, 2, 2, false),
    }
}

#[test]
fn unsupported_retained_merge_and_split_preserve_model_events_render_and_both_saves() {
    let source = load();
    let hwp = source.export_hwp_with_adapter_snapshot().unwrap();
    let hwpx = source.export_hwpx_native().unwrap();
    for source in [None, Some(hwp.as_slice()), Some(hwpx.as_slice())] {
        for mode in 0..5 {
            let mut c = source.map_or_else(load, |b| DocumentCore::from_bytes(b).unwrap());
            let model = serde_json::to_value(&c.document().sections[0].paragraphs).unwrap();
            let event_log = c.serialize_event_log();
            let render = c.render_page_svg_native(0).unwrap();
            let before = (
                c.export_hwp_with_adapter_snapshot().unwrap(),
                c.export_hwpx_native().unwrap(),
            );
            let error = topology_edit(&mut c, mode).unwrap_err().to_string();
            assert!(error.contains("아직 지원하지 않습니다"));
            assert_eq!(
                serde_json::to_value(&c.document().sections[0].paragraphs).unwrap(),
                model
            );
            assert_eq!(c.serialize_event_log(), event_log);
            assert_eq!(c.render_page_svg_native(0).unwrap(), render);
            // Dynamic transaction/restoration equality, not a fixed layout golden.
            assert_eq!(c.export_hwp_with_adapter_snapshot().unwrap(), before.0);
            assert_eq!(c.export_hwpx_native().unwrap(), before.1);
        }
    }
}

#[test]
fn one_by_one_retained_split_remains_a_noop() {
    for range in [false, true] {
        let mut c = load();
        let ci = root(&c).0;
        let before = c.export_hwpx_native().unwrap();
        let render = c.render_page_svg_native(0).unwrap();
        if range {
            c.split_table_cells_in_range_native(0, 0, ci, 0, 0, 1, 1, 1, 1, false)
                .unwrap();
        } else {
            c.split_table_cell_into_native(0, 0, ci, 0, 0, 1, 1, false, true)
                .unwrap();
        }
        assert_eq!(c.export_hwpx_native().unwrap(), before);
        assert_eq!(c.render_page_svg_native(0).unwrap(), render);
    }
}

fn load_native_topology(name: &str) -> DocumentCore {
    DocumentCore::from_pdf_bytes(
        &std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("pdf/issue7998/cell-regions/{name}.pdf")),
        )
        .unwrap(),
        &PdfImportOptions::default(),
    )
    .unwrap()
}

fn assert_native_merged_paragraphs(c: &DocumentCore, row: u16, col: u16, want: [&str; 2]) {
    use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
    fn collect(n: &RenderNode, ci: usize, cell: usize, out: &mut [Vec<(String, BoundingBox)>; 2]) {
        if let RenderNodeType::TextRun(t) = &n.node_type {
            if n.visible && !n.editor_only && !t.text.is_empty() {
                if let Some(path) = t.cell_context.as_ref().map(|c| c.path.as_slice()) {
                    if path.len() == 1
                        && path[0].control_index == ci
                        && path[0].cell_index == cell
                        && path[0].cell_para_index < 2
                    {
                        out[path[0].cell_para_index].push((t.text.clone(), n.bbox));
                    }
                }
            }
        }
        for child in &n.children {
            collect(child, ci, cell, out);
        }
    }
    let (ci, table) = root(c);
    assert_eq!(table.cells.len(), 2);
    let cell = table
        .cells
        .iter()
        .position(|c| c.row == row && c.col == col)
        .unwrap();
    assert_eq!(table.cells[cell].paragraphs.len(), 2);
    let tree = c.build_page_render_tree(0).unwrap();
    fn cell_box(n: &RenderNode, ci: usize, cell: usize) -> Option<BoundingBox> {
        if matches!(&n.node_type, RenderNodeType::Table(t) if t.control_index == Some(ci) && t.cell_context.is_none())
        {
            return n.children.iter().find_map(|n| match &n.node_type {
                RenderNodeType::TableCell(t) if t.model_cell_index == Some(cell as u32) => {
                    Some(n.bbox)
                }
                _ => None,
            });
        }
        n.children.iter().find_map(|n| cell_box(n, ci, cell))
    }
    let frame = cell_box(&tree.root, ci, cell).unwrap();
    let mut runs = [vec![], vec![]];
    collect(&tree.root, ci, cell, &mut runs);
    for (found, expected) in runs.iter().zip(want) {
        assert_eq!(
            found.iter().map(|r| r.0.as_str()).collect::<String>(),
            expected,
            "text and final cell/paragraph ownership"
        );
    }
    for (_, bounds) in runs.iter().flatten() {
        assert!(
            bounds.x >= frame.x - 0.01 && bounds.x + bounds.width <= frame.x + frame.width + 0.01
        );
        assert!(
            bounds.y >= frame.y - 0.01 && bounds.y + bounds.height <= frame.y + frame.height + 0.01,
            "merged text belongs inside its cell, not a visually masked overflow"
        );
    }
    let end = runs[0]
        .iter()
        .map(|r| r.1.y + r.1.height)
        .fold(f64::NEG_INFINITY, f64::max);
    let start = runs[1].iter().map(|r| r.1.y).fold(f64::INFINITY, f64::min);
    assert!(
        end <= start + 0.01,
        "successive merged paragraphs must not overlap: {end} > {start}"
    );
}

#[test]
fn native_merges_rebase_separate_cell_line_origins_and_keep_text_after_both_saves() {
    // The independent Hancom Print has two successive paragraphs, not two runs
    // at the same source-cell origin. No absolute glyph coordinate is pinned.
    for (name, row, col, er, ec, want) in [
        ("colspan", 1, 0, 1, 1, ["LOWER LEFT", "LOWER RIGHT"]),
        ("rowspan", 0, 1, 1, 1, ["UPPER RIGHT", "LOWER RIGHT"]),
    ] {
        let mut c = load_native_topology(name);
        let ci = root(&c).0;
        let all_text = text(&c.document().sections[0].paragraphs);
        c.merge_table_cells_native(0, 0, ci, row, col, er, ec)
            .unwrap();
        assert_eq!(text(&c.document().sections[0].paragraphs), all_text);
        assert_native_merged_paragraphs(&c, row, col, want);
        for bytes in [
            c.export_hwp_with_adapter_snapshot().unwrap(),
            c.export_hwpx_native().unwrap(),
        ] {
            let reopened = DocumentCore::from_bytes(&bytes).unwrap();
            assert_eq!(text(&reopened.document().sections[0].paragraphs), all_text);
            assert_eq!(
                (root(&reopened).1.row_count, root(&reopened).1.col_count),
                (2, 2)
            );
            assert_native_merged_paragraphs(&reopened, row, col, want);
        }
    }
}

const ROW_GRID: &str = "META-INF/rhwp-table-row-grids.json";
fn merged_native() -> DocumentCore {
    let mut c = load_native_topology("rowspan");
    let ci = root(&c).0;
    c.merge_table_cells_native(0, 0, ci, 0, 1, 1, 1).unwrap();
    c
}
fn zip_part(bytes: &[u8], name: &str) -> Vec<u8> {
    use std::io::Read;
    let mut z = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let mut b = vec![];
    z.by_name(name).unwrap().read_to_end(&mut b).unwrap();
    b
}
fn change_part(bytes: &[u8], name: &str, replace: Option<&[u8]>) -> Vec<u8> {
    use std::io::{Read, Write};
    let mut z = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let mut target = zip::ZipWriter::new(std::io::Cursor::new(vec![]));
    for i in 0..z.len() {
        let mut p = z.by_index(i).unwrap();
        let n = p.name().unwrap().into_owned();
        if n == name && replace.is_none() {
            continue;
        }
        target
            .start_file(
                &n,
                zip::write::SimpleFileOptions::default().compression_method(p.compression()),
            )
            .unwrap();
        let mut b = vec![];
        p.read_to_end(&mut b).unwrap();
        target
            .write_all(if n == name { replace.unwrap() } else { &b })
            .unwrap();
    }
    target.finish().unwrap().into_inner()
}
fn table_topology(c: &DocumentCore) -> serde_json::Value {
    let t = root(c).1;
    serde_json::json!({"rows":t.row_count,"cols":t.col_count,"cells":t.cells.iter().map(|c|(c.row,c.col,c.row_span,c.col_span,c.width,c.height)).collect::<Vec<_>>()})
}
fn assert_xml_rows(bytes: &[u8], count: usize) {
    let part = zip_part(bytes, "Contents/section0.xml");
    let x = roxmltree::Document::parse(std::str::from_utf8(&part).unwrap()).unwrap();
    let t = x
        .descendants()
        .find(|n| n.has_tag_name(("http://www.hancom.co.kr/hwpml/2011/paragraph", "tbl")))
        .unwrap();
    assert_eq!(
        t.attribute("rowCnt").unwrap().parse::<usize>().unwrap(),
        count
    );
    let rows: Vec<_> = t
        .children()
        .filter(|n| n.is_element() && n.tag_name().name() == "tr")
        .collect();
    assert_eq!(rows.len(), count);
    assert!(
        rows.iter().all(|r| r
            .children()
            .any(|c| c.is_element() && c.tag_name().name() == "tc")),
        "HWPX must not contain the all-covered empty row rejected by Hancom"
    );
}
#[test]
fn hwpx_projects_redundant_rows_and_restores_the_original_split_grid() {
    let c = merged_native();
    let before = table_topology(&c);
    let text_before = text(&c.document().sections[0].paragraphs);
    let bytes = c.export_hwpx_native().unwrap();
    assert_xml_rows(&bytes, 1);
    assert_eq!(table_topology(&c), before);
    let mut restored = DocumentCore::from_bytes(&bytes).unwrap();
    assert_eq!(table_topology(&restored), before);
    assert_eq!(
        text(&restored.document().sections[0].paragraphs),
        text_before
    );
    let ci = root(&restored).0;
    restored.split_table_cell_native(0, 0, ci, 0, 1).unwrap();
    assert_eq!(root(&restored).1.row_count, 2);
    assert_eq!(root(&restored).1.cells.len(), 3);
    assert_eq!(
        text(&restored.document().sections[0].paragraphs),
        text_before
    );
    let plain = DocumentCore::from_bytes(&change_part(&bytes, ROW_GRID, None)).unwrap();
    assert_eq!(root(&plain).1.row_count, 1);
    assert_eq!(root(&plain).1.cells.len(), 2);
    assert_eq!(text(&plain.document().sections[0].paragraphs), text_before);
    assert_eq!(
        root(&plain)
            .1
            .cells
            .iter()
            .map(|c| (c.width, c.height))
            .collect::<Vec<_>>(),
        root(&c)
            .1
            .cells
            .iter()
            .map(|c| (c.width, c.height))
            .collect::<Vec<_>>()
    );
}
#[test]
fn projected_later_row_origins_roundtrip_after_structure_edit() {
    let mut c = merged_native();
    let ci = root(&c).0;
    c.insert_table_row_native(0, 0, ci, 1, true).unwrap();
    let before = table_topology(&c);
    assert_eq!(root(&c).1.row_count, 3);
    let bytes = c.export_hwpx_native().unwrap();
    assert_xml_rows(&bytes, 2);
    let restored = DocumentCore::from_bytes(&bytes).unwrap();
    assert_eq!(table_topology(&restored), before);
}
#[test]
fn stale_or_malformed_row_grid_records_do_not_override_standard_cells() {
    let c = merged_native();
    let bytes = c.export_hwpx_native().unwrap();
    let original: serde_json::Value = serde_json::from_slice(&zip_part(&bytes, ROW_GRID)).unwrap();
    let mut variants = vec![];
    let mut stale = original.clone();
    stale["records"][0]["fingerprint"][0] =
        serde_json::json!((original["records"][0]["fingerprint"][0].as_u64().unwrap() + 1) % 256);
    variants.push(stale);
    for boundaries in [
        serde_json::json!([0, 2, 1]),
        serde_json::json!([1, 2]),
        serde_json::json!([0, 0]),
    ] {
        let mut v = original.clone();
        v["records"][0]["boundaries"] = boundaries;
        variants.push(v);
    }
    let mut duplicate = original.clone();
    let r = duplicate["records"][0].clone();
    duplicate["records"].as_array_mut().unwrap().push(r);
    variants.push(duplicate);
    for v in variants {
        let changed = change_part(&bytes, ROW_GRID, Some(&serde_json::to_vec(&v).unwrap()));
        let d = DocumentCore::from_bytes(&changed).unwrap();
        assert_eq!(root(&d).1.row_count, 1);
        assert_eq!(root(&d).1.cells.len(), 2);
        assert_eq!(
            text(&d.document().sections[0].paragraphs),
            text(&c.document().sections[0].paragraphs)
        );
    }
}
#[test]
fn normal_rows_do_not_add_a_row_grid_sidecar() {
    let c = load_native_topology("rowspan");
    let b = c.export_hwpx_native().unwrap();
    let mut z = zip::ZipArchive::new(std::io::Cursor::new(&b)).unwrap();
    assert!(z.by_name(ROW_GRID).is_err());
    assert_xml_rows(&b, 2);
}
#[test]
fn row_projection_rejects_ambiguous_ids_and_physical_holes() {
    let mut c = merged_native();
    let original = root(&c).1.clone();
    c.document_mut().sections[0].paragraphs[0]
        .controls
        .push(Control::Table(Box::new(original)));
    // 행 격자는 같은 개체 번호의 사본에 새 번호를 주지만, PDF에서 가져온 칸 테두리 소유는
    // 여전히 표 정체성으로 가려야 하므로 모호한 사본의 저장을 거절한다.
    assert!(c
        .export_hwpx_native()
        .unwrap_err()
        .to_string()
        .contains("ambiguous cell border paint ownership"));
    let mut c = merged_native();
    let ci = root(&c).0;
    let Control::Table(t) = &mut c.document_mut().sections[0].paragraphs[0].controls[ci] else {
        panic!()
    };
    for cell in &mut t.cells {
        cell.row_span = 1;
    }
    assert!(c
        .export_hwpx_native()
        .unwrap_err()
        .to_string()
        .contains("빈 논리 행"));
    let mut c = merged_native();
    let ci = root(&c).0;
    let Control::Table(t) = &mut c.document_mut().sections[0].paragraphs[0].controls[ci] else {
        panic!()
    };
    t.cells.push(t.cells[0].clone());
    assert!(c
        .export_hwpx_native()
        .unwrap_err()
        .to_string()
        .contains("빈 논리 행"));
    let mut c = merged_native();
    let ci = root(&c).0;
    let Control::Table(t) = &mut c.document_mut().sections[0].paragraphs[0].controls[ci] else {
        panic!()
    };
    t.zones.push(rhwp::model::table::TableZone {
        start_row: 1,
        end_row: 1,
        start_col: 0,
        end_col: 1,
        border_fill_id: t.border_fill_id,
    });
    assert!(c
        .export_hwpx_native()
        .unwrap_err()
        .to_string()
        .contains("빈 논리 행"));
}

fn topology_scope_load() -> (DocumentCore, usize) {
    let c = DocumentCore::from_pdf_bytes(
        &std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("pdf/issue7998/retained-border-structure/retained-pen.pdf"),
        )
        .unwrap(),
        &PdfImportOptions::default(),
    )
    .unwrap();
    let ci = c.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .position(|v| matches!(v, Control::Table(_)))
        .unwrap();
    (c, ci)
}
fn topology_table(c: &mut DocumentCore, ci: usize) -> &mut Table {
    let Control::Table(t) = &mut c.document_mut().sections[0].paragraphs[0].controls[ci] else {
        panic!()
    };
    t
}
fn clear_topology_paint(ctrl: &mut Control) {
    match ctrl {
        Control::Shape(s) => {
            s.common_mut().cell_border_paint = None;
            s.common_mut().cell_frame_paint = false;
            s.common_mut().outside_cell_border_paint = None;
        }
        Control::Picture(p) => {
            p.common.cell_border_paint = None;
            p.common.cell_frame_paint = false;
            p.common.outside_cell_border_paint = None;
        }
        _ => {}
    }
}
fn clear_topology_cells(t: &mut Table) {
    for p in t.cells.iter_mut().flat_map(|c| &mut c.paragraphs) {
        for ctrl in &mut p.controls {
            clear_topology_paint(ctrl);
        }
    }
}
// In-memory ownership counterexamples, not independent Print/fidelity fixtures.
#[test]
fn outside_binding_alone_blocks_target() {
    let (mut c, ci) = topology_scope_load();
    clear_topology_cells(topology_table(&mut c, ci));
    let before = serde_json::to_vec(&c.document().sections[0].paragraphs).unwrap();
    assert!(c.merge_table_cells_native(0, 0, ci, 0, 0, 0, 1).is_err());
    assert_eq!(
        before,
        serde_json::to_vec(&c.document().sections[0].paragraphs).unwrap()
    );
}
#[test]
fn another_tables_owned_paint_does_not_block_target() {
    let (mut c, ci) = topology_scope_load();
    let original =
        serde_json::to_value(&c.document().sections[0].paragraphs[0].controls[ci]).unwrap();
    let mut candidate = topology_table(&mut c, ci).clone();
    clear_topology_cells(&mut candidate);
    candidate.common.instance_id = 9999;
    let next = c.document().sections[0].paragraphs[0].controls.len();
    c.document_mut().sections[0].paragraphs[0]
        .controls
        .push(Control::Table(Box::new(candidate)));
    c.merge_table_cells_native(0, 0, next, 0, 0, 0, 1).unwrap();
    assert_eq!(
        serde_json::to_value(&c.document().sections[0].paragraphs[0].controls[ci]).unwrap(),
        original
    );
}
#[test]
fn ordinary_drawings_are_not_border_authority() {
    let (mut c, ci) = topology_scope_load();
    for ctrl in &mut c.document_mut().sections[0].paragraphs[0].controls {
        clear_topology_paint(ctrl);
    }
    clear_topology_cells(topology_table(&mut c, ci));
    c.merge_table_cells_native(0, 0, ci, 0, 0, 0, 1).unwrap();
}
#[test]
fn child_owned_paint_does_not_block_parent() {
    let (mut c, ci) = topology_scope_load();
    let nested = topology_table(&mut c, ci).clone();
    for ctrl in &mut c.document_mut().sections[0].paragraphs[0].controls {
        clear_topology_paint(ctrl);
    }
    let t = topology_table(&mut c, ci);
    clear_topology_cells(t);
    t.common.instance_id = 9999;
    let place = t.cells[0]
        .paragraphs
        .iter_mut()
        .flat_map(|p| &mut p.controls)
        .find(|v| matches!(v, Control::Table(_)))
        .unwrap();
    *place = Control::Table(Box::new(nested));
    c.merge_table_cells_native(0, 0, ci, 0, 0, 0, 1).unwrap();
    let t = topology_table(&mut c, ci);
    assert!(t.cells.iter().flat_map(|c|&c.paragraphs).flat_map(|p|&p.controls).any(|v|matches!(v,Control::Table(t)if t.cells.iter().flat_map(|c|&c.paragraphs).flat_map(|p|&p.controls).any(|v|matches!(v,Control::Shape(s)if s.common().cell_border_paint.is_some())||matches!(v,Control::Picture(p)if p.common.cell_border_paint.is_some())))));
}

// Stage28: generated, explicitly closed compound perimeters. These are not the
// unmodified Hancom Print (whose endpoint extensions remain outside the proof).
fn load_merged_retained(name: &str) -> DocumentCore {
    DocumentCore::from_pdf_bytes(
        &std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("pdf/issue7998/merged-neighbor/{name}.pdf")),
        )
        .unwrap(),
        &PdfImportOptions::default(),
    )
    .unwrap()
}
fn merged_structure_edit(
    c: &mut DocumentCore,
    mode: usize,
) -> Result<String, rhwp::error::HwpError> {
    let ci = root(c).0;
    match mode {
        0..=5 => apply(c, mode),
        6 => c.insert_table_row_native(0, 0, ci, 1, true),
        7 => c.insert_table_column_native(0, 0, ci, 1, true),
        8 => c.delete_table_row_native(0, 0, ci, 0),
        _ => c.delete_table_column_native(0, 0, ci, 0),
    }
}
fn assert_merged_owned_sides(c: &DocumentCore) {
    use rhwp::model::border_paint::CellSide;
    assert!(tagged(c) > 0, "source ink has actual editable ownership");
    for cell in &root(c).1.cells {
        let mask = cell
            .paragraphs
            .iter()
            .flat_map(|p| &p.controls)
            .filter_map(|ctrl| {
                let common = match ctrl {
                    Control::Shape(s) => s.common(),
                    Control::Picture(p) => &p.common,
                    _ => return None,
                };
                let t = common.cell_border_paint.as_ref()?;
                Some(if t.sides != 0 {
                    t.sides
                } else {
                    match t.side {
                        CellSide::Left => 1,
                        CellSide::Right => 2,
                        CellSide::Top => 4,
                        CellSide::Bottom => 8,
                    }
                })
            })
            .fold(0, |a, b| a | b);
        assert_eq!(
            mask, 15,
            "all four real sides keep ownership, including long merged sides"
        );
        let fill = &c.document().doc_info.border_fills[cell.border_fill_id as usize - 1];
        assert!(fill.borders[..4]
            .iter()
            .all(|b| b.line_type == BorderLineType::None));
    }
}
fn assert_merged_text_in_cells(c: &DocumentCore) {
    use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
    fn find(n: &RenderNode, ci: usize) -> Option<&RenderNode> {
        if matches!(&n.node_type, RenderNodeType::Table(t) if t.control_index == Some(ci) && t.cell_context.is_none())
        {
            return Some(n);
        }
        n.children.iter().find_map(|n| find(n, ci))
    }
    fn runs(n: &RenderNode, ci: usize, cell: usize, out: &mut Vec<(String, BoundingBox)>) {
        if let RenderNodeType::TextRun(t) = &n.node_type {
            if n.visible
                && !n.editor_only
                && !t.text.is_empty()
                && t.cell_context.as_ref().is_some_and(|p| {
                    p.path.len() == 1
                        && p.path[0].control_index == ci
                        && p.path[0].cell_index == cell
                })
            {
                out.push((t.text.clone(), n.bbox));
            }
        }
        for n in &n.children {
            runs(n, ci, cell, out);
        }
    }
    let (ci, model) = root(c);
    let tree = c.build_page_render_tree(0).unwrap();
    let table = find(&tree.root, ci).unwrap();
    let frames: Vec<_> = table
        .children
        .iter()
        .filter_map(|n| {
            if let RenderNodeType::TableCell(t) = &n.node_type {
                Some((t.model_cell_index.unwrap() as usize, n.bbox))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(frames.len(), model.cells.len());
    for (index, frame) in frames {
        let mut found = vec![];
        runs(&tree.root, ci, index, &mut found);
        assert_eq!(
            found.iter().map(|t| t.0.as_str()).collect::<String>(),
            text(&model.cells[index].paragraphs)
        );
        for (_, b) in found {
            assert!(b.x >= frame.x - 0.01 && b.x + b.width <= frame.x + frame.width + 0.01);
            assert!(
                b.x >= tree.root.bbox.x - 0.01
                    && b.x + b.width <= tree.root.bbox.x + tree.root.bbox.width + 0.01,
                "source text remains on the paper after column insertion"
            );
            assert!(
                b.y >= frame.y - 0.01 && b.y + b.height <= frame.y + frame.height + 0.01,
                "actual text must remain in its logical cell"
            );
        }
    }
}
#[test]
fn merged_compound_source_has_real_perimeter_owners_without_inventing_interior_edges() {
    for (name, span, label) in [
        ("colspan-retained", (1, 2), "MERGED HEADER"),
        ("rowspan-retained", (2, 1), "MERGED LEFT"),
        ("rowspan-top-retained", (2, 1), "MERGED LEFT"),
    ] {
        let c = load_merged_retained(name);
        let table = root(&c).1;
        assert_eq!(
            (table.row_count, table.col_count, table.cells.len()),
            (2, 2, 3)
        );
        let cell = table
            .cells
            .iter()
            .find(|c| text(&c.paragraphs).contains(label))
            .unwrap();
        assert_eq!((cell.row_span, cell.col_span), span);
        assert_merged_owned_sides(&c);
        assert_merged_text_in_cells(&c);
    }
}
#[test]
fn merged_retained_neighbors_inherit_real_sides_across_structure_edits_and_both_saves() {
    // Counts follow the source's three logical cells and the insertion/deletion
    // operation; a new cut inside a merged cell expands its span, not its text.
    for (name, modes) in [
        ("colspan-retained", &[0, 1, 2, 3, 5, 6, 7, 8, 9][..]),
        ("rowspan-retained", &[2, 3, 7, 9][..]),
        ("rowspan-top-retained", &[0, 6][..]),
    ] {
        for &mode in modes {
            let mut c = load_merged_retained(name);
            let original = text(&c.document().sections[0].paragraphs);
            merged_structure_edit(&mut c, mode).unwrap();
            let t = root(&c).1;
            let dims = match mode {
                0 | 1 | 6 => (3, 2),
                2 | 3 | 7 => (2, 3),
                8 => (1, 2),
                _ => (2, 1),
            };
            assert_eq!((t.row_count, t.col_count), dims);
            assert_eq!(
                t.cells.len(),
                if mode >= 5 && mode != 6 && mode != 7 {
                    2
                } else if name.starts_with("colspan") && mode == 3 {
                    4
                } else {
                    5
                }
            );
            let after = text(&c.document().sections[0].paragraphs);
            if mode <= 3 || mode == 6 || mode == 7 {
                assert_eq!(after, original)
            } else {
                let removed = match (name, mode) {
                    ("colspan-retained", 5) => "LOWER RIGHT",
                    ("colspan-retained", 8) => "MERGED HEADER",
                    ("colspan-retained", 9) => "LOWER LEFT",
                    _ => "MERGED LEFT",
                };
                assert!(!after.contains(removed));
                assert_eq!(after, original.replace(removed, ""));
            }
            assert_merged_owned_sides(&c);
            assert_merged_text_in_cells(&c);
            let topology: Vec<_> = t
                .cells
                .iter()
                .map(|c| (c.row, c.col, c.row_span, c.col_span))
                .collect();
            for bytes in [
                c.export_hwp_with_adapter_snapshot().unwrap(),
                c.export_hwpx_native().unwrap(),
            ] {
                let n = DocumentCore::from_bytes(&bytes).unwrap();
                assert_eq!(text(&n.document().sections[0].paragraphs), after);
                assert_eq!(
                    root(&n)
                        .1
                        .cells
                        .iter()
                        .map(|c| (c.row, c.col, c.row_span, c.col_span))
                        .collect::<Vec<_>>(),
                    topology
                );
                assert_merged_owned_sides(&n);
                assert_merged_text_in_cells(&n);
            }
        }
    }
}
#[test]
fn merged_retained_unproven_grid_and_varying_side_reject_before_document_changes() {
    for (name, modes) in [
        ("colspan-retained", &[4][..]),
        ("rowspan-retained", &[0, 1, 4, 5, 6, 8][..]),
        ("colspan-varying-side", &[0, 1, 2, 3, 4, 5][..]),
    ] {
        for &mode in modes {
            let mut c = load_merged_retained(name);
            assert!(tagged(&c) > 0);
            let before = (
                serde_json::to_value(&c.document().sections[0].paragraphs).unwrap(),
                c.serialize_event_log(),
                c.export_hwp_with_adapter_snapshot().unwrap(),
                c.export_hwpx_native().unwrap(),
                c.render_page_svg_native(0).unwrap(),
            );
            let error = merged_structure_edit(&mut c, mode).unwrap_err().to_string();
            if name == "rowspan-retained" && (mode == 0 || mode == 6) {
                assert!(error.contains("실제 셀 배치"));
            }
            assert_eq!(
                before,
                (
                    serde_json::to_value(&c.document().sections[0].paragraphs).unwrap(),
                    c.serialize_event_log(),
                    c.export_hwp_with_adapter_snapshot().unwrap(),
                    c.export_hwpx_native().unwrap(),
                    c.render_page_svg_native(0).unwrap()
                )
            );
        }
    }
}
