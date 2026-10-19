//! Strict glyph geometry must remain finite in the f32 replay coordinate space.

use rhwp::paint::*;
use rhwp::renderer::canvaskit_policy::{analyze_canvaskit_replay_plan, CanvasKitReplayMode};
use rhwp::renderer::layer_renderer::{
    analyze_text_variant_selection, TextVariantSelectionOptions, VariantRejectReason,
    VariantSelectedReason,
};
use rhwp::renderer::render_tree::{BoundingBox, FieldMarkerType, TextRunNode};
use rhwp::renderer::{PathCommand, TextStyle};

include!("../support/issue_536_portable_glyph_geometry.rs");

#[test]
fn existing_payload_contracts_accept_finite_negative_origins() {
    assert!(colrv0().has_colrv0_resolved_layer_contract());
    for kind in [
        ColorPaintGraphNodeKind::SolidPath,
        ColorPaintGraphNodeKind::LinearGradientPath,
        ColorPaintGraphNodeKind::RadialGradientPath,
        ColorPaintGraphNodeKind::SweepGradientPath,
    ] {
        assert!(
            colrv1(kind).has_colrv1_supported_graph_contract(),
            "{kind:?}"
        );
    }
    assert!(bitmap().has_strict_visual_contract());
    assert!(svg().has_static_sanitized_contract());
}

#[test]
fn color_paths_reject_finite_coordinates_outside_f32() {
    assert!(colrv0().has_colrv0_resolved_layer_contract());
    assert!(colrv1(ColorPaintGraphNodeKind::SolidPath).has_colrv1_supported_graph_contract());
    for value in [-1e100, 1e100] {
        let path_cases = [
            PathCommand::MoveTo(value, 0.0),
            PathCommand::MoveTo(0.0, value),
            PathCommand::LineTo(value, 0.0),
            PathCommand::LineTo(0.0, value),
            PathCommand::CurveTo(value, 1.0, 2.0, 3.0, 8.0, 8.0),
            PathCommand::CurveTo(1.0, value, 2.0, 3.0, 8.0, 8.0),
            PathCommand::CurveTo(1.0, 1.0, value, 3.0, 8.0, 8.0),
            PathCommand::CurveTo(1.0, 1.0, 2.0, value, 8.0, 8.0),
            PathCommand::CurveTo(1.0, 1.0, 2.0, 3.0, value, 8.0),
            PathCommand::CurveTo(1.0, 1.0, 2.0, 3.0, 8.0, value),
            PathCommand::ArcTo(value, 8.0, 0.0, false, false, 8.0, 8.0),
            PathCommand::ArcTo(8.0, value, 0.0, false, false, 8.0, 8.0),
            PathCommand::ArcTo(8.0, 8.0, value, false, false, 8.0, 8.0),
            PathCommand::ArcTo(8.0, 8.0, 0.0, false, false, value, 8.0),
            PathCommand::ArcTo(8.0, 8.0, 0.0, false, false, 8.0, value),
        ];
        for (index, command) in path_cases.into_iter().enumerate() {
            let mut layers = colrv0();
            layers.layers[0].commands.as_mut().unwrap()[1] = command.clone();
            assert!(
                !layers.has_colrv0_resolved_layer_contract(),
                "COLRv0 path coordinate {index}: {value}"
            );
            let mut layers = colrv1(ColorPaintGraphNodeKind::SolidPath);
            layers.paint_graph.as_mut().unwrap().nodes[0]
                .solid_path
                .as_mut()
                .unwrap()
                .commands[1] = command;
            assert!(
                !layers.has_colrv1_supported_graph_contract(),
                "COLRv1 path coordinate {index}: {value}"
            );
        }
    }
}

#[test]
fn color_paths_and_affines_accept_signed_f32_limits() {
    for limit in [-f64::from(f32::MAX), f64::from(f32::MAX)] {
        let mut layers = colrv0();
        layers.layers[0].commands.as_mut().unwrap()[0] = PathCommand::MoveTo(limit, limit);
        layers.layers[0].transform_to_run.as_mut().unwrap().e = limit;
        assert!(layers.has_colrv0_resolved_layer_contract());
        let mut layers = colrv1(ColorPaintGraphNodeKind::SolidPath);
        layers.paint_graph.as_mut().unwrap().nodes[0]
            .solid_path
            .as_mut()
            .unwrap()
            .commands[0] = PathCommand::MoveTo(limit, limit);
        assert!(layers.has_colrv1_supported_graph_contract());
        let mut bitmap = bitmap();
        bitmap.transform_to_run.as_mut().unwrap().e = limit;
        assert!(bitmap.has_strict_visual_contract());
        let mut svg = svg();
        svg.transform_to_run.as_mut().unwrap().f = limit;
        assert!(svg.has_static_sanitized_contract());
    }
}

