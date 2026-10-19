//! Proven source/page ownership and actual edit/save contracts; no rendering golden.
#![cfg(feature = "pdf-import")]
use rhwp::{
    model::{border_paint, control::Control, shape::ShapeObject, table::Table},
    parser::pdf::PdfImportOptions,
    DocumentCore,
};
use std::{fs, path::PathBuf};
fn load(key: &str) -> DocumentCore {
    DocumentCore::from_pdf_bytes(
        &fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(format!("pdf/issue7998/periodic-borders/{key}.pdf")),
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
        .find_map(|(i, c)| match c {
            Control::Table(t) => Some((i, t.as_ref())),
            _ => None,
        })
        .unwrap()
}
fn outside(c: &DocumentCore) -> Vec<u32> {
    c.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .filter_map(|c| match c {
            Control::Shape(s) if s.common().outside_cell_border_paint.is_some() => {
                Some(s.common().instance_id)
            }
            _ => None,
        })
        .collect()
}
fn untouched(c: &DocumentCore) -> serde_json::Value {
    let v: Vec<_> = c.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .filter(|c| {
            !matches!(c, Control::Table(_))
                && !matches!(c,Control::Shape(s)if s.common().outside_cell_border_paint.is_some())
        })
        .collect();
    serde_json::to_value(v).unwrap()
}
fn apply(c: &mut DocumentCore, cell: usize, props: &str) -> serde_json::Value {
    let i = root(c).0;
    serde_json::from_str(&c.set_cell_properties_native(0, 0, i, cell, props).unwrap()).unwrap()
}
fn inverse(c: &mut DocumentCore, r: &serde_json::Value) {
    let i = root(c).0;
    let cells: Vec<_> = r["changes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| serde_json::json!({"cellIdx":v["cellIdx"],"id":v["beforeId"]}))
        .collect();
    let ids: Vec<_> = r["borderPaintUndoId"].as_u64().into_iter().collect();
    c.apply_cell_border_fill_ids_native(
        0,
        0,
        i,
        &serde_json::json!({"cells":cells,"zones":[],"borderPaintUndoIds":ids}).to_string(),
    )
    .unwrap();
    c.remove_border_fill_tails_native(
        &serde_json::json!({"fromLen":r["borderFillLenBefore"],"dirtyWas":r["docInfoDirtyBefore"]})
            .to_string(),
    )
    .unwrap();
}
const LEFT: &str = r##"{"borderLeft":{"type":1,"width":5,"color":"#d00000"}}"##;
#[test]
fn external_stroke_replacement_preserves_other_cells_fill_and_slots() {
    for key in ["occluded-inner-dot", "thin-fill-occluder"] {
        for props in [
            LEFT,
            r##"{"borderLeft":{"type":0,"width":0,"color":"#000000"}}"##,
        ] {
            let mut c = load(key);
            let before = c.export_hwpx_native().unwrap();
            let index = root(&c).0;
            let keep = untouched(&c);
            let old = outside(&c);
            assert_eq!(
                old.len(),
                8,
                "four outer strokes split across two cell intervals each"
            );
            let r = apply(&mut c, 0, props);
            assert_eq!(outside(&c).len(), old.len() - 1);
            assert_eq!(root(&c).0, index);
            assert_eq!(
                untouched(&c),
                keep,
                "fill/content may not be claimed by stroke bbox"
            );
            inverse(&mut c, &r);
            assert_eq!(c.export_hwpx_native().unwrap(), before);
        }
    }
}
#[test]
fn native_files_keep_page_owner_and_allow_the_same_edit_after_reopen() {
    let c = load("thin-fill-occluder");
    let expected = border_paint::encode(c.document()).unwrap();
    assert_eq!(&expected.as_ref().unwrap()[..8], b"RBPO\0\0\0\x02");
    for bytes in [
        c.export_hwp_with_adapter_snapshot().unwrap(),
        c.export_hwpx_native().unwrap(),
    ] {
        let mut n = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(border_paint::encode(n.document()).unwrap(), expected);
        let before = n.export_hwpx_native().unwrap();
        let r = apply(&mut n, 0, LEFT);
        assert_eq!(outside(&n).len(), 7);
        inverse(&mut n, &r);
        assert_eq!(n.export_hwpx_native().unwrap(), before);
    }
}
#[test]
fn stale_shape_table_and_early_slots_reject_before_mutation() {
    for mode in ["shape", "table", "slot"] {
        let mut c = load("thin-fill-occluder");
        let mut d = c.document().clone();
        let p = &mut d.sections[0].paragraphs[0];
        let i = p
            .controls
            .iter()
            .position(|c| matches!(c, Control::Table(_)))
            .unwrap();
        if mode == "table" {
            let Control::Table(t) = &mut p.controls[i] else {
                unreachable!()
            };
            t.common.horizontal_offset += 100;
        } else {
            let j=p.controls.iter().position(|c|matches!(c,Control::Shape(s)if s.common().outside_cell_border_paint.is_some())).unwrap();
            if mode == "slot" {
                let v = p.controls.remove(j);
                p.controls.insert(0, v);
            } else {
                let Control::Shape(s) = &mut p.controls[j] else {
                    unreachable!()
                };
                s.common_mut().horizontal_offset += 100;
            }
        }
        c.set_document(d);
        let before = c.export_hwpx_native().unwrap();
        let i = root(&c).0;
        let props = if mode != "table" {
            r##"{"borderTop":{"type":1,"width":5,"color":"#d00000"}}"##
        } else {
            LEFT
        };
        assert!(
            c.set_cell_properties_native(0, 0, i, 0, props).is_err(),
            "{mode}"
        );
        assert_eq!(c.export_hwpx_native().unwrap(), before);
    }
}
#[test]
fn parent_journal_survives_text_inverse_and_two_cell_edits() {
    let mut c = load("thin-fill-occluder");
    let before = c.export_hwpx_native().unwrap();
    let a = apply(&mut c, 0, LEFT);
    let b = apply(&mut c, 2, LEFT);
    assert_eq!(outside(&c).len(), 6);
    let i = root(&c).0;
    c.insert_text_in_cell_native(0, 0, i, 0, 0, 0, "X").unwrap();
    c.delete_text_in_cell_native(0, 0, i, 0, 0, 0, 1).unwrap();
    inverse(&mut c, &b);
    inverse(&mut c, &a);
    assert_eq!(c.export_hwpx_native().unwrap(), before);
}
#[test]
fn bounded_codec_rejects_truncated_or_unknown_outside_records() {
    let c = load("thin-fill-occluder");
    let bytes = border_paint::encode(c.document()).unwrap().unwrap();
    let mut invalid: Vec<Vec<u8>> = [0, 7, 11, bytes.len() - 1]
        .iter()
        .map(|&n| bytes[..n].to_vec())
        .collect();
    let mut unknown = bytes.clone();
    unknown[40] = 0x40;
    invalid.push(unknown);
    for bad in invalid {
        let mut d = c.document().clone();
        for ctrl in &mut d.sections[0].paragraphs[0].controls {
            if let Control::Shape(s) = ctrl {
                s.common_mut().outside_cell_border_paint = None;
            }
        }
        border_paint::decode(&mut d, &bad);
        assert!(d.sections[0].paragraphs[0].controls.iter().all(
            |c| !matches!(c,Control::Shape(s)if s.common().outside_cell_border_paint.is_some())
        ));
    }
}

#[test]
fn missing_undo_target_returns_error_instead_of_panicking() {
    let mut c = load("thin-fill-occluder");
    let i = root(&c).0;
    let r = apply(&mut c, 0, LEFT);
    c.document_mut().sections.clear();
    let p = serde_json::json!({"borderPaintUndoIds":[r["borderPaintUndoId"]]});
    assert!(c
        .apply_cell_border_fill_ids_native(0, 0, i, &p.to_string())
        .is_err());
    assert!(c.document().sections.is_empty());
}

#[test]
fn unchanged_dialog_preserves_page_and_cell_source_ownership() {
    let mut c = load("thin-fill-occluder");
    let i = root(&c).0;
    let before = border_paint::encode(c.document()).unwrap();
    let mut p: serde_json::Value = serde_json::from_str(
        &c.get_cell_properties_by_cell_path_native(0, 0, &[(i, 0, 0)], 0)
            .unwrap(),
    )
    .unwrap();
    p.as_object_mut()
        .unwrap()
        .retain(|k, _| k.starts_with("border"));
    p["borderPaintSides"] = serde_json::json!([false, false, false, false]);
    let r = apply(&mut c, 0, &p.to_string());
    assert!(r["borderPaintUndoId"].is_null());
    assert_eq!(border_paint::encode(c.document()).unwrap(), before);
    assert_eq!(outside(&c).len(), 8);
}
