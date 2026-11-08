//! Public v2 exports must reject unanchored or unreplayable strict text variants.

use rhwp::paint::*;
use rhwp::renderer::layer_renderer::{
    analyze_text_variant_selection, TextVariantSelectionOptions, VariantRejectReason,
    VariantSelectionBackend,
};
use rhwp::renderer::render_tree::{BoundingBox, TextRunNode};
use rhwp::renderer::{PathCommand, TextStyle};

const FONT: &[u8] = include_bytes!("../fixtures/fonts/RHWPBitmapSvgGlyphSmoke.ttf");

fn native_tree() -> PageLayerTree {
    let bbox = BoundingBox::new(10.0, 20.0, 16.0, 16.0);
    let run = TextRunNode {
        text: "\u{E100}".to_string(),
        style: TextStyle {
            font_family: "fixture".to_string(),
            font_size: 16.0,
            ..Default::default()
        },
        char_shape_id: Some(7),
        para_shape_id: None,
        section_index: None,
        para_index: None,
        char_start: None,
        cell_context: None,
        is_para_end: false,
        is_line_break_end: false,
        rotation: 0.0,
        is_vertical: false,
        char_overlap: None,
        border_fill_id: 0,
        baseline: 12.0,
        field_marker: Default::default(),
        layout_positions: None,
        display_text: None,
    };
    let mut tree = PageLayerTree::new(
        100.0,
        100.0,
        LayerNode::leaf(bbox, None, vec![PaintOp::text_run(bbox, run)]),
    );
    lower_font_native_glyph_sidecars(
        &mut tree.root,
        &mut tree.resources,
        &[EmbeddedFontFace {
            char_shape_id: 7,
            language_index: 0,
            family: "fixture",
            alternate_family: None,
            bytes: FONT,
            face_index: 0,
        }],
    );
    tree
}

fn glyph_run_mut(tree: &mut PageLayerTree) -> &mut LayerGlyphRunPaint {
    let LayerNodeKind::Leaf { ops } = &mut tree.root.kind else {
        panic!("fixture root is a leaf");
    };
    ops.iter_mut()
        .find_map(|op| match op {
            PaintOp::GlyphRun { run, .. } => Some(run.as_mut()),
            _ => None,
        })
        .expect("real font produces an exact GlyphRun sidecar")
}

fn outline_mut(tree: &mut PageLayerTree) -> &mut LayerGlyphOutlinePaint {
    let LayerNodeKind::Leaf { ops } = &mut tree.root.kind else {
        panic!("fixture root is a leaf");
    };
    ops.iter_mut()
        .find_map(|op| match op {
            PaintOp::GlyphOutline { outline, .. } => Some(outline.as_mut()),
            _ => None,
        })
        .expect("real font produces a resource-backed outline sidecar")
}

fn monochrome_tree() -> PageLayerTree {
    let mut tree = native_tree();
    let outline = outline_mut(&mut tree);
    // Keep the real font's source/variant binding, replacing only its bitmap
    // visual with a bounded public path payload to isolate outline validation.
    outline.payload_kind = GlyphOutlinePayloadKind::MonochromeFill;
    outline.bitmap_glyph = None;
    outline.paths = vec![LayerGlyphOutlinePath {
        glyph_id: 1,
        source_range_utf8: outline.source.utf8_range,
        glyph_range: GlyphRange::new(0, 1),
        commands: vec![
            PathCommand::MoveTo(0.0, 0.0),
            PathCommand::LineTo(8.0, 0.0),
            PathCommand::LineTo(8.0, 8.0),
            PathCommand::ClosePath,
        ],
        fill_rule: GlyphOutlineFillRule::NonZero,
    }];
    tree
}

#[test]
fn compat_export_rejects_an_unconsumed_sidecar_group_for_the_same_source() {
    let mut tree = native_tree();
    let before: serde_json::Value =
        serde_json::from_str(&tree.to_json_v2_compat().unwrap()).unwrap();
    assert_eq!(
        before["root"]["ops"][0]["variants"]
            .as_array()
            .unwrap()
            .len(),
        3
    );

    let outline = outline_mut(&mut tree);
    outline.variant.equivalence_group = "second-group".to_string();
    outline.variant.anchor_op_id = Some("second-group".to_string());

    assert!(
        tree.to_json_v2_compat().is_err(),
        "a sidecar without an exported fallback slot must not silently disappear"
    );
}

#[test]
fn strict_glyph_run_export_checks_the_exact_font_glyph_count() {
    let face = ttf_parser::Face::parse(FONT, 0).unwrap();
    let glyph_count = u32::from(face.number_of_glyphs());
    assert!(glyph_count > 1 && glyph_count < u32::from(u16::MAX));

    let mut valid = native_tree();
    assert!(valid.to_json_v2_strict_glyph_run().is_ok());
    glyph_run_mut(&mut valid).glyph_ids[0] = glyph_count - 1;
    assert!(
        valid.to_json_v2_strict_glyph_run().is_ok(),
        "the last valid glyph id must remain inside the exact face's range"
    );

    for invalid_id in [glyph_count, u32::from(u16::MAX)] {
        let mut invalid = native_tree();
        glyph_run_mut(&mut invalid).glyph_ids[0] = invalid_id;
        assert!(
            invalid.to_json_v2_strict_glyph_run().is_err(),
            "glyph {invalid_id} does not exist in a face with {glyph_count} glyphs"
        );
    }
}