#[test]
fn payload_transforms_reject_each_finite_coordinate_outside_f32() {
    for value in [-1e100, 1e100] {
        for index in 0..6 {
            let mut transform = affine();
            *[
                &mut transform.a,
                &mut transform.b,
                &mut transform.c,
                &mut transform.d,
                &mut transform.e,
                &mut transform.f,
            ][index] = value;
            let mut layers = colrv0();
            layers.layers[0].transform_to_run = Some(transform);
            assert!(
                !layers.has_colrv0_resolved_layer_contract(),
                "{index}: {value}"
            );
            let mut bitmap = bitmap();
            bitmap.transform_to_run = Some(transform);
            assert!(!bitmap.has_strict_visual_contract(), "{index}: {value}");
            let mut svg = svg();
            svg.transform_to_run = Some(transform);
            assert!(!svg.has_static_sanitized_contract(), "{index}: {value}");
            let mut layers = colrv1(ColorPaintGraphNodeKind::SolidPath);
            let graph = layers.paint_graph.as_mut().unwrap();
            let mut parent = graph.nodes[0].clone();
            parent.node_id = 2;
            parent.kind = ColorPaintGraphNodeKind::Transform;
            parent.solid_path = None;
            parent.transform = Some(ColorPaintTransformNode {
                child_node_id: 1,
                transform: affine(),
            });
            graph.nodes.push(parent);
            graph.root_node_id = 2;
            assert!(layers.has_colrv1_supported_graph_contract());
            layers.paint_graph.as_mut().unwrap().nodes[1]
                .transform
                .as_mut()
                .unwrap()
                .transform = transform;
            assert!(
                !layers.has_colrv1_supported_graph_contract(),
                "{index}: {value}"
            );
        }
    }
}

#[test]
fn gradient_geometry_rejects_finite_coordinates_outside_f32() {
    for value in [-1e100, 1e100] {
        for index in 0..4 {
            let mut layers = colrv1(ColorPaintGraphNodeKind::LinearGradientPath);
            assert!(layers.has_colrv1_supported_graph_contract());
            let gradient = &mut layers.paint_graph.as_mut().unwrap().nodes[0]
                .linear_gradient_path
                .as_mut()
                .unwrap()
                .gradient;
            *[
                &mut gradient.x0,
                &mut gradient.y0,
                &mut gradient.x1,
                &mut gradient.y1,
            ][index] = value;
            assert!(
                !layers.has_colrv1_supported_graph_contract(),
                "linear {index}: {value}"
            );
        }
        for index in 0..3 {
            let mut layers = colrv1(ColorPaintGraphNodeKind::RadialGradientPath);
            assert!(layers.has_colrv1_supported_graph_contract());
            let gradient = &mut layers.paint_graph.as_mut().unwrap().nodes[0]
                .radial_gradient_path
                .as_mut()
                .unwrap()
                .gradient;
            *[&mut gradient.cx, &mut gradient.cy, &mut gradient.radius][index] = value;
            assert!(
                !layers.has_colrv1_supported_graph_contract(),
                "radial {index}: {value}"
            );
        }
        for index in 0..2 {
            let mut layers = colrv1(ColorPaintGraphNodeKind::SweepGradientPath);
            assert!(layers.has_colrv1_supported_graph_contract());
            let gradient = &mut layers.paint_graph.as_mut().unwrap().nodes[0]
                .sweep_gradient_path
                .as_mut()
                .unwrap()
                .gradient;
            *[&mut gradient.cx, &mut gradient.cy][index] = value;
            assert!(
                !layers.has_colrv1_supported_graph_contract(),
                "sweep {index}: {value}"
            );
        }
    }
}

#[test]
fn bitmap_and_svg_bounds_reject_out_of_range_components_and_edges() {
    assert!(bitmap().has_strict_visual_contract());
    assert!(svg().has_static_sanitized_contract());
    let edge = f64::from(f32::MAX) * 0.75;
    let mut cases = vec![
        BoundingBox::new(edge, 0.0, edge, 8.0),
        BoundingBox::new(0.0, edge, 8.0, edge),
    ];
    for value in [-1e100, 1e100] {
        cases.extend([
            BoundingBox::new(value, 0.0, 8.0, 8.0),
            BoundingBox::new(0.0, value, 8.0, 8.0),
        ]);
    }
    cases.extend([
        BoundingBox::new(0.0, 0.0, 1e100, 8.0),
        BoundingBox::new(0.0, 0.0, 8.0, 1e100),
    ]);
    for (index, bounds) in cases.into_iter().enumerate() {
        let mut bitmap = bitmap();
        bitmap.placement = bounds;
        assert!(!bitmap.has_strict_visual_contract(), "bitmap case {index}");
        let mut svg = svg();
        svg.view_box = bounds;
        assert!(!svg.has_static_sanitized_contract(), "SVG case {index}");
    }
}

