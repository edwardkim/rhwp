#![cfg(feature = "pdf-import")]
use rhwp::{
    model::{bin_data::BinDataBytes, border_paint, control::Control, image::Picture, table::Table},
    parser::pdf::PdfImportOptions,
    renderer::render_tree::{BoundingBox, ImageNode, PageRenderTree, RenderNode, RenderNodeType},
    DocumentCore,
};
use serde_json::{json, Value};
use std::{fs, path::PathBuf};
fn load(key: &str) -> DocumentCore {
    DocumentCore::from_pdf_bytes(
        &fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(format!("pdf/issue7998/compound-borders/{key}.pdf")),
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
fn apply(c: &mut DocumentCore, action: &str) -> Value {
    let edge = json!({"type":if action=="right"{1}else{0},"width":if action=="right"{5}else{0},"color":"#d00000"});
    let mut p = json!({});
    for (i, key) in ["borderLeft", "borderRight", "borderTop", "borderBottom"]
        .iter()
        .enumerate()
    {
        if action == "all"
            || (action == "left" && i == 0)
            || (action == "right" && i == 1)
            || (action == "top" && i == 2)
        {
            p[*key] = edge.clone();
        }
    }
    let ci = root(c).0;
    serde_json::from_str(
        &c.set_cell_properties_native(0, 0, ci, 0, &p.to_string())
            .unwrap(),
    )
    .unwrap()
}
fn undo(c: &mut DocumentCore, r: &Value) {
    let ci = root(c).0;
    let cells: Vec<_> = r["changes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| json!({"cellIdx":v["cellIdx"],"id":v["beforeId"]}))
        .collect();
    c.apply_cell_border_fill_ids_native(
        0,
        0,
        ci,
        &json!({"cells":cells,"zones":[],"borderPaintUndoIds":[r["borderPaintUndoId"]]})
            .to_string(),
    )
    .unwrap();
    c.remove_border_fill_tails_native(
        &json!({"fromLen":r["borderFillLenBefore"],"dirtyWas":r["docInfoDirtyBefore"]}).to_string(),
    )
    .unwrap();
}
fn child(c: &DocumentCore) -> Vec<Value> {
    root(c)
        .1
        .cells
        .iter()
        .flat_map(|c| &c.paragraphs)
        .flat_map(|p| &p.controls)
        .filter(|c| matches!(c, Control::Table(_)))
        .map(|c| serde_json::to_value(c).unwrap())
        .collect()
}
fn pictures(t: &Table) -> Vec<(usize, &Picture)> {
    t.cells
        .iter()
        .enumerate()
        .flat_map(|(ci, c)| {
            c.paragraphs.iter().flat_map(move |p| {
                p.controls.iter().filter_map(move |c| {
                    if let Control::Picture(p) = c {
                        Some((ci, p.as_ref()))
                    } else {
                        None
                    }
                })
            })
        })
        .collect()
}
fn mask(t: border_paint::CellBorderPaint) -> u8 {
    if t.sides != 0 {
        t.sides
    } else {
        match t.side {
            border_paint::CellSide::Left => 1,
            border_paint::CellSide::Right => 2,
            border_paint::CellSide::Top => 4,
            border_paint::CellSide::Bottom => 8,
        }
    }
}
fn roles(c: &DocumentCore) -> Vec<(u32, u8)> {
    pictures(root(c).1)
        .into_iter()
        .filter(|(ci, _)| *ci == 0)
        .filter_map(|(_, p)| {
            p.common
                .cell_border_paint
                .map(|t| (p.common.instance_id, mask(t)))
        })
        .collect()
}
#[test]
fn shared_axis_replacement_preserves_other_axes_neighbors_and_child() {
    for key in ["compound-solid", "compound-dash", "compound-alpha"] {
        let mut c = load(key);
        let nested = child(&c);
        let text = root(&c)
            .1
            .cells
            .iter()
            .flat_map(|c| &c.paragraphs)
            .map(|p| p.text.clone())
            .collect::<Vec<_>>();
        let untouched = serde_json::to_value(&root(&c).1.cells[2..]).unwrap();
        let old = pictures(root(&c).1)
            .into_iter()
            .filter_map(|(ci, p)| {
                p.common
                    .cell_border_paint
                    .map(|t| (ci, p.common.instance_id, mask(t)))
            })
            .collect::<Vec<_>>();
        let r = apply(&mut c, "right");
        assert!(r["borderPaintUndoId"].is_number());
        assert_eq!(child(&c), nested);
        assert_eq!(
            root(&c)
                .1
                .cells
                .iter()
                .flat_map(|c| &c.paragraphs)
                .map(|p| p.text.clone())
                .collect::<Vec<_>>(),
            text
        );
        assert_eq!(
            serde_json::to_value(&root(&c).1.cells[2..]).unwrap(),
            untouched
        );
        let after = pictures(root(&c).1)
            .into_iter()
            .filter_map(|(ci, p)| {
                p.common
                    .cell_border_paint
                    .map(|t| (ci, p.common.instance_id, mask(t)))
            })
            .collect::<Vec<_>>();
        let expected = old
            .into_iter()
            .filter_map(|(ci, id, m)| {
                let m = m & !match ci {
                    0 => 2,
                    1 => 1,
                    _ => 0,
                };
                (m != 0).then_some((ci, id, m))
            })
            .collect::<Vec<_>>();
        assert_eq!(
            after, expected,
            "shared edge must not consume another axis with the same source identity"
        );
    }
}
#[test]
fn joint_survives_one_axis_deletion_until_its_last_ink_owner_is_removed() {
    let mut c = load("compound-solid");
    let joint = roles(&c).into_iter().find(|(_, m)| *m == 5).unwrap().0;
    apply(&mut c, "left");
    assert!(roles(&c).contains(&(joint, 4)));
    apply(&mut c, "top");
    assert!(!roles(&c).iter().any(|(id, _)| *id == joint));
}
#[test]
fn both_formats_keep_editing_roles_and_commands_undo_after_text_inverse() {
    for (key, action) in [
        ("compound-solid", "left"),
        ("compound-solid", "right"),
        ("compound-dash", "all"),
        ("compound-dash", "right"),
        ("compound-alpha", "all"),
        ("compound-alpha", "left"),
    ] {
        let mut c = load(key);
        let original = c.export_hwpx_native().unwrap();
        let r = apply(&mut c, action);
        assert!(r["borderPaintUndoId"].is_number());
        let changed = c.export_hwpx_native().unwrap();
        for bytes in [
            c.export_hwp_with_adapter_snapshot().unwrap(),
            changed.clone(),
        ] {
            let mut n = DocumentCore::from_bytes(&bytes).unwrap();
            assert_eq!(
                border_paint::encode(n.document()).unwrap(),
                border_paint::encode(c.document()).unwrap()
            );
            let old = n.export_hwpx_native().unwrap();
            let r = apply(&mut n, "top");
            if r["borderPaintUndoId"].is_number() {
                undo(&mut n, &r);
                assert_eq!(n.export_hwpx_native().unwrap(), old);
            }
        }
        let ci = root(&c).0;
        c.insert_text_in_cell_native(0, 0, ci, 0, 0, 0, "X")
            .unwrap();
        c.delete_text_in_cell_native(0, 0, ci, 0, 0, 0, 1).unwrap();
        undo(&mut c, &r);
        assert_eq!(c.export_hwpx_native().unwrap(), original);
        let r = apply(&mut c, action);
        assert_eq!(c.export_hwpx_native().unwrap(), changed);
        undo(&mut c, &r);
    }
}
#[test]
fn stale_source_or_noncanonical_side_mask_rejects_before_mutation() {
    for mode in [
        "crop",
        "geometry",
        "resource",
        "bad-mask",
        "noncanonical-mask",
    ] {
        let mut c = load("compound-solid");
        let ci = root(&c).0;
        let mut d = c.document().clone();
        let Control::Table(t) = &mut d.sections[0].paragraphs[0].controls[ci] else {
            unreachable!()
        };
        let p = t.cells[0]
            .paragraphs
            .iter_mut()
            .flat_map(|p| &mut p.controls)
            .find_map(|c| match c {
                Control::Picture(p) if p.common.cell_border_paint.is_some_and(|t| t.sides == 5) => {
                    Some(p)
                }
                _ => None,
            })
            .unwrap();
        match mode {
            "crop" => p.crop.left += 1,
            "geometry" => p.common.width += 1,
            "bad-mask" => p.common.cell_border_paint.as_mut().unwrap().sides = 16,
            "noncanonical-mask" => p.common.cell_border_paint.as_mut().unwrap().sides = 1,
            _ => {
                let id = p.image_attr.bin_data_id;
                let index = d.bin_data_content.iter().position(|c| c.id == id).unwrap();
                let mut bytes = d.bin_data_content[index]
                    .data
                    .load_limited(256 * 1024 * 1024)
                    .unwrap();
                let last = bytes.len() - 1;
                bytes[last] ^= 1;
                d.bin_data_content[index].data = BinDataBytes::Loaded(bytes);
            }
        }
        c.set_document(d);
        let before = serde_json::to_value(&c.document().sections[0].paragraphs[0]).unwrap();
        let fills = c.document().doc_info.border_fills.len();
        assert!(
            c.set_cell_properties_native(
                0,
                0,
                ci,
                0,
                r##"{"borderLeft":{"type":0,"width":0,"color":"#000000"}}"##
            )
            .is_err(),
            "{mode}"
        );
        assert_eq!(
            serde_json::to_value(&c.document().sections[0].paragraphs[0]).unwrap(),
            before
        );
        assert_eq!(c.document().doc_info.border_fills.len(), fills);
    }
}
fn clear(ps: &mut [rhwp::model::paragraph::Paragraph]) {
    for p in ps {
        for c in &mut p.controls {
            match c {
                Control::Picture(p) => {
                    p.common.cell_border_paint = None;
                    p.common.cell_frame_paint = false;
                    p.common.outside_cell_border_paint = None
                }
                Control::Table(t) => {
                    for cell in &mut t.cells {
                        clear(&mut cell.paragraphs)
                    }
                }
                _ => {}
            }
        }
    }
}
#[test]
fn frame_masks_roundtrip_and_malformed_side_sets_fail_closed() {
    let c = load("compound-solid");
    let bytes = border_paint::encode(c.document()).unwrap().unwrap();
    assert_eq!(bytes[7], 6);
    let mut cleared = c.document().clone();
    clear(&mut cleared.sections[0].paragraphs);
    let mut good = cleared.clone();
    border_paint::decode(&mut good, &bytes);
    assert_eq!(border_paint::encode(&good).unwrap().as_ref(), Some(&bytes));
    for flag in [0, 0x20, 0x3f] {
        let mut bad = bytes.clone();
        let mut at = 12;
        while at < bad.len() {
            let outside = bad[at + 28] & 128 != 0;
            bad[at + 28] = (bad[at + 28] & 0xc0) | flag;
            at += if outside { 101 } else { 69 };
        }
        let mut d = cleared.clone();
        border_paint::decode(&mut d, &bad);
        assert!(border_paint::encode(&d).unwrap().is_none());
    }
}
fn without_style_ids(v: &mut Value) {
    match v {
        Value::Object(o) => {
            o.remove("border_fill_id");
            for v in o.values_mut() {
                without_style_ids(v)
            }
        }
        Value::Array(a) => {
            for v in a {
                without_style_ids(v)
            }
        }
        _ => {}
    }
}
#[test]
fn decoration_extensions_and_connected_joins_remain_unchanged_drawings() {
    for key in [
        "compound-connected-axis",
        "compound-interior",
        "compound-extension",
        "compound-joined",
    ] {
        let mut c = load(key);
        let mut before = serde_json::to_value(&c.document().sections[0].paragraphs[0]).unwrap();
        let r = apply(&mut c, "right");
        assert!(r["borderPaintUndoId"].is_null(), "{key}");
        let mut after = serde_json::to_value(&c.document().sections[0].paragraphs[0]).unwrap();
        without_style_ids(&mut before);
        without_style_ids(&mut after);
        assert_eq!(before, after);
    }
}
fn image(id: u32, x: f64, y: f64, w: f64, h: f64) -> RenderNode {
    let mut image = ImageNode::new(9, None);
    image.crop = Some((0, 0, 100, 100));
    RenderNode::new(
        id,
        RenderNodeType::Image(image),
        BoundingBox::new(x, y, w, h),
    )
}
#[test]
fn adjacent_narrow_source_crops_are_not_occluding_duplicates() {
    for (dx, dy, w, h, expected) in [
        (0.3, 10., 0.3, 40., 40.),
        (0., 40. - f64::EPSILON * 64., 10., 40., 40.),
        (0., 10., 10., 40., 10.),
    ] {
        let mut t = PageRenderTree::new(0, 600., 800.);
        let a = t.next_id();
        let b = t.next_id();
        t.root.children.push(image(a, 10., 20., w, h));
        t.root.children.push(image(b, 10. + dx, 20. + dy, w, 20.));
        t.clip_overlapping_same_bin_images();
        assert_eq!(t.root.children[0].bbox.height, expected);
    }
}
fn image_nodes(n: &RenderNode, out: &mut Vec<(usize, BoundingBox, Option<(i32, i32, i32, i32)>)>) {
    if let RenderNodeType::Image(i) = &n.node_type {
        if i.cell_index.is_none() {
            if let Some(ci) = i.control_index {
                out.push((ci, n.bbox, i.crop));
            }
        }
    }
    for c in &n.children {
        image_nodes(c, out);
    }
}
#[test]
fn compound_page_pieces_keep_source_frames_and_selected_regions_in_layout() {
    let c = load("compound-alpha");
    let mut nodes = vec![];
    image_nodes(&c.build_page_render_tree(0).unwrap().root, &mut nodes);
    let ps = &c.document().sections[0].paragraphs[0];
    let mut checked = 0;
    for (ci, p) in ps.controls.iter().enumerate().filter_map(|(i, c)| {
        if let Control::Picture(p) = c {
            Some((i, p))
        } else {
            None
        }
    }) {
        if p.common.outside_cell_border_paint.is_none() {
            continue;
        }
        let (_, bbox, crop) = nodes.iter().find(|(index, _, _)| *index == ci).unwrap();
        assert!(
            (bbox.height - f64::from(p.common.height) / 75.).abs() < 1e-9,
            "a page fragment cannot lose its physically owned stroke to an adjacent crop"
        );
        assert_eq!(
            *crop,
            Some((p.crop.left, p.crop.top, p.crop.right, p.crop.bottom))
        );
        checked += 1;
    }
    assert!(checked > 0);
}

#[test]
fn erased_edges_cannot_survive_as_ownerless_bitmap_padding() {
    for key in ["compound-solid", "compound-dash", "compound-alpha"] {
        let mut c = load(key);
        let source = pictures(root(&c).1)
            .iter()
            .find_map(|(_, p)| p.common.cell_border_paint.map(|_| p.image_attr.bin_data_id))
            .unwrap();
        let nested = child(&c);
        apply(&mut c, "all");
        assert!(
            !pictures(root(&c).1)
                .iter()
                .any(|(ci, p)| *ci == 0 && p.image_attr.bin_data_id == source),
            "empty crops must not restore erased stroke samples in a native consumer"
        );
        assert!(
            pictures(root(&c).1)
                .iter()
                .any(|(ci, p)| *ci != 0 && p.image_attr.bin_data_id == source),
            "unselected source axes remain"
        );
        assert_eq!(child(&c), nested);
        assert!(!c.document().sections[0].paragraphs[0].controls.iter().any(|ctrl|matches!(ctrl,Control::Picture(p) if p.image_attr.bin_data_id==source && p.common.outside_cell_border_paint.is_none())));
    }
}