#[test]
fn strict_monochrome_export_rejects_an_empty_path_inside_a_nonempty_vector() {
    let mut tree = monochrome_tree();
    assert!(tree.to_json_v2_strict_glyph_outline().is_ok());
    let outline = outline_mut(&mut tree);
    assert_eq!(outline.paths.len(), 1);
    assert!(!outline.paths[0].commands.is_empty());
    outline.paths[0].commands.clear();

    assert!(
        tree.to_json_v2_strict_glyph_outline().is_err(),
        "a nonempty paths vector does not prove that its path commands are replayable"
    );
}

#[test]
fn strict_export_rejects_finite_coordinates_outside_the_portable_scalar_range() {
    let valid = native_tree();
    assert!(valid.to_json_v2_strict_glyph_run().is_ok());

    for target in ["position", "placement"] {
        let mut invalid = native_tree();
        let run = glyph_run_mut(&mut invalid);
        if target == "position" {
            run.positions[0].x = f64::MAX;
        } else {
            run.placement.run_to_page.e = f64::MAX;
        }
        assert!(
            invalid.to_json_v2_strict_glyph_run().is_err(),
            "finite f64 {target} overflows the portable replay scalar"
        );
    }

    let mut outline = monochrome_tree();
    assert!(outline.to_json_v2_strict_glyph_outline().is_ok());
    outline_mut(&mut outline).placement.run_to_page.e = f64::MAX;
    assert!(
        outline.to_json_v2_strict_glyph_outline().is_err(),
        "outline placement requires the same portable scalar range as GlyphRun"
    );
}

#[test]
fn browser_selector_checks_portable_scalars_and_exact_face_glyph_range() {
    let mut control = native_tree();
    let LayerNodeKind::Leaf { ops } = &mut control.root.kind else {
        unreachable!("leaf fixture")
    };
    ops.retain(|op| !matches!(op, PaintOp::GlyphOutline { .. }));
    let options = TextVariantSelectionOptions {
        backend: VariantSelectionBackend::CanvasKitBrowser,
        ..TextVariantSelectionOptions::canvaskit()
    };
    let reports = analyze_text_variant_selection(&control, options);
    assert_eq!(reports.len(), 1);
    assert_eq!(
        reports[0].selected_variant_kind,
        Some(TextVariantKind::GlyphRun)
    );
    assert!(!reports[0].fallback_required);

    for value in [1e100, -1e100] {
        for (field, expected_reason) in [
            ("placement", VariantRejectReason::PlacementNotFinite),
            ("position", VariantRejectReason::PositionNotFinite),
            ("advance", VariantRejectReason::AdvanceNotFinite),
        ] {
            let mut invalid = control.clone();
            let run = glyph_run_mut(&mut invalid);
            match field {
                "placement" => run.placement.run_to_page.e = value,
                "position" => run.positions[0].x = value,
                "advance" => run.advances = Some(vec![LayerVector { dx: value, dy: 0.0 }]),
                _ => unreachable!(),
            }
            let reports = analyze_text_variant_selection(&invalid, options);
            assert_eq!(reports.len(), 1);
            let report = &reports[0];
            assert_eq!(report.selected_variant_kind, Some(TextVariantKind::TextRun));
            assert!(report.fallback_required);
            assert_eq!(report.rejected_variants.len(), 1);
            assert!(report.rejected_variants[0]
                .reasons
                .contains(&expected_reason));
        }
    }
    for value in [0.0, -16.0, f64::from(f32::MAX), -f64::from(f32::MAX)] {
        let mut valid = control.clone();
        glyph_run_mut(&mut valid).positions[0].x = value;
        let reports = analyze_text_variant_selection(&valid, options);
        assert_eq!(reports.len(), 1);
        assert_eq!(
            reports[0].selected_variant_kind,
            Some(TextVariantKind::GlyphRun)
        );
    }
    // Fitting u16 does not prove membership in this exact font face.
    let glyph_count = u32::from(ttf_parser::Face::parse(FONT, 0).unwrap().number_of_glyphs());
    assert!(glyph_count > 1 && glyph_count < u32::from(u16::MAX));
    for invalid_id in [glyph_count, u32::from(u16::MAX)] {
        let mut invalid = control.clone();
        glyph_run_mut(&mut invalid).glyph_ids[0] = invalid_id;
        let reports = analyze_text_variant_selection(&invalid, options);
        assert_eq!(reports.len(), 1);
        let report = &reports[0];
        assert_eq!(report.selected_variant_kind, Some(TextVariantKind::TextRun));
        assert!(report.fallback_required);
        assert!(report.rejected_variants[0]
            .reasons
            .contains(&VariantRejectReason::GlyphIdOutOfRange));
    }
}