#[test]
fn bitmap_and_svg_bounds_accept_edges_exactly_at_f32_limit() {
    let limit = f64::from(f32::MAX);
    for bounds in [
        BoundingBox::new(limit * 0.5, limit * 0.5, limit * 0.5, limit * 0.5),
        BoundingBox::new(-limit, -limit, limit, limit),
    ] {
        let mut bitmap = bitmap();
        bitmap.placement = bounds;
        assert!(bitmap.has_strict_visual_contract());
        let mut svg = svg();
        svg.view_box = bounds;
        assert!(svg.has_static_sanitized_contract());
    }
}

#[test]
fn svg_intrinsic_size_rejects_finite_values_outside_f32() {
    assert!(svg().has_static_sanitized_contract());
    for size in [
        LayerVector { dx: 1e100, dy: 8.0 },
        LayerVector { dx: 8.0, dy: 1e100 },
    ] {
        let mut payload = svg();
        payload.intrinsic_size = Some(size);
        assert!(!payload.has_static_sanitized_contract());
    }
    let mut payload = svg();
    payload.intrinsic_size = Some(LayerVector {
        dx: f64::from(f32::MAX),
        dy: f64::from(f32::MAX),
    });
    assert!(payload.has_static_sanitized_contract());
}

#[test]
fn strict_stroke_rejects_finite_values_outside_f32() {
    let stroke = GlyphOutlineStrokeStyle {
        color: 0,
        width: 1.0,
        join: GlyphOutlineStrokeJoin::Miter,
        cap: GlyphOutlineStrokeCap::Butt,
        miter_limit: 4.0,
        paint_order: GlyphOutlinePaintOrder::FillThenStroke,
    };
    assert!(stroke.is_strict_subset());
    assert!(!GlyphOutlineStrokeStyle {
        width: 1e100,
        ..stroke.clone()
    }
    .is_strict_subset());
    assert!(!GlyphOutlineStrokeStyle {
        miter_limit: 1e100,
        ..stroke.clone()
    }
    .is_strict_subset());
    assert!(GlyphOutlineStrokeStyle {
        width: f64::from(f32::MAX),
        miter_limit: f64::from(f32::MAX),
        ..stroke
    }
    .is_strict_subset());
}

#[test]
fn strict_outline_selector_keeps_text_fallback_when_geometry_exceeds_f32() {
    let tree = monochrome_selector_tree();
    let options = TextVariantSelectionOptions::canvaskit_strict_outline();
    let reports = analyze_text_variant_selection(&tree, options);
    assert_eq!(reports.len(), 1);
    assert_eq!(
        reports[0].selected_variant_kind,
        Some(TextVariantKind::GlyphOutline)
    );
    assert_eq!(
        reports[0].selected_reason,
        VariantSelectedReason::GlyphOutlineStrictProfile
    );
    assert!(!reports[0].fallback_required);
    assert!(reports[0].rejected_variants.is_empty());

    for value in [-1e100, 1e100] {
        for field in ["path", "transform", "baseline"] {
            let mut invalid = tree.clone();
            let LayerNodeKind::Leaf { ops } = &mut invalid.root.kind else {
                unreachable!("leaf fixture")
            };
            let PaintOp::GlyphOutline { outline, .. } = &mut ops[1] else {
                unreachable!("strict outline fixture")
            };
            match field {
                "path" => outline.paths[0].commands[1] = PathCommand::LineTo(value, 0.0),
                "transform" => outline.placement.run_to_page.e = value,
                "baseline" => outline.placement.baseline_y = value,
                _ => unreachable!(),
            }
            let reports = analyze_text_variant_selection(&invalid, options);
            assert_eq!(reports.len(), 1);
            let report = &reports[0];
            assert_eq!(
                report.selected_variant_kind,
                Some(TextVariantKind::TextRun),
                "{field}: {value}"
            );
            assert_eq!(report.selected_variant_id.as_deref(), Some("textRun"));
            assert_eq!(
                report.selected_reason,
                VariantSelectedReason::DefaultTextRunFallback
            );
            assert!(report.fallback_required);
            assert_eq!(report.rejected_variants.len(), 1);
            assert_eq!(report.rejected_variants[0].variant_id, "glyphOutline");
            assert_eq!(
                report.rejected_variants[0].reasons,
                vec![VariantRejectReason::UnsupportedOutlinePayload],
                "only the changed scalar should reject {field}: {value}"
            );
        }
    }
}

#[test]
fn strict_outline_selector_obeys_the_portable_command_limit() {
    let mut tree = monochrome_selector_tree();
    for (command_count, expected) in [
        (4096, TextVariantKind::GlyphOutline),
        (4097, TextVariantKind::TextRun),
    ] {
        let LayerNodeKind::Leaf { ops } = &mut tree.root.kind else {
            unreachable!("leaf fixture")
        };
        let PaintOp::GlyphOutline { outline, .. } = &mut ops[1] else {
            unreachable!("outline fixture")
        };
        outline.paths[0]
            .commands
            .resize(command_count, PathCommand::LineTo(0.0, 0.0));
        let reports = analyze_text_variant_selection(
            &tree,
            TextVariantSelectionOptions::canvaskit_strict_outline(),
        );
        assert_eq!(reports.len(), 1);
        let report = &reports[0];
        assert_eq!(report.selected_variant_kind, Some(expected));
        assert_eq!(report.fallback_required, command_count > 4096);
        if command_count > 4096 {
            assert_eq!(report.rejected_variants.len(), 1);
            assert_eq!(
                report.rejected_variants[0].reasons,
                vec![VariantRejectReason::UnsupportedOutlinePayload]
            );
        } else {
            assert!(report.rejected_variants.is_empty());
        }
    }
}

