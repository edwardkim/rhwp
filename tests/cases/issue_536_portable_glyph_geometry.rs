//! Strict glyph geometry must remain finite in the f32 replay coordinate space.

use rhwp::paint::*;
use rhwp::renderer::layer_renderer::{
    analyze_text_variant_selection, TextVariantSelectionOptions, VariantRejectReason,
    VariantSelectedReason,
};
use rhwp::renderer::render_tree::{BoundingBox, FieldMarkerType, TextRunNode};
use rhwp::renderer::{PathCommand, TextStyle};

fn color() -> ResolvedColor {
    ResolvedColor {
        color_space: Some("sRGB".to_string()),
        rgba: [0.25, 0.5, 0.75, 1.0],
    }
}

fn commands() -> Vec<PathCommand> {
    vec![
        PathCommand::MoveTo(-2.0, -3.0),
        PathCommand::LineTo(8.0, 0.0),
        PathCommand::LineTo(0.0, 8.0),
        PathCommand::ClosePath,
    ]
}

fn affine() -> LayerAffineTransform {
    LayerAffineTransform {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: -2.0,
        f: -3.0,
    }
}

fn source_font(format: ColorGlyphFormat) -> FontColorGlyphRef {
    FontColorGlyphRef {
        face_key: Some("fixture-face".to_string()),
        glyph_id: Some(1),
        palette_index: Some(0),
        color_format: Some(format),
    }
}

fn colrv0() -> ColorLayersPayload {
    ColorLayersPayload {
        color_format: ColorGlyphFormat::ColrV0,
        source_font_ref: Some(source_font(ColorGlyphFormat::ColrV0)),
        palette_ref: None,
        layers: vec![ColorLayerNode {
            layer_index: Some(0),
            glyph_id: Some(1),
            glyph_range: Some(GlyphRange { start: 0, end: 1 }),
            source_range_utf8: Some(TextSourceRange { start: 0, end: 1 }),
            source_font_ref: Some(source_font(ColorGlyphFormat::ColrV0)),
            commands: Some(commands()),
            fill: Some(color()),
            fill_rule: Some(GlyphOutlineFillRule::NonZero),
            palette_index: Some(0),
            color: None,
            opacity: Some(1.0),
            transform_to_run: Some(affine()),
        }],
        paint_graph: None,
        source_range_utf8: Some(TextSourceRange { start: 0, end: 1 }),
        glyph_range: Some(GlyphRange { start: 0, end: 1 }),
    }
}

fn colrv1(kind: ColorPaintGraphNodeKind) -> ColorLayersPayload {
    let mut node = ColorPaintGraphNode {
        node_id: 1,
        kind,
        solid_path: None,
        linear_gradient_path: None,
        radial_gradient_path: None,
        sweep_gradient_path: None,
        transform: None,
        source_range_utf8: Some(TextSourceRange { start: 0, end: 1 }),
        glyph_range: Some(GlyphRange { start: 0, end: 1 }),
        source_font_ref: Some(source_font(ColorGlyphFormat::ColrV1)),
    };
    let stops = vec![
        ColorGradientStop {
            offset: 0.0,
            color: color(),
        },
        ColorGradientStop {
            offset: 1.0,
            color: ResolvedColor {
                rgba: [1.0, 0.0, 0.0, 1.0],
                ..color()
            },
        },
    ];
    match kind {
        ColorPaintGraphNodeKind::SolidPath => {
            node.solid_path = Some(ColorPaintSolidPathNode {
                commands: commands(),
                fill: color(),
                fill_rule: GlyphOutlineFillRule::NonZero,
                source_glyph_id: Some(1),
                palette_index: Some(0),
            });
        }
        ColorPaintGraphNodeKind::LinearGradientPath => {
            node.linear_gradient_path = Some(ColorPaintLinearGradientPathNode {
                commands: commands(),
                gradient: ColorLinearGradient {
                    x0: -2.0,
                    y0: -3.0,
                    x1: 8.0,
                    y1: 8.0,
                    stops,
                },
                fill_rule: GlyphOutlineFillRule::NonZero,
                source_glyph_id: Some(1),
                palette_index: Some(0),
            });
        }
        ColorPaintGraphNodeKind::RadialGradientPath => {
            node.radial_gradient_path = Some(ColorPaintRadialGradientPathNode {
                commands: commands(),
                gradient: ColorRadialGradient {
                    cx: -2.0,
                    cy: -3.0,
                    radius: 8.0,
                    stops,
                },
                fill_rule: GlyphOutlineFillRule::NonZero,
                source_glyph_id: Some(1),
                palette_index: Some(0),
            });
        }
        ColorPaintGraphNodeKind::SweepGradientPath => {
            node.sweep_gradient_path = Some(ColorPaintSweepGradientPathNode {
                commands: commands(),
                gradient: ColorSweepGradient {
                    cx: -2.0,
                    cy: -3.0,
                    start_angle_degrees: 0.0,
                    end_angle_degrees: 360.0,
                    stops,
                },
                fill_rule: GlyphOutlineFillRule::NonZero,
                source_glyph_id: Some(1),
                palette_index: Some(0),
            });
        }
        _ => panic!("fixture expects a supported leaf"),
    }
    ColorLayersPayload {
        color_format: ColorGlyphFormat::ColrV1,
        source_font_ref: Some(source_font(ColorGlyphFormat::ColrV1)),
        palette_ref: None,
        layers: Vec::new(),
        paint_graph: Some(ColorPaintGraphPayload {
            root_node_id: 1,
            nodes: vec![node],
        }),
        source_range_utf8: Some(TextSourceRange { start: 0, end: 1 }),
        glyph_range: Some(GlyphRange { start: 0, end: 1 }),
    }
}

