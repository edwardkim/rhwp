//! Qualified source and edited Print evidence: stage16 and p4-border-replacement.
//! Ownership/content and real writer/undo relations; no rendering golden or px pins.
#![cfg(feature = "pdf-import")]
use rhwp::{
    model::{
        border_paint::{CellBorderPaint, CellSide},
        control::Control,
        table::Table,
    },
    parser::pdf::PdfImportOptions,
    DocumentCore,
};
use std::{collections::BTreeMap, path::PathBuf};
fn load(key: &str) -> DocumentCore {
    DocumentCore::from_pdf_bytes(
        &std::fs::read(
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
        .find_map(|(i, c)| {
            if let Control::Table(t) = c {
                Some((i, t.as_ref()))
            } else {
                None
            }
        })
        .unwrap()
}
fn bindings(t: &Table) -> BTreeMap<(usize, u32), CellBorderPaint> {
    t.cells
        .iter()
        .enumerate()
        .flat_map(|(ci, c)| {
            c.paragraphs.iter().flat_map(move |p| {
                p.controls.iter().filter_map(move |c| {
                    if let Control::Shape(s) = c {
                        s.common()
                            .cell_border_paint
                            .map(|b| ((ci, s.common().instance_id), b))
                    } else {
                        None
                    }
                })
            })
        })
        .collect()
}
fn content(t: &Table) -> Vec<serde_json::Value> {
    t.cells
        .iter()
        .flat_map(|c| {
            c.paragraphs.iter().flat_map(|p| {
                p.controls
                    .iter()
                    .filter(
                        |c| !matches!(c,Control::Shape(s)if s.common().cell_border_paint.is_some()),
                    )
                    .map(|c| serde_json::to_value(c).unwrap())
            })
        })
        .collect()
}
fn text(t: &Table) -> Vec<String> {
    t.cells
        .iter()
        .flat_map(|c| c.paragraphs.iter().map(|p| p.text.clone()))
        .collect()
}
fn model(c: &DocumentCore) -> serde_json::Value {
    serde_json::to_value(root(c).1).unwrap()
}
fn apply(c: &mut DocumentCore, cell: usize, props: &str) -> serde_json::Value {
    let ci = root(c).0;
    serde_json::from_str(&c.set_cell_properties_native(0, 0, ci, cell, props).unwrap()).unwrap()
}
fn inverse(c: &mut DocumentCore, records: &[serde_json::Value]) {
    let ci = root(c).0;
    let mut cells = BTreeMap::new();
    let mut ids = vec![];
    for r in records {
        for x in r["changes"].as_array().unwrap() {
            cells
                .entry(x["cellIdx"].as_u64().unwrap())
                .or_insert(x["beforeId"].as_u64().unwrap());
        }
        if let Some(id) = r["borderPaintUndoId"].as_u64() {
            ids.push(id);
        }
    }
    let cells: Vec<_> = cells
        .into_iter()
        .map(|(cell, id)| serde_json::json!({"cellIdx":cell,"id":id}))
        .collect();
    c.apply_cell_border_fill_ids_native(
        0,
        0,
        ci,
        &serde_json::json!({"cells":cells,"zones":[],"borderPaintUndoIds":ids}).to_string(),
    )
    .unwrap();
    c.remove_border_fill_tails_native(&serde_json::json!({"fromLen":records[0]["borderFillLenBefore"],"dirtyWas":records[0]["docInfoDirtyBefore"]}).to_string()).unwrap();
}
const RED: &str = r##"{"borderRight":{"type":1,"width":5,"color":"#d00000"}}"##;
#[test]
fn sparse_side_replaces_both_physical_owners_and_preserves_other_content() {
    for key in ["phase-shifted-dot", "wrong-width-dot", "thin-fill-occluder"] {
        let mut c = load(key);
        let before = model(&c);
        let old = bindings(root(&c).1);
        let want_text = text(root(&c).1);
        let want_content = content(root(&c).1);
        let expected: BTreeMap<_, _> = old
            .iter()
            .filter(|((cell, _), b)| {
                !((*cell == 0 && b.side == CellSide::Right)
                    || (*cell == 1 && b.side == CellSide::Left))
            })
            .map(|(k, v)| (*k, *v))
            .collect();
        assert!(expected.len() < old.len());
        let response = apply(&mut c, 0, RED);
        assert_eq!(bindings(root(&c).1), expected);
        assert_eq!(text(root(&c).1), want_text);
        assert_eq!(content(root(&c).1), want_content);
        for cell in [0, 1] {
            let t = root(&c).1;
            let b = &c.document().doc_info.border_fills[t.cells[cell].border_fill_id as usize - 1];
            assert_eq!(b.borders[if cell == 0 { 1 } else { 0 }].color, 0xd0);
        }
        for bytes in [
            c.export_hwp_with_adapter_snapshot().unwrap(),
            c.export_hwpx_native().unwrap(),
        ] {
            let n = DocumentCore::from_bytes(&bytes).unwrap();
            assert_eq!(bindings(root(&n).1), expected);
            assert_eq!(text(root(&n).1), want_text);
        }
        inverse(&mut c, &[response]);
        assert!(
            model(&c) == before,
            "inverse must restore exact source controls, nested content, slots and ownership"
        );
    }
}
#[test]
fn explicit_none_changes_source_paint_even_when_native_id_does_not_change() {
    let mut c = load("phase-shifted-dot");
    let before = model(&c);
    let r = apply(
        &mut c,
        0,
        r##"{"borderRight":{"type":0,"width":0,"color":"#000000"},"borderPaintSides":[false,true,false,false]}"##,
    );
    assert!(r["borderPaintUndoId"].is_number());
    assert!(r["changes"].as_array().unwrap().is_empty());
    assert!(bindings(root(&c).1).len() < bindings(root(&load("phase-shifted-dot")).1).len());
    inverse(&mut c, &[r]);
    assert!(model(&c) == before);
}
#[test]
fn full_unchanged_dialog_and_background_only_preserve_source_roles() {
    for action in ["noop", "background"] {
        let mut c = load("phase-shifted-dot");
        let old = bindings(root(&c).1);
        let ci = root(&c).0;
        let mut value: serde_json::Value = serde_json::from_str(
            &c.get_cell_properties_by_cell_path_native(0, 0, &[(ci, 0, 0)], 0)
                .unwrap(),
        )
        .unwrap();
        value
            .as_object_mut()
            .unwrap()
            .retain(|k, _| k.starts_with("border"));
        value["borderPaintSides"] = serde_json::json!([false, false, false, false]);
        if action == "background" {
            value["fillType"] = "solid".into();
            value["fillColor"] = "#ccffff".into();
        }
        let r = apply(&mut c, 0, &value.to_string());
        assert!(r["borderPaintUndoId"].is_null());
        assert_eq!(bindings(root(&c).1), old);
    }
}
#[test]
fn stale_bindings_and_simultaneous_resize_fail_before_any_mutation() {
    for mode in ["stale", "resize"] {
        let mut c = load("thin-fill-occluder");
        let ci = root(&c).0;
        if mode == "stale" {
            let mut d = c.document().clone();
            if let Control::Table(t) = &mut d.sections[0].paragraphs[0].controls[ci] {
                for ctrl in t.cells[0]
                    .paragraphs
                    .iter_mut()
                    .flat_map(|p| &mut p.controls)
                {
                    if let Control::Shape(s) = ctrl {
                        if s.common()
                            .cell_border_paint
                            .is_some_and(|b| b.side == CellSide::Right)
                        {
                            s.common_mut().horizontal_offset += 100;
                            break;
                        }
                    }
                }
            }
            c.set_document(d);
        }
        let before = c.export_hwpx_native().unwrap();
        let props = if mode == "resize" {
            r##"{"borderRight":{"type":1,"width":5,"color":"#d00000"},"width":12000}"##
        } else {
            RED
        };
        assert!(c.set_cell_properties_native(0, 0, ci, 0, props).is_err());
        assert_eq!(c.export_hwpx_native().unwrap(), before);
    }
}
#[test]
fn reverse_batch_restores_two_changes_in_one_paragraph_and_rejects_unknown_ids() {
    let mut c = load("phase-shifted-dot");
    let before = model(&c);
    let a = apply(&mut c, 0, RED);
    let b = apply(
        &mut c,
        0,
        r##"{"borderBottom":{"type":1,"width":5,"color":"#0033cc"}}"##,
    );
    let changed = model(&c);
    let ci = root(&c).0;
    assert!(c
        .apply_cell_border_fill_ids_native(
            0,
            0,
            ci,
            r#"{"cells":[{"cellIdx":0,"id":1}],"borderPaintUndoIds":[999999]}"#
        )
        .is_err());
    assert!(model(&c) == changed);
    inverse(&mut c, &[a, b]);
    assert!(model(&c) == before);
}
#[test]
fn border_undo_after_text_edit_and_its_inverse_keeps_original_text_and_paint() {
    let mut c = load("phase-shifted-dot");
    let before = model(&c);
    let ci = root(&c).0;
    let a = apply(&mut c, 0, RED);
    c.insert_text_in_cell_native(0, 0, ci, 0, 0, 0, "X")
        .unwrap();
    c.delete_text_in_cell_native(0, 0, ci, 0, 0, 0, 1).unwrap();
    inverse(&mut c, &[a]);
    assert!(model(&c) == before);
}