#[test]
fn bitmap_and_svg_bounds_stay_positive_in_f32_replay_space() {
    for bounds in [
        BoundingBox::new(0.0, 0.0, 1e-100, 8.0),
        BoundingBox::new(0.0, 0.0, 8.0, 1e-100),
        BoundingBox::new(1.0, 0.0, 2f64.powi(-25), 8.0),
        BoundingBox::new(0.0, 1.0, 8.0, 2f64.powi(-25)),
        // The two consumers narrow in different orders; either can collapse.
        BoundingBox::new(1.0 + 2f64.powi(-25), 0.0, 3.0 * 2f64.powi(-26), 8.0),
        BoundingBox::new(1.0 - 2f64.powi(-26), 0.0, 5.0 * 2f64.powi(-26), 8.0),
        // The f64 edge fits, but the sum of the narrowed components overflows.
        BoundingBox::new(
            f64::from(f32::MAX) - 2f64.powi(103) + 2f64.powi(77),
            0.0,
            2f64.powi(103) - 2f64.powi(77),
            8.0,
        ),
    ] {
        let mut bitmap = bitmap();
        bitmap.placement = bounds;
        assert!(!bitmap.has_strict_visual_contract(), "bitmap: {bounds:?}");
        let mut svg = svg();
        svg.view_box = bounds;
        assert!(!svg.has_static_sanitized_contract(), "SVG: {bounds:?}");
    }
    for size in [
        LayerVector {
            dx: 1e-100,
            dy: 8.0,
        },
        LayerVector {
            dx: 8.0,
            dy: 1e-100,
        },
    ] {
        let mut svg = svg();
        svg.intrinsic_size = Some(size);
        assert!(!svg.has_static_sanitized_contract(), "size: {size:?}");
    }
    // A representable subnormal has native edges, but the decimal wire loses them.
    let minimum = f64::from(f32::from_bits(1));
    let mut bitmap = bitmap();
    bitmap.placement = BoundingBox::new(0.0, 0.0, minimum, minimum);
    assert!(!bitmap.has_strict_visual_contract());
    let mut svg = svg();
    svg.view_box = bitmap.placement;
    svg.intrinsic_size = Some(LayerVector {
        dx: minimum,
        dy: minimum,
    });
    assert!(!svg.has_static_sanitized_contract());
}

#[test]
fn shared_svg_selector_preserves_the_required_fallback_font_for_invalid_destination() {
    let mut tree = monochrome_selector_tree();
    let svg_ref = tree
        .resources
        .intern_svg_fragment("<path d=\"M0 0H10V10Z\"/>");
    let LayerNodeKind::Leaf { ops } = &mut tree.root.kind else {
        unreachable!()
    };
    let PaintOp::GlyphOutline { outline, .. } = &mut ops[1] else {
        unreachable!()
    };
    outline.payload_kind = GlyphOutlinePayloadKind::SvgGlyph;
    outline.paths.clear();
    outline.svg_glyph = Some(SvgGlyphPayload { svg_ref, ..svg() });
    let options = TextVariantSelectionOptions {
        allow_svg_glyph: true,
        ..TextVariantSelectionOptions::canvaskit_strict_outline()
    };
    assert_eq!(
        analyze_text_variant_selection(&tree, options)[0].selected_variant_kind,
        Some(TextVariantKind::GlyphOutline)
    );
    assert!(
        analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default)
            .required_font_families
            .is_empty()
    );
    let edge = f64::from(f32::MAX) * 0.75;
    for bounds in [
        BoundingBox::new(1e100, 0.0, 8.0, 8.0),
        BoundingBox::new(0.0, -1e100, 8.0, 8.0),
        BoundingBox::new(edge, 0.0, edge, 8.0),
        BoundingBox::new(0.0, edge, 8.0, edge),
        BoundingBox::new(0.0, 0.0, 1e-100, 8.0),
        BoundingBox::new(1.0, 0.0, 2f64.powi(-25), 8.0),
    ] {
        let mut invalid = tree.clone();
        let LayerNodeKind::Leaf { ops } = &mut invalid.root.kind else {
            unreachable!()
        };
        let PaintOp::GlyphOutline { bbox, .. } = &mut ops[1] else {
            unreachable!()
        };
        *bbox = bounds;
        let reports = analyze_text_variant_selection(&invalid, options);
        assert_eq!(
            reports[0].selected_variant_kind,
            Some(TextVariantKind::TextRun),
            "{bounds:?}"
        );
        assert!(reports[0].fallback_required);
        assert!(reports[0].rejected_variants[0]
            .reasons
            .contains(&VariantRejectReason::UnsupportedSvgGlyph));
        let plan = analyze_canvaskit_replay_plan(&invalid, CanvasKitReplayMode::Default);
        assert_eq!(plan.required_font_families, ["Test"], "{bounds:?}");
        assert!(plan.required_font_families_complete);
    }
}