fn bitmap() -> BitmapGlyphPayload {
    BitmapGlyphPayload {
        image_ref: ImageResourceId(0),
        source_range_utf8: TextSourceRange { start: 0, end: 1 },
        glyph_range: GlyphRange { start: 0, end: 1 },
        placement: BoundingBox::new(-2.0, -3.0, 8.0, 8.0),
        alpha_premultiplied: false,
        scaling_policy: BitmapGlyphScalingPolicy::SourceExact,
        filtering: BitmapGlyphFiltering::Linear,
        transform_to_run: Some(affine()),
    }
}

fn svg() -> SvgGlyphPayload {
    SvgGlyphPayload {
        svg_ref: SvgResourceId(0),
        source_range_utf8: TextSourceRange { start: 0, end: 1 },
        glyph_range: GlyphRange { start: 0, end: 1 },
        view_box: BoundingBox::new(-2.0, -3.0, 8.0, 8.0),
        intrinsic_size: Some(LayerVector { dx: 8.0, dy: 8.0 }),
        static_sanitized: true,
        script_allowed: false,
        animation_allowed: false,
        external_resources_allowed: false,
        interactivity_allowed: false,
        transform_to_run: Some(affine()),
    }
}

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
    let bounds = BoundingBox::new(0.0, 0.0, 24.0, 24.0);
    let style = TextStyle {
        font_family: "Test".to_string(),
        font_size: 12.0,
        shade_color: 0x00ff_ffff,
        ..Default::default()
    };
    let mut tree = PageLayerTree::new(
        100.0,
        100.0,
        LayerNode::leaf(
            bounds,
            None,
            vec![PaintOp::text_run(
                bounds,
                TextRunNode {
                    text: "A".to_string(),
                    style: style.clone(),
                    char_shape_id: None,
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
                    field_marker: FieldMarkerType::None,
                    layout_positions: None,
                    display_text: None,
                },
            )],
        ),
    );
    let LayerNodeKind::Leaf { ops } = &mut tree.root.kind else {
        unreachable!("leaf fixture")
    };
    let PaintOp::TextRun {
        source: Some(source),
        ..
    } = &ops[0]
    else {
        unreachable!("PageLayerTree binds the fallback source")
    };
    let mut variant = PaintVariantMeta::text_run_default("text-0");
    variant.variant_id = "glyphOutline".to_string();
    variant.variant_kind = TextVariantKind::GlyphOutline;
    variant.is_default_fallback = false;
    variant.quality = Some(TextVariantQuality::Exact);
    variant.anchor_op_id = Some("text-0".to_string());
    ops.push(PaintOp::glyph_outline(
        bounds,
        LayerGlyphOutlinePaint {
            source: source.clone(),
            variant,
            payload_kind: GlyphOutlinePayloadKind::MonochromeFill,
            color_layers: None,
            bitmap_glyph: None,
            svg_glyph: None,
            paint_style: PaintTextStyle::from(&style),
            placement: TextRunPlacement {
                run_to_page: affine(),
                baseline_y: 12.0,
            },
            paths: vec![LayerGlyphOutlinePath {
                glyph_id: 1,
                source_range_utf8: TextSourceRange::new(0, 1),
                glyph_range: GlyphRange::new(0, 1),
                commands: commands(),
                fill_rule: GlyphOutlineFillRule::NonZero,
            }],
            stroke: None,
            diagnostics: GlyphRunDiagnostics {
                quality: TextVariantQuality::Exact,
                replay_eligibility: GlyphRunReplayEligibility::Portable,
                strict_visual_eligible: true,
                max_origin_delta_px: 0.0,
                max_advance_delta_px: 0.0,
                max_residual_after_adjustment_px: 0.0,
                cluster_mismatch_count: 0,
                missing_glyph_count: 0,
                used_fallback_font_count: 0,
                reason: None,
            },
        },
    ));

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
