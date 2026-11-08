// 휴대 가능한 glyph geometry 계약 검사의 공유 fixture.
// 호출 case의 타입·이름 공간을 유지하며 별도 test binary를 만들지 않는다.

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
        composite: None,
        clip: None,
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
        font_em_box: None,
        intrinsic_size: Some(LayerVector { dx: 8.0, dy: 8.0 }),
        static_sanitized: true,
        script_allowed: false,
        animation_allowed: false,
        external_resources_allowed: false,
        interactivity_allowed: false,
        transform_to_run: Some(affine()),
    }
}

fn monochrome_selector_tree() -> PageLayerTree {
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

    tree
}