#[test]
fn shared_bitmap_and_svg_selector_preserves_fonts_when_decimal_wire_extents_collapse() {
    for (field, values) in [
        (
            "bitmapPlacement",
            vec![f64::from(f32::from_bits(1)), 0.000_49],
        ),
        ("svgViewBox", vec![f64::from(f32::from_bits(1)), 0.000_49]),
        (
            "svgDestination",
            vec![f64::from(f32::from_bits(1)), 0.000_49],
        ),
        (
            "svgIntrinsic",
            vec![f64::from(f32::from_bits(1)), 0.000_000_49],
        ),
    ] {
        for value in values {
            let mut tree = monochrome_selector_tree();
            let svg_ref = tree
                .resources
                .intern_svg_fragment("<path d=\"M0 0H10V10Z\"/>");
            let image_ref = tree
                .resources
                .intern_image_bytes(include_bytes!("../../assets/logo/logo-32.png"));
            let LayerNodeKind::Leaf { ops } = &mut tree.root.kind else {
                unreachable!()
            };
            let PaintOp::GlyphOutline { bbox, outline, .. } = &mut ops[1] else {
                unreachable!()
            };
            outline.paths.clear();
            if field == "bitmapPlacement" {
                outline.payload_kind = GlyphOutlinePayloadKind::BitmapGlyph;
                outline.bitmap_glyph = Some(BitmapGlyphPayload {
                    image_ref,
                    placement: BoundingBox::new(0.0, 0.0, value, 8.0),
                    ..bitmap()
                });
            } else {
                outline.payload_kind = GlyphOutlinePayloadKind::SvgGlyph;
                let mut payload = SvgGlyphPayload { svg_ref, ..svg() };
                match field {
                    "svgViewBox" => payload.view_box = BoundingBox::new(0.0, 0.0, value, 8.0),
                    "svgDestination" => *bbox = BoundingBox::new(0.0, 0.0, value, 8.0),
                    "svgIntrinsic" => {
                        payload.intrinsic_size = Some(LayerVector { dx: value, dy: 8.0 })
                    }
                    _ => unreachable!(),
                }
                outline.svg_glyph = Some(payload);
            }
            let options = TextVariantSelectionOptions {
                allow_bitmap_glyph: true,
                allow_svg_glyph: true,
                ..TextVariantSelectionOptions::canvaskit_strict_outline()
            };
            // Exercise the real serializer rather than a numerical approximation
            // of its precision: this is the payload CanvasKit actually receives.
            let wire: serde_json::Value = serde_json::from_str(&tree.to_json()).unwrap();
            let op = &wire["root"]["ops"][1];
            let serialized_extent = match field {
                "bitmapPlacement" => &op["bitmapGlyph"]["placement"]["width"],
                "svgViewBox" => &op["svgGlyph"]["viewBox"]["width"],
                "svgDestination" => &op["bbox"]["width"],
                "svgIntrinsic" => &op["svgGlyph"]["intrinsicSize"]["width"],
                _ => unreachable!(),
            };
            assert_eq!(serialized_extent.as_f64(), Some(0.0), "{field}: {value}");
            let report = &analyze_text_variant_selection(&tree, options)[0];
            assert_eq!(
                report.selected_variant_kind,
                Some(TextVariantKind::TextRun),
                "{field}: {value}"
            );
            assert!(report.fallback_required);
            let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);
            assert_eq!(plan.required_font_families, ["Test"], "{field}: {value}");
            assert!(plan.required_font_families_complete);
            if field != "svgDestination" {
                assert!(op.get("payloadResourceKey").is_none(), "{field}: {value}");
            }
        }
    }
}

