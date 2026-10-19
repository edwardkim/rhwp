//! Common ownership/metadata only; no new rendering golden or pixel position pin.
#![cfg(feature = "pdf-import")]
use rhwp::{
    model::{
        border_paint::{self, CellBorderPaint, CellSide},
        control::Control,
        paragraph::Paragraph,
        shape::ShapeObject,
    },
    parser::pdf::PdfImportOptions,
    DocumentCore,
};
use std::{collections::BTreeMap, fs, path::PathBuf};
fn source(key: &str) -> DocumentCore {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("pdf/issue7998/periodic-borders/{key}.pdf"));
    DocumentCore::from_pdf_bytes(&fs::read(p).unwrap(), &PdfImportOptions::default()).unwrap()
}
fn walk(ps: &[Paragraph], f: &mut impl FnMut(&Control)) {
    for p in ps {
        for c in &p.controls {
            f(c);
            if let Control::Table(t) = c {
                for cell in &t.cells {
                    walk(&cell.paragraphs, f);
                }
            }
        }
    }
}
fn bindings(c: &DocumentCore) -> BTreeMap<u32, CellBorderPaint> {
    bindings_doc(c.document())
}
fn bindings_doc(d: &rhwp::model::document::Document) -> BTreeMap<u32, CellBorderPaint> {
    let mut out = BTreeMap::new();
    for s in &d.sections {
        walk(&s.paragraphs, &mut |c| {
            if let Control::Shape(s) = c {
                if let Some(paint) = s.common().cell_border_paint {
                    assert!(matches!(s.as_ref(), ShapeObject::Rectangle(_)));
                    assert!(out.insert(s.common().instance_id, paint).is_none());
                }
            } else if let Control::Picture(p) = c {
                if let Some(paint) = p.common.cell_border_paint {
                    assert!(
                        p.border_paint_fingerprint.is_some(),
                        "a raster side needs its source seal"
                    );
                    assert!(out.insert(p.common.instance_id, paint).is_none());
                }
            }
        });
    }
    out
}
#[test]
fn tens_of_thousands_of_bindings_fit_the_unchanged_byte_budget() {
    fn remap(ps: &mut [Paragraph], offset: u32) {
        for p in ps {
            for c in &mut p.controls {
                match c {
                    Control::Table(t) => {
                        t.common.instance_id += offset;
                        for cell in &mut t.cells {
                            remap(&mut cell.paragraphs, offset);
                        }
                    }
                    Control::Shape(s) => {
                        s.common_mut().instance_id += offset;
                        if let Some(d) = s.drawing_mut() {
                            d.inst_id += offset;
                            if let Some(tb) = &mut d.text_box {
                                remap(&mut tb.paragraphs, offset);
                            }
                        }
                    }
                    Control::Picture(p) => {
                        p.common.instance_id += offset;
                        p.instance_id += offset;
                    }
                    _ => {}
                }
            }
        }
    }
    let c = source("phase-shifted-dot");
    let template = c.document().sections[0].clone();
    let mut d = c.document().clone();
    d.sections.clear();
    for i in 0..160 {
        let mut s = template.clone();
        remap(&mut s.paragraphs, i * 100_000);
        d.sections.push(s);
    }
    let expected = bindings_doc(&d);
    assert!(expected.len() > 40_000);
    let bytes = border_paint::encode(&d).unwrap().unwrap();
    assert!(bytes.len() <= border_paint::MAX_BYTES);
    for s in &mut d.sections {
        clear(&mut s.paragraphs, false);
    }
    border_paint::decode(&mut d, &bytes);
    assert_eq!(
        bindings_doc(&d),
        expected,
        "budget compaction must not drop owners"
    );
}
fn clear(ps: &mut [Paragraph], moved: bool) {
    for p in ps {
        for c in &mut p.controls {
            match c {
                Control::Shape(s) => {
                    let a = s.common_mut();
                    if moved && a.cell_border_paint.is_some() {
                        a.horizontal_offset = a.horizontal_offset.wrapping_add(1);
                    }
                    a.cell_border_paint = None;
                    a.cell_frame_paint = false;
                }
                Control::Table(t) => {
                    for c in &mut t.cells {
                        clear(&mut c.paragraphs, moved);
                    }
                }
                _ => {}
            }
        }
    }
}
#[test]
fn retained_separator_pieces_keep_their_owner_in_both_real_writers() {
    for key in [
        "phase-shifted-dot",
        "wrong-width-dot",
        "occluded-inner-dot",
        "thin-fill-occluder",
    ] {
        let c = source(key);
        let expected = bindings(&c);
        assert!(
            !expected.is_empty(),
            "positive source separators must reach actual IR: {key}"
        );
        let mut origins = BTreeMap::<u32, Vec<CellSide>>::new();
        for b in expected.values() {
            origins.entry(b.source_id).or_default().push(b.side);
        }
        assert!(
            origins.values().any(
                |s| (s.contains(&CellSide::Left) && s.contains(&CellSide::Right))
                    || (s.contains(&CellSide::Top) && s.contains(&CellSide::Bottom))
            ),
            "shared source pieces retain both distinct physical owners"
        );
        if matches!(key, "occluded-inner-dot" | "thin-fill-occluder") {
            let mut covers = 0;
            walk(&c.document().sections[0].paragraphs, &mut |ctrl| {
                if let Control::Shape(s) = ctrl {
                    if let ShapeObject::Rectangle(r) = s.as_ref() {
                        if r.drawing
                            .fill
                            .solid
                            .as_ref()
                            .is_some_and(|f| f.background_color == 0xffffff)
                        {
                            covers += 1;
                            assert!(
                                r.common.cell_border_paint.is_none(),
                                "the authored white occluder is fill, not a source stroke"
                            );
                        }
                    }
                }
            });
            assert!(covers > 0);
        }
        let bytes = border_paint::encode(c.document()).unwrap().unwrap();
        for saved in [
            c.export_hwp_with_adapter_snapshot_with_report().unwrap(),
            c.export_hwpx_native_with_report().unwrap(),
        ] {
            let n = DocumentCore::from_bytes(saved.bytes()).unwrap();
            assert_eq!(
                bindings(&n),
                expected,
                "IDs, side and common source survive {key}"
            );
            assert_eq!(border_paint::encode(n.document()).unwrap().unwrap(), bytes);
            assert!(saved
                .content_loss()
                .losses()
                .iter()
                .any(|l| l.path.contains("셀 테두리 그림 소유")));
        }
    }
}
#[test]
fn content_images_and_authored_background_fills_keep_independent_owners() {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("pdf/issue7998");
    let mut images = 0;
    let mut backgrounds = 0;
    let native_fills = |c: &DocumentCore| {
        let mut values = Vec::new();
        walk(&c.document().sections[0].paragraphs, &mut |ctrl| {
            if let Control::Table(t) = ctrl {
                for cell in &t.cells {
                    let fill = &c.document().doc_info.border_fills
                        [usize::from(cell.border_fill_id) - 1]
                        .fill;
                    if let Some(s) = &fill.solid {
                        if !matches!(s.background_color, 0 | 0xffffff) {
                            values.push((
                                t.common.instance_id,
                                cell.row,
                                cell.col,
                                s.background_color,
                            ));
                        }
                    }
                }
            }
        });
        values
    };
    for key in ["nested-dash-image", "nested-periodic-background"] {
        let c = DocumentCore::from_pdf_bytes(
            &fs::read(base.join(format!("{key}.pdf"))).unwrap(),
            &PdfImportOptions::default(),
        )
        .unwrap();
        // The independent generator authors black strokes, a colored area fill
        // and a separate image XObject. Color is not a production classifier.
        walk(
            &c.document().sections[0].paragraphs,
            &mut |ctrl| match ctrl {
                Control::Picture(p) => {
                    // The independent generator's image XObject is 2x2 pixels.
                    // Rasterized source strokes are a different provenance.
                    if p.img_dim == (150, 150) {
                        images += 1;
                        assert!(p.common.cell_border_paint.is_none());
                        assert!(p.border_paint_fingerprint.is_none());
                    }
                }
                Control::Shape(s) => {
                    if let ShapeObject::Rectangle(r) = s.as_ref() {
                        if r.drawing
                            .fill
                            .solid
                            .as_ref()
                            .is_some_and(|f| !matches!(f.background_color, 0 | 0xffffff))
                        {
                            backgrounds += 1;
                            assert!(r.common.cell_border_paint.is_none());
                        }
                    }
                }
                _ => {}
            },
        );
        let fills = native_fills(&c);
        if key == "nested-periodic-background" {
            assert_eq!(fills.len(), 1, "one authored area becomes one cell fill");
            assert_eq!(backgrounds, 0, "no background drawing fragments remain");
        }
        let expected = bindings(&c);
        for bytes in [
            c.export_hwp_with_adapter_snapshot().unwrap(),
            c.export_hwpx_native().unwrap(),
        ] {
            let saved = DocumentCore::from_bytes(&bytes).unwrap();
            assert_eq!(bindings(&saved), expected);
            assert_eq!(
                native_fills(&saved),
                fills,
                "real native cell fill survives both formats"
            );
        }
        backgrounds += fills.len();
    }
    assert!(
        images > 0 && backgrounds > 0,
        "exercise real independent non-border content"
    );
}
#[test]
fn native_borders_and_unconfirmed_outer_tables_do_not_receive_drawing_ownership() {
    for key in ["compatible-inner-dot", "underreach-dot"] {
        let c = source(key);
        assert!(bindings(&c).is_empty());
        assert!(border_paint::encode(c.document()).unwrap().is_none());
    }
}
#[test]
fn stale_geometry_owner_and_ambiguous_ids_cannot_attach_border_metadata() {
    let c = source("phase-shifted-dot");
    let bytes = border_paint::encode(c.document()).unwrap().unwrap();
    assert_eq!(bytes[7], 6);
    assert_eq!((bytes.len() - 12) % 69, 0);
    for mode in [
        "fingerprint",
        "owner",
        "duplicate",
        "version",
        "geometry",
        "truncated",
        "count",
        "side",
        "budget",
    ] {
        let mut data = bytes.clone();
        match mode {
            "fingerprint" => {
                for r in data[12..].chunks_exact_mut(69) {
                    r[33] ^= 1;
                }
            }
            "owner" => {
                for r in data[12..].chunks_exact_mut(69) {
                    r[4..8].fill(0);
                }
            }
            "duplicate" => {
                let record = data[12..81].to_vec();
                let count = u32::from_le_bytes(data[8..12].try_into().unwrap());
                data[8..12].copy_from_slice(&(count + 1).to_le_bytes());
                data.extend(record);
            }
            "version" => data[7] = 99,
            "truncated" => {
                data.pop();
            }
            "count" => data[8..12].copy_from_slice(&u32::MAX.to_le_bytes()),
            "side" => {
                for r in data[12..].chunks_exact_mut(69) {
                    r[28] = 255;
                }
            }
            "budget" => data = vec![0; border_paint::MAX_BYTES + 1],
            _ => {}
        }
        let mut d = c.document().clone();
        for s in &mut d.sections {
            clear(&mut s.paragraphs, mode == "geometry");
        }
        border_paint::decode(&mut d, &data);
        let mut n = source("phase-shifted-dot");
        n.set_document(d);
        assert!(
            bindings(&n).is_empty(),
            "{mode} must fail closed without repainting or moving objects"
        );
        assert!(
            border_paint::encode(n.document()).unwrap().is_none(),
            "{mode}: no frame authority restored"
        );
    }
    let mut d = c.document().clone();
    let duplicate = d.sections[0].paragraphs[0]
        .controls
        .iter()
        .find_map(|c| {
            if let Control::Table(t) = c {
                t.cells
                    .iter()
                    .flat_map(|c| &c.paragraphs)
                    .flat_map(|p| &p.controls)
                    .find(
                        |c| matches!(c,Control::Shape(s)if s.common().cell_border_paint.is_some()),
                    )
                    .cloned()
            } else {
                None
            }
        })
        .unwrap();
    d.sections[0].paragraphs[0].controls.push(duplicate);
    assert!(border_paint::encode(&d).is_err());
}
