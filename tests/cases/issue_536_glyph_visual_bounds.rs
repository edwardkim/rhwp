//! Geometry contracts for recording bounds; these are not visual PDF evidence.

use rhwp::paint::*;
use rhwp::renderer::render_tree::BoundingBox;
use rhwp::renderer::{PathCommand, TextStyle};

fn affine(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> LayerAffineTransform {
    LayerAffineTransform { a, b, c, d, e, f }
}

fn rectangle(x: f64, y: f64, width: f64, height: f64) -> Vec<PathCommand> {
    vec![
        PathCommand::MoveTo(x, y),
        PathCommand::LineTo(x + width, y),
        PathCommand::LineTo(x + width, y + height),
        PathCommand::LineTo(x, y + height),
        PathCommand::ClosePath,
    ]
}

fn outline() -> LayerGlyphOutlinePaint {
    let mut variant = PaintVariantMeta::text_run_default("bounds-source");
    variant.variant_kind = TextVariantKind::GlyphOutline;
    variant.is_default_fallback = false;
    LayerGlyphOutlinePaint {
        source: TextSourceSpan {
            id: TextSourceId(0),
            utf8_range: TextSourceRange::new(0, 1),
            utf16_range: TextSourceRange::new(0, 1),
            stable_source_key: None,
        },
        variant,
        payload_kind: GlyphOutlinePayloadKind::MonochromeFill,
        color_layers: None,
        bitmap_glyph: None,
        svg_glyph: None,
        paint_style: PaintTextStyle::from(&TextStyle {
            font_size: 1.0,
            ..Default::default()
        }),
        placement: TextRunPlacement {
            run_to_page: affine(1.0, 0.0, 0.0, 1.0, 0.0, 0.0),
            baseline_y: 0.0,
        },
        paths: vec![LayerGlyphOutlinePath {
            glyph_id: 1,
            source_range_utf8: TextSourceRange::new(0, 1),
            glyph_range: GlyphRange::new(0, 1),
            commands: rectangle(0.0, 0.0, 4.0, 2.0),
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
    }
}

fn visual(outline: LayerGlyphOutlinePaint) -> BoundingBox {
    PaintOp::glyph_outline(BoundingBox::new(0.0, 0.0, 1.0, 1.0), outline).visual_bounds()
}

fn contains(bounds: BoundingBox, x: f64, y: f64) {
    assert!(
        x >= bounds.x - 1e-9
            && x <= bounds.x + bounds.width + 1e-9
            && y >= bounds.y - 1e-9
            && y <= bounds.y + bounds.height + 1e-9,
        "({x}, {y}) must be contained in {bounds:?}"
    );
}

fn same_bounds(actual: BoundingBox, expected: BoundingBox) {
    assert_eq!(
        [actual.x, actual.y, actual.width, actual.height],
        [expected.x, expected.y, expected.width, expected.height]
    );
}

fn color() -> ResolvedColor {
    ResolvedColor {
        color_space: Some("sRGB".to_owned()),
        rgba: [0.0, 0.0, 0.0, 1.0],
    }
}

fn font_ref(format: ColorGlyphFormat) -> FontColorGlyphRef {
    FontColorGlyphRef {
        face_key: Some("bounds-face".to_owned()),
        glyph_id: Some(1),
        palette_index: Some(0),
        color_format: Some(format),
    }
}

fn graph_node(id: u32, kind: ColorPaintGraphNodeKind) -> ColorPaintGraphNode {
    ColorPaintGraphNode {
        node_id: id,
        kind,
        solid_path: None,
        linear_gradient_path: None,
        radial_gradient_path: None,
        sweep_gradient_path: None,
        transform: None,
        composite: None,
        clip: None,
        source_range_utf8: Some(TextSourceRange::new(0, 1)),
        glyph_range: Some(GlyphRange::new(0, 1)),
        source_font_ref: Some(font_ref(ColorGlyphFormat::ColrV1)),
    }
}

fn leaf(id: u32, kind: ColorPaintGraphNodeKind) -> ColorPaintGraphNode {
    let mut node = graph_node(id, kind);
    let commands = rectangle(0.0, 0.0, 4.0, 2.0);
    let stops = vec![
        ColorGradientStop {
            offset: 0.0,
            color: color(),
        },
        ColorGradientStop {
            offset: 1.0,
            color: color(),
        },
    ];
    match kind {
        ColorPaintGraphNodeKind::SolidPath => {
            node.solid_path = Some(ColorPaintSolidPathNode {
                commands,
                fill: color(),
                fill_rule: GlyphOutlineFillRule::NonZero,
                source_glyph_id: Some(1),
                palette_index: Some(0),
            })
        }
        ColorPaintGraphNodeKind::LinearGradientPath => {
            node.linear_gradient_path = Some(ColorPaintLinearGradientPathNode {
                commands,
                gradient: ColorLinearGradient {
                    x0: 0.0,
                    y0: 0.0,
                    x1: 4.0,
                    y1: 0.0,
                    stops,
                },
                fill_rule: GlyphOutlineFillRule::NonZero,
                source_glyph_id: Some(1),
                palette_index: Some(0),
            })
        }
        ColorPaintGraphNodeKind::RadialGradientPath => {
            node.radial_gradient_path = Some(ColorPaintRadialGradientPathNode {
                commands,
                gradient: ColorRadialGradient {
                    cx: 2.0,
                    cy: 1.0,
                    radius: 2.0,
                    stops,
                },
                fill_rule: GlyphOutlineFillRule::NonZero,
                source_glyph_id: Some(1),
                palette_index: Some(0),
            })
        }
        ColorPaintGraphNodeKind::SweepGradientPath => {
            node.sweep_gradient_path = Some(ColorPaintSweepGradientPathNode {
                commands,
                gradient: ColorSweepGradient {
                    cx: 2.0,
                    cy: 1.0,
                    start_angle_degrees: 0.0,
                    end_angle_degrees: 360.0,
                    stops,
                },
                fill_rule: GlyphOutlineFillRule::NonZero,
                source_glyph_id: Some(1),
                palette_index: Some(0),
            })
        }
        _ => unreachable!("leaf fixture"),
    }
    node
}

fn graph_outline(nodes: Vec<ColorPaintGraphNode>) -> LayerGlyphOutlinePaint {
    let graph = ColorPaintGraphPayload {
        root_node_id: 0,
        nodes,
    };
    assert!(graph.has_colrv1_supported_graph_contract());
    let mut outline = outline();
    outline.payload_kind = GlyphOutlinePayloadKind::ColorLayers;
    outline.paths.clear();
    outline.color_layers = Some(ColorLayersPayload {
        color_format: ColorGlyphFormat::ColrV1,
        source_font_ref: Some(font_ref(ColorGlyphFormat::ColrV1)),
        palette_ref: None,
        layers: Vec::new(),
        paint_graph: Some(graph),
        source_range_utf8: Some(TextSourceRange::new(0, 1)),
        glyph_range: Some(GlyphRange::new(0, 1)),
    });
    outline
}

#[test]
fn transformed_monochrome_bounds_are_exported_without_changing_layout() {
    let logical = BoundingBox::new(0.0, 0.0, 1.0, 1.0);
    let mut outline = outline();
    outline.paths[0].commands = rectangle(-5.0, -3.0, 12.0, 5.0);
    outline.placement.run_to_page = affine(2.0, 0.0, 1.0, 4.0, 109.0, 200.0);
    let mut op = PaintOp::glyph_outline(logical, outline);
    let bounds = op.paint_bounds();
    same_bounds(bounds.logical, logical);
    contains(bounds.visual, 96.0, 188.0);
    contains(bounds.visual, 125.0, 208.0);
    let tree = PageLayerTree::new(20.0, 20.0, LayerNode::leaf(logical, None, vec![op.clone()]));
    same_bounds(tree.root.bounds, logical);
    let json: serde_json::Value = serde_json::from_str(&tree.to_json()).unwrap();
    let root = &json["root"]["visualBounds"];
    assert!(root["x"].as_f64().unwrap() + root["width"].as_f64().unwrap() >= 125.0);
    assert!(root["y"].as_f64().unwrap() + root["height"].as_f64().unwrap() >= 208.0);
    let PaintOp::GlyphOutline { outline, .. } = &mut op else {
        unreachable!()
    };
    outline.placement.run_to_page.e += 20.0;
    contains(op.visual_bounds(), 145.0, 208.0);
}

fn colrv0_outline(transform: LayerAffineTransform) -> LayerGlyphOutlinePaint {
    let mut outline = outline();
    outline.payload_kind = GlyphOutlinePayloadKind::ColorLayers;
    outline.paths.clear();
    let payload = ColorLayersPayload {
        color_format: ColorGlyphFormat::ColrV0,
        source_font_ref: Some(font_ref(ColorGlyphFormat::ColrV0)),
        palette_ref: None,
        layers: vec![ColorLayerNode {
            layer_index: Some(0),
            glyph_id: Some(1),
            glyph_range: Some(GlyphRange::new(0, 1)),
            source_range_utf8: Some(TextSourceRange::new(0, 1)),
            source_font_ref: Some(font_ref(ColorGlyphFormat::ColrV0)),
            commands: Some(rectangle(0.0, 0.0, 4.0, 2.0)),
            fill: Some(color()),
            fill_rule: Some(GlyphOutlineFillRule::NonZero),
            palette_index: Some(0),
            color: None,
            opacity: Some(1.0),
            transform_to_run: Some(transform),
        }],
        paint_graph: None,
        source_range_utf8: Some(TextSourceRange::new(0, 1)),
        glyph_range: Some(GlyphRange::new(0, 1)),
    };
    assert!(payload.has_colrv0_resolved_layer_contract());
    outline.color_layers = Some(payload);
    outline
}

#[test]
fn colrv0_applies_layer_transform_before_run_placement() {
    let mut outline = colrv0_outline(affine(0.0, 1.0, -1.0, 0.0, 10.0, -20.0));
    outline.placement.run_to_page = affine(2.0, 0.0, 0.0, 3.0, 100.0, 200.0);
    let bounds = visual(outline);
    contains(bounds, 116.0, 140.0);
    contains(bounds, 120.0, 152.0);
    assert_eq!(bounds.x + bounds.width, 120.0);
    assert_eq!(bounds.y + bounds.height, 152.0);
}

#[test]
fn colrv1_shared_leaves_keep_each_parent_transform_for_every_paint_kind() {
    for kind in [
        ColorPaintGraphNodeKind::SolidPath,
        ColorPaintGraphNodeKind::LinearGradientPath,
        ColorPaintGraphNodeKind::RadialGradientPath,
        ColorPaintGraphNodeKind::SweepGradientPath,
    ] {
        let mut root = graph_node(0, ColorPaintGraphNodeKind::Composite);
        root.composite = Some(ColorPaintCompositeNode {
            source_node_id: 1,
            backdrop_node_id: 2,
            mode: ColorPaintCompositeMode::SourceOver,
        });
        let mut left = graph_node(1, ColorPaintGraphNodeKind::Transform);
        left.transform = Some(ColorPaintTransformNode {
            child_node_id: 3,
            transform: affine(1.0, 0.0, 0.0, 1.0, -100.0, 0.0),
        });
        let mut right = graph_node(2, ColorPaintGraphNodeKind::Transform);
        right.transform = Some(ColorPaintTransformNode {
            child_node_id: 3,
            transform: affine(1.0, 0.0, 0.0, 1.0, 100.0, 0.0),
        });
        let mut outline = graph_outline(vec![root, left, right, leaf(3, kind)]);
        outline.placement.run_to_page = affine(1.0, 0.0, 0.0, 1.0, 10.0, 20.0);
        let bounds = visual(outline);
        contains(bounds, -90.0, 20.0);
        contains(bounds, 114.0, 22.0);
    }
}

#[test]
fn nested_graph_clips_stay_in_their_parent_space() {
    let mut outer = graph_node(0, ColorPaintGraphNodeKind::Clip);
    outer.clip = Some(ColorPaintClipNode {
        child_node_id: 1,
        clip_commands: rectangle(10.0, 10.0, 4.0, 4.0),
        fill_rule: GlyphOutlineFillRule::NonZero,
    });
    let mut transform = graph_node(1, ColorPaintGraphNodeKind::Transform);
    transform.transform = Some(ColorPaintTransformNode {
        child_node_id: 2,
        transform: affine(1.0, 0.0, 0.0, 1.0, 8.0, 12.0),
    });
    let mut inner = graph_node(2, ColorPaintGraphNodeKind::Clip);
    inner.clip = Some(ColorPaintClipNode {
        child_node_id: 3,
        clip_commands: rectangle(0.0, 0.0, 3.0, 1.0),
        fill_rule: GlyphOutlineFillRule::EvenOdd,
    });
    let mut outline = graph_outline(vec![
        outer,
        transform,
        inner,
        leaf(3, ColorPaintGraphNodeKind::SolidPath),
    ]);
    outline.placement.run_to_page = affine(2.0, 0.0, 0.0, 3.0, 100.0, 200.0);
    let bounds = visual(outline.clone());
    contains(bounds, 120.0, 236.0);
    contains(bounds, 122.0, 239.0);
    assert_eq!(bounds.x + bounds.width, 122.0);
    assert_eq!(bounds.y + bounds.height, 239.0);
    outline
        .color_layers
        .as_mut()
        .unwrap()
        .paint_graph
        .as_mut()
        .unwrap()
        .nodes[1]
        .transform
        .as_mut()
        .unwrap()
        .transform
        .e = 100.0;
    let mut empty = outline.clone();
    empty.color_layers = None;
    // A disjoint clip adds no ink extent beyond the logical fallback estimate.
    same_bounds(visual(outline), visual(empty));
}

#[test]
fn curve_arc_and_miter_extents_survive_placement() {
    let mut cubic = outline();
    cubic.paths[0].commands = vec![
        PathCommand::MoveTo(0.0, 0.0),
        PathCommand::CurveTo(0.0, 100.0, 100.0, 100.0, 100.0, 0.0),
    ];
    cubic.placement.run_to_page = affine(0.0, 1.0, -1.0, 0.0, 200.0, 100.0);
    // This cubic's t=1/2 point is (50, 75), then rotated and translated.
    contains(visual(cubic), 125.0, 150.0);
    let mut arc = outline();
    arc.paths[0].commands = vec![
        PathCommand::MoveTo(10.0, 0.0),
        PathCommand::ArcTo(10.0, 10.0, 0.0, false, true, -10.0, 0.0),
    ];
    arc.placement.run_to_page = affine(1.0, 0.0, 0.0, 1.0, 300.0, 200.0);
    let bounds = visual(arc);
    for point in [(310.0, 200.0), (300.0, 210.0), (290.0, 200.0)] {
        contains(bounds, point.0, point.1);
    }
    let mut miter = outline();
    miter.paths[0].commands = vec![
        PathCommand::MoveTo(0.0, 0.0),
        PathCommand::LineTo(1.0, 10.0),
        PathCommand::LineTo(2.0, 0.0),
    ];
    miter.payload_kind = GlyphOutlinePayloadKind::MonochromeFillStroke;
    miter.stroke = Some(GlyphOutlineStrokeStyle {
        color: 0,
        width: 2.0,
        join: GlyphOutlineStrokeJoin::Miter,
        cap: GlyphOutlineStrokeCap::Butt,
        miter_limit: 11.0,
        paint_order: GlyphOutlinePaintOrder::FillThenStroke,
    });
    miter.placement.run_to_page = affine(3.0, 0.0, 0.0, 2.0, 100.0, 100.0);
    // The two offset edges intersect at y=10+sqrt(101), within this miter limit.
    contains(
        visual(miter),
        103.0,
        100.0 + 2.0 * (10.0 + 101.0_f64.sqrt()),
    );
}

#[test]
fn bitmap_and_svg_placements_are_not_translated_by_run_to_page_again() {
    let mut bitmap = outline();
    bitmap.payload_kind = GlyphOutlinePayloadKind::BitmapGlyph;
    bitmap.paths.clear();
    bitmap.placement.run_to_page.e = 10_000.0;
    bitmap.bitmap_glyph = Some(BitmapGlyphPayload {
        image_ref: ImageResourceId(0),
        source_range_utf8: TextSourceRange::new(0, 1),
        glyph_range: GlyphRange::new(0, 1),
        placement: BoundingBox::new(100.0, 200.0, 4.0, 2.0),
        alpha_premultiplied: false,
        scaling_policy: BitmapGlyphScalingPolicy::SourceExact,
        filtering: BitmapGlyphFiltering::Linear,
        transform_to_run: Some(affine(1.0, 0.0, 0.0, 1.0, 5.0, -5.0)),
    });
    let mut svg = bitmap.clone();
    svg.bitmap_glyph = None;
    svg.payload_kind = GlyphOutlinePayloadKind::SvgGlyph;
    svg.svg_glyph = Some(SvgGlyphPayload {
        svg_ref: SvgResourceId(0),
        source_range_utf8: TextSourceRange::new(0, 1),
        glyph_range: GlyphRange::new(0, 1),
        view_box: BoundingBox::new(0.0, 0.0, 4.0, 2.0),
        font_em_box: Some(BoundingBox::new(100.0, 200.0, 4.0, 2.0)),
        intrinsic_size: None,
        static_sanitized: true,
        script_allowed: false,
        animation_allowed: false,
        external_resources_allowed: false,
        interactivity_allowed: false,
        transform_to_run: Some(affine(1.0, 0.0, 0.0, 1.0, 5.0, -5.0)),
    });
    for bounds in [visual(bitmap), visual(svg)] {
        assert_eq!(bounds.x + bounds.width, 109.0);
        assert_eq!(bounds.y + bounds.height, 197.0);
    }
}

#[test]
fn malformed_paths_and_cyclic_graphs_keep_the_logical_fallback_estimate() {
    let mut empty = outline();
    empty.paths.clear();
    let expected = visual(empty);
    let mut malformed = outline();
    malformed.paths[0].commands = vec![PathCommand::MoveTo(f64::NAN, 0.0)];
    same_bounds(visual(malformed), expected);
    let mut cyclic = graph_outline(vec![leaf(0, ColorPaintGraphNodeKind::SolidPath)]);
    let graph = cyclic
        .color_layers
        .as_mut()
        .unwrap()
        .paint_graph
        .as_mut()
        .unwrap();
    let mut node = graph_node(0, ColorPaintGraphNodeKind::Transform);
    node.transform = Some(ColorPaintTransformNode {
        child_node_id: 0,
        transform: affine(1.0, 0.0, 0.0, 1.0, 100.0, 100.0),
    });
    graph.nodes[0] = node;
    same_bounds(visual(cyclic), expected);
}

#[test]
fn f32_path_and_matrix_input_rounding_are_included_before_cancellation() {
    let mut path = outline();
    path.paths[0].commands = rectangle(16_777_217.0, 0.0, 4.0, 2.0);
    let colrv0 = colrv0_outline(affine(1.0, 0.0, 0.0, 1.0, 16_777_217.0, 0.0));
    let mut transform = graph_node(0, ColorPaintGraphNodeKind::Transform);
    transform.transform = Some(ColorPaintTransformNode {
        child_node_id: 1,
        transform: affine(1.0, 0.0, 0.0, 1.0, 16_777_217.0, 0.0),
    });
    let colrv1 = graph_outline(vec![transform, leaf(1, ColorPaintGraphNodeKind::SolidPath)]);
    for mut outline in [path, colrv0, colrv1] {
        outline.paint_style.font_size = 12.0;
        outline.placement.run_to_page.e = -16_777_216.0;
        let logical = BoundingBox::new(10.0, 10.0, 1.0, 1.0);
        let op = PaintOp::glyph_outline(logical, outline);
        same_bounds(op.bounds(), logical);
        // 2^24+1 rounds to 2^24 at the Native input boundary, before the
        // translation cancels it. The f64-only estimate starts at x=1.
        contains(op.visual_bounds(), 0.0, 1.0);
        contains(op.visual_bounds(), 5.0, 1.0);
    }
}

#[test]
fn f32_matrix_composition_rounding_is_included_before_point_mapping() {
    let mut outline = colrv0_outline(affine(4097.0, 0.0, 0.0, 1.0, 0.0, 0.0));
    outline.color_layers.as_mut().unwrap().layers[0].commands =
        Some(rectangle(1.0, 0.0, 0.25, 2.0));
    outline.placement.run_to_page = affine(4097.0, 0.0, 0.0, 1.0, -16_785_408.0, 0.0);
    let op = PaintOp::glyph_outline(BoundingBox::new(10.0, 10.0, 1.0, 1.0), outline);
    // All inputs are exact f32 values. The composed scale 4097^2 is rounded
    // from 16_785_409 to 16_785_408 before x=1 is mapped, yielding x=0.
    contains(op.visual_bounds(), 0.0, 1.0);
}

#[test]
fn graph_clip_encloses_separately_rounded_path_transformations() {
    let mut transform = graph_node(0, ColorPaintGraphNodeKind::Transform);
    transform.transform = Some(ColorPaintTransformNode {
        child_node_id: 1,
        transform: affine(1.0, 0.0, 0.0, 1.0, 16_777_216.0, 0.0),
    });
    let mut clip = graph_node(1, ColorPaintGraphNodeKind::Clip);
    clip.clip = Some(ColorPaintClipNode {
        child_node_id: 2,
        clip_commands: rectangle(1.0, 0.0, 2.0, 2.0),
        fill_rule: GlyphOutlineFillRule::NonZero,
    });
    let mut fill = leaf(2, ColorPaintGraphNodeKind::SolidPath);
    fill.solid_path.as_mut().unwrap().commands = rectangle(-1.0, 0.0, 6.0, 2.0);
    let mut outline = graph_outline(vec![transform, clip, fill]);
    outline.placement.run_to_page.e = -16_777_216.0;
    let op = PaintOp::glyph_outline(BoundingBox::new(10.0, 10.0, 1.0, 1.0), outline);
    // Native transforms the clip path by +2^24 and then -2^24. Its original
    // x=1..3 becomes x=0..4; composing the matrices first would miss this.
    contains(op.visual_bounds(), 0.0, 1.0);
}