#[test]
fn bitmap_and_svg_resource_keys_use_full_fixed_decimals_at_large_f32_values() {
    for (value, integer) in [
        (1e21, "1000000000000000000000"),
        (
            f64::from(f32::MAX),
            "340282346638528859811704183484516925440",
        ),
    ] {
        let mut tree = monochrome_selector_tree();
        let svg_ref = tree
            .resources
            .intern_svg_fragment("<path d=\"M0 0H10V10Z\"/>");
        let LayerNodeKind::Leaf { ops } = &mut tree.root.kind else {
            unreachable!()
        };
        let PaintOp::GlyphOutline { outline, .. } = &mut ops[1] else {
            unreachable!()
        };
        outline.payload_kind = GlyphOutlinePayloadKind::SvgGlyph;
        outline.paths.clear();
        outline.svg_glyph = Some(SvgGlyphPayload {
            svg_ref,
            view_box: BoundingBox::new(0.0, -3.0, value, 8.0),
            intrinsic_size: Some(LayerVector { dx: value, dy: 8.0 }),
            transform_to_run: Some(LayerAffineTransform {
                e: -value,
                ..affine()
            }),
            ..svg()
        });
        let key = outline.payload_resource_key().unwrap();
        assert!(key.contains(&format!("viewBox:0.000,-3.000,{integer}.000,8.000:")));
        assert!(key.contains(&format!("intrinsicSize:{integer}.000000,8.000000:")));
        assert!(key.ends_with(&format!(
            "transform:1.000000,0.000000,0.000000,1.000000,-{integer}.000000,-3.000000"
        )));
        let wire: serde_json::Value = serde_json::from_str(&tree.to_json()).unwrap();
        let op = &wire["root"]["ops"][1];
        assert!(op["payloadResourceKey"]
            .as_str()
            .unwrap()
            .starts_with(&format!("{key}:resource:")));
        assert_eq!(
            op["svgGlyph"]["intrinsicSize"]["width"].as_f64(),
            Some(value)
        );
        assert_eq!(op["svgGlyph"]["transformToRun"]["e"].as_f64(), Some(-value));
        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);
        assert!(plan.required_font_families.is_empty());
        assert!(plan.required_font_families_complete);
    }
}

#[test]
fn bitmap_and_svg_resource_keys_preserve_the_fixed_decimal_negative_zero_sign() {
    let mut tree = monochrome_selector_tree();
    let LayerNodeKind::Leaf { ops } = &mut tree.root.kind else {
        unreachable!()
    };
    let PaintOp::GlyphOutline { outline, .. } = &mut ops[1] else {
        unreachable!()
    };
    outline.payload_kind = GlyphOutlinePayloadKind::BitmapGlyph;
    outline.paths.clear();
    outline.bitmap_glyph = Some(BitmapGlyphPayload {
        placement: BoundingBox::new(-0.0, -3.0, 8.0, 8.0),
        transform_to_run: Some(LayerAffineTransform {
            e: -0.0,
            ..affine()
        }),
        ..bitmap()
    });
    let key = outline.payload_resource_key().unwrap();
    assert!(key.contains("placement:-0.000,-3.000,8.000,8.000:"));
    assert!(key.ends_with("transform:1.000000,0.000000,0.000000,1.000000,-0.000000,-3.000000"));
    let wire: serde_json::Value = serde_json::from_str(&tree.to_json()).unwrap();
    let op = &wire["root"]["ops"][1];
    assert_eq!(op["payloadResourceKey"], key);
    assert!(op["bitmapGlyph"]["placement"]["x"]
        .as_f64()
        .unwrap()
        .is_sign_negative());
    assert!(op["bitmapGlyph"]["transformToRun"]["e"]
        .as_f64()
        .unwrap()
        .is_sign_negative());
}

#[test]
fn shared_svg_selector_keeps_fallback_fonts_when_view_box_mapping_overflows() {
    let max = f64::from(f32::MAX);
    let normal = SvgGlyphPayload {
        view_box: BoundingBox::new(0.0, 0.0, 0.001, 1.0),
        ..svg()
    };
    assert!(normal.has_portable_view_box_mapping(BoundingBox::new(0.0, 0.0, 1.0, 1.0)));
    assert!(svg().has_portable_view_box_mapping(BoundingBox::new(-10.0, -10.0, 20.0, 20.0)));
    for (view_box, destination) in [
        (
            BoundingBox::new(0.0, 0.0, 0.001, 1.0),
            BoundingBox::new(0.0, 0.0, max, 1.0),
        ),
        (
            BoundingBox::new(0.0, 0.0, 1.0, 0.001),
            BoundingBox::new(0.0, 0.0, 1.0, max),
        ),
        (
            BoundingBox::new(1e20, 0.0, 1e14, 1.0),
            BoundingBox::new(0.0, 0.0, 1e36, 1.0),
        ),
        (
            BoundingBox::new(-4.0, 0.0, 1.0, 1.0),
            BoundingBox::new(max * 0.75, 0.0, max * 0.125, 1.0),
        ),
        // The original ratio is finite f32; rounding view width to .001
        // makes the actual CanvasKit wire ratio exceed f32::MAX.
        (
            BoundingBox::new(0.0, 0.0, 0.001_49, 1.0),
            BoundingBox::new(0.0, 0.0, max * 0.001_25, 1.0),
        ),
    ] {
        let mut tree = monochrome_selector_tree();
        let svg_ref = tree
            .resources
            .intern_svg_fragment("<path d=\"M0 0H10V10Z\"/>");
        let LayerNodeKind::Leaf { ops } = &mut tree.root.kind else {
            unreachable!()
        };
        let PaintOp::GlyphOutline { bbox, outline, .. } = &mut ops[1] else {
            unreachable!()
        };
        *bbox = destination;
        outline.payload_kind = GlyphOutlinePayloadKind::SvgGlyph;
        outline.paths.clear();
        outline.svg_glyph = Some(SvgGlyphPayload {
            svg_ref,
            view_box,
            ..svg()
        });
        let options = TextVariantSelectionOptions {
            allow_svg_glyph: true,
            ..TextVariantSelectionOptions::canvaskit_strict_outline()
        };
        let report = &analyze_text_variant_selection(&tree, options)[0];
        assert_eq!(
            report.selected_variant_kind,
            Some(TextVariantKind::TextRun),
            "{view_box:?} => {destination:?}"
        );
        assert!(report.fallback_required);
        assert!(report.rejected_variants[0]
            .reasons
            .contains(&VariantRejectReason::UnsupportedSvgGlyph));
        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);
        assert_eq!(plan.required_font_families, ["Test"]);
        assert!(plan.required_font_families_complete);
    }
}

