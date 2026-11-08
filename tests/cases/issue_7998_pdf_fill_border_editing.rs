#![cfg(feature = "pdf-import")]
use rhwp::{
    model::{
        border_paint::{self, CellSide},
        control::Control,
        shape::ShapeObject,
        table::Table,
    },
    parser::pdf::PdfImportOptions,
    DocumentCore,
};
fn load(key: &str) -> DocumentCore {
    DocumentCore::from_pdf_bytes(
        &std::fs::read(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(format!("pdf/issue7998/fill-borders/{key}.pdf")),
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
const RIGHT: &str = r##"{"borderRight":{"type":1,"width":5,"color":"#d00000"}}"##;
const LEFT: &str = r##"{"borderLeft":{"type":0,"width":0,"color":"#000000"}}"##;
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
fn rects(t: &Table) -> Vec<(usize, &rhwp::model::shape::RectangleShape)> {
    t.cells
        .iter()
        .enumerate()
        .flat_map(|(ci, c)| {
            c.paragraphs.iter().flat_map(move |p| {
                p.controls.iter().filter_map(move |c| match c {
                    Control::Shape(s) => match s.as_ref() {
                        ShapeObject::Rectangle(r) => Some((ci, r)),
                        _ => None,
                    },
                    _ => None,
                })
            })
        })
        .collect()
}
#[test]
fn shared_filled_band_replaces_both_physical_sides_and_preserves_child() {
    let mut c = load("fill-parent");
    let before = c.export_hwpx_native().unwrap();
    let t = root(&c).1;
    let old: Vec<_> = rects(t).iter().map(|(_, r)| r.common.instance_id).collect();
    let selected: Vec<_> = rects(t)
        .into_iter()
        .filter(|(ci, r)| {
            r.common.cell_border_paint.is_some_and(|v| {
                (*ci == 0 && v.side == CellSide::Right) || (*ci == 1 && v.side == CellSide::Left)
            })
        })
        .map(|(_, r)| r.common.instance_id)
        .collect();
    assert_eq!(selected.len(), 2);
    let child = |c: &DocumentCore| {
        root(c)
            .1
            .cells
            .iter()
            .flat_map(|c| &c.paragraphs)
            .flat_map(|p| &p.controls)
            .filter(|c| matches!(c, Control::Table(_)))
            .map(|c| serde_json::to_value(c).unwrap())
            .collect::<Vec<_>>()
    };
    let nested = child(&c);
    let r = apply(&mut c, RIGHT);
    assert!(r["borderPaintUndoId"].is_number());
    assert_eq!(
        rects(root(&c).1)
            .iter()
            .map(|(_, r)| r.common.instance_id)
            .collect::<Vec<_>>(),
        old.into_iter()
            .filter(|id| !selected.contains(id))
            .collect::<Vec<_>>()
    );
    assert_eq!(child(&c), nested);
    undo(&mut c, &r);
    assert_eq!(c.export_hwpx_native().unwrap(), before);
}
#[test]
fn outer_fill_uses_page_closure_and_undo_survives_text_inverse() {
    let mut c = load("fill-parent");
    let before = c.export_hwpx_native().unwrap();
    let outside = |c: &DocumentCore| {
        c.document().sections[0].paragraphs[0]
            .controls
            .iter()
            .filter(
                |c| matches!(c,Control::Shape(s)if s.common().outside_cell_border_paint.is_some()),
            )
            .count()
    };
    let n = outside(&c);
    assert!(n > 0);
    let r = apply(&mut c, LEFT);
    assert!(outside(&c) < n);
    let i = root(&c).0;
    c.insert_text_in_cell_native(0, 0, i, 0, 0, 0, "X").unwrap();
    c.delete_text_in_cell_native(0, 0, i, 0, 0, 0, 1).unwrap();
    undo(&mut c, &r);
    assert_eq!(c.export_hwpx_native().unwrap(), before);
}
#[test]
fn both_saves_restore_filled_band_editing_owner() {
    let c = load("fill-parent");
    let roles = border_paint::encode(c.document()).unwrap().unwrap();
    for bytes in [
        c.export_hwp_with_adapter_snapshot().unwrap(),
        c.export_hwpx_native().unwrap(),
    ] {
        let mut n = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(
            border_paint::encode(n.document()).unwrap(),
            Some(roles.clone())
        );
        let before = n.export_hwpx_native().unwrap();
        let r = apply(&mut n, RIGHT);
        assert!(r["borderPaintUndoId"].is_number());
        undo(&mut n, &r);
        assert_eq!(n.export_hwpx_native().unwrap(), before);
    }
}
fn native_fill(c: &DocumentCore) -> rhwp::model::style::Fill {
    c.document().doc_info.border_fills[usize::from(root(c).1.cells[0].border_fill_id) - 1]
        .fill
        .clone()
}
fn effective_fill(c: &DocumentCore) -> (rhwp::model::style::FillType, Option<u32>) {
    let f = native_fill(c);
    assert_eq!(f.alpha, 0);
    assert!(f.gradient.is_none() && f.image.is_none());
    (
        f.fill_type,
        f.solid.map(|s| {
            assert!(s.pattern_type == 0 || s.pattern_type == -1);
            assert_eq!(s.pattern_color, 0);
            s.background_color
        }),
    )
}
fn no_background_drawings(t: &Table) {
    for (_, r) in rects(t) {
        assert!(
            r.common.width <= 100 || r.common.height <= 100,
            "cell background must not be a drawing"
        );
    }
    for c in &t.cells {
        for p in &c.paragraphs {
            for ctrl in &p.controls {
                if let Control::Table(child) = ctrl {
                    no_background_drawings(child);
                }
            }
        }
    }
}
#[test]
fn background_is_native_cell_fill_and_border_edit_preserves_it() {
    let mut c = load("fill-background");
    let before = native_fill(&c);
    assert_eq!(before.fill_type, rhwp::model::style::FillType::Solid);
    assert_eq!(before.solid.unwrap().background_color, 0xfff2e6);
    no_background_drawings(root(&c).1);
    let children = root(&c).1.cells[0]
        .paragraphs
        .iter()
        .flat_map(|p| &p.controls)
        .filter(|c| matches!(c, Control::Table(_)))
        .count();
    assert_eq!(children, 1);
    apply(&mut c, RIGHT);
    apply(&mut c, LEFT);
    assert_eq!(native_fill(&c), before);
    no_background_drawings(root(&c).1);
}
#[test]
fn cell_fill_color_clear_and_both_saves_keep_editable_paragraphs() {
    for props in [
        r##"{"fillType":"solid","fillColor":"#20b050"}"##,
        r##"{"fillType":"none"}"##,
    ] {
        let mut c = load("fill-background");
        let original_fill = native_fill(&c);
        let other = serde_json::to_value(&root(&c).1.cells[1..]).unwrap();
        let result = apply(&mut c, props);
        assert_eq!(serde_json::to_value(&root(&c).1.cells[1..]).unwrap(), other);
        assert_ne!(native_fill(&c), original_fill);
        for bytes in [
            c.export_hwp_with_adapter_snapshot().unwrap(),
            c.export_hwpx_native().unwrap(),
        ] {
            let n = DocumentCore::from_bytes(&bytes).unwrap();
            assert_eq!(effective_fill(&n), effective_fill(&c));
            no_background_drawings(root(&n).1);
            assert!(root(&n).1.cells[0]
                .paragraphs
                .iter()
                .any(|p| p.text.contains("BEFORE CHILD")));
        }
        let ci = root(&c).0;
        c.insert_text_in_cell_native(0, 0, ci, 0, 0, 0, "X")
            .unwrap();
        c.delete_text_in_cell_native(0, 0, ci, 0, 0, 0, 1).unwrap();
        undo(&mut c, &result);
        // The style command restores style and edited content. Text inverse may
        // invalidate stored LineSeg caches independently of the fill command.
        assert_eq!(native_fill(&c), original_fill);
        assert!(root(&c).1.cells[0]
            .paragraphs
            .iter()
            .any(|p| p.text.contains("BEFORE CHILD")));
        assert_eq!(serde_json::to_value(&root(&c).1.cells[1..]).unwrap(), other);
        let mut unchanged = load("fill-background");
        let saved = unchanged.export_hwpx_native().unwrap();
        let r = apply(&mut unchanged, props);
        undo(&mut unchanged, &r);
        assert_eq!(unchanged.export_hwpx_native().unwrap(), saved);
    }
}
#[test]
fn overlapping_fill_is_not_certified_by_color_or_full_edge_coverage() {
    for key in ["fill-occluder", "fill-dark-occluder"] {
        let mut c = load(key);
        let fill = |c: &DocumentCore| {
            let mut values = rects(root(c).1)
                .into_iter()
                .filter(|(_, r)| r.common.width <= 100 && r.common.height > 100)
                .map(|(_, r)| {
                    assert!(r.common.cell_border_paint.is_none());
                    serde_json::to_value(r).unwrap()
                })
                .collect::<Vec<_>>();
            for ctrl in &c.document().sections[0].paragraphs[0].controls {
                if let Control::Shape(s) = ctrl {
                    if let ShapeObject::Rectangle(r) = s.as_ref() {
                        assert!(r.common.outside_cell_border_paint.is_none());
                        values.push(serde_json::to_value(r).unwrap())
                    }
                }
            }
            values
        };
        let before = fill(&c);
        assert!(!before.is_empty());
        apply(&mut c, LEFT);
        assert_eq!(fill(&c), before, "{key}");
    }
}

#[test]
fn partial_foreground_and_competing_rectangles_keep_source_paint() {
    fn visit(
        c: &DocumentCore,
        ps: &[rhwp::model::paragraph::Paragraph],
        fills: &mut usize,
        paint: &mut usize,
    ) {
        for p in ps {
            for ctl in &p.controls {
                match ctl {
                    Control::Table(t) => {
                        for cell in &t.cells {
                            *fills += usize::from(
                                c.document().doc_info.border_fills
                                    [usize::from(cell.border_fill_id) - 1]
                                    .fill
                                    .fill_type
                                    != rhwp::model::style::FillType::None,
                            );
                            visit(c, &cell.paragraphs, fills, paint);
                        }
                    }
                    Control::Shape(s) => {
                        if let ShapeObject::Rectangle(r) = s.as_ref() {
                            *paint += usize::from(
                                r.drawing.text_box.is_none()
                                    && r.common.width > 100
                                    && r.common.height > 100,
                            );
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    for key in ["fill-highlight", "fill-foreground", "fill-competing"] {
        let c = load(key);
        let (mut fills, mut paint) = (0, 0);
        visit(
            &c,
            &c.document().sections[0].paragraphs,
            &mut fills,
            &mut paint,
        );
        assert_eq!(
            fills, 0,
            "{key}: ambiguous rectangle must not claim fill editing"
        );
        assert!(paint > 0, "{key}: source paint remains");
    }
}
