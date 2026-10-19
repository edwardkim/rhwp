#![cfg(feature = "pdf-import")]
use rhwp::{
    model::{
        border_paint::{self, CellSide},
        control::Control,
        image::Picture,
        table::Table,
    },
    parser::pdf::PdfImportOptions,
    DocumentCore,
};
use std::{fs, path::PathBuf};
fn load() -> DocumentCore {
    DocumentCore::from_pdf_bytes(
        &fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("pdf/issue7998/nested-dash-parent.pdf"),
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
fn pictures(t: &Table) -> Vec<&Picture> {
    t.cells
        .iter()
        .flat_map(|c| &c.paragraphs)
        .flat_map(|p| &p.controls)
        .filter_map(|c| {
            if let Control::Picture(p) = c {
                Some(p.as_ref())
            } else {
                None
            }
        })
        .collect()
}
const RIGHT: &str = r##"{"borderRight":{"type":1,"width":5,"color":"#d00000"}}"##;
fn apply(c: &mut DocumentCore, s: &str) -> serde_json::Value {
    let i = root(c).0;
    serde_json::from_str(&c.set_cell_properties_native(0, 0, i, 0, s).unwrap()).unwrap()
}
fn undo(c: &mut DocumentCore, r: &serde_json::Value) {
    let i = root(c).0;
    let cells: Vec<_> = r["changes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| serde_json::json!({"cellIdx":x["cellIdx"],"id":x["beforeId"]}))
        .collect();
    c.apply_cell_border_fill_ids_native(0,0,i,&serde_json::json!({"cells":cells,"zones":[],"borderPaintUndoIds":[r["borderPaintUndoId"]]}).to_string()).unwrap();
    c.remove_border_fill_tails_native(
        &serde_json::json!({"fromLen":r["borderFillLenBefore"],"dirtyWas":r["docInfoDirtyBefore"]})
            .to_string(),
    )
    .unwrap();
}
#[test]
fn replaces_both_shared_raster_halves_and_preserves_nested_content() {
    let mut c = load();
    let before = c.export_hwpx_native().unwrap();
    let t = root(&c).1;
    let old: Vec<_> = pictures(t).iter().map(|p| p.common.instance_id).collect();
    let selected: Vec<_> = t
        .cells
        .iter()
        .enumerate()
        .flat_map(|(ci, cell)| {
            cell.paragraphs.iter().flat_map(move |p| {
                p.controls.iter().filter_map(move |c| match c {
                    Control::Picture(p)
                        if p.common.cell_border_paint.is_some_and(|v| {
                            (ci == 0 && v.side == CellSide::Right)
                                || (ci == 1 && v.side == CellSide::Left)
                        }) =>
                    {
                        Some(p.common.instance_id)
                    }
                    _ => None,
                })
            })
        })
        .collect();
    assert_eq!(selected.len(), 2);
    let nested: Vec<_> = t
        .cells
        .iter()
        .flat_map(|c| &c.paragraphs)
        .flat_map(|p| &p.controls)
        .filter(|c| matches!(c, Control::Table(_)))
        .map(|c| serde_json::to_value(c).unwrap())
        .collect();
    let r = apply(&mut c, RIGHT);
    assert!(r["borderPaintUndoId"].is_number());
    let want: Vec<_> = old.into_iter().filter(|i| !selected.contains(i)).collect();
    assert_eq!(
        pictures(root(&c).1)
            .iter()
            .map(|p| p.common.instance_id)
            .collect::<Vec<_>>(),
        want
    );
    let after: Vec<_> = root(&c)
        .1
        .cells
        .iter()
        .flat_map(|c| &c.paragraphs)
        .flat_map(|p| &p.controls)
        .filter(|c| matches!(c, Control::Table(_)))
        .map(|c| serde_json::to_value(c).unwrap())
        .collect();
    assert_eq!(nested, after);
    undo(&mut c, &r);
    assert_eq!(c.export_hwpx_native().unwrap(), before);
}
#[test]
fn both_native_saves_reopen_and_edit_with_same_resource_seal() {
    let c = load();
    let expected = border_paint::encode(c.document()).unwrap().unwrap();
    assert_eq!(&expected[..8], b"RBPO\0\0\0\x06");
    for bytes in [
        c.export_hwp_with_adapter_snapshot().unwrap(),
        c.export_hwpx_native().unwrap(),
    ] {
        let mut n = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(
            border_paint::encode(n.document()).unwrap(),
            Some(expected.clone())
        );
        let before = n.export_hwpx_native().unwrap();
        let r = apply(&mut n, RIGHT);
        undo(&mut n, &r);
        assert_eq!(n.export_hwpx_native().unwrap(), before);
    }
}
#[test]
fn changed_crop_geometry_or_resource_rejects_atomically() {
    for mode in ["crop", "geometry", "resource", "reference", "transform"] {
        let mut c = load();
        let mut d = c.document().clone();
        let i = root(&c).0;
        let Control::Table(t) = &mut d.sections[0].paragraphs[0].controls[i] else {
            unreachable!()
        };
        let p = t.cells[0]
            .paragraphs
            .iter_mut()
            .flat_map(|p| &mut p.controls)
            .find_map(|c| match c {
                Control::Picture(p)
                    if p.common
                        .cell_border_paint
                        .is_some_and(|v| v.side == CellSide::Right) =>
                {
                    Some(p)
                }
                _ => None,
            })
            .unwrap();
        match mode {
            "crop" => p.crop.left += 1,
            "geometry" => p.common.width += 1,
            "reference" => p.image_attr.bin_data_id = 0,
            "transform" => p.shape_attr.render_tx = 1.,
            _ => {
                let id = p.image_attr.bin_data_id;
                let idx = usize::from(id - 1);
                let mut bytes = d.bin_data_content[idx]
                    .data
                    .load_limited(256 * 1024 * 1024)
                    .unwrap();
                let last = bytes.len() - 1;
                bytes[last] ^= 1;
                d.bin_data_content[idx].data = rhwp::model::bin_data::BinDataBytes::Loaded(bytes);
            }
        }
        c.set_document(d);
        let before = c.export_hwpx_native().unwrap();
        let i = root(&c).0;
        assert!(
            c.set_cell_properties_native(0, 0, i, 0, RIGHT).is_err(),
            "{mode}"
        );
        assert_eq!(c.export_hwpx_native().unwrap(), before);
    }
}
#[test]
fn outer_raster_replacement_and_text_inverse_restore_all_source_pieces() {
    let mut c = load();
    let before = c.export_hwpx_native().unwrap();
    let count = |c: &DocumentCore| {
        c.document().sections[0].paragraphs[0]
            .controls
            .iter()
            .filter(
                |c| matches!(c,Control::Picture(p)if p.common.outside_cell_border_paint.is_some()),
            )
            .count()
    };
    let old = count(&c);
    assert!(old > 0);
    let r = apply(
        &mut c,
        r##"{"borderLeft":{"type":0,"width":0,"color":"#000000"}}"##,
    );
    assert!(count(&c) < old);
    let i = root(&c).0;
    c.insert_text_in_cell_native(0, 0, i, 0, 0, 0, "X").unwrap();
    c.delete_text_in_cell_native(0, 0, i, 0, 0, 0, 1).unwrap();
    undo(&mut c, &r);
    assert_eq!(c.export_hwpx_native().unwrap(), before);
}
#[test]
fn untagged_content_picture_cannot_be_deleted_by_bbox_overlap() {
    let mut c = load();
    let mut d = c.document().clone();
    let i = root(&c).0;
    let Control::Table(t) = &mut d.sections[0].paragraphs[0].controls[i] else {
        unreachable!()
    };
    let mut copied = pictures(t)
        .into_iter()
        .find(|p| {
            p.common
                .cell_border_paint
                .is_some_and(|v| v.side == CellSide::Right)
        })
        .unwrap()
        .clone();
    copied.common.cell_border_paint = None;
    copied.border_paint_fingerprint = None;
    copied.common.instance_id = 1234567;
    copied.instance_id = 1234567;
    let keep = serde_json::to_value(&copied).unwrap();
    t.cells[0].paragraphs[0]
        .controls
        .push(Control::Picture(Box::new(copied)));
    c.set_document(d);
    apply(&mut c, RIGHT);
    assert!(pictures(root(&c).1)
        .iter()
        .any(|p| serde_json::to_value(p).unwrap() == keep));
}
#[test]
fn metadata_picture_kind_and_version_mismatch_grant_no_authority() {
    let c = load();
    let bytes = border_paint::encode(c.document()).unwrap().unwrap();
    for mode in ["kind", "version", "source", "truncated"] {
        let mut d = c.document().clone();
        fn clear(ps: &mut [rhwp::model::paragraph::Paragraph]) {
            for p in ps {
                for c in &mut p.controls {
                    match c {
                        Control::Table(t) => {
                            for c in &mut t.cells {
                                clear(&mut c.paragraphs)
                            }
                        }
                        Control::Picture(p) => {
                            p.common.cell_border_paint = None;
                            p.common.cell_frame_paint = false;
                            p.common.outside_cell_border_paint = None;
                            p.border_paint_fingerprint = None;
                        }
                        _ => {}
                    }
                }
            }
        }
        clear(&mut d.sections[0].paragraphs);
        let mut bad = bytes.clone();
        match mode {
            "kind" => bad[40] ^= 0x40,
            "version" => bad[7] = 0xff,
            "source" => bad[41..45].fill(0),
            _ => {
                bad.pop();
            }
        }
        border_paint::decode(&mut d, &bad);
        if mode != "kind" {
            assert!(border_paint::encode(&d).unwrap().is_none(), "{mode}");
        } else {
            let data = border_paint::encode(&d).unwrap();
            assert_ne!(data, Some(bytes.clone()));
        }
    }
}

#[test]
fn separate_source_image_xobject_and_curved_fill_keep_independent_role() {
    for key in [
        "nested-dash-image",
        "compound-backdrop",
        "compound-foreground",
    ] {
        let c = DocumentCore::from_pdf_bytes(
            &fs::read(
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("pdf/issue7998/{key}.pdf")),
            )
            .unwrap(),
            &PdfImportOptions::default(),
        )
        .unwrap();
        fn walk(
            ps: &[rhwp::model::paragraph::Paragraph],
            independent: &mut usize,
            curve: &mut usize,
        ) {
            for p in ps {
                for c in &p.controls {
                    match c {
                        Control::Table(t) => {
                            for c in &t.cells {
                                walk(&c.paragraphs, independent, curve)
                            }
                        }
                        Control::Picture(p) => {
                            if p.img_dim == (150, 150) {
                                *independent += 1;
                                assert!(p.common.cell_border_paint.is_none());
                                assert!(p.border_paint_fingerprint.is_none());
                            }
                            if p.common.cell_border_paint.is_none() {
                                *curve += 1;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        let (mut image, mut area) = (0, 0);
        walk(&c.document().sections[0].paragraphs, &mut image, &mut area);
        if key == "nested-dash-image" {
            assert!(image > 0);
        } else {
            assert!(area > 0);
        }
    }
}
#[test]
fn source_phase_and_alpha_use_common_paint_without_losing_picture_semantics() {
    use rhwp::{
        paint::{LayerBuilder, LayerNode, LayerNodeKind, PaintOp, RenderProfile},
        renderer::render_tree::{RenderNode, RenderNodeType},
    };
    let bytes = fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("pdf/issue7998/raster-strokes/raster-phase-alpha.pdf"),
    )
    .unwrap();
    let c = DocumentCore::from_pdf_bytes(&bytes, &PdfImportOptions::default()).unwrap();
    fn images(n: &RenderNode) -> usize {
        usize::from(matches!(n.node_type, RenderNodeType::Image(_)))
            + n.children.iter().map(images).sum::<usize>()
    }
    fn paths(n: &LayerNode, out: &mut Vec<rhwp::renderer::render_tree::PathNode>) {
        match &n.kind {
            LayerNodeKind::Group { children, .. } => {
                for c in children {
                    paths(c, out)
                }
            }
            LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
                paths(child, out)
            }
            LayerNodeKind::Leaf { ops } => {
                for op in ops {
                    if let PaintOp::Path { path, .. } = op {
                        if (path.style.opacity - 0.4).abs() < 1e-6 {
                            out.push(path.as_ref().clone());
                        }
                    }
                }
            }
        }
    }
    for bytes in [
        None,
        Some(c.export_hwp_with_adapter_snapshot().unwrap()),
        Some(c.export_hwpx_native().unwrap()),
    ] {
        let n = bytes
            .as_deref()
            .map(DocumentCore::from_bytes)
            .transpose()
            .unwrap();
        let n = n.as_ref().unwrap_or(&c);
        let tree = n.build_page_render_tree(0).unwrap();
        assert!(
            images(&tree.root) > 0,
            "semantic ImageNode must remain editable"
        );
        let layer = LayerBuilder::new(RenderProfile::Print).build(&tree);
        let mut p = vec![];
        paths(&layer.root, &mut p);
        assert!(
            !p.is_empty(),
            "source alpha must reach common paint after native reopen"
        );
        let mut phase = false;
        for path in p {
            let mut widths = vec![];
            for q in path.commands.chunks_exact(5) {
                if let [rhwp::renderer::PathCommand::MoveTo(x, _), rhwp::renderer::PathCommand::LineTo(a, _), ..] =
                    q
                {
                    let w = (a - x).abs();
                    if w > 1e-6 {
                        widths.push(w);
                    }
                }
            }
            if widths.len() > 3
                && (widths[1] / widths[0] - 4.).abs() < 0.03
                && (widths[2] / widths[1] - 1.).abs() < 0.01
            {
                phase = true;
            }
        }
        assert!(
            phase,
            "PDF [2 2] phase 1.5 starts with 0.5pt, followed by full 2pt ink"
        );
    }
}