#[test]
fn shared_svg_selector_uses_the_explicit_font_em_destination() {
    let mut tree = monochrome_selector_tree();
    let svg_ref = tree
        .resources
        .intern_svg_fragment("<path d=\"M0 0H10V10Z\"/>");
    let LayerNodeKind::Leaf { ops } = &mut tree.root.kind else {
        unreachable!()
    };
    let PaintOp::GlyphOutline { bbox, outline, .. } = &mut ops[1] else {
        unreachable!()
    };
    *bbox = BoundingBox::new(1e100, 0.0, 8.0, 8.0);
    outline.payload_kind = GlyphOutlinePayloadKind::SvgGlyph;
    outline.paths.clear();
    outline.svg_glyph = Some(SvgGlyphPayload {
        svg_ref,
        font_em_box: Some(BoundingBox::new(0.0, 0.0, 8.0, 8.0)),
        ..svg()
    });
    let options = TextVariantSelectionOptions {
        allow_svg_glyph: true,
        ..TextVariantSelectionOptions::canvaskit_strict_outline()
    };
    assert_eq!(
        analyze_text_variant_selection(&tree, options)[0].selected_variant_kind,
        Some(TextVariantKind::GlyphOutline)
    );
    assert!(
        analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default)
            .required_font_families
            .is_empty()
    );
    for invalid in [
        BoundingBox::new(0.0, 0.0, 1e-100, 8.0),
        BoundingBox::new(1.0, 0.0, 2f64.powi(-25), 8.0),
    ] {
        let LayerNodeKind::Leaf { ops } = &mut tree.root.kind else {
            unreachable!()
        };
        let PaintOp::GlyphOutline { outline, .. } = &mut ops[1] else {
            unreachable!()
        };
        outline.svg_glyph.as_mut().unwrap().font_em_box = Some(invalid);
        assert_eq!(
            analyze_text_variant_selection(&tree, options)[0].selected_variant_kind,
            Some(TextVariantKind::TextRun)
        );
        assert_eq!(
            analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default)
                .required_font_families,
            ["Test"]
        );
    }
}

#[test]
fn shared_svg_mapping_checks_the_explicit_font_em_viewport() {
    for (compatibility_width, em_width, expected) in [
        (f64::from(f32::MAX), 1.0, TextVariantKind::GlyphOutline),
        (1.0, f64::from(f32::MAX), TextVariantKind::TextRun),
    ] {
        let mut tree = monochrome_selector_tree();
        let svg_ref = tree
            .resources
            .intern_svg_fragment("<path d=\"M0 0H10V10Z\"/>");
        let LayerNodeKind::Leaf { ops } = &mut tree.root.kind else {
            unreachable!()
        };
        let PaintOp::GlyphOutline { bbox, outline, .. } = &mut ops[1] else {
            unreachable!()
        };
        *bbox = BoundingBox::new(0.0, 0.0, compatibility_width, 1.0);
        outline.payload_kind = GlyphOutlinePayloadKind::SvgGlyph;
        outline.paths.clear();
        outline.svg_glyph = Some(SvgGlyphPayload {
            svg_ref,
            view_box: BoundingBox::new(0.0, 0.0, 0.001, 1.0),
            font_em_box: Some(BoundingBox::new(0.0, 0.0, em_width, 1.0)),
            ..svg()
        });
        let options = TextVariantSelectionOptions {
            allow_svg_glyph: true,
            ..TextVariantSelectionOptions::canvaskit_strict_outline()
        };
        assert_eq!(
            analyze_text_variant_selection(&tree, options)[0].selected_variant_kind,
            Some(expected)
        );
        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);
        assert_eq!(
            plan.required_font_families.is_empty(),
            expected == TextVariantKind::GlyphOutline
        );
        assert!(plan.required_font_families_complete);
    }
}

#[test]
fn shared_selector_preserves_fonts_when_positive_six_decimal_scalars_disappear() {
    for family in ["stroke", "radial"] {
        for (value, admitted) in [
            (f64::from(f32::from_bits(1)), false),
            (1e-7, false),
            (1e-6, true),
        ] {
            let mut tree = monochrome_selector_tree();
            let LayerNodeKind::Leaf { ops } = &mut tree.root.kind else {
                unreachable!()
            };
            let PaintOp::GlyphOutline { outline, .. } = &mut ops[1] else {
                unreachable!()
            };
            if family == "stroke" {
                outline.payload_kind = GlyphOutlinePayloadKind::MonochromeFillStroke;
                outline.stroke = Some(GlyphOutlineStrokeStyle {
                    color: 0,
                    width: value,
                    join: GlyphOutlineStrokeJoin::Miter,
                    cap: GlyphOutlineStrokeCap::Butt,
                    miter_limit: 4.0,
                    paint_order: GlyphOutlinePaintOrder::FillThenStroke,
                });
            } else {
                outline.payload_kind = GlyphOutlinePayloadKind::ColorLayers;
                outline.paths.clear();
                let mut layers = colrv1(ColorPaintGraphNodeKind::RadialGradientPath);
                layers.paint_graph.as_mut().unwrap().nodes[0]
                    .radial_gradient_path
                    .as_mut()
                    .unwrap()
                    .gradient
                    .radius = value;
                outline.color_layers = Some(layers);
            }
            let wire: serde_json::Value = serde_json::from_str(&tree.to_json()).unwrap();
            let op = &wire["root"]["ops"][1];
            let exported = if family == "stroke" {
                &op["stroke"]["width"]
            } else {
                &op["colorLayers"]["paintGraph"]["nodes"][0]["radialGradientPath"]["gradient"]
                    ["radius"]
            };
            assert_eq!(exported.as_f64(), Some(if admitted { 1e-6 } else { 0.0 }));
            let options = TextVariantSelectionOptions {
                allow_colrv1_stage1_color_graph: true,
                ..TextVariantSelectionOptions::canvaskit_strict_outline()
            };
            let report = &analyze_text_variant_selection(&tree, options)[0];
            assert_eq!(
                report.selected_variant_kind,
                Some(if admitted {
                    TextVariantKind::GlyphOutline
                } else {
                    TextVariantKind::TextRun
                }),
                "{family}: {value}"
            );
            assert_eq!(report.fallback_required, !admitted);
            let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);
            assert_eq!(
                plan.required_font_families.is_empty(),
                admitted,
                "{family}: {value}"
            );
            if !admitted {
                assert_eq!(plan.required_font_families, ["Test"]);
            }
            assert!(plan.required_font_families_complete);
        }
    }
}

#[test]
fn shared_selector_requires_full_circle_sweep_on_the_decimal_wire() {
    for (start, end, admitted) in [
        (0.000_000_499_8, 360.000_000_500_2, false),
        (0.000_001, 360.000_001, true),
    ] {
        let mut tree = monochrome_selector_tree();
        let LayerNodeKind::Leaf { ops } = &mut tree.root.kind else {
            unreachable!()
        };
        let PaintOp::GlyphOutline { outline, .. } = &mut ops[1] else {
            unreachable!()
        };
        outline.payload_kind = GlyphOutlinePayloadKind::ColorLayers;
        outline.paths.clear();
        let mut layers = colrv1(ColorPaintGraphNodeKind::SweepGradientPath);
        let gradient = &mut layers.paint_graph.as_mut().unwrap().nodes[0]
            .sweep_gradient_path
            .as_mut()
            .unwrap()
            .gradient;
        gradient.start_angle_degrees = start;
        gradient.end_angle_degrees = end;
        outline.color_layers = Some(layers);
        let wire: serde_json::Value = serde_json::from_str(&tree.to_json()).unwrap();
        let gradient = &wire["root"]["ops"][1]["colorLayers"]["paintGraph"]["nodes"][0]
            ["sweepGradientPath"]["gradient"];
        let wire_span = gradient["endAngleDegrees"].as_f64().unwrap()
            - gradient["startAngleDegrees"].as_f64().unwrap();
        assert_eq!((wire_span - 360.0).abs() <= 1e-9, admitted);
        let options = TextVariantSelectionOptions {
            allow_colrv1_stage1_color_graph: true,
            ..TextVariantSelectionOptions::canvaskit_strict_outline()
        };
        assert_eq!(
            analyze_text_variant_selection(&tree, options)[0].selected_variant_kind,
            Some(if admitted {
                TextVariantKind::GlyphOutline
            } else {
                TextVariantKind::TextRun
            })
        );
        let plan = analyze_canvaskit_replay_plan(&tree, CanvasKitReplayMode::Default);
        assert_eq!(plan.required_font_families.is_empty(), admitted);
        if !admitted {
            assert_eq!(plan.required_font_families, ["Test"]);
        }
    }
}
